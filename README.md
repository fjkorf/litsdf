# litsdf

A real-time SDF (Signed Distance Function) shape editor built with Rust, Bevy, and egui.

![litsdf editor — Mushroom Garden demo](tests/screenshots/demo_mushroom.png)

Create complex 3D objects by combining SDF primitives attached to a bone hierarchy. Shapes are ray-marched in a WGSL fragment shader with PBR lighting and rendered in real time. Animate with a visual node graph editor.

## Demo Scenes

Six built-in demo scenes accessible via **File > Demo Scenes**:

| Demo | Features Showcased |
|------|--------------------|
| **Primitive Gallery** | All 13 SDF primitives arranged in an arc |
| **Boolean Sampler** | Subtraction, SmoothIntersection, ChamferUnion, SmoothSubtraction |
| **Modifier Parade** | Rounding, Onion shell, Twist, Bend, Elongation, Repetition |
| **Mushroom Garden** | Cosine Palette, Cellular, Ridged, Gradient Snow color modes, noise displacement |
| **Robot Friend** | Metallic materials, Fresnel, ChamferUnion, node-driven bone animation |
| **Abstract Sculpture** | SmoothIntersection, Twist modifier, color cycling animation, custom scene settings |

## Features

### SDF Rendering
- **13 SDF primitives** — Sphere, Box, RoundBox, Cylinder, CappedCone, Torus, Capsule, Plane, Ellipsoid, Octahedron, Pyramid, HexPrism, RoundCone
- **8 combination operations** — Union, Intersection, Subtraction, Smooth variants, Chamfer Union/Intersection
- **6 domain modifiers** — Rounding, Shell, Twist, Bend, Elongation, Repetition
- **6 color modes** — Solid, Cosine Palette, Noise Tint, Cellular/Voronoi, Ridged Multifractal, Gradient Snow
- **PBR lighting** — Cook-Torrance BRDF, gradient sky environment with sun reflection spot
- **Shader codegen** — scene topology compiles to unrolled WGSL, hot-reloaded by Bevy

### Node Editor
- **24 node types** — oscillators (sin, square, triangle, sawtooth), math (add, multiply, mix, clamp, abs, modulo, ease, remap), animation (exponential impulse, smoothstep, noise 1D), cosine palette, vec3 ops
- **27-pin ShapeOutput** — drives position, rotation, scale, color, material, noise, and all modifier params
- **7-pin BoneOutput** — drives bone transforms
- **Graph persistence** — ProjectFile saves scene + node graphs in single YAML
- **7 presets** — bob, spin, pulse, orbit, color cycle + bone variants
- **Color-coded nodes** by category

### Editor
- **Menu bar** — File, Edit, Add, View with keyboard shortcut hints
- **Keyboard shortcuts** — Cmd+S/O/N/Z/D/C/V, Delete, Escape, F, H, Alt+H, G/R/S/E/P, 1/3/7/5
- **Bone tree** — visibility toggles, context menus, drag-to-reparent, inline rename
- **Properties panel** — scene settings, rendering params, shape/bone editing
- **Viewport** — orbit, pan, zoom, click-to-select, compass gizmo
- **Gizmo modes** — G (translate), R (rotate), S (scale), E (elongation), P (repetition) with bone-local RGB axis handles
- **Camera views** — 1 (front), 3 (right), 7 (top), 5 (toggle perspective/orthographic)
- **Copy/paste** — Cmd+C/V for shapes
- **Render sequence** — export numbered PNGs for video assembly

### CLI
- **25 subcommands** across scene, bone, shape, and modifier operations
- Pure data manipulation, no GPU required

## Running

```sh
cargo run --bin litsdf                                      # editor
cargo run --bin litsdf-viewer -- scene.yaml                 # viewer
cargo run -p litsdf-cli -- scene info scene.yaml            # CLI
cargo test --workspace                                      # 66 tests
LITSDF_SCREENSHOT=out.png cargo run --bin litsdf            # screenshot
LITSDF_RENDER_SEQUENCE=frames,60,30 cargo run --bin litsdf  # render 60 frames
```

## Architecture

4-crate Cargo workspace:

| Crate | Purpose | Dependencies |
|-------|---------|-------------|
| `litsdf_core` | Data model, SDF math, persistence | glam, serde |
| `litsdf_render` | Bevy rendering, shader codegen | core, Bevy |
| `litsdf_editor` | UI, node editor, project files | core, render, egui-snarl, litui |
| `litsdf_cli` | Command-line scene manipulation | core, clap |

See `knowledge/` for 20 detailed documentation files and `knowledge/api/API.md` for auto-generated API reference (179 items).

## Dependencies

- [Bevy](https://bevyengine.org/) 0.18
- [egui](https://github.com/emilk/egui) 0.33 + [bevy_egui](https://github.com/vladbat00/bevy_egui) 0.39
- [egui-snarl](https://github.com/zakarumych/egui-snarl) 0.9
- [litui](https://github.com/fjkorf/litui)
- [clap](https://github.com/clap-rs/clap) 4
