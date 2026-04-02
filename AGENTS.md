# Agent Guide for litsdf

This document is for AI agents working on litsdf. It covers project goals, architecture, conventions, and critical knowledge that may not be obvious from reading the code alone.

## Project Goals

litsdf is an interactive SDF (Signed Distance Function) editor. The user creates 3D objects by:
1. Building a bone hierarchy (skeleton) in the left tree panel
2. Attaching SDF primitive shapes to bones
3. Editing shape properties (type, size, position, color, combine op) in the right panel
4. Viewing the result in realtime via ray-marched rendering

The app targets artists and technical users who want to create complex SDF objects visually.

## Core Principles

- **Bevy-first**: Use Bevy's ECS, rendering, and plugin patterns. Don't fight the engine.
- **litui for declarative UI**: Properties panel, dialogs, and file browser are defined in markdown files under `content/`. The bone tree is an exception — it uses raw egui because litui's foreach can't express recursive structures.
- **Realtime feedback**: Every property change must update the 3D viewport immediately. The `dirty` flag on `SdfSceneState` triggers shader parameter rebuild.
- **Compact YAML persistence**: Scenes save/load as human-readable YAML. Default values are omitted via `#[serde(skip_serializing_if)]` — a shape with default transform/material produces just its id, name, and primitive.
- **Viewport picking**: Click shapes in the 3D viewport to select them. CPU-side ray marching with the same SDF formulas as the shader.
- **Test coverage**: 21 unit tests for data model, scene flattening, property sync, compact YAML. Screenshot tests for visual regression.

## Architecture Overview

```
┌──────────────────┬─────────────────────────┬──────────────────┐
│ Left Panel       │                         │ Right Panel      │
│ (pure egui)      │   Bevy 3D Viewport      │ (litui)          │
│                  │   Ray-marched SDF        │                  │
│ Bone tree with   │                         │ Bone properties  │
│ CollapsingHeader │                         │ Shape list       │
│ +Bone +Shape     │                         │ Shape properties │
│ Delete, Gizmos   │                         │ Save/Load        │
└──────────────────┴─────────────────────────┴──────────────────┘
```

### Data Flow

```
User edits slider → ui.md.state.tx changes
  → sync_shape_properties() detects change, writes to SdfShape.transform
  → sets scene.dirty = true
  → sync_scene_to_shader() rebuilds SdfShaderParams
  → mutates SdfMaterial asset (triggers Bevy AssetChanged)
  → GPU re-uploads uniform buffer
  → fragment shader reads new data, renders updated scene
```

### Workspace Layout

```
crates/
  litsdf_core/           — No Bevy dependency
    src/
      lib.rs             — pub mod exports
      models.rs          — BoneId, SdfBone, SdfShape, SdfPrimitive, ShapeMaterial, etc.
      scene.rs           — compute_bone_world_transforms, flatten_bone_tree, anim_eval
      sdf.rs             — CPU-side SDF primitives (sd_sphere, sd_box, etc.)
      persistence.rs     — YAML save/load, scenes directory
  litsdf_render/         — Bevy rendering plugin
    src/
      lib.rs             — SdfRenderPlugin registration
      shader.rs          — SdfMaterial, ShaderShape, SdfShaderParams (GPU types)
      scene_sync.rs      — SdfSceneState resource, sync_scene_to_shader, build_shader_params
      camera.rs          — OrbitCamera, setup_camera, orbit_camera
      gizmos.rs          — Bone gizmos + compass gizmo
      picking.rs         — CPU ray march pick + drag handles
    assets/shaders/
      sdf_raymarch.wgsl  — WGSL ray marching shader
  litsdf_editor/         — Editor UI plugin
    src/
      lib.rs             — SdfEditorPlugin registration
      ui/mod.rs          — EditorUi resource, editor_ui system, manual panel rendering
      ui/tree.rs         — Pure egui recursive bone tree
      ui/populate.rs     — Populate litui state from scene
      ui/sync.rs         — Sync slider values back to model
      ui/handlers.rs     — Button click handlers
      ui/helpers.rs      — Primitive/combo conversion
      undo.rs            — UndoHistory + undo_redo_system
      testing.rs         — Screenshot capture systems
    content/             — litui markdown UI definitions
src/
  main.rs              — Editor binary
  bin/viewer.rs        — Viewer binary (no UI)
scenes/                — Example YAML scene files
knowledge/             — Architecture documentation
scripts/               — generate-api-docs.py
    sdf_raymarch.wgsl — WGSL ray marching fragment shader
knowledge/          — This documentation
tests/
  screenshot_test.sh — Integration test script
  screenshots/       — Captured test screenshots
```

## Critical Technical Knowledge

### Bevy 0.18 Specifics

**Material bind group index**: Custom materials use `@group(3)` in WGSL, NOT `@group(2)`. Group 0 = view/globals, Group 1 = light probes, Group 2 = mesh transforms, Group 3 = material bindings. Using `@group(2)` causes a pipeline validation error. The constant is `MATERIAL_BIND_GROUP_INDEX = 3` in `bevy_pbr::material`.

**Uniform vs storage for material data**: Use `#[uniform(0)]` with inline data, not `#[storage(0, read_only)]` with `Handle<ShaderStorageBuffer>`. The storage buffer approach breaks realtime updates because mutating the buffer asset doesn't trigger material change detection — the bind group never gets rebuilt. With inline uniform, `materials.get_mut(handle)` triggers `AssetChanged` automatically.

**EventReader → MessageReader**: In Bevy 0.18, events are messages. Use `bevy::ecs::message::MessageReader` instead of the removed `EventReader`.

**AmbientLight**: Is a camera component in 0.18, not a standalone resource. Requires `affects_lightmapped_meshes` field.

**EguiPrimaryContextPass**: UI systems must run in this schedule, not `Update`. The egui frame context (`Context::run`) only exists during this schedule. Running litui's `show_all` or egui panel code in `Update` causes "Called available_rect() before Context::run()" panic.

**Backface culling**: The SDF bounding cuboid (40x40x40) has the camera inside it. The Material's `specialize()` method must set `descriptor.primitive.cull_mode = None` or nothing renders.

### SDF Rendering

The shader (`assets/shaders/sdf_raymarch.wgsl`) implements:
- **8 SDF primitives**: Sphere, Box, RoundBox, Cylinder, CappedCone, Torus, Capsule, Plane
- **6 combination ops**: Union, Intersection, Subtraction + smooth variants (with k parameter)
- **Ray marching**: 128 steps max, epsilon 0.001
- **Normals**: Tetrahedron technique (4 SDF evaluations)
- **Lighting**: Directional Phong + soft shadows + ambient occlusion
- **Per-shape color**: Tracked via closest shape index during ray march

The shader receives a flat `SdfShaderParams` uniform with `shape_count` and `shapes[32]` array. It knows nothing about bones — all hierarchy is resolved CPU-side.

### Bone Hierarchy

Bones are purely spatial organizers. Each bone has:
- `id: BoneId` (UUID, root uses `Uuid::nil()`)
- `name: String`
- `transform: ShapeTransform` (local, relative to parent)
- `children: Vec<SdfBone>` (recursive tree)
- `shapes: Vec<SdfShape>` (shapes attached to this bone)

`build_shader_params()` in `scene.rs`:
1. `compute_bone_world_transforms()` — walks the bone tree, multiplies parent * local matrices
2. `flatten_bone_tree()` — DFS traversal, computes world position for each shape (bone_world * shape_local), emits flat `ShaderShape` array
3. Each shape uses its own `combination` op. First shape overall gets Union (ignored by shader).

The root bone is always at origin, immutable, cannot be deleted. New scenes always have a root bone.

### UI Architecture

**Left panel (pure egui)**: Rendered in `render_tree_panel()` using `egui::SidePanel::left`. The bone tree uses `egui::collapsing_header::CollapsingState` recursively — egui manages open/close state internally via persistent IDs keyed by `BoneId`.

**Right panel (litui)**: Rendered via `app::render_properties(ui, &mut state)` inside a manually-created `egui::SidePanel::right`. litui state (`AppState`) is populated each frame from the scene model.

**Windows (litui)**: Add Shape and File Browser dialogs use litui's `render_add_shape` and `render_file_browser` inside `egui::Window` calls. The `show_add_shape`/`show_file_browser` bools on AppState control visibility.

**Why not show_all()**: litui's `show_all()` renders ALL panels. Since the left panel is pure egui, we can't use `show_all()` — it would render the litui shapes.md placeholder over our egui tree. Instead, we manually create each panel container and call individual litui render functions.

### Button Click Detection (litui pattern)

litui buttons generate a `u32` click counter field (e.g., `on_confirm_add_count`). Each frame:
1. Compare current count against `prev_on_confirm_add` stored on `EditorUi`
2. If current > prev, the button was clicked this frame
3. Update prev to current

For per-row buttons in foreach (e.g., shape select), use `HashMap<ShapeId, u32>` to track previous counts by ID.

**Critical**: When bone selection changes, skip `sync_shape_properties` for that frame — the UI state still holds the old shape's values and would overwrite the new shape.

### Property Sync

Properties flow in one direction per concern:
- **Scene → UI**: Only on selection change (`populate_shape_properties` checks `prev_selected_shape`)
- **UI → Scene**: Every frame (`sync_shape_properties` compares UI values against model)

This prevents the UI from overwriting slider edits every frame while still loading correct values on selection change.

### Viewport Picking (picking.rs)

CPU-side ray marching for click-to-select in the 3D viewport. When the user clicks (not drags) in the viewport:

1. `camera.viewport_to_world(transform, cursor_pos)` converts screen click to `Ray3d`
2. `pick_shape(ray, scene)` ray marches using the same SDF formulas as the shader (all 8 primitives ported to Rust)
3. On hit, evaluates each shape individually at the hit point to find which one is closest
4. Sets `selected_shape` and `selected_bone` on `SdfSceneState`

Click vs drag detection: tracks mouse press position, only picks if movement < 3px. Uses `just_released(MouseButton::Left)` to avoid conflicting with orbit camera drag. Respects `EguiWantsInput` — no picking when clicking UI panels.

Runs in `Update` schedule (not EguiPrimaryContextPass) since it reads mouse input.

### Compass Gizmo (gizmos.rs)

An egui overlay in the bottom-left corner showing camera-relative XYZ axes:

1. Gets camera rotation, computes inverse (view rotation)
2. Projects each world axis (X, Y, Z) through the view rotation to get 2D screen directions
3. Draws colored arrows (red X, green Y, blue Z) with labels using `egui::Painter::arrow()`
4. Depth-sorts axes so nearer ones draw on top
5. Uses `egui::Area` with `interactable(false)` so it doesn't capture clicks

Runs in `EguiPrimaryContextPass` (needs egui context).

### Compact YAML (models.rs)

All model structs use `#[serde(default, skip_serializing_if)]` to omit default values:

- `ShapeTransform` fields: translation omitted if [0,0,0], rotation if [0,0,0], scale if 1.0
- `ShapeMaterial` fields: color omitted if white, roughness if 0.5, metallic if 0.0
- `SdfShape`: transform/material omitted if fully default, modifiers if empty, combination if Union
- `SdfBone`: transform omitted if identity, children/shapes if empty
- `SdfScene`: combination omitted if Union

Per-field skipping (not just per-struct) means a transform with only translation produces just `translation: [1.5, 0.0, 0.0]` without rotation or scale.

Helper functions: `is_zero()`, `is_one()`, `is_half()`, `is_zero_array()`, `is_white()` + matching default providers `one()`, `half()`, `white()`.

`CombinationOp` implements `Default` (Union) and `is_default()`.

All fields have `#[serde(default)]` for backwards compatibility — loading old verbose YAML files still works.

### Testing

**Unit tests** (21 total):
- `models::tests` — YAML round-trips, bone find/remove, compact YAML (omits defaults, keeps non-defaults, round-trip stability)
- `scene::tests` — Shader param flattening, bone transform composition
- `persistence::tests` — Save/load, file listing
- `ui::tests` — Click-through flow (bone→shape→edit), no-dirty-when-unchanged, nested bone selection

**Screenshot tests**:
- `LITSDF_SCREENSHOT=path.png cargo run` — single frame capture
- `LITSDF_TEST_SEQUENCE=dir/ cargo run` — multi-step sequence with bone selection changes
- `./tests/screenshot_test.sh` — runs default + nested + multi-bone scenes

### Litui Integration

litui is a macro-based UI framework that generates egui code from markdown files. The `define_litui_app!` macro in `ui/mod.rs` processes:
- `content/properties.md` — right panel with sliders, comboboxes, color picker, foreach shape list
- `content/add_shape.md` — window dialog
- `content/file_browser.md` — window dialog

It generates `AppState` struct with fields for every widget, `render_*` functions per page, and row structs for foreach loops (`Bone_shapesRow`, `File_rowsRow`).

litui is a local dependency at `../egui-md-macro/crates/litui`. It requires `eframe = "0.33"` and `egui = "0.33"` as peer dependencies.

### Common Pitfalls

1. **Adding `@group(2)` bindings in WGSL** — This is the mesh bind group, not material. Use `@group(3)`.
2. **Mutating storage buffers expecting material update** — Use inline `#[uniform(0)]` instead.
3. **Running UI code in Update schedule** — Use `EguiPrimaryContextPass`.
4. **Overwriting slider edits** — Don't populate properties every frame, only on selection change.
5. **Selection carrying old values** — Skip sync on the frame selection changes.
6. **litui widget configs** — Every `{config}` in markdown must match a key in `widgets:` section. Button labels with spaces need underscores.
7. **Foreach field references** — `{field}` in litui table cells auto-declares String fields on the row struct. `{field}` in button labels does NOT auto-declare.
