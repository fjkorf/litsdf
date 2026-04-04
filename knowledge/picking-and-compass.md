# Viewport Picking and Compass Gizmo

## Viewport Picking (src/picking.rs)

### How It Works

When the user clicks in the 3D viewport, the picking system:

1. **Converts screen position to world ray** using Bevy's `Camera::viewport_to_world(transform, cursor_pos)` which returns a `Ray3d` (origin + direction)
2. **Ray marches on the CPU** using the same SDF formulas as the GPU shader, ported to Rust
3. **On hit**, evaluates each shape individually at the hit point to find which one is closest (by smallest absolute SDF distance)
4. **Sets selection** — `selected_shape` and `selected_bone` on `SdfSceneState`

### Click vs Drag

The orbit camera also uses left mouse button for dragging. To distinguish:
- On `just_pressed`: record cursor position in `ClickTracker` resource
- On `just_released`: compare current position to press position
- If movement < 3px: this was a click → do picking
- If movement >= 3px: this was a drag → orbit camera already handled it

### CPU SDF Primitives

All 13 primitives are implemented in `litsdf_core::sdf` (Sphere, Box, RoundBox, Cylinder, CappedCone, Torus, Capsule, Plane, Ellipsoid, Octahedron, Pyramid, HexPrism, RoundCone). The picking module delegates to `core::sdf::eval_primitive()` rather than duplicating the match — a single source of truth for CPU SDF evaluation.

Rotation uses the same Euler angle function as the shader (`rotate_point`).

### World Shape Resolution

The picking system reuses `compute_bone_world_transforms` from `scene.rs` to get bone world matrices, then computes each shape's world-space transform (bone_world * shape_local) and evaluates the SDF at the ray march hit point.

### Per-Shape vs Combined Scene

The shader combines shapes via union/intersection/subtraction. For picking, we need to know WHICH individual shape was hit. The approach:
- Ray march uses `min()` of all shapes (simple union) to find the surface
- At the hit point, evaluate each shape independently and pick the one with smallest `abs(distance)`

This means picking always works regardless of combination ops.

### Public API

`get_selected_world_pos(scene: &SdfSceneState) -> Option<Vec3>` — resolves the world position of the currently selected shape (bone_world * shape_local). Used by the editor for frame selection (F key).

### System Registration

```rust
.init_resource::<picking::ClickTracker>()
.add_systems(Update, picking::pick_system)
```

Runs in `Update` (not `EguiPrimaryContextPass`) because it reads mouse input and camera transforms.

---

## Camera Controls (src/camera.rs)

The `OrbitCamera` component supports:
- **Left drag**: Orbit (yaw/pitch rotation around target)
- **Middle drag**: Pan (shifts target position, speed scales with distance)
- **Scroll**: Zoom (distance clamped to 0.5–50.0)
- **Frame selection**: `frame_target: Option<Vec3>` — one-shot, set by editor (F key), consumed by camera system to snap target and set distance to 3.0

Orbit and pan are suppressed when egui wants pointer input or when a drag handle is active.

---

## Drag Handle Modes (src/picking.rs)

The `GizmoMode` resource controls what property the drag handles edit:

| Mode | Key | Visual | Edits | Works On |
|------|-----|--------|-------|----------|
| **Translate** | G | Arrow lines + sphere tips | transform.translation | Shapes + Bones |
| **Rotate** | R | Circular arcs per axis | transform.rotation | Shapes + Bones |
| **Elongation** | E | Double-headed lines | ShapeModifier::Elongation | Shapes only |
| **Repetition** | P | Lines + cross tips | ShapeModifier::Repetition.period | Shapes only |

All modes use RGB for X/Y/Z axes. The drag projects screen delta onto the constrained world axis, scaled by camera distance. Mode keys are guarded by `!ctx.wants_keyboard_input()` (don't fire during text editing).

When a **bone** is selected (no shape), translate and rotate handles appear at the bone's world position. Elongation and Repetition modes are ignored for bones (bones don't have modifiers).

The status bar shows the current gizmo mode name.

---

## Compass Gizmo (src/gizmos.rs)

### How It Works

A small orientation indicator in the bottom-left corner showing camera-relative XYZ axes.

1. Gets camera's `Transform.rotation` and computes its inverse (the view rotation)
2. For each world axis (X, Y, Z): multiplies by view rotation to get camera-space direction
3. Projects to 2D by taking (x, -y) — flips Y because screen Y is down
4. Draws colored arrows using `egui::Painter::arrow()` with labels

### Depth Sorting

Axes are sorted by their Z depth (in camera space) so that closer axes draw on top of further ones. This prevents visual confusion when axes overlap.

### egui Rendering

Uses `egui::Area::new("compass").fixed_pos(...).interactable(false)`:
- `fixed_pos` — anchored to screen position regardless of scrolling
- `interactable(false)` — doesn't capture mouse clicks (clicking through to the viewport picking)
- `allocate_space` — reserves a fixed rect for drawing
- `painter.arrow()` — draws shaft + arrowhead
- `painter.text()` — axis labels at arrow tips
- `painter.circle_filled()` — white center dot

### System Registration

```rust
.add_systems(bevy_egui::EguiPrimaryContextPass, (ui::editor_ui, gizmos::draw_compass))
```

Runs in `EguiPrimaryContextPass` because it needs the egui `Context` for drawing.
