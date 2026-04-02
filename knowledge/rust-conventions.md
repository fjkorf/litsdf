# Rust Conventions in litsdf

## Edition and Toolchain

- Rust edition 2024
- Minimum Rust 1.92

## Project Structure

Cargo workspace with four crates and three binaries:

```
crates/
  litsdf_core/    — Data model, SDF math, persistence (no Bevy)
  litsdf_render/  — Bevy rendering plugin (material, camera, picking)
  litsdf_editor/  — Editor UI plugin (egui/litui panels, undo)
  litsdf_cli/     — CLI tool for scene file manipulation (no Bevy)
```

Binaries: `litsdf` (editor), `litsdf-viewer` (viewport only), `litsdf-cli` (command line).

All logic lives in the library crates. `main.rs` entry points wire plugins and env var handling. The CLI crate depends only on `litsdf_core` + `clap` — no GPU required.

## Module Organization

Modules are split when they exceed ~300 lines or have distinct responsibilities. The `ui/` directory is a module directory with `mod.rs`:

```
litsdf_editor/src/ui/
  mod.rs          — public types, editor_ui system, menu bar, status bar, shortcut dispatch, tests
  shortcuts.rs    — keyboard shortcut constant definitions (egui KeyboardShortcut)
  tree.rs         — pure egui bone tree with visibility toggles and context menus
  populate.rs     — litui state population from scene model
  handlers.rs     — button click handlers
  sync.rs         — UI → model property sync
  helpers.rs      — primitive/combo conversion utilities, PRIM_NAMES constant
```

The node editor lives in a separate module directory:

```
litsdf_editor/src/nodes/
  mod.rs          — re-exports
  types.rs        — SdfNode enum (13 variants), PinType, pin counts/labels
  viewer.rs       — SnarlViewer<SdfNode> implementation (pin rendering, connection validation, context menus)
  eval.rs         — graph evaluation (topological sort, per-node compute, ShapeOutputValues/BoneOutputValues)
  presets.rs      — starter graph templates
```

The render crate includes a shader codegen module:

```
litsdf_render/src/
  codegen.rs        — generate_shader(), topology_hash(), PREAMBLE/POSTAMBLE constants
  shader.rs         — SdfMaterial, ShaderShape, SdfShaderParams structs
  scene_sync.rs     — SdfSceneState, build_shader_params, codegen trigger on topology change
```

The CLI crate uses a `commands/` module directory:

```
litsdf_cli/src/commands/
  mod.rs          — shared load/save/mutate helpers
  scene.rs        — scene new, rename, light, info, tree
  bone.rs         — bone add, remove, rename, move, rotate, duplicate, reparent, list
  shape.rs        — shape add, remove, set, set-type, duplicate, reparent, color-mode, noise, list
  modifier.rs     — modifier set, clear, list
```

Functions are `pub(crate)` when shared within the crate but not part of the public API.

## Newtype ID Pattern

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ShapeId(pub Uuid);
```

`#[serde(transparent)]` means it serializes as a plain UUID string in YAML. The newtype prevents mixing up `ShapeId` and `BoneId` at compile time.

`BoneId` has additional methods:

```rust
impl BoneId {
    pub fn root() -> Self { Self(Uuid::nil()) }
    pub fn is_root(&self) -> bool { self.0.is_nil() }
}
```

## Enum Pattern for Primitives

```rust
pub enum SdfPrimitive {
    Sphere { radius: f32 },
    Box { half_extents: [f32; 3] },
    ...
}
```

Each variant carries its own parameters. Helper methods provide uniform access:
- `label() -> &'static str` — display name
- `default_for(name: &str) -> Self` — factory from string name

## Error Handling

- `persistence` functions return `Result<T, String>` — simple error messages for UI display
- Bevy systems use `Res<T>` / `ResMut<T>` which can't fail
- `find_bone`/`find_shape` return `Option` — callers handle None with early returns

## Serde for YAML (Compact Serialization)

All model types derive `Serialize, Deserialize` with `skip_serializing_if` for compaction:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SdfShape {
    pub id: ShapeId,
    pub name: String,
    pub primitive: SdfPrimitive,
    #[serde(default, skip_serializing_if = "ShapeTransform::is_default")]
    pub transform: ShapeTransform,
    #[serde(default, skip_serializing_if = "ShapeMaterial::is_default")]
    pub material: ShapeMaterial,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<ShapeModifier>,
    #[serde(default, skip_serializing_if = "CombinationOp::is_default")]
    pub combination: CombinationOp,
}
```

**Key pattern**: `#[serde(default)]` on deserialization fills missing fields with defaults. `skip_serializing_if` omits fields matching their default on serialization. Both are needed together for round-trip stability.

**Per-field helpers** in models.rs: `is_zero()`, `is_one()`, `is_half()`, `is_zero_array()`, `is_white()` — each checks a specific default value. Matching providers: `one()`, `half()`, `white()` — used by `#[serde(default = "one")]`.

**Per-field vs per-struct**: Fields within structs are skipped individually. A ShapeTransform with only translation set serializes as just `translation: [1.5, 0.0, 0.0]` without rotation or scale. If ALL fields are default, the whole struct is omitted from its parent.

Enums serialize as YAML tagged values:

```yaml
primitive: !Sphere
  radius: 1.0
combination: !SmoothUnion
  k: 0.5
```

Simple variants serialize as plain strings: `combination: Union`

## Testing Conventions

- Tests live in `#[cfg(test)] mod tests` within each module
- Helper functions (`make_test_scene`, `make_state`) build test fixtures
- Tests simulate multi-frame sequences by calling populate/sync functions directly
- Screenshot tests use env vars and shell scripts, not Rust test harness (requires GPU)

## Common Patterns

### Dirty flag for deferred work

```rust
scene.dirty = true;  // set anywhere
// Later, in sync system:
if !state.dirty { return; }
state.dirty = false;
// do expensive work
```

### Selection change detection

```rust
let changed = scene.selected_shape != ui.prev_selected_shape;
ui.prev_selected_shape = scene.selected_shape;
if !changed { return; }
// load new selection data
```

### Recursive tree traversal

```rust
fn walk(bone: &SdfBone) {
    for shape in &bone.shapes { ... }
    for child in &bone.children {
        walk(child);
    }
}
```

Used in: `find_bone`, `find_shape`, `find_bone_by_name`, `remove_bone`, `remove_shape`, `extract_bone`, `extract_shape`, `reparent_bone`, `reparent_shape`, `duplicate_deep`, `all_shapes`, `bone_count`, `shape_count`, `flatten_bone_tree`, `compute_bone_world_transforms`, `render_bone_tree`.
