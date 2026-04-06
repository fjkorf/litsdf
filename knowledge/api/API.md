# litsdf API Reference

Generated from source by `python3 scripts/generate-api-docs.py`.
Contains full type definitions, function signatures, and module dependencies.
Run after any code change that adds, removes, or modifies types or functions.

---

## `crates/litsdf_core/src/models.rs`

### Structs

#### `ShapeId` (line 22)

```rust
pub struct ShapeId(pub Uuid);
```


#### `BoneId` (line 26)

```rust
pub struct BoneId(pub Uuid);
```


#### `RotationLimits` (line 50)

```rust
pub struct RotationLimits {
    #[serde(default, skip_serializing_if = "is_none_limit")]
    pub pitch: Option<[f32; 2]>,
    #[serde(default, skip_serializing_if = "is_none_limit")]
    pub yaw: Option<[f32; 2]>,
    #[serde(default, skip_serializing_if = "is_none_limit")]
    pub roll: Option<[f32; 2]>,
}
```


#### `BonePhysicsProps` (line 64)

```rust
pub struct BonePhysicsProps {
    #[serde(default, skip_serializing_if = "is_zero")]
    pub mass: f32,
    #[serde(default = "default_damping", skip_serializing_if = "is_default_damping")]
    pub damping: f32,
    #[serde(default, skip_serializing_if = "RotationLimits::is_default")]
    pub rotation_limits: RotationLimits,
}
```


#### `SdfBone` (line 86)

```rust
pub struct SdfBone {
    pub id: BoneId,
    pub name: String,
    #[serde(default, skip_serializing_if = "ShapeTransform::is_default")]
    pub transform: ShapeTransform,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub visible: bool,
    #[serde(default, skip_serializing_if = "BonePhysicsProps::is_default")]
    pub physics: BonePhysicsProps,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<SdfBone>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shapes: Vec<SdfShape>,
}
```


#### `SceneSettings` (line 325)

```rust
pub struct SceneSettings {
    // Fill light
    #[serde(default = "default_fill_color", skip_serializing_if = "is_default_fill_color")]
    pub fill_color: [f32; 3],
    #[serde(default = "default_fill_intensity", skip_serializing_if = "is_default_fill_intensity")]
    pub fill_intensity: f32,
    // Back light
    #[serde(default = "default_back_color", skip_serializing_if = "is_default_back_color")]
    pub back_color: [f32; 3],
    #[serde(default = "default_back_intensity", skip_serializing_if = "is_default_back_intensity")]
    pub back_intensity: f32,
    // SSS
    #[serde(default = "default_sss_color", skip_serializing_if = "is_default_sss_color")]
    pub sss_color: [f32; 3],
    #[serde(default = "default_sss_intensity", skip_serializing_if = "is_default_sss_intensity")]
    pub sss_intensity: f32,
    // AO
    #[serde(default = "default_ao_intensity", skip_serializing_if = "is_default_ao_intensity")]
    pub ao_intensity: f32,
    // Shadows
    #[serde(default = "default_shadow_softness", skip_serializing_if = "is_default_shadow_softness")]
    pub shadow_softness: f32,
    // Post-processing
    #[serde(default = "default_vignette", skip_serializing_if = "is_default_vignette")]
    pub vignette_intensity: f32,
    // Environment
    #[serde(default = "default_ground_color", skip_serializing_if = "is_default_ground_color")]
    pub ground_color: [f32; 3],
    #[serde(default = "default_sun_sharpness", skip_serializing_if = "is_default_sun_sharpness")]
    pub sun_sharpness: f32,
    #[serde(default = "default_sun_brightness", skip_serializing_if = "is_default_sun_brightness")]
    pub sun_brightness: f32,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub show_environment: bool,
    // Physics
    #[serde(default = "default_gravity", skip_serializing_if = "is_default_gravity")]
    pub gravity: f32,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub ground_plane: bool,
}
```


#### `SdfScene` (line 422)

```rust
pub struct SdfScene {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub root_bone: SdfBone,
    #[serde(default, skip_serializing_if = "CombinationOp::is_default")]
    pub combination: CombinationOp,
    #[serde(default = "default_light_dir", skip_serializing_if = "is_default_light_dir")]
    pub light_dir: [f32; 3],
    #[serde(default, skip_serializing_if = "SceneSettings::is_default")]
    pub settings: SceneSettings,
}
```


#### `SceneInfo` (line 440)

```rust
pub struct SceneInfo {
    pub name: String,
    pub bone_count: usize,
    pub shape_count: usize,
}
```

Summary information about a scene.

#### `SceneInfo` (line 440)

```rust
pub struct SceneInfo {
    pub name: String,
    pub bone_count: usize,
    pub shape_count: usize,
}
```


#### `SdfShape` (line 527)

```rust
pub struct SdfShape {
    pub id: ShapeId,
    pub name: String,
    pub primitive: SdfPrimitive,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub visible: bool,
    #[serde(default, skip_serializing_if = "ShapeTransform::is_default")]
    pub transform: ShapeTransform,
    #[serde(default, skip_serializing_if = "ShapeMaterial::is_default")]
    pub material: ShapeMaterial,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<ShapeModifier>,
    #[serde(default, skip_serializing_if = "CombinationOp::is_default")]
    pub combination: CombinationOp,
}
```


#### `ShapeTransform` (line 561)

```rust
pub struct ShapeTransform {
    #[serde(default, skip_serializing_if = "is_zero_array")]
    pub translation: [f32; 3],
    #[serde(default, skip_serializing_if = "is_zero_array")]
    pub rotation: [f32; 3],
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub scale: f32,
}
```


#### `ShapeMaterial` (line 585)

```rust
pub struct ShapeMaterial {
    #[serde(default = "white", skip_serializing_if = "is_white")]
    pub color: [f32; 3],
    #[serde(default = "half", skip_serializing_if = "is_half")]
    pub roughness: f32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub metallic: f32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub fresnel_power: f32,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub color_mode: u32, // 0=solid, 1=cosine palette
    #[serde(default, skip_serializing_if = "is_zero_array")]
    pub palette_a: [f32; 3],
    #[serde(default, skip_serializing_if = "is_zero_array")]
    pub palette_b: [f32; 3],
    #[serde(default, skip_serializing_if = "is_zero_array")]
    pub palette_c: [f32; 3],
    #[serde(default, skip_serializing_if = "is_zero_array")]
    pub palette_d: [f32; 3],
    #[serde(default, skip_serializing_if = "is_zero")]
    pub noise_amplitude: f32,
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub noise_frequency: f32,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub noise_octaves: u32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub smooth_symmetry: f32,
}
```


### Enums

#### `ColliderApprox` (line 37)

```rust
pub enum ColliderApprox {
    Sphere { radius: f32 },
    Capsule { radius: f32, half_height: f32 },
    Box { half_extents: [f32; 3] },
}
```

Simple collider shape for physics approximation (no Bevy dependency).

#### `ColliderApprox` (line 37)

```rust
pub enum ColliderApprox {
    Sphere { radius: f32 },
    Capsule { radius: f32, half_height: f32 },
    Box { half_extents: [f32; 3] },
}
```


#### `SdfPrimitive` (line 544)

```rust
pub enum SdfPrimitive {
    Sphere { radius: f32 },
    Box { half_extents: [f32; 3] },
    RoundBox { half_extents: [f32; 3], rounding: f32 },
    Cylinder { height: f32, radius: f32 },
    CappedCone { height: f32, r1: f32, r2: f32 },
    Torus { major_radius: f32, minor_radius: f32 },
    Capsule { radius: f32, half_height: f32 },
    Plane { normal: [f32; 3], offset: f32 },
    Ellipsoid { radii: [f32; 3] },
    Octahedron { size: f32 },
    Pyramid { height: f32, base: f32 },
    HexPrism { height: f32, radius: f32 },
    RoundCone { r1: f32, r2: f32, height: f32 },
}
```


#### `CombinationOp` (line 639)

```rust
pub enum CombinationOp {
    Union,
    Intersection,
    Subtraction,
    SmoothUnion { k: f32 },
    SmoothIntersection { k: f32 },
    SmoothSubtraction { k: f32 },
    ChamferUnion { k: f32 },
    ChamferIntersection { k: f32 },
}
```


#### `ShapeModifier` (line 659)

```rust
pub enum ShapeModifier {
    Rounding(f32),
    Onion(f32),
    Twist(f32),
    Bend(f32),
    Elongation([f32; 3]),
    Repetition { period: [f32; 3], count: [u32; 3] },
}
```


### Functions

#### `root` (line 29)

```rust
    pub fn root() -> Self
```


#### `is_root` (line 30)

```rust
    pub fn is_root(&self) -> bool
```


#### `is_default` (line 60)

```rust
    pub fn is_default(&self) -> bool
```


#### `is_default` (line 80)

```rust
    pub fn is_default(&self) -> bool
```


#### `root` (line 102)

```rust
    pub fn root() -> Self
```


#### `new` (line 114)

```rust
    pub fn new(name: impl Into<String>) -> Self
```


#### `find_bone` (line 126)

```rust
    pub fn find_bone(&self, id: BoneId) -> Option<&SdfBone>
```


#### `find_bone_mut` (line 134)

```rust
    pub fn find_bone_mut(&mut self, id: BoneId) -> Option<&mut SdfBone>
```


#### `find_shape` (line 142)

```rust
    pub fn find_shape(&self, id: ShapeId) -> Option<(&SdfShape, BoneId)>
```


#### `find_shape_mut` (line 152)

```rust
    pub fn find_shape_mut(&mut self, id: ShapeId) -> Option<(&mut SdfShape, BoneId)>
```


#### `find_shape_by_name` (line 163)

```rust
    pub fn find_shape_by_name(&self, name: &str) -> Option<(&SdfShape, BoneId)>
```


#### `all_shapes` (line 173)

```rust
    pub fn all_shapes(&self) -> Vec<(&SdfShape, BoneId)>
```


#### `remove_shape` (line 184)

```rust
    pub fn remove_shape(&mut self, id: ShapeId) -> bool
```


#### `remove_bone` (line 194)

```rust
    pub fn remove_bone(&mut self, id: BoneId) -> bool
```


#### `duplicate_deep` (line 208)

```rust
    pub fn duplicate_deep(&self) -> Self
```

Deep clone with fresh UUIDs for this bone, all children, and all shapes.

#### `duplicate_deep` (line 208)

```rust
    pub fn duplicate_deep(&self) -> Self
```


#### `restore_names` (line 230)

```rust
    fn restore_names(&mut self, original: &SdfBone)
```

Restore original names after duplicate_deep (only top-level gets " Copy").

#### `find_bone_by_name` (line 240)

```rust
    pub fn find_bone_by_name(&self, name: &str) -> Option<&SdfBone>
```


#### `find_bone_by_name_mut` (line 248)

```rust
    pub fn find_bone_by_name_mut(&mut self, name: &str) -> Option<&mut SdfBone>
```


#### `reparent_shape` (line 257)

```rust
    pub fn reparent_shape(&mut self, shape_id: ShapeId, target_bone_id: BoneId) -> bool
```

Remove a shape from anywhere in the tree and add it to the target bone.

#### `reparent_shape` (line 257)

```rust
    pub fn reparent_shape(&mut self, shape_id: ShapeId, target_bone_id: BoneId) -> bool
```


#### `extract_shape` (line 267)

```rust
    pub fn extract_shape(&mut self, id: ShapeId) -> Option<SdfShape>
```


#### `reparent_bone` (line 279)

```rust
    pub fn reparent_bone(&mut self, bone_id: BoneId, target_bone_id: BoneId) -> bool
```

Remove a bone from anywhere in the tree and add it as a child of target.
Returns false if bone_id == target or target is a descendant of bone_id (cycle).

#### `reparent_bone` (line 279)

```rust
    pub fn reparent_bone(&mut self, bone_id: BoneId, target_bone_id: BoneId) -> bool
```


#### `extract_bone` (line 291)

```rust
    pub fn extract_bone(&mut self, id: BoneId) -> Option<SdfBone>
```


#### `bone_count` (line 302)

```rust
    pub fn bone_count(&self) -> usize
```

Count all descendant bones (not including self).

#### `bone_count` (line 302)

```rust
    pub fn bone_count(&self) -> usize
```


#### `shape_count` (line 307)

```rust
    pub fn shape_count(&self) -> usize
```

Count all shapes in this bone and all descendants.

#### `shape_count` (line 307)

```rust
    pub fn shape_count(&self) -> usize
```


#### `has_physics_bones` (line 312)

```rust
    pub fn has_physics_bones(bone: &SdfBone) -> bool
```

Check if any bone in this subtree has physics mass > 0.

#### `has_physics_bones` (line 312)

```rust
    pub fn has_physics_bones(bone: &SdfBone) -> bool
```


#### `reset_transform` (line 317)

```rust
    pub fn reset_transform(&mut self)
```


#### `is_default` (line 416)

```rust
    pub fn is_default(&self) -> bool
```


#### `new` (line 454)

```rust
    pub fn new(name: impl Into<String>) -> Self
```

Create an empty scene with a root bone and default light.

#### `new` (line 454)

```rust
    pub fn new(name: impl Into<String>) -> Self
```


#### `info` (line 465)

```rust
    pub fn info(&self) -> SceneInfo
```


#### `tree_string` (line 474)

```rust
    pub fn tree_string(&self) -> String
```

ASCII tree representation of the scene hierarchy.

#### `tree_string` (line 474)

```rust
    pub fn tree_string(&self) -> String
```


#### `default_scene` (line 507)

```rust
    pub fn default_scene() -> Self
```


#### `is_default` (line 571)

```rust
    pub fn is_default(&self) -> bool
```


#### `is_default` (line 615)

```rust
    pub fn is_default(&self) -> bool
```


#### `is_default` (line 655)

```rust
    pub fn is_default(&self) -> bool
```


#### `duplicate` (line 669)

```rust
    pub fn duplicate(&self) -> Self
```


#### `reset_transform` (line 676)

```rust
    pub fn reset_transform(&mut self)
```


#### `clear_modifiers` (line 680)

```rust
    pub fn clear_modifiers(&mut self)
```


#### `default_sphere` (line 684)

```rust
    pub fn default_sphere() -> Self
```


#### `new` (line 697)

```rust
    pub fn new(name: impl Into<String>, primitive: SdfPrimitive) -> Self
```


#### `label` (line 712)

```rust
    pub fn label(&self) -> &'static str
```


#### `default_for` (line 730)

```rust
    pub fn default_for(name: &str) -> Self
```


## `crates/litsdf_core/src/scene.rs`

Scene computation: bone world transforms, flattening bone tree to flat shape array.
No Bevy dependency — uses glam directly.

### Structs

#### `FlatShape` (line 37)

```rust
pub struct FlatShape {
    pub primitive_type: u32,
    pub params: Vec4,
    pub combination_op: u32,
    pub smooth_k: f32,
    pub translation: Vec3,
    pub rotation: Vec3,
    pub scale: f32,
    pub color: Vec3,
    pub roughness: f32,
    pub metallic: f32,
    pub fresnel_power: f32,
    pub color_mode: u32,
    pub palette_a: Vec3,
    pub palette_b: Vec3,
    pub palette_c: Vec3,
    pub palette_d: Vec3,
    pub modifier_flags: u32,
    pub rounding: f32,
    pub onion_thickness: f32,
    pub twist_amount: f32,
    pub bend_amount: f32,
    pub elongation: Vec3,
    pub rep_period: Vec3,
    pub noise_amplitude: f32,
    pub noise_frequency: f32,
    pub noise_octaves: u32,
    pub smooth_symmetry: f32,
}
```

A flattened shape ready for GPU encoding.

#### `FlatShape` (line 37)

```rust
pub struct FlatShape {
    pub primitive_type: u32,
    pub params: Vec4,
    pub combination_op: u32,
    pub smooth_k: f32,
    pub translation: Vec3,
    pub rotation: Vec3,
    pub scale: f32,
    pub color: Vec3,
    pub roughness: f32,
    pub metallic: f32,
    pub fresnel_power: f32,
    pub color_mode: u32,
    pub palette_a: Vec3,
    pub palette_b: Vec3,
    pub palette_c: Vec3,
    pub palette_d: Vec3,
    pub modifier_flags: u32,
    pub rounding: f32,
    pub onion_thickness: f32,
    pub twist_amount: f32,
    pub bend_amount: f32,
    pub elongation: Vec3,
    pub rep_period: Vec3,
    pub noise_amplitude: f32,
    pub noise_frequency: f32,
    pub noise_octaves: u32,
    pub smooth_symmetry: f32,
}
```


### Functions

#### `compute_bone_world_transforms` (line 12)

```rust
pub fn compute_bone_world_transforms(
    bone: &SdfBone,
    parent: Mat4,
    overrides: &HashMap<BoneId, ShapeTransform>,
) -> HashMap<BoneId, Mat4>
```

Compute world-space transform for every bone in the tree.
The `overrides` map allows external systems (e.g., node graphs) to provide
per-bone transform overrides computed before this function is called.

#### `compute_bone_world_transforms` (line 12)

```rust
pub fn compute_bone_world_transforms(
    bone: &SdfBone,
    parent: Mat4,
    overrides: &HashMap<BoneId, ShapeTransform>,
) -> HashMap<BoneId, Mat4>
```


#### `flatten_bone_tree` (line 68)

```rust
pub fn flatten_bone_tree(
    bone: &SdfBone,
    world_transforms: &HashMap<BoneId, Mat4>,
    output: &mut Vec<FlatShape>,
)
```

Flatten bone tree into a linear list of world-space shapes.

#### `flatten_bone_tree` (line 68)

```rust
pub fn flatten_bone_tree(
    bone: &SdfBone,
    world_transforms: &HashMap<BoneId, Mat4>,
    output: &mut Vec<FlatShape>,
)
```


### Module Dependencies

```rust
use crate::models::*;
use crate::models::*;
```

## `crates/litsdf_core/src/sdf.rs`

CPU-side SDF primitive evaluation functions.
These mirror the WGSL shader implementations for picking and other CPU uses.

### Functions

#### `sd_sphere` (line 7)

```rust
pub fn sd_sphere(p: Vec3, r: f32) -> f32
```


#### `sd_box` (line 11)

```rust
pub fn sd_box(p: Vec3, b: Vec3) -> f32
```


#### `sd_round_box` (line 16)

```rust
pub fn sd_round_box(p: Vec3, b: Vec3, r: f32) -> f32
```


#### `sd_cylinder` (line 21)

```rust
pub fn sd_cylinder(p: Vec3, h: f32, r: f32) -> f32
```


#### `sd_capped_cone` (line 26)

```rust
pub fn sd_capped_cone(p: Vec3, h: f32, r1: f32, r2: f32) -> f32
```


#### `sd_torus` (line 39)

```rust
pub fn sd_torus(p: Vec3, major: f32, minor: f32) -> f32
```


#### `sd_capsule` (line 44)

```rust
pub fn sd_capsule(p: Vec3, r: f32, h: f32) -> f32
```


#### `sd_plane` (line 50)

```rust
pub fn sd_plane(p: Vec3, n: Vec3, d: f32) -> f32
```


#### `sd_ellipsoid` (line 54)

```rust
pub fn sd_ellipsoid(p: Vec3, r: Vec3) -> f32
```


#### `rotate_point` (line 61)

```rust
pub fn rotate_point(p: Vec3, euler: Vec3) -> Vec3
```


#### `sd_octahedron` (line 72)

```rust
pub fn sd_octahedron(p: Vec3, s: f32) -> f32
```


#### `sd_pyramid` (line 83)

```rust
pub fn sd_pyramid(p: Vec3, h: f32, base: f32) -> f32
```


#### `sd_hex_prism` (line 90)

```rust
pub fn sd_hex_prism(p: Vec3, h: f32, r: f32) -> f32
```


#### `sd_round_cone` (line 96)

```rust
pub fn sd_round_cone(p: Vec3, r1: f32, r2: f32, h: f32) -> f32
```


#### `eval_primitive` (line 107)

```rust
pub fn eval_primitive(p: Vec3, prim: &SdfPrimitive) -> f32
```

Evaluate an SDF primitive at a point in the primitive's local space.

#### `eval_primitive` (line 107)

```rust
pub fn eval_primitive(p: Vec3, prim: &SdfPrimitive) -> f32
```


### Module Dependencies

```rust
use crate::models::SdfPrimitive;
```

## `crates/litsdf_core/src/persistence.rs`

### Functions

#### `scenes_dir` (line 5)

```rust
pub fn scenes_dir() -> PathBuf
```


#### `save_scene` (line 12)

```rust
pub fn save_scene(scene: &SdfScene, path: &Path) -> Result<(), String>
```


#### `load_scene` (line 20)

```rust
pub fn load_scene(path: &Path) -> Result<SdfScene, String>
```


#### `list_scenes` (line 42)

```rust
pub fn list_scenes(dir: &Path) -> Vec<String>
```


### Module Dependencies

```rust
use crate::models::SdfScene;
use crate::models::SdfScene;
```

## `crates/litsdf_render/src/lib.rs`

### Structs

#### `SdfRenderPlugin` (line 11)

```rust
pub struct SdfRenderPlugin;
```


## `crates/litsdf_render/src/shader.rs`

### Structs

#### `SdfMaterial` (line 11)

```rust
pub struct SdfMaterial {
    #[uniform(0)]
    pub params: SdfShaderParams,
}
```


#### `SdfShaderParams` (line 33)

```rust
pub struct SdfShaderParams {
    pub shape_count: u32,
    pub time: f32,
    pub _pad_h: Vec2,
    pub light_dir: Vec3,
    pub _pad_l: f32,
    pub shapes: [ShaderShape; MAX_SHAPES],
    // Scene settings (after shapes array to preserve array offset)
    pub fill_color: Vec3,
    pub fill_intensity: f32,
    pub back_color: Vec3,
    pub back_intensity: f32,
    pub sss_color: Vec3,
    pub sss_intensity: f32,
    pub ao_intensity: f32,
    pub shadow_softness: f32,
    pub vignette_intensity: f32,
    pub ground_color: Vec3,
    pub sun_sharpness: f32,
    pub sun_brightness: f32,
    pub show_environment: u32,
    pub _pad_s: Vec2,
}
```


#### `ShaderShape` (line 58)

```rust
pub struct ShaderShape {
    // Geometry
    pub primitive_type: u32,
    pub combination_op: u32,
    pub smooth_k: f32,
    pub _pad0: f32,
    pub params: Vec4,
    pub translation: Vec3,
    pub _pad1: f32,
    pub rotation: Vec3,
    pub scale: f32,
    // Material
    pub color: Vec3,
    pub roughness: f32,
    pub metallic: f32,
    pub fresnel_power: f32,
    pub color_mode: u32,
    pub _pad3: f32,
    pub palette_a: Vec3,
    pub _pad4: f32,
    pub palette_b: Vec3,
    pub _pad5: f32,
    pub palette_c: Vec3,
    pub _pad6: f32,
    pub palette_d: Vec3,
    pub _pad7: f32,
    // Modifiers
    pub modifier_flags: u32,
    pub rounding: f32,
    pub onion_thickness: f32,
    pub twist_amount: f32,
    pub bend_amount: f32,
    pub _pad_mod0: Vec3,
    pub elongation: Vec3,
    pub _pad_mod1: f32,
    pub rep_period: Vec3,
    pub _pad_mod2: f32,
    // Noise
    pub noise_amplitude: f32,
    pub noise_frequency: f32,
    pub noise_octaves: u32,
    pub smooth_symmetry: f32,
}
```


### Constants

#### `MAX_SHAPES` (line 8)

```rust
pub const MAX_SHAPES: usize = 32;
```


## `crates/litsdf_render/src/scene_sync.rs`

### Structs

#### `BonePhysicsReading` (line 10)

```rust
pub struct BonePhysicsReading {
    pub position: [f32; 3],
    pub linear_velocity: [f32; 3],
    pub angular_velocity: [f32; 3],
    // Collision data
    pub is_colliding: bool,
    pub contact_count: u32,
    pub contact_normal: [f32; 3],
    // Raycast data (downward ray)
    pub ray_hit_distance: f32, // f32::MAX if no hit
    pub ray_hit_normal: [f32; 3],
}
```

Physics state readable by node graphs (no Bevy types).

#### `BonePhysicsReading` (line 10)

```rust
pub struct BonePhysicsReading {
    pub position: [f32; 3],
    pub linear_velocity: [f32; 3],
    pub angular_velocity: [f32; 3],
    // Collision data
    pub is_colliding: bool,
    pub contact_count: u32,
    pub contact_normal: [f32; 3],
    // Raycast data (downward ray)
    pub ray_hit_distance: f32, // f32::MAX if no hit
    pub ray_hit_normal: [f32; 3],
}
```


#### `BoneForceOutputs` (line 25)

```rust
pub struct BoneForceOutputs {
    pub force: [f32; 3],
    pub torque: [f32; 3],
}
```

Force/torque outputs from node graphs to apply to Avian entities.

#### `BoneForceOutputs` (line 25)

```rust
pub struct BoneForceOutputs {
    pub force: [f32; 3],
    pub torque: [f32; 3],
}
```


#### `SdfSceneState` (line 31)

```rust
pub struct SdfSceneState {
    pub scene: SdfScene,
    pub selected_shape: Option<ShapeId>,
    pub selected_bone: Option<BoneId>,
    pub show_bone_gizmos: bool,
    pub dirty: bool,
    pub topology_hash: u64,
    pub use_avian: bool,
    pub physics_paused: bool,
    pub physics_readings: HashMap<BoneId, BonePhysicsReading>,
    pub force_outputs: HashMap<BoneId, BoneForceOutputs>,
}
```


#### `SdfBoundingEntity` (line 62)

```rust
pub struct SdfBoundingEntity;
```


### Functions

#### `setup_initial_scene` (line 64)

```rust
pub fn setup_initial_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SdfMaterial>>,
    state: Res<SdfSceneState>,
)
```


#### `sync_scene_to_shader` (line 78)

```rust
pub fn sync_scene_to_shader(
    mut state: ResMut<SdfSceneState>,
    mut materials: ResMut<Assets<SdfMaterial>>,
    query: Query<&MeshMaterial3d<SdfMaterial>, With<SdfBoundingEntity>>,
    time: Res<Time>,
)
```


#### `build_shader_params` (line 106)

```rust
pub fn build_shader_params(scene_data: &SdfScene) -> SdfShaderParams
```


### Module Dependencies

```rust
use crate::shader::{SdfMaterial, SdfShaderParams, ShaderShape, MAX_SHAPES};
```

## `crates/litsdf_render/src/camera.rs`

### Structs

#### `OrbitCamera` (line 7)

```rust
pub struct OrbitCamera {
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub target: Vec3,
    /// Set by editor to trigger a one-shot frame-selection move.
    pub frame_target: Option<Vec3>,
    /// Set by editor to toggle orthographic projection.
    pub toggle_ortho: bool,
    /// Viewport center offset [x, y] in normalized screen coords (-0.5 to 0.5).
    /// Used to shift the rendered view to match the visible area between UI panels.
    pub viewport_offset: [f32; 2],
}
```


### Functions

#### `setup_camera` (line 21)

```rust
pub fn setup_camera(mut commands: Commands)
```


#### `orbit_camera` (line 42)

```rust
pub fn orbit_camera(
    mut mouse_motion: MessageReader<MouseMotion>,
    mut scroll: MessageReader<MouseWheel>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut OrbitCamera, &mut Transform, &mut Projection)>,
    egui_wants: Option<Res<EguiWantsInput>>,
    drag_state: Option<Res<crate::picking::DragState>>,
)
```


## `crates/litsdf_render/src/gizmos.rs`

### Functions

#### `draw_bone_gizmos` (line 9)

```rust
pub fn draw_bone_gizmos(
    mut gizmos: Gizmos,
    state: Res<SdfSceneState>,
)
```


#### `draw_compass` (line 52)

```rust
pub fn draw_compass(
    mut contexts: bevy_egui::EguiContexts,
    camera: Query<&Transform, With<OrbitCamera>>,
)
```


### Module Dependencies

```rust
use crate::camera::OrbitCamera;
use crate::scene_sync::SdfSceneState;
```

## `crates/litsdf_render/src/picking.rs`

### Structs

#### `ClickTracker` (line 119)

```rust
pub struct ClickTracker { press_pos: Option<Vec2> }
```


#### `DragState` (line 122)

```rust
pub struct DragState {
    pub active: bool,
    pub axis: Vec3,         // local axis direction in world space (for projection)
    pub axis_index: usize,  // 0=X, 1=Y, 2=Z (which component to modify)
    pub start_world_pos: Vec3,
    pub start_value: [f32; 3],
    pub start_cursor: Vec2,
    pub screen_axis: Vec2,
}
```


### Enums

#### `GizmoMode` (line 19)

```rust
pub enum GizmoMode {
    #[default]
    Translate,
    Rotate,
    Scale,
    Elongation,
    Repetition,
}
```


### Functions

#### `label` (line 29)

```rust
    pub fn label(&self) -> &'static str
```


#### `pick_shape` (line 90)

```rust
pub fn pick_shape(ray: Ray3d, scene: &SdfScene) -> Option<(ShapeId, BoneId)>
```


#### `pick_system` (line 132)

```rust
pub fn pick_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<OrbitCamera>>,
    mut scene: ResMut<SdfSceneState>,
    egui_wants: Option<Res<EguiWantsInput>>,
    mut tracker: ResMut<ClickTracker>,
)
```


#### `get_local_axes` (line 170)

```rust
fn get_local_axes(scene: &SdfSceneState) -> [Vec3; 3]
```

Get the local coordinate axes for the selected shape/bone in world space.
These are the parent bone's world rotation applied to X/Y/Z unit vectors.

#### `draw_handles` (line 210)

```rust
pub fn draw_handles(
    mut gizmos: Gizmos,
    scene: Res<SdfSceneState>,
    drag: Res<DragState>,
    mode: Res<GizmoMode>,
)
```


#### `drag_system` (line 290)

```rust
pub fn drag_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<OrbitCamera>>,
    mut scene: ResMut<SdfSceneState>,
    mut drag: ResMut<DragState>,
    egui_wants: Option<Res<EguiWantsInput>>,
    mode: Res<GizmoMode>,
)
```


#### `get_selected_world_pos` (line 441)

```rust
pub fn get_selected_world_pos(scene: &SdfSceneState) -> Option<Vec3>
```


### Module Dependencies

```rust
use crate::camera::OrbitCamera;
use crate::scene_sync::SdfSceneState;
```

## `crates/litsdf_cli/src/commands/mod.rs`

### Functions

#### `load` (line 11)

```rust
pub fn load(path: &Path) -> Result<SdfScene, String>
```

Load a scene from a YAML file.

#### `load` (line 11)

```rust
pub fn load(path: &Path) -> Result<SdfScene, String>
```


#### `save` (line 16)

```rust
pub fn save(scene: &SdfScene, path: &Path) -> Result<(), String>
```

Save a scene to a YAML file.

#### `save` (line 16)

```rust
pub fn save(scene: &SdfScene, path: &Path) -> Result<(), String>
```


#### `mutate` (line 21)

```rust
pub fn mutate(path: &Path, f: impl FnOnce(&mut SdfScene) -> Result<String, String>) -> Result<(), String>
```

Load, apply a mutation, save back.

#### `mutate` (line 21)

```rust
pub fn mutate(path: &Path, f: impl FnOnce(&mut SdfScene) -> Result<String, String>) -> Result<(), String>
```


## `crates/litsdf_cli/src/commands/scene.rs`

### Enums

#### `SceneCmd` (line 7)

```rust
pub enum SceneCmd {
    /// Create a new empty scene
    New {
        /// Scene name
        name: String,
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Rename a scene
    Rename {
        /// Scene file
        file: PathBuf,
        /// New name
        name: String,
    },
    /// Set light direction
    Light {
        /// Scene file
        file: PathBuf,
        /// X component
        x: f32,
        /// Y component
        y: f32,
        /// Z component
        z: f32,
    },
    /// Show scene info
    Info {
        /// Scene file
        file: PathBuf,
    },
    /// Print scene tree structure
    Tree {
        /// Scene file
        file: PathBuf,
    },
}
```


### Functions

#### `run` (line 46)

```rust
pub fn run(cmd: SceneCmd) -> Result<(), String>
```


## `crates/litsdf_cli/src/commands/bone.rs`

### Enums

#### `BoneCmd` (line 6)

```rust
pub enum BoneCmd {
    /// Add a new bone
    Add {
        /// Scene file
        file: PathBuf,
        /// Parent bone name
        #[arg(long, default_value = "Root")]
        parent: String,
        /// Bone name
        #[arg(long)]
        name: String,
        /// Translation x,y,z
        #[arg(long, value_parser = parse_vec3)]
        translate: Option<[f32; 3]>,
    },
    /// Remove a bone (reparents children and shapes to parent)
    Remove {
        /// Scene file
        file: PathBuf,
        /// Bone name
        #[arg(long)]
        name: String,
        /// Also delete all children and shapes recursively
        #[arg(long)]
        recursive: bool,
    },
    /// Rename a bone
    Rename {
        /// Scene file
        file: PathBuf,
        /// Current bone name
        #[arg(long)]
        name: String,
        /// New name
        #[arg(long)]
        to: String,
    },
    /// Set bone translation
    Move {
        /// Scene file
        file: PathBuf,
        /// Bone name
        #[arg(long)]
        name: String,
        /// Translation x,y,z
        #[arg(long, value_parser = parse_vec3)]
        translate: [f32; 3],
    },
    /// Set bone rotation (degrees)
    Rotate {
        /// Scene file
        file: PathBuf,
        /// Bone name
        #[arg(long)]
        name: String,
        /// Rotation rx,ry,rz in degrees
        #[arg(long, value_parser = parse_vec3)]
        rotation: [f32; 3],
    },
    /// Deep-duplicate a bone with all its children and shapes
    Duplicate {
        /// Scene file
        file: PathBuf,
        /// Bone name to duplicate
        #[arg(long)]
        name: String,
        /// Name for the copy
        #[arg(long)]
        r#as: Option<String>,
    },
    /// Move a bone to a new parent
    Reparent {
        /// Scene file
        file: PathBuf,
        /// Bone name to move
        #[arg(long)]
        name: String,
        /// New parent bone name
        #[arg(long)]
        parent: String,
    },
    /// List all bones
    List {
        /// Scene file
        file: PathBuf,
    },
}
```


### Functions

#### `run` (line 112)

```rust
pub fn run(cmd: BoneCmd) -> Result<(), String>
```


## `crates/litsdf_cli/src/commands/shape.rs`

### Enums

#### `ShapeCmd` (line 6)

```rust
pub enum ShapeCmd {
    /// Add a shape to a bone
    Add {
        /// Scene file
        file: PathBuf,
        /// Bone name
        #[arg(long)]
        bone: String,
        /// Primitive type (Sphere, Box, RoundBox, Cylinder, CappedCone, Torus, Capsule, Plane, Ellipsoid)
        #[arg(long = "type")]
        prim_type: String,
        /// Shape name
        #[arg(long)]
        name: Option<String>,
        /// Primitive parameters a,b,c,d
        #[arg(long, value_parser = parse_vec4)]
        params: Option<[f32; 4]>,
    },
    /// Remove a shape
    Remove {
        /// Scene file
        file: PathBuf,
        /// Shape name
        #[arg(long)]
        name: String,
    },
    /// Set shape properties
    Set {
        /// Scene file
        file: PathBuf,
        /// Shape name
        #[arg(long)]
        name: String,
        /// Translation x,y,z
        #[arg(long, value_parser = parse_vec3)]
        translate: Option<[f32; 3]>,
        /// Rotation rx,ry,rz in degrees
        #[arg(long, value_parser = parse_vec3)]
        rotate: Option<[f32; 3]>,
        /// Uniform scale
        #[arg(long)]
        scale: Option<f32>,
        /// Color r,g,b (0-1)
        #[arg(long, value_parser = parse_vec3)]
        color: Option<[f32; 3]>,
        /// Roughness (0-1)
        #[arg(long)]
        roughness: Option<f32>,
        /// Metallic (0-1)
        #[arg(long)]
        metallic: Option<f32>,
        /// Fresnel/rim power
        #[arg(long)]
        fresnel: Option<f32>,
        /// Combination operation (Union, Intersection, Subtraction, SmoothUnion, SmoothIntersection, SmoothSubtraction)
        #[arg(long)]
        combine: Option<String>,
        /// Blend radius for smooth operations
        #[arg(long)]
        blend_k: Option<f32>,
    },
    /// Change primitive type and parameters
    SetType {
        /// Scene file
        file: PathBuf,
        /// Shape name
        #[arg(long)]
        name: String,
        /// Primitive type
        #[arg(long = "type")]
        prim_type: String,
        /// Primitive parameters a,b,c,d
        #[arg(long, value_parser = parse_vec4)]
        params: Option<[f32; 4]>,
    },
    /// Duplicate a shape
    Duplicate {
        /// Scene file
        file: PathBuf,
        /// Shape name
        #[arg(long)]
        name: String,
        /// Name for the copy
        #[arg(long)]
        r#as: Option<String>,
    },
    /// Move a shape to a different bone
    Reparent {
        /// Scene file
        file: PathBuf,
        /// Shape name
        #[arg(long)]
        name: String,
        /// Target bone name
        #[arg(long)]
        bone: String,
    },
    /// Set color mode and palette
    ColorMode {
        /// Scene file
        file: PathBuf,
        /// Shape name
        #[arg(long)]
        name: String,
        /// Mode: solid, palette, noise
        #[arg(long)]
        mode: String,
        /// Palette A (bias) r,g,b
        #[arg(long, value_parser = parse_vec3)]
        palette_a: Option<[f32; 3]>,
        /// Palette B (amplitude) r,g,b
        #[arg(long, value_parser = parse_vec3)]
        palette_b: Option<[f32; 3]>,
        /// Palette C (frequency) r,g,b
        #[arg(long, value_parser = parse_vec3)]
        palette_c: Option<[f32; 3]>,
        /// Palette D (phase) r,g,b
        #[arg(long, value_parser = parse_vec3)]
        palette_d: Option<[f32; 3]>,
    },
    /// Set noise parameters
    Noise {
        /// Scene file
        file: PathBuf,
        /// Shape name
        #[arg(long)]
        name: String,
        /// Noise amplitude
        #[arg(long)]
        amp: Option<f32>,
        /// Noise frequency
        #[arg(long)]
        freq: Option<f32>,
        /// Noise octaves
        #[arg(long)]
        oct: Option<u32>,
    },
    /// List all shapes
    List {
        /// Scene file
        file: PathBuf,
    },
}
```


### Functions

#### `run` (line 202)

```rust
pub fn run(cmd: ShapeCmd) -> Result<(), String>
```


## `crates/litsdf_cli/src/commands/modifier.rs`

### Enums

#### `ModifierCmd` (line 6)

```rust
pub enum ModifierCmd {
    /// Set a modifier on a shape (replaces existing of same type)
    Set {
        /// Scene file
        file: PathBuf,
        /// Shape name
        #[arg(long)]
        shape: String,
        /// Rounding radius
        #[arg(long)]
        rounding: Option<f32>,
        /// Onion shell thickness
        #[arg(long)]
        onion: Option<f32>,
        /// Twist amount
        #[arg(long)]
        twist: Option<f32>,
        /// Bend amount
        #[arg(long)]
        bend: Option<f32>,
        /// Elongation x,y,z
        #[arg(long, value_parser = parse_vec3)]
        elongate: Option<[f32; 3]>,
        /// Repetition period x,y,z
        #[arg(long, value_parser = parse_vec3)]
        repeat: Option<[f32; 3]>,
    },
    /// Clear all modifiers from a shape
    Clear {
        /// Scene file
        file: PathBuf,
        /// Shape name
        #[arg(long)]
        shape: String,
    },
    /// List modifiers on a shape
    List {
        /// Scene file
        file: PathBuf,
        /// Shape name
        #[arg(long)]
        shape: String,
    },
}
```


### Functions

#### `set_modifier` (line 70)

```rust
fn set_modifier(modifiers: &mut Vec<ShapeModifier>, new: ShapeModifier)
```

Replace or add a modifier, removing any existing modifier of the same discriminant.

#### `run` (line 76)

```rust
pub fn run(cmd: ModifierCmd) -> Result<(), String>
```


## `crates/litsdf_editor/src/lib.rs`

### Structs

#### `SdfEditorPlugin` (line 10)

```rust
pub struct SdfEditorPlugin;
```


## `crates/litsdf_editor/src/ui/mod.rs`

### Structs

#### `EditorUi` (line 37)

```rust
pub struct EditorUi {
    pub md: app::LituiApp,
    pub(crate) prev_on_delete_shape: u32,
    pub(crate) prev_on_edit_yaml: u32,
    pub(crate) prev_on_apply_yaml: u32,
    pub(crate) prev_on_confirm_add: u32,
    pub(crate) prev_on_reset_transform: u32,

    pub(crate) prev_on_confirm_save: u32,
    pub(crate) prev_pick_file_counts: Vec<u32>,
    pub(crate) file_browser_save_mode: bool,
    pub(crate) prev_selected_shape: Option<ShapeId>,
    pub(crate) prev_selected_bone: Option<BoneId>,
    pub(crate) prev_shape_clicks: HashMap<ShapeId, u32>,
    pub(crate) shape_order: Vec<ShapeId>,
    // Node editor state
    pub(crate) show_node_editor: bool,
    pub(crate) node_graphs: HashMap<ShapeId, Snarl<SdfNode>>,
    pub(crate) bone_graphs: HashMap<BoneId, Snarl<SdfNode>>,
    pub(crate) node_style: SnarlStyle,
    // Graph undo (separate from scene undo)
    pub(crate) graph_undo_stack: Vec<(ShapeId, Snarl<SdfNode>)>,
    pub(crate) rename_state: Option<(tree::RenameTarget, String)>,
    pub(crate) clipboard: Option<litsdf_core::models::SdfShape>,
    // Animation / physics playback
    pub(crate) animation_time: f32,
    pub(crate) animation_playing: bool,
    pub(crate) animation_enabled: bool,
    pub(crate) physics_enabled: bool,
    pub(crate) rest_pose: Option<litsdf_core::models::SdfBone>,
    pub(crate) physics_states: HashMap<BoneId, litsdf_core::physics::BonePhysicsState>,
    pub(crate) show_help: bool,
    pub(crate) show_description: bool,
}
```


#### `TreePanelActions` (line 110)

```rust
struct TreePanelActions {
    select_bone: Option<BoneId>,
    select_shape: Option<ShapeId>,
    add_bone: bool,
    add_shape: bool,
    delete_selected: bool,
    show_gizmos: Option<bool>,
    context_action: tree::ContextAction,
}
```

Actions collected from the left panel to apply after rendering.

### Functions

#### `editor_ui` (line 124)

```rust
pub fn editor_ui(
    mut contexts: bevy_egui::EguiContexts,
    mut ui: ResMut<EditorUi>,
    mut scene: ResMut<SdfSceneState>,
    mut undo_history: ResMut<crate::undo::UndoHistory>,
    drag_state: Res<litsdf_render::picking::DragState>,
    mut gizmo_mode: ResMut<litsdf_render::picking::GizmoMode>,
    mut camera_query: Query<(&mut OrbitCamera, &mut Camera)>,
    time: Res<Time>,
    mut pending_graphs: Option<ResMut<crate::demos::PendingGraphs>>,
)
```


### Module Dependencies

```rust
use crate::nodes::{SdfNode, SdfNodeViewer};
```

## `crates/litsdf_editor/src/ui/populate.rs`

### Functions

#### `populate_bone_shapes` (line 11)

```rust
pub fn populate_bone_shapes(ui: &mut EditorUi, scene: &SdfSceneState)
```


#### `populate_shape_properties` (line 34)

```rust
pub fn populate_shape_properties(ui: &mut EditorUi, scene: &SdfSceneState)
```


#### `populate_bone_properties` (line 85)

```rust
pub fn populate_bone_properties(ui: &mut EditorUi, scene: &SdfSceneState)
```


#### `populate_file_browser` (line 111)

```rust
pub fn populate_file_browser(ui: &mut EditorUi)
```


## `crates/litsdf_editor/src/ui/sync.rs`

### Functions

#### `sync_shape_properties` (line 10)

```rust
pub fn sync_shape_properties(ui: &mut EditorUi, scene: &mut SdfSceneState)
```


#### `sync_bone_properties` (line 118)

```rust
pub fn sync_bone_properties(ui: &mut EditorUi, scene: &mut SdfSceneState)
```


## `crates/litsdf_editor/src/ui/handlers.rs`

### Functions

#### `handle_confirm_add` (line 7)

```rust
pub fn handle_confirm_add(ui: &mut EditorUi, scene: &mut SdfSceneState)
```


#### `handle_delete_shape` (line 24)

```rust
pub fn handle_delete_shape(ui: &mut EditorUi, scene: &mut SdfSceneState)
```


#### `handle_edit_yaml` (line 35)

```rust
pub fn handle_edit_yaml(ui: &mut EditorUi, scene: &SdfSceneState)
```


#### `handle_apply_yaml` (line 47)

```rust
pub fn handle_apply_yaml(ui: &mut EditorUi, scene: &mut SdfSceneState)
```


#### `handle_shape_selection` (line 68)

```rust
pub fn handle_shape_selection(ui: &mut EditorUi, scene: &mut SdfSceneState) -> bool
```


#### `handle_reset_transform` (line 86)

```rust
pub fn handle_reset_transform(ui: &mut EditorUi, scene: &mut SdfSceneState)
```


#### `handle_save_load` (line 100)

```rust
pub fn handle_save_load(_ui: &mut EditorUi)
```


#### `handle_file_browser` (line 104)

```rust
pub fn handle_file_browser(ui: &mut EditorUi, scene: &mut SdfSceneState)
```


## `crates/litsdf_editor/src/ui/helpers.rs`

### Functions

#### `prim_to_index` (line 9)

```rust
pub fn prim_to_index(p: &SdfPrimitive) -> usize
```


#### `prim_params` (line 27)

```rust
pub fn prim_params(p: &SdfPrimitive) -> (f64, f64, f64, f64)
```


#### `set_prim_params` (line 45)

```rust
pub fn set_prim_params(p: &mut SdfPrimitive, a: f32, b: f32, c: f32, d: f32)
```


#### `combo_to_index` (line 63)

```rust
pub fn combo_to_index(c: &CombinationOp) -> usize
```


#### `combo_smooth_k` (line 76)

```rust
pub fn combo_smooth_k(c: &CombinationOp) -> f32
```


#### `index_to_combo` (line 87)

```rust
pub fn index_to_combo(i: usize, k: f32) -> CombinationOp
```


### Constants

#### `PRIM_NAMES` (line 3)

```rust
pub const PRIM_NAMES: &[&str] = &[
    "Sphere", "Box", "RoundBox", "Cylinder", "CappedCone",
    "Torus", "Capsule", "Plane", "Ellipsoid",
    "Octahedron", "Pyramid", "HexPrism", "RoundCone",
];
```


## `crates/litsdf_editor/src/ui/tree.rs`

### Structs

#### `TreeResult` (line 44)

```rust
pub struct TreeResult {
    pub action: TreeAction,
    pub context: ContextAction,
}
```


### Enums

#### `DragPayload` (line 6)

```rust
pub enum DragPayload {
    Shape(ShapeId),
    Bone(BoneId),
}
```

Drag-and-drop payload for reparenting.

#### `DragPayload` (line 6)

```rust
pub enum DragPayload {
    Shape(ShapeId),
    Bone(BoneId),
}
```


#### `TreeAction` (line 12)

```rust
pub enum TreeAction {
    None,
    SelectBone(BoneId),
    SelectShape(ShapeId, BoneId),
}
```

Selection action returned by the tree renderer.

#### `TreeAction` (line 12)

```rust
pub enum TreeAction {
    None,
    SelectBone(BoneId),
    SelectShape(ShapeId, BoneId),
}
```


#### `RenameTarget` (line 20)

```rust
pub enum RenameTarget {
    Bone(BoneId),
    Shape(ShapeId),
}
```

Identifies an item being renamed inline.

#### `RenameTarget` (line 20)

```rust
pub enum RenameTarget {
    Bone(BoneId),
    Shape(ShapeId),
}
```


#### `ContextAction` (line 26)

```rust
pub enum ContextAction {
    None,
    AddChildBone(BoneId),
    AddShapeToBone(BoneId, String),
    DuplicateBone(BoneId),
    DuplicateShape(ShapeId),
    DeleteBone(BoneId),
    DeleteBoneRecursive(BoneId),
    DeleteShape(ShapeId),
    ToggleBoneVisibility(BoneId),
    ToggleShapeVisibility(ShapeId),
    ReparentBone { bone: BoneId, new_parent: BoneId },
    ReparentShape { shape: ShapeId, new_bone: BoneId },
    RenameBone(BoneId, String),
    RenameShape(ShapeId, String),
    ToggleBonePhysics(BoneId),
}
```

Context menu action returned by the tree renderer.

#### `ContextAction` (line 26)

```rust
pub enum ContextAction {
    None,
    AddChildBone(BoneId),
    AddShapeToBone(BoneId, String),
    DuplicateBone(BoneId),
    DuplicateShape(ShapeId),
    DeleteBone(BoneId),
    DeleteBoneRecursive(BoneId),
    DeleteShape(ShapeId),
    ToggleBoneVisibility(BoneId),
    ToggleShapeVisibility(ShapeId),
    ReparentBone { bone: BoneId, new_parent: BoneId },
    ReparentShape { shape: ShapeId, new_bone: BoneId },
    RenameBone(BoneId, String),
    RenameShape(ShapeId, String),
    ToggleBonePhysics(BoneId),
}
```


### Functions

#### `collect_bone_list` (line 50)

```rust
fn collect_bone_list(bone: &SdfBone, out: &mut Vec<(BoneId, String)>)
```

Flat list of (BoneId, name) for reparent submenus.

#### `render_bone_tree` (line 58)

```rust
pub fn render_bone_tree(
    ui: &mut egui::Ui,
    bone: &SdfBone,
    selected_bone: Option<BoneId>,
    selected_shape: Option<ShapeId>,
    rename_state: &mut Option<(RenameTarget, String)>,
) -> TreeResult
```

Renders the bone tree recursively using egui CollapsingHeader.

#### `render_bone_tree` (line 58)

```rust
pub fn render_bone_tree(
    ui: &mut egui::Ui,
    bone: &SdfBone,
    selected_bone: Option<BoneId>,
    selected_shape: Option<ShapeId>,
    rename_state: &mut Option<(RenameTarget, String)>,
) -> TreeResult
```


## `crates/litsdf_editor/src/undo.rs`

### Structs

#### `UndoHistory` (line 9)

```rust
pub struct UndoHistory {
    undo_stack: Vec<SdfScene>,
    redo_stack: Vec<SdfScene>,
}
```


### Functions

#### `push` (line 24)

```rust
    pub fn push(&mut self, scene: SdfScene)
```


#### `undo` (line 32)

```rust
    pub fn undo(&mut self, current: &SdfScene) -> Option<SdfScene>
```


#### `redo` (line 38)

```rust
    pub fn redo(&mut self, current: &SdfScene) -> Option<SdfScene>
```


#### `undo_len` (line 44)

```rust
    pub fn undo_len(&self) -> usize
```


#### `redo_len` (line 45)

```rust
    pub fn redo_len(&self) -> usize
```


#### `undo_redo_system` (line 48)

```rust
pub fn undo_redo_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut scene: ResMut<SdfSceneState>,
    mut history: ResMut<UndoHistory>,
)
```


#### `snapshot_before_mutation` (line 78)

```rust
pub fn snapshot_before_mutation(scene: &SdfScene) -> SdfScene
```

Call this before a mutation to snapshot the current scene.
Returns the cloned scene to pass to UndoHistory::push after mutation.

#### `snapshot_before_mutation` (line 78)

```rust
pub fn snapshot_before_mutation(scene: &SdfScene) -> SdfScene
```


## `crates/litsdf_editor/src/testing.rs`

### Structs

#### `ScreenshotConfig` (line 8)

```rust
pub struct ScreenshotConfig {
    pub path: String,
    pub capture_frame: u32,
    pub exit_frame: u32,
}
```


#### `TestSequence` (line 31)

```rust
pub struct TestSequence {
    pub dir: String,
    pub frame: u32,
    pub step: u32,
}
```


#### `RenderSequence` (line 100)

```rust
pub struct RenderSequence {
    pub dir: String,
    pub total_frames: u32,
    pub fps: f32,
    pub current_frame: u32,
    pub frames_per_capture: u32, // render frames between captures (for settling)
    pub internal_frame: u32,
}
```

Render a sequence of frames to numbered PNGs for video assembly.
Controlled by LITSDF_RENDER_SEQUENCE env var: "output_dir,total_frames,fps"

#### `RenderSequence` (line 100)

```rust
pub struct RenderSequence {
    pub dir: String,
    pub total_frames: u32,
    pub fps: f32,
    pub current_frame: u32,
    pub frames_per_capture: u32, // render frames between captures (for settling)
    pub internal_frame: u32,
}
```


### Functions

#### `auto_screenshot` (line 14)

```rust
pub fn auto_screenshot(
    mut commands: Commands,
    config: Res<ScreenshotConfig>,
    mut frame: Local<u32>,
)
```


#### `test_sequence_system` (line 37)

```rust
pub fn test_sequence_system(
    mut commands: Commands,
    mut seq: ResMut<TestSequence>,
    mut scene: ResMut<SdfSceneState>,
)
```


#### `render_sequence_system` (line 109)

```rust
pub fn render_sequence_system(
    mut commands: Commands,
    mut seq: ResMut<RenderSequence>,
    mut scene: ResMut<SdfSceneState>,
)
```


---

## Module Stratification

Stratification = (outgoing + 1) / (incoming + 1). Low = foundational, high = leaf.

| Module | Out | In | Strat | Role |
|--------|-----|-----|-------|------|
| `core::models` | 0 | 36 | 0.03 | foundation |
| `render::camera` | 0 | 3 | 0.25 | foundation |
| `core::scene` | 1 | 5 | 0.33 | foundation |
| `render::scene_sync` | 3 | 11 | 0.33 | foundation |
| `render::shader` | 0 | 1 | 0.50 | core |
| `core::persistence` | 1 | 2 | 0.67 | core |
| `core::lib` | 0 | 0 | 1.00 | core |
| `core::physics` | 1 | 1 | 1.00 | core |
| `render::lib` | 0 | 0 | 1.00 | core |
| `editor::lib` | 0 | 0 | 1.00 | core |
| `editor::nodes::expression` | 0 | 0 | 1.00 | core |
| `editor::nodes::mod` | 0 | 0 | 1.00 | core |
| `editor::nodes::presets` | 0 | 0 | 1.00 | core |
| `editor::nodes::types` | 0 | 0 | 1.00 | core |
| `editor::nodes::viewer` | 0 | 0 | 1.00 | core |
| `editor::ui::panel_tests` | 0 | 0 | 1.00 | core |
| `editor::ui::shortcuts` | 0 | 0 | 1.00 | core |
| `core::sdf` | 1 | 0 | 2.00 | connector |
| `editor::demos::abstract_sculpture` | 1 | 0 | 2.00 | connector |
| `editor::demos::boolean_sampler` | 1 | 0 | 2.00 | connector |
| `editor::demos::damping_lab` | 1 | 0 | 2.00 | connector |
| `editor::demos::hanging_chain` | 1 | 0 | 2.00 | connector |
| `editor::demos::lemmings` | 1 | 0 | 2.00 | connector |
| `editor::demos::mod` | 1 | 0 | 2.00 | connector |
| `editor::demos::modifier_parade` | 1 | 0 | 2.00 | connector |
| `editor::demos::mushroom_garden` | 1 | 0 | 2.00 | connector |
| `editor::demos::pendulum` | 1 | 0 | 2.00 | connector |
| `editor::demos::primitive_gallery` | 1 | 0 | 2.00 | connector |
| `editor::demos::robot_friend` | 1 | 0 | 2.00 | connector |
| `editor::demos::speed_glow` | 1 | 0 | 2.00 | connector |
| `editor::demos::walker` | 1 | 0 | 2.00 | connector |
| `editor::demos::wave_force` | 1 | 0 | 2.00 | connector |
| `editor::nodes::eval` | 1 | 0 | 2.00 | connector |
| `editor::project` | 1 | 0 | 2.00 | connector |
| `editor::ui::helpers` | 1 | 0 | 2.00 | connector |
| `editor::ui::populate` | 1 | 0 | 2.00 | connector |
| `editor::ui::sync` | 1 | 0 | 2.00 | connector |
| `editor::ui::tree` | 1 | 0 | 2.00 | connector |
| `cli::commands::bone` | 1 | 0 | 2.00 | connector |
| `cli::commands::modifier` | 1 | 0 | 2.00 | connector |
| `cli::commands::shape` | 1 | 0 | 2.00 | connector |
| `render::picking` | 4 | 1 | 2.50 | leaf |
| `render::codegen` | 2 | 0 | 3.00 | leaf |
| `editor::testing` | 2 | 0 | 3.00 | leaf |
| `editor::ui::handlers` | 2 | 0 | 3.00 | leaf |
| `editor::ui::modifier_stack` | 2 | 0 | 3.00 | leaf |
| `editor::undo` | 2 | 0 | 3.00 | leaf |
| `cli::commands::mod` | 2 | 0 | 3.00 | leaf |
| `cli::commands::scene` | 2 | 0 | 3.00 | leaf |
| `render::avian_physics` | 4 | 0 | 5.00 | leaf |
| `render::gizmos` | 4 | 0 | 5.00 | leaf |
| `editor::ui::mod` | 4 | 0 | 5.00 | leaf |

