# SDF Rendering in litsdf

## What is SDF?

A Signed Distance Function returns the shortest distance from any point in space to the nearest surface. Positive = outside, zero = on surface, negative = inside. This property enables ray marching: step along a ray by the SDF value (safe because nothing is closer), repeat until you hit the surface.

## Ray Marching Algorithm (sdf_raymarch.wgsl)

```
for each pixel:
  ray_origin = camera position
  ray_dir = normalize(world_position - camera)
  t = 0
  for 128 steps:
    p = ray_origin + ray_dir * t
    d = sdf_scene(p)          // evaluate all shapes
    if d < 0.001: HIT         // close enough to surface
    t += d                    // advance by safe distance
    if t > 100: MISS          // too far, discard pixel
```

## Primitives (13 types)

Each primitive is a function `sd_*(p, params) -> float`:

| Type | Encoding | Params | Formula essence |
|------|----------|--------|----------------|
| Sphere | 0 | radius | `length(p) - r` |
| Box | 1 | half_extents (vec3) | `length(max(abs(p)-b, 0))` |
| RoundBox | 2 | half_extents + rounding | Box distance minus rounding |
| Cylinder | 3 | height, radius | 2D circle in XZ + height clamp |
| CappedCone | 4 | height, r1, r2 | Interpolated radii along Y |
| Torus | 5 | major, minor | Ring distance |
| Capsule | 6 | radius, half_height | Line segment minus radius |
| Plane | 7 | normal, offset | `dot(p, normal) + offset` |
| Ellipsoid | 8 | radii (vec3) | Scaled sphere approximation |
| Octahedron | 9 | size | Diamond shape via plane folding |
| Pyramid | 10 | height, base | Square-base pyramid |
| HexPrism | 11 | height, radius | Hexagonal column |
| RoundCone | 12 | r1, r2, height | Cone with spherical caps |

CPU-side picking uses `core::sdf::eval_primitive()` which dispatches all 13 types.

## Combination Operations

Shapes combine sequentially — each shape's result is merged with the accumulated SDF:

| Op | Formula | Effect |
|----|---------|--------|
| Union | `min(a, b)` | Both shapes visible |
| Intersection | `max(a, b)` | Only overlap |
| Subtraction | `max(a, -b)` | Carve b from a |
| SmoothUnion | polynomial blend with k | Soft merge |
| SmoothIntersection | polynomial blend | Soft overlap |
| SmoothSubtraction | polynomial blend | Soft carve |

The `k` parameter controls blend radius. Higher k = wider, softer blend. At k=0, smooth variants behave like hard versions.

## Normals

Computed via tetrahedron technique — 4 SDF evaluations at offset positions. More efficient than central differences (6 evaluations). The gradient of the SDF gives the surface normal direction.

## Lighting (PBR — Cook-Torrance BRDF)

Physically-based rendering with energy conservation:

- **Diffuse:** Lambertian `albedo / PI * NdotL`, weighted by `kD = (1-F)*(1-metallic)` (metals have zero diffuse)
- **Specular:** Cook-Torrance microfacet BRDF: `DFG / (4*NdotV*NdotL)`
  - **D** = GGX/Trowbridge-Reitz normal distribution (roughness-based lobe shape)
  - **F** = Fresnel-Schlick approximation (`F0 = mix(0.04, albedo, metallic)`)
  - **G** = Smith geometry with Schlick-GGX (microfacet self-shadowing)
- **Ambient:** Hemisphere diffuse (sky/ground interpolation) + reflective sky specular with roughness-adjusted Fresnel
- **Soft shadows:** Ray march from hit point toward light, track `min(k*h/t)` for penumbra
- **Ambient occlusion:** Sample SDF along normal at increasing distances
- **SSS:** Approximate subsurface scattering (red tint when backlit)
- **Artistic override:** `fresnel_power > 1.0` adds extra rim glow on top of physical Fresnel

## Shader Data Layout

The CPU sends a flat `SdfShaderParams` uniform at `@group(3) @binding(0)`:

```
struct SdfParams {
    shape_count: u32,
    _pad: vec3<f32>,
    shapes: array<ShaderShape, 32>,
}

struct ShaderShape {
    primitive_type: u32,    // 0-12
    combination_op: u32,    // 0-5
    smooth_k: f32,
    _pad0: f32,
    params: vec4<f32>,      // primitive-specific
    translation: vec3<f32>,
    _pad1: f32,
    rotation: vec3<f32>,    // euler radians
    scale: f32,
    color: vec3<f32>,
    _pad2: f32,
}
```

The shader loops `shapes[0..shape_count]`, evaluating each and combining with `combine()`. The first shape's combine op is ignored (it seeds the accumulator).

## Bounding Geometry

The SDF is rendered on a large Cuboid (40x40x40) centered at origin. The camera sits inside it. Backface culling is disabled via `Material::specialize()`. The fragment shader ray-marches from the camera through each pixel of the cuboid's faces. Pixels that miss all shapes call `discard`.

## Shader File Location

The WGSL shader lives at `crates/litsdf_render/assets/shaders/sdf_raymarch.wgsl`. The `assets/shaders/` directory at the workspace root is a **symlink** to this location. Bevy loads from the root `assets/` path at runtime. Always edit the crate copy — the symlink ensures Bevy sees the same file.

## ShaderShape Struct Alignment (Critical Gotcha)

The `ShaderShape` struct must be **byte-identical** between Rust (`shader.rs`) and WGSL (`sdf_raymarch.wgsl`). Bevy's `ShaderType` derive (via `encase`) computes sizes using WGSL std140 alignment rules — `Vec3` is padded to 16 bytes, array strides are multiples of 16.

**When adding or removing fields:**
1. Edit both `shader.rs` AND `sdf_raymarch.wgsl` together
2. Run `cargo test -p litsdf_render` — the `shader_shape_size_matches_wgsl` test catches size mismatches
3. If sizes don't match, the viewport will silently render blank (no panic, no error — just wrong data offsets)

Current sizes: ShaderShape = 256 bytes, SdfShaderParams = 8288 bytes (32 header + 32 * 256 shapes + 64 settings).

**Critical gotcha:** `SdfShaderParams` scene settings fields MUST be placed AFTER the `shapes` array. Inserting fields before the array shifts its offset and Metal silently fails to render (blank viewport, no error, no panic). This was discovered empirically and is documented as a critical rule in CLAUDE.md.

## Scene Settings

`SceneSettings` (in `SdfScene`) exposes lighting and post-processing parameters:
- **Fill light:** color, intensity (sky hemisphere)
- **Back light:** color, intensity (warm bounce)
- **SSS:** color, intensity (subsurface scattering tint)
- **AO intensity** — ambient occlusion strength
- **Shadow softness** — soft shadow penumbra
- **Vignette intensity** — edge darkening

All have serde defaults matching the original hardcoded values. The shader reads these from `params.*` after the shapes array.

## Noise Functions

- **Value noise** (`noise3d`) — trilinear with quintic smoothing
- **FBM** (`fbm`) — octave-based frequency/amplitude scaling
- **Ridged FBM** (`fbm_ridged`) — `abs(noise)` per octave, creates ridge/vein patterns
- **Cellular/Voronoi** (`cellular3d`) — returns `vec2(f1, f2)` distances to nearest two cell centers

Note: `calc_normal` uses the tetrahedron technique (4 SDF evaluations at offset points), which correctly accounts for noise displacement. Analytical noise derivatives are not needed.

## Combination Operations (8 types)

| Op | Encoding | Description |
|----|----------|-------------|
| Union | 0 | min(a, b) |
| Intersection | 1 | max(a, b) |
| Subtraction | 2 | max(a, -b) |
| SmoothUnion | 3 | polynomial blend with k |
| SmoothIntersection | 4 | polynomial blend with k |
| SmoothSubtraction | 5 | polynomial blend with k |
| ChamferUnion | 6 | beveled edge at junction |
| ChamferIntersection | 7 | beveled edge at intersection |

## Color Modes (6 modes)

| Mode | Value | Description |
|------|-------|-------------|
| Solid | 0 | Plain color |
| Cosine Palette | 1 | `a + b * cos(2π(ct + d))` |
| Noise Tint | 2 | Color modulated by FBM noise |
| Cellular | 3 | Voronoi cell pattern with edge detection |
| Ridged | 4 | Ridged multifractal — `abs(noise)` creates ridge patterns |
| Gradient Snow | 5 | Normal-based: smoothstep on normal.y blends white onto upward faces |

Modes 0-4 are evaluated in `get_shape_color()` (per-shape, before material blending). Mode 5 is evaluated in the fragment shader after the normal is computed, since it depends on the blended surface normal. `MatResult.color_mode` carries the dominant shape's mode to the fragment stage.

## Shader Code Generation

The shader is split into three parts:
- `sdf_preamble.wgsl` (426 lines) — structs, noise, all 13 primitive SDFs, BRDF functions, domain modifiers, eval_shape, get_shape_color, combine_blend
- Generated body (~50 lines) — `sdf_scene()` and `sdf_scene_material()` with unrolled shape evaluation
- `sdf_postamble.wgsl` (181 lines) — normals, shadows, AO, ray march, PBR lighting, fragment shader

`codegen::generate_shader(scene)` walks the flattened shape list and emits static indexed code: `params.shapes[0]`, `params.shapes[1]`... with no loop, no switch. The existing uniform array is still used for parameter passing — codegen only changes the evaluation structure.

`codegen::topology_hash(scene)` detects when recompilation is needed (shape count, primitive types, combination ops, modifier flags change). On topology change, the generated WGSL is written to `assets/shaders/sdf_raymarch.wgsl` and Bevy hot-reloads it.

## Animation

Animation is handled by the **node editor** (egui-snarl) in the editor crate, not by shader fields. There are no `anim_*` fields on shapes or bones. Node graphs evaluate per-frame on the CPU and write computed values into shape/bone transform properties, which then flow to the shader via the normal sync path.

Old YAML files with `anim_*` fields will fail to load with a descriptive migration message.

## References

- Inigo Quilez SDF primitives: iquilezles.org/articles/distfunctions/
- Smooth minimum: iquilezles.org/articles/smin/
- Ray marching: iquilezles.org/articles/raymarchingdf/
