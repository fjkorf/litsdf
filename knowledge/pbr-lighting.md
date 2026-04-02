# PBR Lighting Upgrade — Cook-Torrance BRDF

Research into replacing the current Blinn-Phong lighting with physically-based rendering.

## Current vs PBR

| Aspect | Current (Blinn-Phong) | PBR (Cook-Torrance) |
|--------|----------------------|---------------------|
| Specular | `pow(NdotH, mix(8,128,1-roughness))` | GGX distribution `D * F * G / (4*NdotV*NdotL)` |
| Fresnel | Manual `fresnel_power` parameter | Automatic from Schlick `F0 + (1-F0)*(1-HdotV)^5` |
| Metallic | Linear `mix(0.04, 1.0, metallic)` | Energy-conserving: metals have zero diffuse |
| Energy | Specular added on top (can exceed 1.0) | `kD + kS <= 1.0` enforced |
| Diffuse | `albedo * NdotL` | `albedo / PI * NdotL` (correct normalization) |

## The Three BRDF Functions

### D — GGX Normal Distribution (~5 lines WGSL)
```
D(NdotH, roughness) = alpha^2 / (PI * ((NdotH^2)*(alpha^2-1)+1)^2)
where alpha = roughness^2
```
Replaces the power-interpolated Blinn-Phong highlight. GGX has heavier tails (more realistic falloff).

### F — Fresnel-Schlick (~3 lines WGSL)
```
F(HdotV, F0) = F0 + (1-F0) * (1-HdotV)^5
where F0 = mix(vec3(0.04), albedo, metallic)
```
Dielectrics: 4% reflectance at normal incidence. Metals: albedo IS the specular color. Replaces `fresnel_power` entirely.

### G — Smith Geometry (~7 lines WGSL)
```
G(NdotV, NdotL, roughness) = G1(NdotV, k) * G1(NdotL, k)
where G1(x, k) = x / (x*(1-k)+k)
and k = (roughness+1)^2 / 8
```
New addition — accounts for microfacet self-shadowing. Without it, rough surfaces appear unrealistically shiny at edges.

## Energy Conservation
```
kS = F  (specular coefficient = Fresnel result)
kD = (1 - kS) * (1 - metallic)  (diffuse coefficient)
```
Two rules: kD + kS <= 1.0, and metals have zero diffuse.

## Per-Light Calculation
```wgsl
let f0 = mix(vec3(0.04), albedo, metallic);
let D = distribution_ggx(n_dot_h, roughness);
let G = geometry_smith(n_dot_v, n_dot_l, roughness);
let F = fresnel_schlick(h_dot_v, f0);
let specular = (D * G * F) / (4.0 * n_dot_v * n_dot_l + 0.0001);
let kD = (vec3(1.0) - F) * (1.0 - metallic);
let diffuse = kD * albedo / PI;
let Lo = (diffuse + specular) * n_dot_l * shadow;
```

## Ambient Lighting Without Cubemaps

**Hemisphere lighting (analytical sky):**
```wgsl
let ambient_diffuse = mix(ground_color, sky_color, 0.5 + 0.5 * nor.y) * albedo;
```

**Ambient specular (reflective sky sample):**
```wgsl
let reflect_dir = reflect(-V, nor);
let sky_reflect = mix(ground_color, sky_color, 0.5 + 0.5 * reflect_dir.y);
let F_ambient = fresnel_schlick_roughness(n_dot_v, f0, roughness);
let ambient_specular = sky_reflect * F_ambient;
```

**Combined:**
```wgsl
let kD_ambient = (vec3(1.0) - F_ambient) * (1.0 - metallic);
let ambient = kD_ambient * ambient_diffuse + ambient_specular;
color = ambient * ao + Lo;
```

## What Changes in the Shader

**Add (~25 lines):** 5 BRDF helper functions (D_GGX, F_Schlick, F_Schlick_Roughness, G_SchlickGGX, G_Smith)

**Replace (~25 lines):** The lighting section (lines 566-617) gets rewritten. Remove:
- `spec_power = mix(8, 128, ...)` — replaced by GGX
- `spec_intensity = mix(0.04, 1.0, metallic)` — replaced by energy conservation
- `specular = pow(...)` — replaced by DFG/(4*NdotV*NdotL)
- Additive `color += specular` — folded into unified Lo
- `view_factor` brightening — Fresnel handles this
- `fresnel_power` rim block — Fresnel handles this

**Keep as-is:** Ray marching, SDF primitives, calc_normal, calc_ao, soft_shadow, sdf_scene, sdf_scene_material, color modes, noise, modifiers, post-processing.

## Struct Impact

`fresnel_power` on `ShapeMaterial` and `ShaderShape` can be removed — Fresnel is automatic from the physics. This saves 4 bytes per shape in the shader struct. However, removing it is a breaking YAML change and requires updating ShaderShape alignment.

**Alternative:** Keep `fresnel_power` as an artistic override multiplier (default 1.0). `F_final = F_schlick * fresnel_power`. This preserves backward compat and gives artists extra control.

## SDF-Specific Advantages

- **AO from SDF** — already implemented (`calc_ao`), physically grounded
- **Soft shadows from SDF** — already implemented (`soft_shadow`), better than shadow maps
- **Specular occlusion** (optional future) — march along reflection vector using SDF, dim specular in crevices (~8-16 extra SDF evals per pixel)

## Sources

- LearnOpenGL PBR Theory/Lighting: learnopengl.com/PBR/Theory
- Google Filament PBR docs: google.github.io/filament/Filament.md.html
- Nadrin/PBR GLSL implementation: github.com/Nadrin/PBR
- Shadertoy PBR raymarcher: shadertoy.com/view/tdlXzr
