# Demo Scenes

Six built-in demo scenes showcase different litsdf features. Accessible via **File > Demo Scenes** menu or the `LITSDF_DEMO` environment variable.

## Loading Demos

**In the editor:** File > Demo Scenes > (pick one)

**From command line:**
```sh
LITSDF_DEMO=mushroom cargo run --bin litsdf
LITSDF_DEMO=robot LITSDF_SCREENSHOT=robot.png cargo run --bin litsdf
```

Valid names: `gallery`, `boolean`, `modifier`, `mushroom`, `robot`, `sculpture`

## Scene Descriptions

### 1. Primitive Gallery
All 13 SDF primitives arranged in an arc on the root bone. Each has a distinct color. No modifiers or smooth operations — pure primitive showcase.

**Demonstrates:** Sphere, Box, RoundBox, Cylinder, CappedCone, Torus, Capsule, Plane, Ellipsoid, Octahedron, Pyramid, HexPrism, RoundCone

### 2. Boolean Sampler
Four groups showing different combination operations:
- **Subtracted:** Sphere with Box carved out
- **Smooth Intersection:** Sphere + Box with SmoothIntersection
- **Chamfer Union:** RoundBox + Sphere with beveled join
- **Bowl:** Ellipsoid with smaller Ellipsoid smooth-subtracted

**Demonstrates:** Subtraction, SmoothIntersection, ChamferUnion, SmoothSubtraction, metallic materials

### 3. Modifier Parade
Seven shapes in a row, each with one domain modifier applied:
Plain, Rounded, Hollow (Onion), Twisted, Bent, Elongated, plus a Repeated sphere grid

**Demonstrates:** All 6 modifiers (Rounding, Onion, Twist, Bend, Elongation, Repetition), HexPrism primitive

### 4. Mushroom Garden
Organic nature scene with two mushrooms on a noisy ground:
- Ground uses Gradient Snow (color_mode 5) with noise displacement
- Mushroom 1 cap uses Cosine Palette (color_mode 1)
- Mushroom 2 cap uses Cellular/Voronoi (color_mode 3)
- Stone uses Ridged Multifractal (color_mode 4)
- Moss uses smooth_symmetry

**Demonstrates:** 4 color modes, noise displacement, SmoothUnion, smooth_symmetry, bone hierarchy, varied roughness

### 5. Robot Friend
Mechanical character with animated arms:
- Body and head joined with ChamferUnion (mechanical look)
- Eyes with high fresnel_power (glowing rim)
- Head gem is metallic Octahedron
- Arms on child bones with bob animation presets (phase-offset)

**Demonstrates:** ChamferUnion, metallic materials, fresnel, Octahedron, node graph bone animation, multi-level bone hierarchy

### 6. Abstract Sculpture
Artistic piece with animated rotation and color cycling:
- Torus smooth-intersected with Octahedron
- Twisted Pyramid smooth-unioned
- Bone spin animation on the form
- Color cycle node preset on the ring
- Custom SceneSettings (dramatic lighting, strong vignette)

**Demonstrates:** SmoothIntersection, Twist modifier, color_cycle node preset, bone_spin_preset, custom SceneSettings

## Feature Coverage

Between the 6 demos, every major feature is demonstrated:
- All 13 primitives
- All 8 combination operations
- All 6 modifiers
- All 6 color modes
- Metallic and fresnel materials
- smooth_symmetry
- Node graph animation (shape + bone)
- Custom SceneSettings
- Multi-level bone hierarchy

## Implementation

Demo scenes live in `litsdf_editor::demos`. Each is a factory function returning `DemoResult { scene, shape_graphs, bone_graphs }`. The `DemoScene` enum dispatches to the correct factory. The File menu iterates `DemoScene::all()` to build the submenu.

The default scene (shown on app launch) is a single gray sphere — demos are discovered via the menu.
