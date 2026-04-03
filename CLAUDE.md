# litsdf — Claude Code Instructions

## Workspace Structure

litsdf is a Cargo workspace with four crates:

- **`litsdf_core`** — Data model, SDF math, scene computation, persistence. **No Bevy dependency.** Uses glam 0.30 directly (same version Bevy uses, so types are compatible).
- **`litsdf_render`** — Bevy rendering plugin. Material, camera, gizmos, picking, scene sync. Depends on core + Bevy.
- **`litsdf_editor`** — Editor UI plugin. litui panels, undo, testing, node editor. Depends on core + render + litui + egui-snarl.
- **`litsdf_cli`** — Command-line tool for manipulating scene YAML files. Depends on core + clap. **No Bevy/GPU dependency.**

Three binaries: `litsdf` (editor), `litsdf-viewer` (3D viewport only), `litsdf-cli` (CLI scene manipulation).

## API Reference

After any code change, run:

```sh
python3 scripts/generate-api-docs.py
```

The generated `knowledge/api/API.md` covers all four crates with full type definitions, function signatures, and cross-crate dependency stratification.

## Knowledge Base

- `knowledge/sdf-rendering.md` — SDF math, ray marching, primitives, shader layout
- `knowledge/bevy-patterns.md` — Bevy 0.18 patterns, Material, schedules, API changes
- `knowledge/bone-hierarchy.md` — Bone tree, transform propagation, selection model
- `knowledge/ui-architecture.md` — Hybrid egui/litui, panel rendering, action pattern
- `knowledge/litui-integration.md` — Markdown syntax, widget types, generated types
- `knowledge/picking-and-compass.md` — Viewport picking, drag handles, compass gizmo
- `knowledge/shape-modifiers.md` — Modifier types, encoding, application order
- `knowledge/rust-conventions.md` — Edition, modules, serde compaction, patterns
- `knowledge/testing.md` — 66 tests across workspace, screenshot tests
- `knowledge/ui-phase-plan.md` — Remaining UI tasks for future work
- `knowledge/glossary.md` — Action glossary (76 actions, gap analysis)
- `knowledge/sdf-math-glossary.md` — SDF math concepts and formulas
- `knowledge/ui-conventions.md` — Professional editor UI conventions research
- `knowledge/cli-design.md` — CLI crate design and command structure
- `knowledge/node-editor-libraries.md` — egui node editor library survey (egui-snarl recommended)
- `knowledge/node-editor-conventions.md` — Industry node editor patterns (Blender, Houdini, Unity, Substance)
- `knowledge/node-property-mapping.md` — Current litsdf features mapped to node concepts
- `knowledge/node-architecture.md` — Node editor architecture design document
- `knowledge/shader-codegen.md` — Per-pixel shader code generation research
- `knowledge/pbr-lighting.md` — PBR (Cook-Torrance) lighting upgrade research
- `knowledge/demo-scenes.md` — 6 demo scenes, feature coverage, loading instructions

## Running

```sh
cargo run --bin litsdf                       # editor
cargo run --bin litsdf-viewer -- scene.yaml  # viewer
cargo run -p litsdf-cli -- scene info s.yaml # CLI
cargo test --workspace                       # 68 tests
LITSDF_SCREENSHOT=path.png cargo run --bin litsdf  # screenshot
```

## Critical Rules

- Material bindings use `@group(3)` in WGSL, NOT `@group(2)`
- Editor UI systems run in `EguiPrimaryContextPass`, NOT `Update` (including undo/redo — handled inline via egui shortcuts)
- Use `#[uniform(0)]` with inline data for SdfMaterial, NOT storage buffers
- Skip property sync on the frame selection changes
- When drag handles are active, suppress orbit camera and property sync
- All model fields need `#[serde(default, skip_serializing_if)]` for compact YAML
- `litsdf_core` must NOT depend on Bevy — use `glam` directly
- `glam` version must match Bevy's internal glam (0.30) to avoid type mismatches
- `ShaderShape` in Rust (`shader.rs`) and WGSL (`sdf_raymarch.wgsl`) must be byte-identical — add/remove fields in BOTH or rendering breaks silently
- `SdfShaderParams` settings fields MUST go AFTER the shapes array, not before — inserting before shapes shifts the array offset and Metal silently fails to render (no error, just blank viewport)
- `assets/shaders/` is a symlink to `crates/litsdf_render/assets/shaders/` — edit only the crate copy, never the symlink target directly
- Animation is done via node graphs (egui-snarl), NOT via model fields — `anim_*` fields were removed; old YAML files with `anim_*` will fail to load with a migration message
- `litsdf_core` must NOT depend on egui-snarl — node graphs live in `litsdf_editor` as `HashMap<ShapeId/BoneId, Snarl<SdfNode>>`
- `compute_bone_world_transforms` takes `&HashMap<BoneId, ShapeTransform>` for overrides, not `time: f32` — pass empty HashMap when no overrides needed
- WGSL `vec3(scalar)` shorthand is NOT supported by Bevy's naga — always use `vec3<f32>(x, x, x)` with explicit components
- Shader codegen writes to `assets/shaders/sdf_raymarch.wgsl` on topology change — Bevy hot-reloads automatically. The preamble/postamble sources are in `crates/litsdf_render/assets/shaders/`
- Lighting uses Cook-Torrance PBR (GGX + Fresnel-Schlick + Smith geometry), NOT Blinn-Phong
