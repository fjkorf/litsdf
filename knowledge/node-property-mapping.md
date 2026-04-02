# Node-Property Mapping — litsdf Features to Node Concepts

Analysis of every current litsdf feature and how it maps to a node-based architecture.

---

## Two Evaluation Contexts

litsdf has two distinct evaluation contexts that a node system must respect:

| Context | Where | When | Examples |
|---------|-------|------|----------|
| **CPU per-frame** | scene.rs | Every frame | Bone transforms, animation sampling, scene flattening |
| **GPU per-pixel** | sdf_raymarch.wgsl | Every pixel, every ray step | SDF evaluation, material lookup, lighting, noise |

Nodes that feed GPU evaluation must compile to WGSL. Nodes that feed CPU evaluation run in Rust.

---

## 1. Shape Geometry

**Current:** SdfPrimitive enum with 9 variants, each with params A-D.

| Input | Type | Node Concept |
|-------|------|-------------|
| Primitive selection | Enum | Primitive Node type (one node per primitive) |
| Params A-D | f32 | Input pins with defaults |

**Output:** Distance field (f32) — per-pixel

**Node design:** One node type per primitive (SphereNode, BoxNode, etc.) each with typed inputs (radius, half_extents, etc.). Output is a Distance socket.

---

## 2. Combination Operations

**Current:** CombinationOp enum (6 variants) with smooth_k parameter.

| Input | Type | Node Concept |
|-------|------|-------------|
| Distance A | f32 | Input pin (left operand) |
| Distance B | f32 | Input pin (right operand) |
| Blend K | f32 | Parameter with default |
| Op type | Enum | Node variant or dropdown |

**Output:** Combined distance (f32) + blend factor for material interpolation

**Node design:** Boolean nodes (Union, Intersection, Subtraction, SmoothUnion, SmoothIntersection, SmoothSubtraction). Two Distance inputs, one Distance output. Subtraction is order-dependent — left input is the base, right is subtracted.

---

## 3. Transform

**Current:** Two levels — bone transforms (CPU) and shape transforms (GPU/shader).

### Bone Transform (CPU per-frame)
| Input | Type | Currently |
|-------|------|-----------|
| Translation XYZ | Vec3 | `bone.transform.translation` |
| Rotation XYZ | Vec3 (degrees) | `bone.transform.rotation` |

**Node design:** A TransformNode with position/rotation/scale inputs. Outputs a Mat4 fed to child bones and shapes.

### Shape Transform (GPU per-pixel)
| Input | Type | Currently |
|-------|------|-----------|
| Translation XYZ | Vec3 | `shape.transform.translation` |
| Rotation XYZ | Vec3 (degrees) | `shape.transform.rotation` |
| Scale | f32 | `shape.transform.scale` |

Applied in shader as inverse transform to query point: `q = rotate(p - trans, -rot) / scale`

**Node design:** Same TransformNode type, but evaluated per-pixel. Inputs can come from constant values, oscillators, noise, etc.

---

## 4. Animation

**Current:** `anim_eval(params, time) = amplitude * sin(time * frequency * TAU + phase)` — hardcoded sin oscillator on every animatable property.

### What exists today
- Bone: `anim_tx/ty/tz/rx/ry/rz` — 6 oscillator triplets [amp, freq, phase]
- Shape: `anim_tx/ty/tz/rx/ry/rz/scale` — 7 oscillator triplets in ShapeMaterial

Each is additive to the base value: `final = base + anim_eval(params, time)`

### Node replacement
This is the primary motivation for the node editor. Instead of hardcoded sin oscillators:

```
TimeNode → OscillatorNode(amp, freq, phase) → TransformNode.position.y
```

**Potential animation nodes:**
| Node | Inputs | Output | Evaluation |
|------|--------|--------|------------|
| Time | — | f32 (seconds) | Per-frame |
| SinOscillator | amplitude, frequency, phase, time | f32 | Per-frame or per-pixel |
| Ramp/Curve | time, keypoints | f32 | Per-frame |
| Noise1D | time, frequency | f32 | Per-frame |
| Math (Add/Mul/Mix) | a, b | f32 | Matches inputs |

The current `[amp, freq, phase]` triplet becomes a `SinOscillator` node connected to Time, with output feeding a transform input.

---

## 5. Material

**Current:** ShapeMaterial with color, roughness, metallic, fresnel, color_mode, palette params.

| Property | Type | Node Concept |
|----------|------|-------------|
| Color | Vec3 | Color input pin |
| Roughness | f32 | Float input pin (0-1) |
| Metallic | f32 | Float input pin (0-1) |
| Fresnel power | f32 | Float input pin |
| Color mode | Enum | Material node variant |
| Palette A/B/C/D | Vec3 x4 | CosinePalette node inputs |

**Node design:** Material output node (like Unity's Master Stack) with typed inputs for each PBR property. Color can come from:
- Constant color
- CosinePalette node (a + b * cos(2pi(c*t + d)))
- Noise color node
- Any Vec3 source

---

## 6. Modifiers

**Current:** Vec<ShapeModifier> — flat list of 6 types, applied in shader.

### Domain Modifiers (transform query point before SDF eval)
| Modifier | Params | Shader Function |
|----------|--------|-----------------|
| Twist | amount (f32) | `op_twist(p, k)` — rotates XZ by Y |
| Bend | amount (f32) | `op_bend(p, k)` — bends XY by X |
| Elongation | xyz (Vec3) | `op_elongate(p, h)` — clamps coords |
| Repetition | period (Vec3) | `op_repeat(p, period)` — mirrors space |

### Distance Modifiers (transform distance after SDF eval)
| Modifier | Params | Effect |
|----------|--------|--------|
| Rounding | radius (f32) | `d -= rounding` |
| Onion | thickness (f32) | `d = abs(d) - thickness` |

**Node design:** Modifier nodes take a Distance input and produce a Distance output. Domain modifiers also take a Position input. Can be chained:

```
Position → Twist(amount) → Bend(amount) → SphereNode → Rounding(0.1) → output
```

---

## 7. Noise

**Current:** FBM noise used for displacement and color modulation. Params: amplitude, frequency, octaves.

| Input | Type | Usage |
|-------|------|-------|
| Position | Vec3 | Spatial coordinate |
| Amplitude | f32 | Displacement strength |
| Frequency | f32 | Spatial scale |
| Octaves | u32 | Layer count |

**Outputs:**
- Displacement (f32) — added to SDF distance
- Color multiplier (f32) — modulates albedo

**Node design:** NoiseNode with position/frequency/octaves inputs, float output. Can feed into:
- Distance add (displacement)
- Color multiply (texture variation)
- Any float input (animation, parameter modulation)

---

## 8. Lighting

**Current:** Three-point lighting hardcoded in shader with scene-level `light_dir`.

Lighting is fundamentally different from other features — it's a post-hit shading calculation, not part of the SDF evaluation. For now, lighting parameters (direction, intensity, color) can remain as scene-level properties rather than nodes.

Future: a lighting node graph (like Blender's shader output) where material nodes feed into a PBR lighting evaluation.

---

## Summary: What Becomes a Node

| Current Feature | Node Type | Status |
|----------------|-----------|--------|
| SdfPrimitive variants | Primitive nodes (Sphere, Box, ...) | Future (Phase D+ shader codegen) |
| CombinationOp | Boolean nodes (Union, Subtract, ...) | Future (Phase D+ shader codegen) |
| ShapeTransform | ShapeOutput pins 0-6 | **Implemented** |
| Animation | Time + Oscillator nodes (5 wave types) | **Implemented** (triplets removed) |
| ShapeMaterial | ShapeOutput pins 7-16 | **Implemented** (color, roughness, metallic, fresnel, noise, symmetry) |
| ShapeModifier | ShapeOutput pins 17-26 | **Implemented** (rounding, onion, twist, bend, elongation, repetition) |
| Cosine palette | CosinePalette node | **Implemented** |
| Bone hierarchy | Stays as-is (spatial organizer) | N/A |
| Lighting | SceneSettings (tunable params) | **Implemented** |

### What Stays Outside the Node Graph
- **Bone hierarchy** — spatial organization, not a per-shape computation
- **Visibility** — boolean flag, not a continuous value
- **Scene name** — metadata
- **Combination ordering** — determined by shape order in bone, not node graph
- **SDF primitive evaluation** — stays in shader (future Phase D+ for shader codegen)
