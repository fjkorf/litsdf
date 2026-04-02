# UI Architecture

## Hybrid Approach: egui + litui

The UI uses two rendering approaches:
- **Menu bar + status bar + left panel (bone tree)**: Pure egui
- **Right panel + dialogs**: litui markdown-driven UI

### Why Hybrid?

litui's `foreach` is flat — it can't express recursive tree structures. The bone tree needs arbitrary-depth nesting with collapsible nodes. egui's `CollapsingState` handles this natively. Menu bars, context menus, and status bars are also pure egui.

The properties panel, add-shape dialog, and file browser are flat forms with sliders/comboboxes/tables — perfect for litui's declarative markdown.

## Panel Rendering (No show_all)

We DON'T call litui's `show_all()`. Instead, we manually create each egui container and call litui's per-page render functions:

```rust
// Menu bar: pure egui
egui::TopBottomPanel::top("menu_bar").show(&ctx, |ui| {
    egui::MenuBar::new().ui(ui, |ui| { ... });
});

// Status bar: pure egui
egui::TopBottomPanel::bottom("status_bar").show(&ctx, |ui| { ... });

// Left panel: pure egui
egui::SidePanel::left("bone_tree").show(&ctx, |ui| {
    tree::render_bone_tree(ui, bone, selected_bone, selected_shape);
});

// Right panel: litui
egui::SidePanel::right("panel_properties").show(&ctx, |ui| {
    app::render_properties(ui, &mut state);
});

// Windows: litui
egui::Window::new("Add Shape").show(&ctx, |ui| {
    app::render_add_shape(ui, &mut state);
});
```

### Why Not show_all()?

`show_all()` renders ALL panels defined in `define_litui_app!`. Since the left panel is pure egui, litui would render a placeholder over our tree. Manual rendering gives us control over which panels litui manages.

## Menu Bar (ui/mod.rs)

Rendered as an `egui::TopBottomPanel::top` before all other panels. Uses `egui::MenuBar::new().ui()` with `ui.menu_button()` for each menu.

| Menu | Items |
|------|-------|
| **File** | New Scene (Cmd+N), Open... (Cmd+O), Save (Cmd+S) |
| **Edit** | Undo (Cmd+Z), Redo (Cmd+Shift+Z), Duplicate (Cmd+D), Delete (Del), Deselect (Esc) |
| **Add** | Bone, separator, 9 primitive types |
| **View** | Bone Gizmos checkbox, Frame Selection (F), Reset Camera |

Menu items display shortcut hints via `Button::new("Label").shortcut_text(ctx.format_shortcut(&shortcut))`.

## Keyboard Shortcuts (ui/shortcuts.rs)

Shortcut constants defined in `shortcuts.rs`. Checked every frame at the top of `editor_ui` via `ctx.input_mut(|i| i.consume_shortcut(&shortcut))` — must be outside menus to fire reliably.

Undo/redo is handled inline in the egui pass (not a separate Bevy system), keeping all shortcuts in one place.

Single-key shortcuts (F, Escape, Delete) use `ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F))`.

All shortcut processing feeds into a `ShortcutAction` enum, dispatched after menu rendering.

## Status Bar (ui/mod.rs)

Rendered as `egui::TopBottomPanel::bottom` after the menu bar. Shows:
- Selection info: "ShapeName (PrimitiveType) on BoneName" or "Bone: BoneName" or "No selection"
- Scene stats: "N bones, M shapes"

## The Tree (ui/tree.rs)

```rust
pub fn render_bone_tree(ui, bone, selected_bone, selected_shape) -> TreeResult {
    // Returns TreeResult { action: TreeAction, context: ContextAction }
}
```

Each bone/shape in the tree has:
- **Eye icon toggle** (👁) — clicks emit `ContextAction::ToggleBoneVisibility` / `ToggleShapeVisibility`
- **Selectable label** — clicks emit `TreeAction::SelectBone` / `SelectShape`
- **Right-click context menu** — bone: Add Child Bone, Add Shape submenu, Duplicate, Delete; shape: Duplicate, Delete

Key points:
- Uses `BoneId.0` (UUID) as persistent egui ID — stable across frames
- `CollapsingState` manages open/close internally — no manual HashMap tracking
- Returns `TreeResult` combining selection actions and context menu actions
- `selectable_label` provides visual highlighting for selected bone/shape
- Context menus use `response.context_menu(|ui| { ... })`

## Action Collection Pattern

The tree panel can't directly mutate `scene` or `ui` because it borrows them in closures. Instead, it returns a `TreePanelActions` struct:

```rust
struct TreePanelActions {
    select_bone: Option<BoneId>,
    select_shape: Option<ShapeId>,
    add_bone: bool,
    add_shape: bool,
    delete_selected: bool,
    show_gizmos: Option<bool>,
    context_action: tree::ContextAction,
}
```

After all panels render, the main `editor_ui` function applies these actions to the scene state.

## litui State Population

Each frame, BEFORE rendering:
1. `populate_bone_shapes()` — fills `bone_shapes` foreach rows from selected bone's shapes
2. `populate_shape_properties()` — fills sliders from selected shape (only on selection change)
3. `populate_bone_properties()` — fills bone sliders (only on selection change)
4. `populate_file_browser()` — lists YAML files in scenes directory

AFTER rendering:
1. `handle_confirm_add()` — checks `on_confirm_add_count` increment
2. `handle_delete_shape()` — checks `on_delete_shape_count` increment
3. `handle_shape_selection()` — checks per-shape `on_select_shape_count` increments
4. `sync_shape_properties()` — writes slider values back to model
5. `sync_bone_properties()` — writes bone slider values back to model

## litui Markdown Files

### properties.md (right panel)

Section order (top to bottom):
1. Scene name textedit
2. Light direction sliders (always visible)
3. Bone properties (conditional: `has_bone_selection`)
   - Name textedit, transform sliders
4. Shape list (foreach `bone_shapes`)
5. Shape properties (conditional: `has_selection`)
   - Primitive type + params, Geometry (combine op), Transform, Material, Noise, Symmetry, Modifiers, Actions

### add_shape.md (window dialog)

Simple: combobox for shape type + confirm button. Opens when `show_add_shape = true`.

### file_browser.md (window dialog)

Textedit for filename, foreach file list with load buttons, save button. Opens when `show_file_browser = true`. Triggered by File menu Save/Open shortcuts.

## Button Click Counter Pattern

litui generates `{config}_count: u32` fields for buttons. Detection:

```rust
if ui.md.state.on_confirm_add_count > ui.prev_on_confirm_add {
    // button was clicked this frame
    do_something();
}
ui.prev_on_confirm_add = ui.md.state.on_confirm_add_count;
```

For per-row buttons in foreach, use `HashMap<Id, u32>`:

```rust
for (i, row) in state.bone_shapes.iter().enumerate() {
    let id = shape_order[i];
    let prev = prev_clicks.get(&id).copied().unwrap_or(0);
    if row.on_select_shape_count > prev {
        // clicked
    }
}
// update all prev counts after
```

## Node Editor (nodes/ module)

Toggled via View > Node Editor. Renders as a resizable `TopBottomPanel::bottom` using egui-snarl.

### Graph Storage

Node graphs are stored in the editor, NOT on the core model:
- `ui.node_graphs: HashMap<ShapeId, Snarl<SdfNode>>` — per-shape graphs
- `ui.bone_graphs: HashMap<BoneId, Snarl<SdfNode>>` — per-bone graphs

This keeps `litsdf_core` free of egui-snarl dependency.

### Panel Behavior

- When a **shape** is selected: shows shape graph with ShapeOutput node (10 pins: pos/rot/scale/color)
- When a **bone** is selected (no shape): shows bone graph with BoneOutput node (7 pins: pos/rot/scale)
- "Create Starter Graph" button adds Time → SinOscillator → Output template
- "Clear Graph" button removes the graph entirely

### Evaluation

After all UI sync, node graphs are evaluated per-frame:
1. For each shape graph: evaluate ShapeOutput pins, override shape transform/material properties
2. For each bone graph: evaluate BoneOutput pins, override bone transform properties
3. If any graph produced values, `scene.dirty = true` (triggers shader re-sync)
4. Node-driven changes skip the undo stack (graph is source of truth, not shape fields)

### Gotcha: `Snarl` Borrow Conflicts

`ui.node_graphs` and `ui.node_style` can't be borrowed simultaneously (one mutable, one immutable). Clone the style before accessing the graph: `let style = ui.node_style.clone();`
