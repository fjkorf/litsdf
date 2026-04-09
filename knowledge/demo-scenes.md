# Demo Scenes

15 built-in demo scenes showcase different litsdf features. Accessible via **File > Demo Scenes** menu or the `LITSDF_DEMO` environment variable.

## Loading Demos

**In the editor:** File > Demo Scenes > (pick one)

**From command line:**
```sh
LITSDF_DEMO=mushroom cargo run --bin litsdf
LITSDF_DEMO=robot LITSDF_SCREENSHOT=robot.png cargo run --bin litsdf
LITSDF_DEMO=chain LITSDF_SCREENSHOT_FRAME=60 LITSDF_SCREENSHOT=chain.png cargo run --bin litsdf
```

Valid names: `gallery`, `boolean`, `modifier`, `mushroom`, `robot`, `sculpture`, `chain`, `pendulum`, `damping`, `speed`, `wave`, `walker`, `lemmings`, `ik`

Each demo has a description that appears in a popup when loaded.

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

### 7. Hanging Chain (Physics)
A chain of 5 capsule links extending horizontally from an anchor sphere. Falls and swings under Avian3D gravity when played.

- 5 child bones chained (each parented to previous), starting horizontal
- Each link: mass 0.5, damping 0.92
- SphericalJoints between links
- Auto-plays on load

**Demonstrates:** Bone physics, gravity, damping, joint constraints, Avian3D integration

### 8. Pendulum (Physics)
An animated arm with a physics-driven weight ball. The arm bobs vertically via a node graph while the weight swings from its offset starting position.

- Animated "Arm" bone (kinematic, mass 0) with bob_preset
- "Weight" child bone (dynamic, mass 1.0, damping 0.90) displaced sideways
- Shows animation-driven parent with physics-driven child

**Demonstrates:** Animation + physics blending, kinematic/dynamic bone interaction, node graph driving physics

### 9. Damping Lab (Physics)
Three color-coded chains hanging from a horizontal bar, each with different damping:
- Bouncy (orange, left): damping 0.70 — oscillates longest
- Normal (blue, center): damping 0.92
- Heavy (green, right): damping 0.98 — settles fastest

First link of each chain starts angled for visible swing.

**Demonstrates:** Damping comparison, visual parameter tuning, multi-chain physics

### 10. Speed Glow (Physics + Nodes)
A pendulum ball whose color shifts from blue (still) to red (fast). Uses `BoneSpeed` physics input node → Remap → ShapeOutput Red/Blue.

- Ball bone (dynamic, mass 1.0, damping 0.995) offset sideways from kinematic Arm bone
- DistanceJoint creates pendulum swing
- Shape graph reads BoneSpeed, remaps to [0,1], drives Red and inverted Blue channels

**Demonstrates:** Physics input nodes, BoneSpeed, node→material reactive color, Avian physics readings

### 11. Wave Force (Nodes → Physics)
A sphere driven by oscillating upward force from the node graph. SinOscillator → BoneOutput Force Y fights gravity.

- Ball bone (dynamic, mass 0.5, damping 0.99) above a ground reference platform
- Bone graph: Time → SinOscillator(amp=15, freq=0.5) → BoneOutput Force Y (pin 8)
- Creates physics-driven bouncing via node graph force outputs

**Demonstrates:** Physics output nodes, BoneOutput Force pins, node→physics force application

### 12. Walker (Game Logic)
A character walks forward when grounded (IsColliding → Gate → Force X) and brakes at edges (RaycastDown → Compare → Gate).

**Demonstrates:** Logic nodes (Compare, Gate), sensing nodes (IsColliding, RaycastDown), physics force output

### 13. Lemmings (Game Logic)
Three color-coded walkers on a split platform with a gap. Each runs the same walker graph independently. Ground plane enabled.

**Demonstrates:** Multi-character node graphs, reusable behavior logic, ground plane collision

### 14. IK Walker (Inverse Kinematics)
Bipedal character with 2-bone IK legs. Body bobs vertically while feet follow oscillating IK targets with alternating stride. FABRIK solver adjusts knee/hip rotations each frame.

- Body: SinOscillator → Y bob (amplitude 0.05, freq 2.0)
- Feet: Phase-offset SinOscillator → IK Target X (stride), abs(sin) → step height → IK Target Y
- `ik_chain_length = 2` on each foot (analytical 2-bone solver)

**Demonstrates:** FABRIK IK solver, BoneOutput IK Target pins, 2-bone analytical solver, procedural walk cycle

## Feature Coverage

Between the 15 demos, every major feature is demonstrated:
- All 13 primitives
- All 8 combination operations
- All 6 modifiers
- All 6 color modes
- Metallic and fresnel materials
- smooth_symmetry
- Node graph animation (shape + bone)
- Custom SceneSettings
- Multi-level bone hierarchy
- Bone physics (gravity, mass, damping)
- Avian3D integration (DistanceJoint, colliders, kinematic/dynamic)
- Animation + physics blending
- Physics input nodes (BoneSpeed → material color)
- Physics output nodes (SinOscillator → Force Y)
- Game logic nodes (Compare, Gate, IsColliding, RaycastDown)
- FABRIK inverse kinematics (2-bone analytical + multi-bone)
- Procedural walk cycle (phase-offset oscillators + IK foot placement)

## Implementation

Demo scenes live in `litsdf_editor::demos`. Each is a factory function returning `DemoResult { scene, shape_graphs, bone_graphs }`. The `DemoScene` enum dispatches to the correct factory. The File menu iterates `DemoScene::all()` to build the submenu.

Physics demos auto-play on load (detected via `has_physics_bones`). Non-physics demos start paused.

The default scene (shown on app launch) is a single gray sphere — demos are discovered via the menu.
