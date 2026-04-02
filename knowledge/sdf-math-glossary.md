# SDF Math Glossary

A comprehensive reference of the mathematical concepts behind SDF modeling, organized by category. Each entry describes the math, where it's implemented in litsdf (if applicable), and whether it preserves distance field properties.

---

## Core Concept: Signed Distance Function

A function `f: R³ → R` where:
- `f(p) > 0` → point `p` is outside the surface
- `f(p) = 0` → point `p` is on the surface
- `f(p) < 0` → point `p` is inside the surface
- `|∇f| = 1` everywhere (Lipschitz continuous with constant 1)

The Lipschitz property guarantees safe sphere tracing — each ray march step advances by `f(p)` and never overshoots the surface.

---

## 1. Ray Marching Pipeline

The mathematical flow from pixel to color:

| Stage | Math | litsdf Location |
|-------|------|-----------------|
| **Ray generation** | `ray(t) = origin + t * direction` from camera model | `fragment()` in sdf_raymarch.wgsl |
| **Sphere tracing** | `t ← t + f(ray(t))` repeated until `f < ε` or `t > t_max` | `ray_march()` in sdf_raymarch.wgsl |
| **Normal computation** | `n = normalize(∇f(p))` via tetrahedron technique (4 evaluations) | `calc_normal()` in sdf_raymarch.wgsl |
| **Material lookup** | Evaluate per-shape SDFs at hit point, blend by proximity | `sdf_scene_material()` in sdf_raymarch.wgsl |
| **Lighting** | Lambert diffuse + Blinn-Phong specular + AO + soft shadows | `fragment()` in sdf_raymarch.wgsl |
| **Post-processing** | Gamma correction, color grading, vignette | `fragment()` in sdf_raymarch.wgsl |

**CPU-side pipeline** (per frame):
| Stage | Math | litsdf Location |
|-------|------|-----------------|
| **Bone transform** | `world = parent × local` with animated rotation/translation | `compute_bone_world_transforms()` in litsdf_core/scene.rs |
| **Shape flattening** | `world_pos = bone_world × shape_local`, decompose to euler+translate | `flatten_bone_tree()` in litsdf_core/scene.rs |
| **Uniform encoding** | Pack flat shapes into GPU struct array | `build_shader_params()` in litsdf_render/scene_sync.rs |

---

## 2. Primitives

### Implemented in litsdf (9)

| Primitive | Formula | Exact? | Params in litsdf |
|-----------|---------|--------|-----------------|
| **Sphere** | `length(p) - r` | Yes | radius |
| **Box** | `length(max(abs(p)-b, 0)) + min(max(q.xyz), 0)` | Yes | half_extents[3] |
| **Round Box** | `sdBox(p, b-r) - r` | Yes | half_extents[3], rounding |
| **Cylinder** | Capped cylinder from radial + height distances | Yes | height, radius |
| **Capped Cone** | Frustum with two radii | Yes | height, r1, r2 |
| **Torus** | `length(vec2(length(p.xz)-R, p.y)) - r` | Yes | major_radius, minor_radius |
| **Capsule** | `length(p - clamp(p.y,-h,h)*Y) - r` | Yes | radius, half_height |
| **Plane** | `dot(p, n) + offset` | Yes | normal[3], offset |
| **Ellipsoid** | `k0*(k0-1)/k1` where k0=length(p/r), k1=length(p/r²) | **Bound** | radii[3] |

### Not implemented (gap)

| Primitive | Value | Difficulty |
|-----------|-------|------------|
| **Box Frame** | Wire-frame box edges | Low |
| **Capped Torus** | Partial torus arc | Low |
| **Link** | Chain link shape | Low |
| **Round Cone** | Cone with spherical caps | **Implemented** |
| **Octahedron** | Diamond shape | **Implemented** |
| **Pyramid** | Square pyramid | **Implemented** |
| **Hexagonal Prism** | 6-sided column | **Implemented** |
| **Triangle/Quad** | Flat 3D polygons | Medium |
| **Bezier curve SDF** | Smooth spline primitives | Medium |
| **2D→3D Revolution** | Sweep 2D profile around axis | Medium (new concept) |
| **2D→3D Extrusion** | Extend 2D shape along axis | Medium (new concept) |

---

## 3. Combination Operations

### Implemented in litsdf (6)

| Operation | Formula | Blend factor | Material blending |
|-----------|---------|-------------|-------------------|
| **Union** | `min(a, b)` | Binary (0 or 1) | Nearest shape wins |
| **Intersection** | `max(a, b)` | Binary | Nearest shape wins |
| **Subtraction** | `max(-a, b)` | Binary | Shape b wins |
| **Smooth Union** | `mix(b,a,h) - k*h*(1-h)` where `h=clamp(0.5+0.5*(b-a)/k)` | Continuous [0,1] | `mix(colorA, colorB, h)` |
| **Smooth Intersection** | Derived from smooth union via negation | Continuous | Blended |
| **Smooth Subtraction** | Derived from smooth union via negation | Continuous | Blended |

Smooth variant is **polynomial (quadratic)**. C1 continuous, rigid (exact outside blend zone), conservative (safe for tracing).

### Not implemented (gap)

| Operation | What it does | Value |
|-----------|-------------|-------|
| **Chamfer Union** | 45° bevel at junction | **Implemented** (ChamferUnion, ChamferIntersection) |
| **Stairs Union** | Terraced steps at junction | Medium — decorative |
| **Columns Union** | Periodic columnar bridge | Medium — decorative |
| **Groove/Tongue** | Channel or ridge at junction | Medium — mechanical detail |
| **Circular smooth min** | Perfect circular fillet profile | High — more physically correct than polynomial |
| **Exponential smooth min** | C-infinity smooth, globally blending | Low priority — polynomial is usually sufficient |
| **Morphing** | `mix(sdfA, sdfB, t)` linear blend between two shapes | Medium — shape-shifting animation |

---

## 4. Domain Operations

### Implemented in litsdf

| Operation | Formula | Where | Preserves Lipschitz? |
|-----------|---------|-------|---------------------|
| **Translation** | `f(p - offset)` | eval_shape (shader) | Yes |
| **Rotation** | `f(R⁻¹ · p)` via Euler angles | eval_shape (shader) | Yes |
| **Uniform scale** | `f(p/s) × s` | eval_shape (shader) | Yes |
| **Smooth symmetry** | `p.x = sqrt(p.x² + k)` | eval_shape (shader) | Yes |
| **Twist** | Rotate XZ plane by `k·p.y` | eval_shape (shader) | **No** |
| **Bend** | Rotate XY plane by `k·p.x` | eval_shape (shader) | **No** |
| **Elongation** | `p - clamp(p, -h, h)` | eval_shape (shader) | Yes |
| **Repetition** | `p - period × round(p/period)` | eval_shape (shader) | Yes (if spacing sufficient) |

### Not implemented (gap)

| Operation | What it does | Value |
|-----------|-------------|-------|
| **Non-uniform scale** | `f(p/s) × min(s)` — lower bound only | Medium |
| **Kaleidoscope** | Angular domain folding for N-fold symmetry | High — decorative, snowflakes, mandalas |
| **Menger fold** | Fractal space subdivision | Medium — fractal geometry |
| **Sierpinski fold** | Tetrahedral fractal folding | Medium |
| **Finite repetition** | `clamp(round(p/s), -limit, limit)` per axis | Medium (repetition exists but count not exposed) |
| **Revolution** | Sweep 2D SDF around Y axis | High — surfaces of revolution |
| **Extrusion** | Extend 2D SDF along Z | High — prismatic shapes |
| **Domain warping** | `f(p + fbm(p))` for organic distortion | Medium — already have noise displacement |

---

## 5. Distance Operations (Post-primitive modifiers)

### Implemented in litsdf

| Operation | Formula | Effect | Preserves Lipschitz? |
|-----------|---------|--------|---------------------|
| **Rounding** | `f(p) - r` | Inflates surface, rounds all edges | Yes |
| **Onion / Shell** | `abs(f(p)) - thickness` | Hollows into shell | Yes |
| **Noise displacement** | `f(p) + amp × fbm(p × freq)` | Organic surface roughness | **No** (depends on amplitude) |

### Not implemented (gap)

| Operation | Formula | Value |
|-----------|---------|-------|
| **Offset surface** | `f(p) - thickness` (same as rounding but conceptually different) | Already implemented as Rounding |
| **Annular** | `abs(f(p)) - r` nested N times for concentric shells | Low — Onion already does one layer |

---

## 6. Noise and Procedural Functions

### Implemented in litsdf

| Function | Formula | Where | Derivatives? |
|----------|---------|-------|-------------|
| **Hash (3D)** | `fract(p.x × p.y × p.z × (p.x+p.y+p.z))` after scaling | sdf_raymarch.wgsl `hash3()` | No |
| **Value noise (3D)** | Trilinear interpolation of hashed corners, quintic smoothing | sdf_raymarch.wgsl `noise3d()` | No (computed but not returned) |
| **FBM** | `Σ amplitude^i × noise(frequency^i × p)` | sdf_raymarch.wgsl `fbm()` | No |
| **Per-tile random** | `hash(floor(p × scale))` | Not standalone function, used conceptually | No |

### Not implemented (gap)

| Function | What it does | Value |
|----------|-------------|-------|
| **Gradient/Perlin noise** | Smoother than value noise, fewer grid artifacts | Medium |
| **Simplex noise** | Even smoother, lower computational cost, fewer artifacts | High |
| **Cellular/Voronoi noise** | Distance to nearest random point — cell patterns, cracks, scales | **Implemented** (cellular3d in shader, color_mode 3) |
| **Worley noise** | Variant of cellular — F1, F2 distances | High |
| **Curl noise** | Divergence-free noise field — smoke, fluid motion | Medium |
| **Noise with analytical derivatives** | Return vec4(value, dx, dy, dz) — needed for proper displaced normals | High |
| **Domain-rotated FBM** | Apply rotation matrix per octave to reduce axis-aligned artifacts | Medium |
| **Ridged multifractal** | `abs(noise)` creates ridge-like patterns (mountains, veins) | **Implemented** (fbm_ridged, color_mode 4) |
| **Turbulence** | `Σ |noise|` — absolute value FBM for turbulent patterns | Medium |
| **Erosion pattern** | FBM with terrain-like features: ridges, valleys, plateaus | Medium |

---

## 7. Color Math

### Implemented in litsdf

| Technique | Formula | Where |
|-----------|---------|-------|
| **Solid color** | Constant RGB per shape | `get_shape_color()` mode 0 |
| **Cosine palette** | `a + b × cos(2π(c×t + d))` | `get_shape_color()` mode 1 |
| **Noise tint** | `color × (0.7 + 0.3 × noise(p))` | `get_shape_color()` mode 2 |
| **Material blending** | `mix(matA, matB, blend_factor)` at smooth boundaries | `sdf_scene_material()` |

### Not implemented (gap)

| Technique | What it does | Value |
|-----------|-------------|-------|
| **Stamping** | Distance-to-feature → smoothstep → color blend (cheeks, makeup) | High |
| **Polar patterns** | Color from `atan2(p.z, p.x)` — radial stripes, iris detail | Medium |
| **Triplanar mapping** | Project 3 planes, blend by normal — texture without UV | High |
| **Distance field coloring** | Use SDF value itself as color driver (glow, contour lines, proximity) | Medium |
| **Gradient-based coloring** | Normal.y → snow deposition, Normal.x → side moss | **Implemented** (color_mode 5, Gradient Snow) |
| **Per-shape color stamps** | Multiple color regions per shape (face features, markings) | High |
| **Ambient color from environment** | Sky color from normal hemisphere, ground bounce | Medium |

---

## 8. Lighting Math

### Implemented in litsdf

| Technique | Formula | Where |
|-----------|---------|-------|
| **Lambert diffuse** | `max(dot(n, L), 0)` | fragment shader |
| **Blinn-Phong specular** | `pow(max(dot(n, H), 0), shininess)` | fragment shader |
| **SDF ambient occlusion** | `1 - Σ (expected - actual) / 2^i` sampling along normal | `calc_ao()` |
| **Soft shadows (penumbra)** | `min(k × h / t)` along shadow ray | `soft_shadow()` |
| **Subsurface scattering** | `clamp(-dot(n, L)) × red_tint` for backlit surfaces | fragment shader |
| **Three-point lighting** | Key (directional) + Fill (hemisphere) + Back (warm bounce) | fragment shader |
| **Fresnel rim** | `pow(1 - dot(n, V), power)` | fragment shader |
| **View-dependent brightening** | `(1 - dot(n, V)) × albedo × factor` | fragment shader |

### Not implemented (gap)

| Technique | What it does | Value |
|-----------|-------------|-------|
| **PBR (Cook-Torrance)** | Physically-based BRDF with GGX distribution | High |
| **Image-based lighting** | Environment map for reflections | Medium |
| **Global illumination approximation** | Cone tracing or SDF-based indirect bounces | Medium |
| **Improved soft shadows (Aaltonen)** | Better penumbra shape using previous step distance | Medium |
| **Volumetric fog** | Ray-march through density field for atmospheric effects | Medium |
| **Screen-space reflections** | Approximate reflections using depth buffer | Low |

---

## 9. Shaping Functions

Mathematical utility functions that remap values. Used for easing, transitions, and signal shaping.

### Implemented in litsdf

| Function | Formula | Where |
|----------|---------|-------|
| **smoothstep** | `3t² - 2t³` (built into WGSL) | Various |
| **clamp** | `min(max(x, lo), hi)` | Various |
| **mix/lerp** | `a + t × (b - a)` | Material blending |
| **sin oscillation** | `amp × sin(time × freq × 2π + phase)` | `animate()` |

### Not implemented (gap — useful for animation and effects)

| Function | Formula | Value |
|----------|---------|-------|
| **smootherstep** | `6t⁵ - 15t⁴ + 10t³` (C2 continuous) | Medium |
| **exponential impulse** | `k×x×exp(1 - k×x)` — sharp attack, decay | **Implemented** (ExpImpulse node) |
| **sustained impulse** | Quadratic attack + exponential release | High (eye movement) |
| **gain** | Contrast control `0.5 × pow(2x, k)` | Medium |
| **parabola** | Symmetric bump `pow(4x(1-x), k)` | Medium |
| **power curve** | Asymmetric bump `k × x^a × (1-x)^b` | Medium |
| **cubic pulse** | Local bump with compact support | Medium |
| **sinc** | `sin(πx)/(πx)` — ringing/bouncing | Low |

---

## 10. Fractal / IFS Techniques

None implemented. All are gap.

| Technique | Method | Value |
|-----------|--------|-------|
| **Menger sponge** | Axis-aligned folds + scale per iteration | Medium |
| **Sierpinski** | Tetrahedral folds + scale per iteration | Medium |
| **Mandelbulb** | Spherical-coordinate power iteration | High (iconic) |
| **Julia set (quaternion)** | Quaternion iteration with distance estimator | Medium |
| **Apollonian gasket** | Sphere inversions | Medium |
| **Koch snowflake** | 2D fold iteration + extrusion/revolution | Low |

---

## 11. Animation Math

### Implemented in litsdf

| Technique | Formula | Where |
|----------|---------|-------|
| **Sinusoidal oscillation** | `base + amp × sin(t × freq × 2π + phase)` | `animate()` (WGSL), `anim_eval()` (Rust) |

### Not implemented (gap — from the Shadertoy talk)

| Technique | Formula | Value |
|----------|---------|-------|
| **Coprime blink** | `max(blink(t, period1), blink(t, period2))` — natural periodic motion | High |
| **Smooth step attack + exp decay** | `smoothstep(0,attack,t) × exp(-decay×t)` | High |
| **Lissajous curves** | `(sin(a×t), sin(b×t))` with non-uniform parameterization | Medium |
| **Cubic power spikiness** | `cos(t)³` for spikier motion than pure sin | Medium |
| **Easing functions** | Ease-in, ease-out, ease-in-out for keyframe interpolation | **Implemented** (EaseInOut node with configurable exponent) |
| **Noise-driven animation** | `fbm(t)` for organic random motion | Medium |

---

## Gap Priority Summary

### High Priority (significant visual/creative impact)

| Gap | Category | Why |
|-----|----------|-----|
| Cellular/Voronoi noise | Noise | Scales, cells, organic patterns |
| Simplex noise | Noise | Better quality, fewer artifacts |
| Noise with derivatives | Noise | Correct normals on displaced surfaces |
| Chamfer union/intersection | Combination | Mechanical/architectural modeling |
| Revolution (2D→3D) | Domain | Bowls, vases, columns from 2D profiles |
| Extrusion (2D→3D) | Domain | Prismatic shapes from 2D profiles |
| Stamping coloring | Color | Face features, markings, decals |
| Gradient-based coloring | Color | Snow, moss, weathering |
| Easing/impulse functions | Animation | Natural motion, blink, expressions |
| PBR lighting (Cook-Torrance) | Lighting | Physically correct specular |

### Medium Priority

| Gap | Category |
|-----|----------|
| Kaleidoscope domain folding | Domain |
| Ridged multifractal noise | Noise |
| Triplanar color mapping | Color |
| Morphing between shapes | Combination |
| More smooth min variants | Combination |
| Mandelbulb fractals | Fractal |
| Lissajous animation | Animation |

### Low Priority

| Gap | Category |
|-----|----------|
| Bezier SDF primitives | Primitive |
| NURBS surfaces | Primitive |
| Screen-space reflections | Lighting |
| Volumetric fog | Lighting |
| Sierpinski/Menger fractals | Fractal |
