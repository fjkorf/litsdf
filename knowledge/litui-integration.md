# litui Integration

## What is litui?

litui is a macro-based UI framework that generates egui code from markdown files. It's a local dependency at `../egui-md-macro/crates/litui`. The macro `define_litui_app!` processes markdown files at compile time and generates:

- `AppState` struct with fields for every widget
- `render_*` functions per page
- Row structs for `foreach` loops
- `LituiApp` struct with `show_all()`, `show_page()`, `show_nav()`

## How litsdf Uses litui

```rust
pub mod app {
    use litui::*;
    define_litui_app! {
        parent: "content/_app.md",
        "content/properties.md",
        "content/add_shape.md",
        "content/file_browser.md",
    }
}
```

This generates `app::AppState`, `app::LituiApp`, `app::render_properties()`, etc.

Note: `content/shapes.md` is NOT in the macro — the left panel is pure egui.

## Markdown Syntax

### Page Frontmatter

```yaml
---
page:
  name: Properties
  label: Properties
  panel: right       # left, right, top, bottom, window
  width: 260.0
  default: true      # one page must be default

widgets:
  slider_cfg:
    min: -10
    max: 10
    label: "X"
  combo_opts:
    options: ["A", "B", "C"]
  button_cfg: {}
  text_cfg:
    hint: "Enter text"
---
```

### Widget Types

| Syntax | State Type | Notes |
|--------|-----------|-------|
| `[slider](field){cfg}` | f64 | Config: min, max, label, suffix |
| `[combobox](field){cfg}` | usize | Config: options list |
| `[checkbox](field)` | bool | No config needed |
| `[textedit](field){cfg}` | String | Config: hint |
| `[color](field)` | [u8; 4] | RGBA color picker |
| `[button](Label){cfg}` | u32 (count) | Click counter |
| `[display](field)` | String | Read-only |
| `[progress](field){cfg}` | f64 | 0.0-1.0 |

### Conditional Blocks

```markdown
::: if has_selection
Content shown when has_selection is true.
:::
```

The field (`has_selection: bool`) is auto-declared on AppState.

### Foreach Loops

```markdown
::: foreach bone_shapes

| {shape_name} | [button](>){on_select_shape} |
|---|---|

:::
```

Generates `Bone_shapesRow` struct with `shape_name: String` and `on_select_shape_count: u32`. The `bone_shapes: Vec<Bone_shapesRow>` field is added to AppState.

### Horizontal Layout

```markdown
::: horizontal
[button](Save){on_save} [button](Load){on_load}
:::
```

### Window Pages

```yaml
page:
  name: AddShape
  panel: window
  open: show_add_shape  # bool field controlling visibility
```

## Generated Types

From litsdf's content files, litui generates:

```rust
pub struct AppState {
    // From properties.md widgets
    pub prim_type: usize,           // combobox
    pub param_a: f64,               // slider
    pub tx: f64,                    // slider
    pub shape_color: [u8; 4],       // color
    pub combo_op: usize,            // combobox
    pub bone_name: String,          // textedit
    pub bone_tx: f64,               // slider

    // From foreach
    pub bone_shapes: Vec<Bone_shapesRow>,
    pub file_rows: Vec<File_rowsRow>,

    // Conditional flags
    pub has_bone_selection: bool,
    pub has_selection: bool,
    pub no_selection: bool,
    pub bone_editable: bool,
    pub bone_is_root: bool,

    // Window visibility
    pub show_add_shape: bool,
    pub show_file_browser: bool,

    // Button counters
    pub on_select_shape_count: u32,  // NO — this is per-row
    pub on_save_count: u32,
    pub on_load_count: u32,
    ...
}

pub struct Bone_shapesRow {
    pub shape_name: String,
    pub on_select_shape_count: u32,
}

pub struct File_rowsRow {
    pub name: String,
    pub on_pick_file_count: u32,
}
```

## Key Constraints

1. **Widget configs must be referenced**: Every key in `widgets:` must appear as `{key}` somewhere in the markdown, or compilation fails.
2. **Button labels**: `[button](Label){config}` — Label can't have spaces (use underscores). `{field}` references in button labels work for top-level foreach but NOT for inner foreach.
3. **Foreach is flat**: No nested foreach, no recursive structures. Use egui directly for trees.
4. **Field auto-declaration**: `{field}` in table cells auto-declares a String field on the row struct. `{field}` in button labels does NOT auto-declare — it must be declared elsewhere.
5. **One default page**: Exactly one page in `define_litui_app!` must have `default: true`.

## Parent Config (_app.md)

Shared styles and theme, no page directive:

```yaml
---
styles:
  selected:
    color: "#4488FF"
    bold: true

nav:
  position: none    # no navigation bar

theme:
  dark:
    panel_fill: "#1E1E2E"
---
```

`nav: position: none` suppresses the navigation bar (important when all pages are panels/windows).
