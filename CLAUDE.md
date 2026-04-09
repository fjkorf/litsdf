# litsdf — Claude Code Instructions

## Workspace Structure

litsdf is a Cargo workspace with four crates:

- **`litsdf_core`** — Data model, SDF math, scene computation, persistence. **No Bevy dependency.** Uses glam 0.30 directly (same version Bevy uses, so types are compatible).
- **`litsdf_render`** — Bevy rendering plugin. Material, camera, gizmos, picking, scene sync, Avian3D physics. Depends on core + Bevy + avian3d.
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
- `knowledge/testing.md` — 68 tests across workspace, screenshot tests
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
- `knowledge/demo-scenes.md` — 15 demo scenes (6 visual + 5 physics + 2 node↔physics + 2 game logic + 1 IK), feature coverage, loading instructions
- `knowledge/litui-feature-request.md` — litui numeric config features (all 5 implemented)
- `knowledge/pbr-lighting.md` also covers gradient sky environment upgrade

## Running

```sh
cargo run --bin litsdf                       # editor
cargo run --bin litsdf-viewer -- scene.yaml  # viewer
cargo run -p litsdf-cli -- scene info s.yaml # CLI
cargo test --workspace                       # 95 tests
LITSDF_SCREENSHOT=path.png cargo run --bin litsdf  # screenshot
```

## Critical Rules

- Material bindings use `@group(3)` in WGSL, NOT `@group(2)`
- Editor UI systems run in `EguiPrimaryContextPass`, NOT `Update` (including undo/redo — handled inline via egui shortcuts)
- Use `#[uniform(0)]` with inline data for SdfMaterial, NOT storage buffers
- Skip property sync on the frame selection changes
- When drag handles are active, suppress orbit camera and property sync
- Gizmo handles use bone-local axes (not world axes) — `get_local_axes()` transforms X/Y/Z by parent bone's world rotation
- All model fields need `#[serde(default, skip_serializing_if)]` for compact YAML
- `litsdf_core` must NOT depend on Bevy — use `glam` directly
- `glam` version must match Bevy's internal glam (0.30) to avoid type mismatches
- `ShaderShape` in Rust (`shader.rs`) and WGSL (`sdf_raymarch.wgsl`) must be byte-identical — add/remove fields in BOTH or rendering breaks silently
- `SdfShaderParams` settings fields MUST go AFTER the shapes array, not before — inserting before shapes shifts the array offset and Metal silently fails to render (no error, just blank viewport)
- `assets/shaders/` is a RUNTIME directory (gitignored, generated). The SOURCE shader lives at `crates/litsdf_render/assets/shaders/sdf_raymarch.wgsl`. Codegen writes to `assets/shaders/sdf_raymarch.wgsl`. Always edit the CRATE copy.
- Animation is done via node graphs (egui-snarl), NOT via model fields — `anim_*` fields were removed; old YAML files with `anim_*` will fail to load with a migration message
- `litsdf_core` must NOT depend on egui-snarl — node graphs live in `litsdf_editor` as `HashMap<ShapeId/BoneId, Snarl<SdfNode>>`
- `compute_bone_world_transforms` takes `&HashMap<BoneId, ShapeTransform>` for overrides, not `time: f32` — pass empty HashMap when no overrides needed
- WGSL `vec3(scalar)` shorthand is NOT supported by Bevy's naga — always use `vec3<f32>(x, x, x)` with explicit components
- Shader codegen writes to `assets/shaders/sdf_raymarch.wgsl` (runtime copy, gitignored) on topology change — Bevy hot-reloads automatically. Source shader + preamble/postamble live in `crates/litsdf_render/assets/shaders/` (committed). On startup, `ensure_runtime_shader()` copies the fallback loop-based shader to the runtime location.
- Lighting uses Cook-Torrance PBR (GGX + Fresnel-Schlick + Smith geometry), NOT Blinn-Phong
- Shader supports orthographic projection — detects via `view.clip_from_view[3][3]` and switches to parallel rays. Toggle with key 5.
- Physics uses Avian3D 0.5 (Bevy 0.18 compatible) via shadow entity layer in `avian_physics.rs`. Bones with mass>0 are Dynamic RigidBody, mass==0 are Kinematic (animation-driven), root is Static.
- Avian runs in `FixedPostUpdate` — sync systems (kinematic→avian, avian→bones, collect_readings) must also run in `FixedPostUpdate`. Spawn/pause systems run in `Update`.
- `DistanceJoint` (NOT SphericalJoint) connects parent-child bones — SphericalJoint with zero compliance locks dynamic bodies when connected to kinematic parents.
- Dynamic bodies need `SleepingDisabled` to prevent premature sleep. Spawn with `Transform` (not Position/Rotation).
- `BonePhysicsProps` (mass, damping, rotation_limits) lives in `litsdf_core` (no Bevy dep). `ColliderApprox` maps SDF shapes to simple colliders.
- `physics_paused` on `SdfSceneState` gates Avian's `Time<Physics>`. Editor sets it from `!physics_enabled`.
- `BonePhysicsReading` (position, velocity) flows Avian→node graphs. `BoneForceOutputs` flows node graphs→Avian.
- `BoneOutput` has 13 pins (7 transform + 3 force + 3 torque). 4 physics input nodes: BoneVelocity, BoneAngularVelocity, BoneWorldPosition, BoneSpeed.
- Custom physics solver (`physics.rs`) remains as fallback when `use_avian = false`.
- IK solver (`ik.rs`) is pure-data in litsdf_core (no Bevy). FABRIK + analytical 2-bone. Called after node graph eval, before physics.
- `BoneOutput` has 17 pins (7 transform + 3 force + 3 torque + 3 IK target + 1 IK weight). `ik_chain_length` on BonePhysicsProps controls solver: 0=auto, 2=analytical, N=FABRIK.
- Node editor has 37 node types across 7 categories: generators, oscillators, math, logic, sensing, physics I/O, output. Plus Expression node with inline math parser.
- Scene settings live in a modal window (Cmd+, or View > Scene Settings), NOT in the Properties panel.
