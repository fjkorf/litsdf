# Action Glossary

Every operation the user can perform in litsdf, organized by semantic group. Each action has a name, parameters, and a CLI equivalent via the `litsdf-cli` crate.

---

## Scene Management

| Action | Parameters | CLI Equivalent | Status |
|--------|-----------|----------------|--------|
| **Create Scene** | name | `scene new "Name"` | Implemented (Cmd+N, CLI) |
| **Rename Scene** | name | `scene rename "Name"` | Implemented (textedit) |
| **Save Scene** | filename | `scene save output.yaml` | Implemented |
| **Load Scene** | filename | `scene load input.yaml` | Implemented |
| **List Scenes** | directory | `scene list` | Implemented (file browser) |
| **Set Light Direction** | x, y, z | `scene light 0.6 0.8 0.4` | Implemented (sliders) |

---

## Bone Operations

| Action | Parameters | CLI Equivalent | Status |
|--------|-----------|----------------|--------|
| **Create Bone** | parent_bone, name | `bone add --parent root --name "Arm"` | Implemented |
| **Select Bone** | bone_id | (editor only) | Implemented (tree click) |
| **Delete Bone** | bone_id | `bone remove --name "Arm"` | Implemented (reparents children+shapes) |
| **Delete Bone + Contents** | bone_id | `bone remove --name "Arm" --recursive` | CLI implemented, editor missing |
| **Rename Bone** | bone_id, name | `bone rename --name "Arm" --to "LeftArm"` | Implemented (textedit) |
| **Move Bone** | bone_id, x, y, z | `bone move --name "Arm" --translate 1.2,0,0` | Implemented (sliders) |
| **Rotate Bone** | bone_id, rx, ry, rz | `bone rotate --name "Arm" --rotation 0,45,0` | Implemented (sliders) |
| **Animate Bone Bob** | bone_id, amp, freq | `bone animate --name "Arm" --bob 0.3,0.5` | Implemented (sliders) |
| **Animate Bone Sway** | bone_id, amp, freq | `bone animate --name "Arm" --sway 5.0,0.2` | Implemented (sliders) |
| **Duplicate Bone** | bone_id | `bone duplicate --name "Arm"` | Implemented (Cmd+D, context menu, CLI) |
| **Reparent Bone** | bone_id, new_parent | `bone reparent --name "Arm" --parent "Body"` | CLI implemented, editor missing |
| **Reset Bone Transform** | bone_id | `bone reset --name "Arm"` | Core helper implemented, no UI/CLI |

---

## Shape Operations

| Action | Parameters | CLI Equivalent | Status |
|--------|-----------|----------------|--------|
| **Create Shape** | bone, primitive_type | `shape add --bone "Arm" --type Sphere` | Implemented |
| **Select Shape** | shape_id | (editor only) | Implemented (tree + viewport click) |
| **Delete Shape** | shape_id | `shape remove --id abc123` | Implemented |
| **Change Primitive** | shape_id, type | `shape set-type --id abc123 --type Box` | Implemented (combobox) |
| **Edit Shape YAML** | shape_id, yaml_text | `shape edit --id abc123 < shape.yaml` | Implemented (modal) |
| **Export Shape YAML** | shape_id | `shape export --id abc123` | Partially — YAML editor shows it, no standalone export |
| **Duplicate Shape** | shape_id | `shape duplicate --name "MySphere"` | Implemented (Cmd+D, context menu, CLI) |
| **Copy Shape** | shape_id | `shape copy --id abc123` | Missing (clipboard) |
| **Paste Shape** | bone_id | `shape paste --bone "Arm"` | Missing (clipboard) |
| **Move Shape to Bone** | shape_id, bone_id | `shape reparent --name "MySphere" --bone "Body"` | CLI implemented, editor missing |
| **Reorder Shapes** | shape_id, position | `shape reorder --id abc123 --position 0` | Missing (affects combine order) |
| **Reset Shape Transform** | shape_id | `shape reset-transform --id abc123` | Core helper implemented, no UI/CLI |
| **Lock/Hide Shape** | shape_id, bool | `shape hide --id abc123` | Partial (visibility toggle in tree, no lock) |

---

## Shape Geometry

| Action | Parameters | CLI Equivalent | Status |
|--------|-----------|----------------|--------|
| **Set Primitive Param A** | shape_id, value | `shape set --id abc123 --param-a 1.0` | Implemented |
| **Set Primitive Param B** | shape_id, value | `shape set --id abc123 --param-b 0.5` | Implemented |
| **Set Primitive Param C** | shape_id, value | `shape set --id abc123 --param-c 0.5` | Implemented |
| **Set Primitive Param D** | shape_id, value | `shape set --id abc123 --param-d 0.1` | Implemented |
| **Set Translate** | shape_id, x, y, z | `shape set --id abc123 --translate 1,0,0` | Implemented |
| **Set Rotation** | shape_id, rx, ry, rz | `shape set --id abc123 --rotate 0,45,0` | Implemented |
| **Set Scale** | shape_id, scale | `shape set --id abc123 --scale 1.5` | Implemented |
| **Set Combination Op** | shape_id, op | `shape set --id abc123 --combine SmoothUnion` | Implemented |
| **Set Blend Radius** | shape_id, k | `shape set --id abc123 --blend-k 0.3` | Implemented |
| **Set Smooth Symmetry** | shape_id, value | `shape set --id abc123 --symmetry 0.01` | Implemented |

---

## Shape Material

| Action | Parameters | CLI Equivalent | Status |
|--------|-----------|----------------|--------|
| **Set Color** | shape_id, r, g, b | `shape set --id abc123 --color 0.8,0.2,0.2` | Implemented |
| **Set Roughness** | shape_id, value | `shape set --id abc123 --roughness 0.5` | Implemented |
| **Set Metallic** | shape_id, value | `shape set --id abc123 --metallic 0.8` | Implemented |
| **Set Rim Glow** | shape_id, value | `shape set --id abc123 --fresnel 2.5` | Implemented |
| **Set Color Mode** | shape_id, mode | `shape set --id abc123 --color-mode palette` | Implemented |
| **Set Palette A/B/C/D** | shape_id, r,g,b per | `shape set --id abc123 --palette-a 0.5,0.5,0.5` | Implemented |

---

## Shape Noise

| Action | Parameters | CLI Equivalent | Status |
|--------|-----------|----------------|--------|
| **Set Noise Amplitude** | shape_id, value | `shape set --id abc123 --noise-amp 0.05` | Implemented |
| **Set Noise Frequency** | shape_id, value | `shape set --id abc123 --noise-freq 4.0` | Implemented |
| **Set Noise Octaves** | shape_id, value | `shape set --id abc123 --noise-oct 3` | Implemented |

---

## Shape Modifiers

| Action | Parameters | CLI Equivalent | Status |
|--------|-----------|----------------|--------|
| **Set Rounding** | shape_id, radius | `modifier set --id abc123 --rounding 0.1` | Implemented |
| **Set Onion Shell** | shape_id, thickness | `modifier set --id abc123 --onion 0.05` | Implemented |
| **Set Twist** | shape_id, amount | `modifier set --id abc123 --twist 2.0` | Implemented |
| **Set Bend** | shape_id, amount | `modifier set --id abc123 --bend 1.5` | Implemented |
| **Set Elongation** | shape_id, x, y, z | `modifier set --id abc123 --elongate 1,0.5,0` | Implemented |
| **Set Repetition** | shape_id, px, py, pz | `modifier set --id abc123 --repeat 2,2,2` | Implemented |
| **Clear All Modifiers** | shape_id | `modifier clear --shape "MySphere"` | CLI implemented, editor missing |

---

## Shape Animation

| Action | Parameters | CLI Equivalent | Status |
|--------|-----------|----------------|--------|
| **Animate Move X** | shape_id, amp, freq | `shape animate --id abc123 --move-x 0.5,1.0` | Implemented |
| **Animate Move Y** | shape_id, amp, freq | `shape animate --id abc123 --move-y 0.3,0.5` | Implemented |
| **Animate Spin Y** | shape_id, amp, freq | `shape animate --id abc123 --spin-y 45,0.3` | Implemented |
| **Animate Pulse** | shape_id, amp, freq | `shape animate --id abc123 --pulse 0.1,0.5` | Implemented |
| **Clear Animation** | shape_id | `shape animate --name "MySphere" --clear` | CLI implemented, editor missing |
| **Animate Move Z** | shape_id, amp, freq | `shape animate --id abc123 --move-z 0.5,1.0` | Model exists, no UI |
| **Animate Pitch/Roll** | shape_id, amp, freq | `shape animate --id abc123 --spin-x 10,0.2` | Model exists, no UI |

---

## Camera (editor/viewer only)

| Action | Parameters | CLI Equivalent | Status |
|--------|-----------|----------------|--------|
| **Orbit** | yaw_delta, pitch_delta | (mouse drag) | Implemented |
| **Zoom** | distance_delta | (scroll wheel) | Implemented |
| **Pan** | x_delta, y_delta | (middle mouse drag) | Implemented |
| **Frame Selection** | — | (F key) | Implemented |
| **Reset Camera** | — | (View menu) | Implemented |

---

## Viewport Interaction (editor only)

| Action | Parameters | CLI Equivalent | Status |
|--------|-----------|----------------|--------|
| **Pick Shape** | screen_position | (left click) | Implemented |
| **Drag Handle** | axis, delta | (left drag on handle) | Implemented |
| **Deselect** | — | (click empty space) | Implemented |

---

## Visual Aids (editor only)

| Action | Parameters | CLI Equivalent | Status |
|--------|-----------|----------------|--------|
| **Toggle Bone Gizmos** | bool | (checkbox) | Implemented |
| **Show Translation Handles** | — | (auto when shape selected) | Implemented |
| **Show Compass** | — | (always on) | Implemented |

---

## History (editor only)

| Action | Parameters | CLI Equivalent | Status |
|--------|-----------|----------------|--------|
| **Undo** | — | (Cmd+Z) | Implemented |
| **Redo** | — | (Cmd+Shift+Z) | Implemented |

---

## Rendering (viewer/CLI)

| Action | Parameters | CLI Equivalent | Status |
|--------|-----------|----------------|--------|
| **Render to Image** | scene, output_path, resolution | `render --scene input.yaml --output render.png --size 1920x1080` | Partial (LITSDF_SCREENSHOT env var) |
| **Render Sequence** | scene, output_dir, frames | `render --scene input.yaml --output-dir frames/ --frames 60` | Missing |

---

## Gap Priority

### Fully Implemented (editor + CLI + core)
- **Create Scene** — Cmd+N, `litsdf-cli scene new`
- **Duplicate Shape** — Cmd+D, context menu, `litsdf-cli shape duplicate`
- **Duplicate Bone** — Cmd+D, context menu, `litsdf-cli bone duplicate`
- **Pan Camera** — middle-mouse drag
- **Frame Selection** — F key
- **Delete key** — Del/Backspace
- **Escape to deselect**
- **Cmd+S / Cmd+O / Cmd+N** — file shortcuts
- **Menu bar** — File, Edit, Add, View
- **Visibility toggles** — eye icon per bone/shape in tree
- **Right-click context menus** — add bone, add shape, duplicate, delete
- **Status bar** — selection info + scene stats

### Implemented in CLI/Core (need editor UI)
- **Reset Transform** — `reset_transform()` on SdfShape and SdfBone (button in properties panel)
- **Clear All Modifiers** — `clear_modifiers()` + `litsdf-cli modifier clear` (button in properties panel)
- **Move Shape to Bone** — `reparent_shape()` + context menu + `litsdf-cli shape reparent`
- **Reparent Bone** — `reparent_bone()` + context menu + `litsdf-cli bone reparent`
- **Delete Bone + Contents** — `extract_bone()` + context menu + `litsdf-cli bone remove --recursive`

### Node Editor (24 node types)
- **Shape node graphs** — ShapeOutput with 27 pins (transform, material, noise, modifiers)
- **Bone node graphs** — BoneOutput with 7 pins (transform)
- **24 node types**: Time, SinOscillator, SquareWave, TriangleWave, SawtoothWave, Constant, ConstantVec3, Add, Multiply, Mix, Clamp, Negate, Abs, Modulo, EaseInOut, Remap, Vec3Compose, Vec3Decompose, CosinePalette, ExpImpulse, SmoothStep, Noise1D, ShapeOutput, BoneOutput
- **Graph persistence** — ProjectFile saves graphs alongside scene YAML
- **7 presets** — bob, spin, pulse, orbit, color_cycle + bone variants
- **Node color-coding** — green (generators), teal (oscillators), blue (math), amber (vec3/color), red (output)

### Shader Improvements (Phase D+F complete)
- **PBR Lighting** — Cook-Torrance BRDF with GGX, Fresnel-Schlick, Smith geometry
- **Shader codegen** — unrolled shape evaluation, topology-driven recompilation via Bevy hot-reload
- **SceneSettings** — tunable fill/back/SSS/AO/shadow/vignette parameters with UI sliders
- **Cellular noise** — Voronoi pattern as color_mode 3
- **Ridged noise** — `abs(noise)` ridge patterns as color_mode 4
- **Gradient Snow** — normal-based snow deposition as color_mode 5
- **Chamfer ops** — ChamferUnion and ChamferIntersection (beveled boolean edges)
- **8 combination operations**, 6 color modes, 13 SDF primitives

### Future (nice-to-have)
- Copy/paste clipboard
- Reorder shapes (drag in tree)
- Drag to reparent (in tree)
- Group selection
- Render sequence (animation frames)
- Modifier stack redesign (ordered, enable/disable)

### Not Needed
- Scene metadata (author, description) — can be done in YAML comments
- Multi-scene tabs — complex, questionable value
