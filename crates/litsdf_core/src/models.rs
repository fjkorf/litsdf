use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Serde helpers for skip_serializing_if ────────────────────────

fn is_zero(v: &f32) -> bool { *v == 0.0 }
fn is_zero_u32(v: &u32) -> bool { *v == 0 }
fn is_one(v: &f32) -> bool { *v == 1.0 }
fn is_half(v: &f32) -> bool { *v == 0.5 }
fn is_zero_array(v: &[f32; 3]) -> bool { *v == [0.0, 0.0, 0.0] }
fn is_white(v: &[f32; 3]) -> bool { *v == [1.0, 1.0, 1.0] }
fn is_true(v: &bool) -> bool { *v }
fn default_true() -> bool { true }
fn one() -> f32 { 1.0 }
fn half() -> f32 { 0.5 }
fn white() -> [f32; 3] { [1.0, 1.0, 1.0] }

// ── IDs ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ShapeId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BoneId(pub Uuid);

impl BoneId {
    pub fn root() -> Self { Self(Uuid::nil()) }
    pub fn is_root(&self) -> bool { self.0.is_nil() }
}

// ── Bone hierarchy ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SdfBone {
    pub id: BoneId,
    pub name: String,
    #[serde(default, skip_serializing_if = "ShapeTransform::is_default")]
    pub transform: ShapeTransform,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub visible: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<SdfBone>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shapes: Vec<SdfShape>,
}

impl SdfBone {
    pub fn root() -> Self {
        Self {
            id: BoneId::root(),
            name: "Root".into(),
            transform: ShapeTransform::default(),
            visible: true,
            children: Vec::new(),
            shapes: Vec::new(),
        }
    }

    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: BoneId(Uuid::new_v4()),
            name: name.into(),
            transform: ShapeTransform::default(),
            visible: true,
            children: Vec::new(),
            shapes: Vec::new(),
        }
    }

    pub fn find_bone(&self, id: BoneId) -> Option<&SdfBone> {
        if self.id == id { return Some(self); }
        for child in &self.children {
            if let Some(found) = child.find_bone(id) { return Some(found); }
        }
        None
    }

    pub fn find_bone_mut(&mut self, id: BoneId) -> Option<&mut SdfBone> {
        if self.id == id { return Some(self); }
        for child in &mut self.children {
            if let Some(found) = child.find_bone_mut(id) { return Some(found); }
        }
        None
    }

    pub fn find_shape(&self, id: ShapeId) -> Option<(&SdfShape, BoneId)> {
        for shape in &self.shapes {
            if shape.id == id { return Some((shape, self.id)); }
        }
        for child in &self.children {
            if let Some(found) = child.find_shape(id) { return Some(found); }
        }
        None
    }

    pub fn find_shape_mut(&mut self, id: ShapeId) -> Option<(&mut SdfShape, BoneId)> {
        let my_id = self.id;
        for shape in &mut self.shapes {
            if shape.id == id { return Some((shape, my_id)); }
        }
        for child in &mut self.children {
            if let Some(found) = child.find_shape_mut(id) { return Some(found); }
        }
        None
    }

    pub fn find_shape_by_name(&self, name: &str) -> Option<(&SdfShape, BoneId)> {
        for shape in &self.shapes {
            if shape.name == name { return Some((shape, self.id)); }
        }
        for child in &self.children {
            if let Some(found) = child.find_shape_by_name(name) { return Some(found); }
        }
        None
    }

    pub fn all_shapes(&self) -> Vec<(&SdfShape, BoneId)> {
        let mut result = Vec::new();
        self.collect_shapes(&mut result);
        result
    }

    fn collect_shapes<'a>(&'a self, out: &mut Vec<(&'a SdfShape, BoneId)>) {
        for shape in &self.shapes { out.push((shape, self.id)); }
        for child in &self.children { child.collect_shapes(out); }
    }

    pub fn remove_shape(&mut self, id: ShapeId) -> bool {
        let before = self.shapes.len();
        self.shapes.retain(|s| s.id != id);
        if self.shapes.len() < before { return true; }
        for child in &mut self.children {
            if child.remove_shape(id) { return true; }
        }
        false
    }

    pub fn remove_bone(&mut self, id: BoneId) -> bool {
        if let Some(pos) = self.children.iter().position(|b| b.id == id) {
            let removed = self.children.remove(pos);
            self.shapes.extend(removed.shapes);
            self.children.extend(removed.children);
            return true;
        }
        for child in &mut self.children {
            if child.remove_bone(id) { return true; }
        }
        false
    }

    /// Deep clone with fresh UUIDs for this bone, all children, and all shapes.
    pub fn duplicate_deep(&self) -> Self {
        let mut clone = Self {
            id: BoneId(Uuid::new_v4()),
            name: format!("{} Copy", self.name),
            transform: self.transform.clone(),
            visible: self.visible,
            children: self.children.iter().map(|c| c.duplicate_deep()).collect(),
            shapes: self.shapes.iter().map(|s| s.duplicate()).collect(),
        };
        // Keep children/shape names from duplicate_deep/duplicate (which append " Copy"),
        // but only the top-level bone gets " Copy" — children keep original names.
        for (child, orig) in clone.children.iter_mut().zip(self.children.iter()) {
            child.restore_names(orig);
        }
        for (shape, orig) in clone.shapes.iter_mut().zip(self.shapes.iter()) {
            shape.name = orig.name.clone();
        }
        clone
    }

    /// Restore original names after duplicate_deep (only top-level gets " Copy").
    fn restore_names(&mut self, original: &SdfBone) {
        self.name = original.name.clone();
        for (shape, orig) in self.shapes.iter_mut().zip(original.shapes.iter()) {
            shape.name = orig.name.clone();
        }
        for (child, orig) in self.children.iter_mut().zip(original.children.iter()) {
            child.restore_names(orig);
        }
    }

    pub fn find_bone_by_name(&self, name: &str) -> Option<&SdfBone> {
        if self.name == name { return Some(self); }
        for child in &self.children {
            if let Some(found) = child.find_bone_by_name(name) { return Some(found); }
        }
        None
    }

    pub fn find_bone_by_name_mut(&mut self, name: &str) -> Option<&mut SdfBone> {
        if self.name == name { return Some(self); }
        for child in &mut self.children {
            if let Some(found) = child.find_bone_by_name_mut(name) { return Some(found); }
        }
        None
    }

    /// Remove a shape from anywhere in the tree and add it to the target bone.
    pub fn reparent_shape(&mut self, shape_id: ShapeId, target_bone_id: BoneId) -> bool {
        // First, extract the shape
        let shape = self.extract_shape(shape_id);
        let Some(shape) = shape else { return false };
        // Then add to target
        let Some(target) = self.find_bone_mut(target_bone_id) else { return false };
        target.shapes.push(shape);
        true
    }

    pub fn extract_shape(&mut self, id: ShapeId) -> Option<SdfShape> {
        if let Some(pos) = self.shapes.iter().position(|s| s.id == id) {
            return Some(self.shapes.remove(pos));
        }
        for child in &mut self.children {
            if let Some(shape) = child.extract_shape(id) { return Some(shape); }
        }
        None
    }

    /// Remove a bone from anywhere in the tree and add it as a child of target.
    /// Returns false if bone_id == target or target is a descendant of bone_id (cycle).
    pub fn reparent_bone(&mut self, bone_id: BoneId, target_bone_id: BoneId) -> bool {
        if bone_id == target_bone_id { return false; }
        // Check for cycle: target must not be a descendant of bone_id
        if let Some(bone) = self.find_bone(bone_id) {
            if bone.find_bone(target_bone_id).is_some() { return false; }
        }
        let Some(bone) = self.extract_bone(bone_id) else { return false };
        let Some(target) = self.find_bone_mut(target_bone_id) else { return false };
        target.children.push(bone);
        true
    }

    pub fn extract_bone(&mut self, id: BoneId) -> Option<SdfBone> {
        if let Some(pos) = self.children.iter().position(|b| b.id == id) {
            return Some(self.children.remove(pos));
        }
        for child in &mut self.children {
            if let Some(bone) = child.extract_bone(id) { return Some(bone); }
        }
        None
    }

    /// Count all descendant bones (not including self).
    pub fn bone_count(&self) -> usize {
        self.children.iter().map(|c| 1 + c.bone_count()).sum()
    }

    /// Count all shapes in this bone and all descendants.
    pub fn shape_count(&self) -> usize {
        self.shapes.len() + self.children.iter().map(|c| c.shape_count()).sum::<usize>()
    }

    pub fn reset_transform(&mut self) {
        self.transform = ShapeTransform::default();
    }
}

// ── Scene settings ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
}

fn default_fill_color() -> [f32; 3] { [0.4, 0.5, 0.7] }
fn is_default_fill_color(v: &[f32; 3]) -> bool { *v == [0.4, 0.5, 0.7] }
fn default_fill_intensity() -> f32 { 0.25 }
fn is_default_fill_intensity(v: &f32) -> bool { *v == 0.25 }
fn default_back_color() -> [f32; 3] { [0.3, 0.2, 0.1] }
fn is_default_back_color(v: &[f32; 3]) -> bool { *v == [0.3, 0.2, 0.1] }
fn default_back_intensity() -> f32 { 0.2 }
fn is_default_back_intensity(v: &f32) -> bool { *v == 0.2 }
fn default_sss_color() -> [f32; 3] { [1.0, 0.2, 0.1] }
fn is_default_sss_color(v: &[f32; 3]) -> bool { *v == [1.0, 0.2, 0.1] }
fn default_sss_intensity() -> f32 { 0.15 }
fn is_default_sss_intensity(v: &f32) -> bool { *v == 0.15 }
fn default_ao_intensity() -> f32 { 3.0 }
fn is_default_ao_intensity(v: &f32) -> bool { *v == 3.0 }
fn default_shadow_softness() -> f32 { 8.0 }
fn is_default_shadow_softness(v: &f32) -> bool { *v == 8.0 }
fn default_vignette() -> f32 { 0.3 }
fn is_default_vignette(v: &f32) -> bool { *v == 0.3 }

impl Default for SceneSettings {
    fn default() -> Self {
        Self {
            fill_color: default_fill_color(),
            fill_intensity: default_fill_intensity(),
            back_color: default_back_color(),
            back_intensity: default_back_intensity(),
            sss_color: default_sss_color(),
            sss_intensity: default_sss_intensity(),
            ao_intensity: default_ao_intensity(),
            shadow_softness: default_shadow_softness(),
            vignette_intensity: default_vignette(),
        }
    }
}

impl SceneSettings {
    pub fn is_default(&self) -> bool { *self == Self::default() }
}

// ── Scene ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SdfScene {
    pub name: String,
    pub root_bone: SdfBone,
    #[serde(default, skip_serializing_if = "CombinationOp::is_default")]
    pub combination: CombinationOp,
    #[serde(default = "default_light_dir", skip_serializing_if = "is_default_light_dir")]
    pub light_dir: [f32; 3],
    #[serde(default, skip_serializing_if = "SceneSettings::is_default")]
    pub settings: SceneSettings,
}

fn default_light_dir() -> [f32; 3] { [0.6, 0.8, 0.4] }
fn is_default_light_dir(v: &[f32; 3]) -> bool { *v == [0.6, 0.8, 0.4] }

/// Summary information about a scene.
#[derive(Debug, Clone)]
pub struct SceneInfo {
    pub name: String,
    pub bone_count: usize,
    pub shape_count: usize,
}

impl std::fmt::Display for SceneInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} bones, {} shapes", self.name, self.bone_count, self.shape_count)
    }
}

impl SdfScene {
    /// Create an empty scene with a root bone and default light.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            root_bone: SdfBone::root(),
            combination: CombinationOp::Union,
            light_dir: default_light_dir(),
            settings: SceneSettings::default(),
        }
    }

    pub fn info(&self) -> SceneInfo {
        SceneInfo {
            name: self.name.clone(),
            bone_count: self.root_bone.bone_count(),
            shape_count: self.root_bone.shape_count(),
        }
    }

    /// ASCII tree representation of the scene hierarchy.
    pub fn tree_string(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("Scene: {}\n", self.name));
        Self::tree_bone(&self.root_bone, &mut out, "", true, true);
        out
    }

    fn tree_bone(bone: &SdfBone, out: &mut String, prefix: &str, is_last: bool, is_root: bool) {
        if is_root {
            out.push_str(&format!("[Bone] {}\n", bone.name));
        } else {
            let connector = if is_last { "└── " } else { "├── " };
            out.push_str(&format!("{}{}[Bone] {}\n", prefix, connector, bone.name));
        }
        let child_prefix = if is_root {
            String::new()
        } else if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };
        let total = bone.shapes.len() + bone.children.len();
        for (i, shape) in bone.shapes.iter().enumerate() {
            let last = i + bone.children.len() == total - 1;
            let sc = if last { "└── " } else { "├── " };
            out.push_str(&format!("{}{}({}) {}\n", child_prefix, sc, shape.primitive.label(), shape.name));
        }
        for (i, child) in bone.children.iter().enumerate() {
            let last = bone.shapes.len() + i == total - 1;
            Self::tree_bone(child, out, &child_prefix, last, false);
        }
    }

    pub fn default_scene() -> Self {
        let mut root = SdfBone::root();

        // Floating island bone
        let mut island_bone = SdfBone::new("Island");

        // Island base — rounded box with noise
        let mut island_base = SdfShape::new("Rock", SdfPrimitive::RoundBox {
            half_extents: [1.2, 0.5, 1.0], rounding: 0.2,
        });
        island_base.material.color = [0.5, 0.4, 0.3];
        island_base.material.roughness = 0.85;
        island_base.material.noise_amplitude = 0.05;
        island_base.material.noise_frequency = 3.0;
        island_base.material.noise_octaves = 2;
        island_base.material.color_mode = 2;
        island_bone.shapes.push(island_base);

        // Grassy mound on top — smooth union with island
        let mut mound = SdfShape::new("Grass", SdfPrimitive::Ellipsoid {
            radii: [1.1, 0.35, 0.9],
        });
        mound.transform.translation = [0.0, 0.35, 0.0];
        mound.material.color = [0.25, 0.55, 0.2];
        mound.material.roughness = 0.7;
        mound.material.noise_amplitude = 0.02;
        mound.material.noise_frequency = 5.0;
        mound.material.noise_octaves = 2;
        mound.combination = CombinationOp::SmoothUnion { k: 0.35 };
        island_bone.shapes.push(mound);

        // Tree bone (child of island)
        let mut tree_bone = SdfBone::new("Tree");
        tree_bone.transform.translation = [0.3, 0.7, -0.1];

        // Trunk
        let mut trunk = SdfShape::new("Trunk", SdfPrimitive::Capsule {
            radius: 0.08, half_height: 0.5,
        });
        trunk.material.color = [0.4, 0.28, 0.15];
        trunk.material.roughness = 0.9;
        trunk.material.color_mode = 2;
        trunk.material.noise_frequency = 8.0;
        tree_bone.shapes.push(trunk);

        // Canopy — sphere with gentle breathing animation
        let mut canopy = SdfShape::new("Canopy", SdfPrimitive::Sphere { radius: 0.5 });
        canopy.transform.translation = [0.0, 0.7, 0.0];
        canopy.material.color = [0.2, 0.5, 0.15];
        canopy.material.roughness = 0.6;
        canopy.material.noise_amplitude = 0.04;
        canopy.material.noise_frequency = 4.0;
        canopy.material.noise_octaves = 2;
        canopy.combination = CombinationOp::SmoothUnion { k: 0.15 };
        tree_bone.shapes.push(canopy);

        island_bone.children.push(tree_bone);

        // Orbiter bone (child of island)
        let mut orbiter_bone = SdfBone::new("Orbiter");
        orbiter_bone.transform.translation = [1.8, 0.8, 0.0];

        // Orbiting torus — metallic, animated
        let mut ring = SdfShape::new("Ring", SdfPrimitive::Torus {
            major_radius: 0.3, minor_radius: 0.07,
        });
        ring.material.color = [0.3, 0.5, 0.9];
        ring.material.roughness = 0.15;
        ring.material.metallic = 0.8;
        ring.material.fresnel_power = 2.5;
        orbiter_bone.shapes.push(ring);

        island_bone.children.push(orbiter_bone);
        root.children.push(island_bone);

        Self {
            name: "Floating Islands".into(),
            root_bone: root,
            combination: CombinationOp::Union,
            light_dir: default_light_dir(),
            settings: SceneSettings::default(),
        }
    }
}

// ── Shape ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShapeTransform {
    #[serde(default, skip_serializing_if = "is_zero_array")]
    pub translation: [f32; 3],
    #[serde(default, skip_serializing_if = "is_zero_array")]
    pub rotation: [f32; 3],
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub scale: f32,
}

impl ShapeTransform {
    pub fn is_default(&self) -> bool { *self == Self::default() }
}

impl Default for ShapeTransform {
    fn default() -> Self {
        Self {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

impl ShapeMaterial {
    pub fn is_default(&self) -> bool { *self == Self::default() }
}

impl Default for ShapeMaterial {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0],
            roughness: 0.5,
            metallic: 0.0,
            fresnel_power: 0.0,
            color_mode: 0,
            palette_a: [0.0, 0.0, 0.0],
            palette_b: [0.0, 0.0, 0.0],
            palette_c: [0.0, 0.0, 0.0],
            palette_d: [0.0, 0.0, 0.0],
            noise_amplitude: 0.0,
            noise_frequency: 1.0,
            noise_octaves: 0,
            smooth_symmetry: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

impl Default for CombinationOp {
    fn default() -> Self { Self::Union }
}

impl CombinationOp {
    pub fn is_default(&self) -> bool { matches!(self, Self::Union) }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShapeModifier {
    Rounding(f32),
    Onion(f32),
    Twist(f32),
    Bend(f32),
    Elongation([f32; 3]),
    Repetition { period: [f32; 3], count: [u32; 3] },
}

impl SdfShape {
    pub fn duplicate(&self) -> Self {
        let mut clone = self.clone();
        clone.id = ShapeId(Uuid::new_v4());
        clone.name = format!("{} Copy", self.name);
        clone
    }

    pub fn reset_transform(&mut self) {
        self.transform = ShapeTransform::default();
    }

    pub fn clear_modifiers(&mut self) {
        self.modifiers.clear();
    }

    pub fn default_sphere() -> Self {
        Self {
            id: ShapeId(Uuid::new_v4()),
            name: "Sphere".into(),
            primitive: SdfPrimitive::Sphere { radius: 1.0 },
            visible: true,
            transform: ShapeTransform::default(),
            material: ShapeMaterial::default(),
            modifiers: Vec::new(),
            combination: CombinationOp::Union,
        }
    }

    pub fn new(name: impl Into<String>, primitive: SdfPrimitive) -> Self {
        Self {
            id: ShapeId(Uuid::new_v4()),
            name: name.into(),
            primitive,
            visible: true,
            transform: ShapeTransform::default(),
            material: ShapeMaterial::default(),
            modifiers: Vec::new(),
            combination: CombinationOp::Union,
        }
    }
}

impl SdfPrimitive {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Sphere { .. } => "Sphere",
            Self::Box { .. } => "Box",
            Self::RoundBox { .. } => "RoundBox",
            Self::Cylinder { .. } => "Cylinder",
            Self::CappedCone { .. } => "CappedCone",
            Self::Torus { .. } => "Torus",
            Self::Capsule { .. } => "Capsule",
            Self::Plane { .. } => "Plane",
            Self::Ellipsoid { .. } => "Ellipsoid",
            Self::Octahedron { .. } => "Octahedron",
            Self::Pyramid { .. } => "Pyramid",
            Self::HexPrism { .. } => "HexPrism",
            Self::RoundCone { .. } => "RoundCone",
        }
    }

    pub fn default_for(name: &str) -> Self {
        match name {
            "Sphere" => Self::Sphere { radius: 1.0 },
            "Box" => Self::Box { half_extents: [0.5, 0.5, 0.5] },
            "RoundBox" => Self::RoundBox { half_extents: [0.5, 0.5, 0.5], rounding: 0.1 },
            "Cylinder" => Self::Cylinder { height: 1.0, radius: 0.5 },
            "CappedCone" => Self::CappedCone { height: 1.0, r1: 0.5, r2: 0.2 },
            "Torus" => Self::Torus { major_radius: 1.0, minor_radius: 0.3 },
            "Capsule" => Self::Capsule { radius: 0.3, half_height: 0.5 },
            "Plane" => Self::Plane { normal: [0.0, 1.0, 0.0], offset: 0.0 },
            "Ellipsoid" => Self::Ellipsoid { radii: [1.0, 0.8, 0.6] },
            "Octahedron" => Self::Octahedron { size: 1.0 },
            "Pyramid" => Self::Pyramid { height: 1.0, base: 1.0 },
            "HexPrism" => Self::HexPrism { height: 1.0, radius: 0.5 },
            "RoundCone" => Self::RoundCone { r1: 0.5, r2: 0.2, height: 1.0 },
            _ => Self::Sphere { radius: 1.0 },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yaml_round_trip() {
        let scene = SdfScene::default_scene();
        let yaml = serde_yaml::to_string(&scene).unwrap();
        let loaded: SdfScene = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(scene.name, loaded.name);
        assert_eq!(scene.root_bone.shapes.len(), loaded.root_bone.shapes.len());
        assert_eq!(scene.combination, loaded.combination);
    }

    #[test]
    fn shape_defaults() {
        let shape = SdfShape::default_sphere();
        assert_eq!(shape.name, "Sphere");
        assert!(matches!(shape.primitive, SdfPrimitive::Sphere { radius } if radius == 1.0));
        assert_eq!(shape.transform.translation, [0.0, 0.0, 0.0]);
        assert_eq!(shape.transform.scale, 1.0);
        assert_eq!(shape.material.color, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn compact_yaml_omits_defaults() {
        let shape = SdfShape::default_sphere();
        let yaml = serde_yaml::to_string(&shape).unwrap();
        // Default transform, material, modifiers, combination should be omitted
        assert!(!yaml.contains("translation"), "default translation should be omitted");
        assert!(!yaml.contains("rotation"), "default rotation should be omitted");
        assert!(!yaml.contains("scale"), "default scale should be omitted");
        assert!(!yaml.contains("color"), "default color should be omitted");
        assert!(!yaml.contains("modifiers"), "empty modifiers should be omitted");
        assert!(!yaml.contains("Union"), "default Union should be omitted");
        // But name and primitive must remain
        assert!(yaml.contains("Sphere"));
        assert!(yaml.contains("name"));
    }

    #[test]
    fn compact_yaml_keeps_non_defaults() {
        let mut shape = SdfShape::default_sphere();
        shape.transform.translation = [1.5, 0.0, 0.0];
        shape.material.color = [0.8, 0.2, 0.2];
        shape.combination = CombinationOp::SmoothUnion { k: 0.5 };

        let yaml = serde_yaml::to_string(&shape).unwrap();
        // Non-default values must be present
        assert!(yaml.contains("1.5"), "non-default translation should be present");
        assert!(yaml.contains("0.8"), "non-default color should be present");
        assert!(yaml.contains("SmoothUnion"), "non-default combination should be present");
        // Default rotation/scale still omitted from transform
        assert!(!yaml.contains("rotation"), "default rotation still omitted");
    }

    #[test]
    fn compact_yaml_round_trips() {
        // Compact → deserialize → reserialize should be identical
        let mut shape = SdfShape::default_sphere();
        shape.transform.translation = [2.0, 0.0, -1.0];
        shape.material.roughness = 0.8;

        let yaml1 = serde_yaml::to_string(&shape).unwrap();
        let loaded: SdfShape = serde_yaml::from_str(&yaml1).unwrap();
        let yaml2 = serde_yaml::to_string(&loaded).unwrap();
        assert_eq!(yaml1, yaml2, "compact YAML should be stable across round-trips");

        // Verify the loaded data matches
        assert_eq!(loaded.transform.translation, [2.0, 0.0, -1.0]);
        assert_eq!(loaded.transform.rotation, [0.0, 0.0, 0.0]); // default filled in
        assert_eq!(loaded.transform.scale, 1.0); // default filled in
        assert_eq!(loaded.material.roughness, 0.8);
        assert_eq!(loaded.material.color, [1.0, 1.0, 1.0]); // default filled in
    }

    #[test]
    fn bone_find_shape() {
        let mut root = SdfBone::root();
        let shape = SdfShape::default_sphere();
        let shape_id = shape.id;
        root.shapes.push(shape);

        let mut child = SdfBone::new("Arm");
        let child_id = child.id;
        let shape2 = SdfShape::new("Box", SdfPrimitive::Box { half_extents: [1.0, 1.0, 1.0] });
        let shape2_id = shape2.id;
        child.shapes.push(shape2);
        root.children.push(child);

        let (found, bone_id) = root.find_shape(shape_id).unwrap();
        assert_eq!(found.id, shape_id);
        assert!(bone_id.is_root());

        let (found, bone_id) = root.find_shape(shape2_id).unwrap();
        assert_eq!(found.id, shape2_id);
        assert_eq!(bone_id, child_id);
    }

    #[test]
    fn bone_remove_reparents() {
        let mut root = SdfBone::root();
        let mut child = SdfBone::new("Arm");
        let child_id = child.id;
        child.shapes.push(SdfShape::default_sphere());
        let mut grandchild = SdfBone::new("Hand");
        grandchild.shapes.push(SdfShape::new("Box", SdfPrimitive::Box { half_extents: [0.5, 0.5, 0.5] }));
        child.children.push(grandchild);
        root.children.push(child);

        assert_eq!(root.all_shapes().len(), 2);
        root.remove_bone(child_id);
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.shapes.len(), 1);
        assert_eq!(root.all_shapes().len(), 2);
    }

    #[test]
    fn all_primitives_serialize() {
        let primitives = [
            SdfPrimitive::Sphere { radius: 1.0 },
            SdfPrimitive::Box { half_extents: [1.0, 2.0, 3.0] },
            SdfPrimitive::RoundBox { half_extents: [1.0, 1.0, 1.0], rounding: 0.1 },
            SdfPrimitive::Cylinder { height: 2.0, radius: 0.5 },
            SdfPrimitive::CappedCone { height: 1.0, r1: 0.5, r2: 0.2 },
            SdfPrimitive::Torus { major_radius: 1.0, minor_radius: 0.3 },
            SdfPrimitive::Capsule { radius: 0.3, half_height: 0.5 },
            SdfPrimitive::Plane { normal: [0.0, 1.0, 0.0], offset: 0.0 },
        ];
        for prim in &primitives {
            let yaml = serde_yaml::to_string(prim).unwrap();
            let loaded: SdfPrimitive = serde_yaml::from_str(&yaml).unwrap();
            assert_eq!(prim, &loaded);
        }
    }

    #[test]
    fn combination_ops_serialize() {
        let ops = [
            CombinationOp::Union,
            CombinationOp::Intersection,
            CombinationOp::Subtraction,
            CombinationOp::SmoothUnion { k: 0.5 },
            CombinationOp::SmoothIntersection { k: 0.3 },
            CombinationOp::SmoothSubtraction { k: 1.0 },
        ];
        for op in &ops {
            let yaml = serde_yaml::to_string(op).unwrap();
            let loaded: CombinationOp = serde_yaml::from_str(&yaml).unwrap();
            assert_eq!(op, &loaded);
        }
    }

    #[test]
    fn modifiers_serialize() {
        let mods = [
            ShapeModifier::Rounding(0.1),
            ShapeModifier::Onion(0.05),
            ShapeModifier::Twist(2.0),
            ShapeModifier::Bend(1.0),
            ShapeModifier::Elongation([0.5, 0.0, 0.0]),
            ShapeModifier::Repetition { period: [2.0, 2.0, 2.0], count: [3, 3, 3] },
        ];
        for m in &mods {
            let yaml = serde_yaml::to_string(m).unwrap();
            let loaded: ShapeModifier = serde_yaml::from_str(&yaml).unwrap();
            assert_eq!(m, &loaded);
        }
    }

    #[test]
    fn bone_hierarchy_serializes() {
        let scene = SdfScene::default_scene();
        let yaml = serde_yaml::to_string(&scene).unwrap();
        let loaded: SdfScene = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(scene.root_bone.id, loaded.root_bone.id);
        assert_eq!(scene.root_bone.shapes.len(), loaded.root_bone.shapes.len());
        assert_eq!(scene.root_bone.children.len(), loaded.root_bone.children.len());
    }

    // ── Phase 1A: Core helper tests ────────────────────────────────

    #[test]
    fn shape_duplicate() {
        let mut shape = SdfShape::default_sphere();
        shape.transform.translation = [1.0, 2.0, 3.0];
        shape.material.color = [0.5, 0.5, 0.5];
        shape.modifiers.push(ShapeModifier::Rounding(0.1));
        let dup = shape.duplicate();
        assert_ne!(dup.id, shape.id);
        assert_eq!(dup.name, "Sphere Copy");
        assert_eq!(dup.transform.translation, [1.0, 2.0, 3.0]);
        assert_eq!(dup.material.color, [0.5, 0.5, 0.5]);
        assert_eq!(dup.modifiers.len(), 1);
    }

    #[test]
    fn shape_reset_transform() {
        let mut shape = SdfShape::default_sphere();
        shape.transform.translation = [5.0, 5.0, 5.0];
        shape.transform.rotation = [45.0, 0.0, 0.0];
        shape.transform.scale = 2.0;
        shape.reset_transform();
        assert_eq!(shape.transform, ShapeTransform::default());
    }

    #[test]
    fn shape_clear_modifiers() {
        let mut shape = SdfShape::default_sphere();
        shape.modifiers.push(ShapeModifier::Rounding(0.1));
        shape.modifiers.push(ShapeModifier::Twist(2.0));
        shape.clear_modifiers();
        assert!(shape.modifiers.is_empty());
    }

    #[test]
    fn bone_duplicate_deep() {
        let mut bone = SdfBone::new("Arm");
        bone.shapes.push(SdfShape::default_sphere());
        let mut child = SdfBone::new("Hand");
        child.shapes.push(SdfShape::new("Finger", SdfPrimitive::Capsule { radius: 0.1, half_height: 0.3 }));
        bone.children.push(child);

        let dup = bone.duplicate_deep();
        assert_ne!(dup.id, bone.id);
        assert_eq!(dup.name, "Arm Copy");
        // Children keep original names
        assert_eq!(dup.children[0].name, "Hand");
        assert_eq!(dup.children[0].shapes[0].name, "Finger");
        // All IDs are fresh
        assert_ne!(dup.children[0].id, bone.children[0].id);
        assert_ne!(dup.shapes[0].id, bone.shapes[0].id);
        assert_ne!(dup.children[0].shapes[0].id, bone.children[0].shapes[0].id);
    }

    #[test]
    fn bone_find_by_name() {
        let mut root = SdfBone::root();
        let mut arm = SdfBone::new("Arm");
        arm.children.push(SdfBone::new("Hand"));
        root.children.push(arm);

        assert!(root.find_bone_by_name("Arm").is_some());
        assert!(root.find_bone_by_name("Hand").is_some());
        assert!(root.find_bone_by_name("Leg").is_none());
        assert_eq!(root.find_bone_by_name("Root").unwrap().id, BoneId::root());
    }

    #[test]
    fn bone_reparent_shape() {
        let mut root = SdfBone::root();
        let shape = SdfShape::default_sphere();
        let shape_id = shape.id;
        root.shapes.push(shape);
        let child = SdfBone::new("Arm");
        let child_id = child.id;
        root.children.push(child);

        assert!(root.reparent_shape(shape_id, child_id));
        assert!(root.shapes.is_empty());
        assert_eq!(root.children[0].shapes.len(), 1);
        assert_eq!(root.children[0].shapes[0].id, shape_id);
    }

    #[test]
    fn bone_reparent_bone() {
        let mut root = SdfBone::root();
        let arm = SdfBone::new("Arm");
        let arm_id = arm.id;
        let body = SdfBone::new("Body");
        let body_id = body.id;
        root.children.push(arm);
        root.children.push(body);

        assert!(root.reparent_bone(arm_id, body_id));
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].name, "Body");
        assert_eq!(root.children[0].children.len(), 1);
        assert_eq!(root.children[0].children[0].name, "Arm");
    }

    #[test]
    fn bone_reparent_bone_prevents_cycle() {
        let mut root = SdfBone::root();
        let mut arm = SdfBone::new("Arm");
        let arm_id = arm.id;
        let hand = SdfBone::new("Hand");
        let hand_id = hand.id;
        arm.children.push(hand);
        root.children.push(arm);

        // Can't reparent Arm under its own child Hand
        assert!(!root.reparent_bone(arm_id, hand_id));
        // Can't reparent to self
        assert!(!root.reparent_bone(arm_id, arm_id));
    }

    #[test]
    fn bone_counts() {
        let mut root = SdfBone::root();
        root.shapes.push(SdfShape::default_sphere());
        let mut arm = SdfBone::new("Arm");
        arm.shapes.push(SdfShape::default_sphere());
        arm.shapes.push(SdfShape::default_sphere());
        let mut hand = SdfBone::new("Hand");
        hand.shapes.push(SdfShape::default_sphere());
        arm.children.push(hand);
        root.children.push(arm);

        assert_eq!(root.bone_count(), 2); // Arm + Hand
        assert_eq!(root.shape_count(), 4); // 1 + 2 + 1
    }

    #[test]
    fn bone_reset_transform() {
        let mut bone = SdfBone::new("Test");
        bone.transform.translation = [1.0, 2.0, 3.0];
        bone.reset_transform();
        assert_eq!(bone.transform, ShapeTransform::default());
    }

    #[test]
    fn scene_new() {
        let scene = SdfScene::new("Test Scene");
        assert_eq!(scene.name, "Test Scene");
        assert!(scene.root_bone.id.is_root());
        assert!(scene.root_bone.children.is_empty());
        assert!(scene.root_bone.shapes.is_empty());
        assert_eq!(scene.light_dir, [0.6, 0.8, 0.4]);
    }

    #[test]
    fn scene_info() {
        let scene = SdfScene::default_scene();
        let info = scene.info();
        assert_eq!(info.name, "Floating Islands");
        assert!(info.bone_count > 0);
        assert!(info.shape_count > 0);
        let s = format!("{}", info);
        assert!(s.contains("Floating Islands"));
    }

    #[test]
    fn scene_tree_string() {
        let mut scene = SdfScene::new("Test");
        let mut arm = SdfBone::new("Arm");
        arm.shapes.push(SdfShape::default_sphere());
        scene.root_bone.children.push(arm);
        let tree = scene.tree_string();
        assert!(tree.contains("Scene: Test"));
        assert!(tree.contains("[Bone] Root"));
        assert!(tree.contains("[Bone] Arm"));
        assert!(tree.contains("(Sphere) Sphere"));
    }
}
