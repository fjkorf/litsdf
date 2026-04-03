//! Scene computation: bone world transforms, flattening bone tree to flat shape array.
//! No Bevy dependency — uses glam directly.

use std::collections::HashMap;
use glam::{EulerRot, Mat4, Quat, Vec3, Vec4};

use crate::models::*;

/// Compute world-space transform for every bone in the tree.
/// The `overrides` map allows external systems (e.g., node graphs) to provide
/// per-bone transform overrides computed before this function is called.
pub fn compute_bone_world_transforms(
    bone: &SdfBone,
    parent: Mat4,
    overrides: &HashMap<BoneId, ShapeTransform>,
) -> HashMap<BoneId, Mat4> {
    let t = overrides.get(&bone.id).unwrap_or(&bone.transform);
    let bt = t.translation;
    let br = t.rotation;

    let local = Mat4::from_scale_rotation_translation(
        Vec3::splat(t.scale),
        Quat::from_euler(EulerRot::XYZ, br[0].to_radians(), br[1].to_radians(), br[2].to_radians()),
        Vec3::new(bt[0], bt[1], bt[2]),
    );
    let world = parent * local;

    let mut result = HashMap::new();
    result.insert(bone.id, world);
    for child in &bone.children {
        result.extend(compute_bone_world_transforms(child, world, overrides));
    }
    result
}

/// A flattened shape ready for GPU encoding.
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

/// Flatten bone tree into a linear list of world-space shapes.
pub fn flatten_bone_tree(
    bone: &SdfBone,
    world_transforms: &HashMap<BoneId, Mat4>,
    output: &mut Vec<FlatShape>,
) {
    if !bone.visible { return; }
    let bone_world = world_transforms[&bone.id];

    for shape in bone.shapes.iter() {
        if !shape.visible { continue; }
        let s_t = shape.transform.translation;
        let s_r = shape.transform.rotation;
        let shape_local = Mat4::from_scale_rotation_translation(
            Vec3::splat(shape.transform.scale),
            Quat::from_euler(EulerRot::XYZ, s_r[0].to_radians(), s_r[1].to_radians(), s_r[2].to_radians()),
            Vec3::new(s_t[0], s_t[1], s_t[2]),
        );

        let world_mat = bone_world * shape_local;
        let (scale, rotation, translation) = world_mat.to_scale_rotation_translation();
        let euler = rotation.to_euler(EulerRot::XYZ);

        let (combo_op, smooth_k) = if output.is_empty() {
            (0u32, 0.0f32)
        } else {
            combo_op_encode(&shape.combination)
        };

        output.push(FlatShape {
            primitive_type: prim_type_encode(&shape.primitive),
            params: prim_params_encode(&shape.primitive),
            combination_op: combo_op,
            smooth_k,
            translation,
            rotation: Vec3::new(euler.0, euler.1, euler.2),
            scale: scale.x,
            color: Vec3::new(shape.material.color[0], shape.material.color[1], shape.material.color[2]),
            roughness: shape.material.roughness,
            metallic: shape.material.metallic,
            fresnel_power: shape.material.fresnel_power,
            color_mode: shape.material.color_mode,
            palette_a: Vec3::from_array(shape.material.palette_a),
            palette_b: Vec3::from_array(shape.material.palette_b),
            palette_c: Vec3::from_array(shape.material.palette_c),
            palette_d: Vec3::from_array(shape.material.palette_d),
            modifier_flags: encode_modifier_flags(&shape.modifiers),
            rounding: get_modifier_f32(&shape.modifiers, 0),
            onion_thickness: get_modifier_f32(&shape.modifiers, 1),
            twist_amount: get_modifier_f32(&shape.modifiers, 2),
            bend_amount: get_modifier_f32(&shape.modifiers, 3),
            elongation: get_modifier_vec3(&shape.modifiers, 4),
            rep_period: get_modifier_vec3(&shape.modifiers, 5),
            noise_amplitude: shape.material.noise_amplitude,
            noise_frequency: shape.material.noise_frequency,
            noise_octaves: shape.material.noise_octaves,
            smooth_symmetry: shape.material.smooth_symmetry,
        });
    }

    for child in &bone.children {
        flatten_bone_tree(child, world_transforms, output);
    }
}

fn combo_op_encode(op: &CombinationOp) -> (u32, f32) {
    match op {
        CombinationOp::Union => (0, 0.0),
        CombinationOp::Intersection => (1, 0.0),
        CombinationOp::Subtraction => (2, 0.0),
        CombinationOp::SmoothUnion { k } => (3, *k),
        CombinationOp::SmoothIntersection { k } => (4, *k),
        CombinationOp::SmoothSubtraction { k } => (5, *k),
        CombinationOp::ChamferUnion { k } => (6, *k),
        CombinationOp::ChamferIntersection { k } => (7, *k),
    }
}

fn prim_type_encode(prim: &SdfPrimitive) -> u32 {
    match prim {
        SdfPrimitive::Sphere { .. } => 0,
        SdfPrimitive::Box { .. } => 1,
        SdfPrimitive::RoundBox { .. } => 2,
        SdfPrimitive::Cylinder { .. } => 3,
        SdfPrimitive::CappedCone { .. } => 4,
        SdfPrimitive::Torus { .. } => 5,
        SdfPrimitive::Capsule { .. } => 6,
        SdfPrimitive::Plane { .. } => 7,
        SdfPrimitive::Ellipsoid { .. } => 8,
        SdfPrimitive::Octahedron { .. } => 9,
        SdfPrimitive::Pyramid { .. } => 10,
        SdfPrimitive::HexPrism { .. } => 11,
        SdfPrimitive::RoundCone { .. } => 12,
    }
}

fn prim_params_encode(prim: &SdfPrimitive) -> Vec4 {
    match prim {
        SdfPrimitive::Sphere { radius } => Vec4::new(*radius, 0.0, 0.0, 0.0),
        SdfPrimitive::Box { half_extents } => Vec4::new(half_extents[0], half_extents[1], half_extents[2], 0.0),
        SdfPrimitive::RoundBox { half_extents, rounding } => Vec4::new(half_extents[0], half_extents[1], half_extents[2], *rounding),
        SdfPrimitive::Cylinder { height, radius } => Vec4::new(*height, *radius, 0.0, 0.0),
        SdfPrimitive::CappedCone { height, r1, r2 } => Vec4::new(*height, *r1, *r2, 0.0),
        SdfPrimitive::Torus { major_radius, minor_radius } => Vec4::new(*major_radius, *minor_radius, 0.0, 0.0),
        SdfPrimitive::Capsule { radius, half_height } => Vec4::new(*radius, *half_height, 0.0, 0.0),
        SdfPrimitive::Plane { normal, offset } => Vec4::new(normal[0], normal[1], normal[2], *offset),
        SdfPrimitive::Ellipsoid { radii } => Vec4::new(radii[0], radii[1], radii[2], 0.0),
        SdfPrimitive::Octahedron { size } => Vec4::new(*size, 0.0, 0.0, 0.0),
        SdfPrimitive::Pyramid { height, base } => Vec4::new(*height, *base, 0.0, 0.0),
        SdfPrimitive::HexPrism { height, radius } => Vec4::new(*height, *radius, 0.0, 0.0),
        SdfPrimitive::RoundCone { r1, r2, height } => Vec4::new(*r1, *r2, *height, 0.0),
    }
}

fn encode_modifier_flags(mods: &[ShapeModifier]) -> u32 {
    use ShapeModifier::*;
    let mut flags = 0u32;
    for m in mods {
        match m {
            Rounding(_) => flags |= 1,
            Onion(_) => flags |= 2,
            Twist(_) => flags |= 4,
            Bend(_) => flags |= 8,
            Elongation(_) => flags |= 16,
            Repetition { .. } => flags |= 32,
        }
    }
    flags
}

fn get_modifier_f32(mods: &[ShapeModifier], kind: u32) -> f32 {
    use ShapeModifier::*;
    for m in mods {
        match (kind, m) {
            (0, Rounding(v)) => return *v,
            (1, Onion(v)) => return *v,
            (2, Twist(v)) => return *v,
            (3, Bend(v)) => return *v,
            _ => {}
        }
    }
    0.0
}

fn get_modifier_vec3(mods: &[ShapeModifier], kind: u32) -> Vec3 {
    use ShapeModifier::*;
    for m in mods {
        match (kind, m) {
            (4, Elongation(v)) => return Vec3::from_array(*v),
            (5, Repetition { period, .. }) => return Vec3::from_array(*period),
            _ => {}
        }
    }
    Vec3::ZERO
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use glam::Mat4;

    #[test]
    fn flatten_default_scene() {
        let scene = SdfScene::default_scene();
        let world_transforms = compute_bone_world_transforms(&scene.root_bone, Mat4::IDENTITY, &HashMap::new());
        let mut flat = Vec::new();
        flatten_bone_tree(&scene.root_bone, &world_transforms, &mut flat);
        assert_eq!(flat.len(), 1, "default scene has one sphere");
        assert_eq!(flat[0].primitive_type, 0); // Sphere
    }

    #[test]
    fn bone_child_offsets_shape() {
        let mut root = SdfBone::root();
        let mut child = SdfBone::new("Arm");
        child.transform.translation = [2.0, 0.0, 0.0];
        child.shapes.push(SdfShape::default_sphere());
        root.children.push(child);

        let world_transforms = compute_bone_world_transforms(&root, Mat4::IDENTITY, &HashMap::new());
        let mut flat = Vec::new();
        flatten_bone_tree(&root, &world_transforms, &mut flat);
        assert_eq!(flat.len(), 1);
        assert!((flat[0].translation.x - 2.0).abs() < 0.001);
    }

    #[test]
    fn nested_bone_transforms_compose() {
        let mut root = SdfBone::root();
        let mut child = SdfBone::new("A");
        child.transform.translation = [1.0, 0.0, 0.0];
        let mut grandchild = SdfBone::new("B");
        grandchild.transform.translation = [0.0, 1.0, 0.0];
        grandchild.shapes.push(SdfShape::default_sphere());
        child.children.push(grandchild);
        root.children.push(child);

        let world_transforms = compute_bone_world_transforms(&root, Mat4::IDENTITY, &HashMap::new());
        let mut flat = Vec::new();
        flatten_bone_tree(&root, &world_transforms, &mut flat);
        assert_eq!(flat.len(), 1);
        assert!((flat[0].translation.x - 1.0).abs() < 0.001);
        assert!((flat[0].translation.y - 1.0).abs() < 0.001);
    }

    #[test]
    fn modifier_flags_encode() {
        let mods = vec![ShapeModifier::Rounding(0.1), ShapeModifier::Twist(2.0)];
        let flags = encode_modifier_flags(&mods);
        assert_eq!(flags & 1, 1);
        assert_eq!(flags & 4, 4);
        assert_eq!(flags & 2, 0);
        assert_eq!(get_modifier_f32(&mods, 0), 0.1);
        assert_eq!(get_modifier_f32(&mods, 2), 2.0);
    }
}
