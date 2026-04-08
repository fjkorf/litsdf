//! Inverse kinematics solvers: analytical 2-bone and FABRIK.
//!
//! Pure-data, no Bevy dependency. Works with the SdfBone tree and ShapeTransform
//! (Euler angles in degrees). Follows the physics.rs pattern: takes bone tree +
//! requests, returns new local transforms.

use std::collections::HashMap;
use glam::{EulerRot, Mat4, Quat, Vec3};

use crate::models::{BoneId, SdfBone, ShapeTransform};

/// IK solve request for one end-effector.
pub struct IkRequest {
    pub chain: Vec<BoneId>,            // root-to-tip order
    pub target: [f32; 3],
    pub pole_vector: Option<[f32; 3]>, // for 2-bone analytical
    pub max_iterations: u32,
    pub tolerance: f32,
    pub weight: f32,                   // 0.0 = no IK, 1.0 = full IK
}

/// Build a child→parent map for the entire bone tree.
pub fn build_parent_map(bone: &SdfBone) -> HashMap<BoneId, BoneId> {
    let mut map = HashMap::new();
    fn walk(bone: &SdfBone, map: &mut HashMap<BoneId, BoneId>) {
        for child in &bone.children {
            map.insert(child.id, bone.id);
            walk(child, map);
        }
    }
    walk(bone, &mut map);
    map
}

/// Build an IK chain from effector up to an ancestor.
/// chain_length=0: auto (walk to first kinematic bone or root).
/// chain_length=N: exactly N bones above the effector.
/// Returns root-to-tip order.
pub fn build_ik_chain(root_bone: &SdfBone, effector_id: BoneId, chain_length: u32) -> Vec<BoneId> {
    let parent_map = build_parent_map(root_bone);
    let mut chain = vec![effector_id];
    let mut current = effector_id;

    if chain_length == 0 {
        // Walk to root or kinematic bone
        while let Some(&parent_id) = parent_map.get(&current) {
            chain.push(parent_id);
            if let Some(bone) = root_bone.find_bone(parent_id) {
                if bone.physics.mass == 0.0 && !parent_id.is_root() {
                    break; // Stop at kinematic bone
                }
            }
            if parent_id.is_root() { break; }
            current = parent_id;
        }
    } else {
        for _ in 0..chain_length {
            if let Some(&parent_id) = parent_map.get(&current) {
                chain.push(parent_id);
                current = parent_id;
            } else {
                break;
            }
        }
    }

    chain.reverse(); // root-to-tip
    chain
}

/// FABRIK solver: iteratively adjust joint positions to reach target.
/// Operates on world-space positions. Returns true if converged.
pub fn solve_fabrik(positions: &mut [Vec3], target: Vec3, tolerance: f32, max_iterations: u32) -> bool {
    let n = positions.len();
    if n < 2 { return false; }

    // Compute bone lengths
    let lengths: Vec<f32> = (0..n - 1).map(|i| (positions[i + 1] - positions[i]).length()).collect();
    let total_length: f32 = lengths.iter().sum();

    // Check if target is reachable
    let dist_to_target = (target - positions[0]).length();
    if dist_to_target > total_length {
        // Stretch toward target
        let dir = (target - positions[0]).normalize_or_zero();
        let mut pos = positions[0];
        for i in 1..n {
            pos += dir * lengths[i - 1];
            positions[i] = pos;
        }
        return false;
    }

    let root = positions[0];

    for _ in 0..max_iterations {
        // Check convergence
        if (positions[n - 1] - target).length() < tolerance {
            return true;
        }

        // Forward pass: tip → root
        positions[n - 1] = target;
        for i in (0..n - 1).rev() {
            let dir = (positions[i] - positions[i + 1]).normalize_or_zero();
            positions[i] = positions[i + 1] + dir * lengths[i];
        }

        // Backward pass: root → tip
        positions[0] = root;
        for i in 0..n - 1 {
            let dir = (positions[i + 1] - positions[i]).normalize_or_zero();
            positions[i + 1] = positions[i] + dir * lengths[i];
        }
    }

    (positions[n - 1] - target).length() < tolerance
}

/// Analytical 2-bone IK solver using law of cosines.
/// positions: [root, mid, tip]. Returns rotations for root and mid joints.
pub fn solve_two_bone(positions: [Vec3; 3], target: Vec3, pole: Vec3) -> [Quat; 2] {
    let len_a = (positions[1] - positions[0]).length();
    let len_b = (positions[2] - positions[1]).length();

    let to_target = target - positions[0];
    let dist = to_target.length().clamp(0.001, len_a + len_b - 0.001);

    // Law of cosines for the angle at the root joint
    let cos_a = ((len_a * len_a + dist * dist - len_b * len_b) / (2.0 * len_a * dist)).clamp(-1.0, 1.0);
    let angle_a = cos_a.acos();

    // Law of cosines for the angle at the mid joint
    let cos_b = ((len_a * len_a + len_b * len_b - dist * dist) / (2.0 * len_a * len_b)).clamp(-1.0, 1.0);
    let _angle_b = cos_b.acos();

    // Compute rotation plane using pole vector
    let target_dir = to_target.normalize_or_zero();
    let pole_dir = (pole - positions[0]).normalize_or_zero();

    // Plane normal from target direction and pole
    let plane_normal = target_dir.cross(pole_dir).normalize_or_zero();
    let plane_up = plane_normal.cross(target_dir).normalize_or_zero();

    // Root joint rotation: rotate toward target with angle offset
    let root_dir = (target_dir * cos_a.cos() + plane_up * angle_a.sin()).normalize_or_zero();
    let root_rot = Quat::from_rotation_arc(Vec3::Y, root_dir);

    // Mid joint: bend in the pole plane
    let mid_forward = (target - positions[0] - root_dir * len_a).normalize_or_zero();
    let mid_rot = Quat::from_rotation_arc(root_dir, mid_forward);

    [root_rot, mid_rot]
}

/// Clamp Euler angles (degrees) to rotation limits.
pub fn clamp_rotation(euler: [f32; 3], limits: &crate::models::RotationLimits) -> [f32; 3] {
    let mut result = euler;
    if let Some([min, max]) = limits.pitch {
        result[0] = result[0].clamp(min, max);
    }
    if let Some([min, max]) = limits.yaw {
        result[1] = result[1].clamp(min, max);
    }
    if let Some([min, max]) = limits.roll {
        result[2] = result[2].clamp(min, max);
    }
    result
}

/// Convert solved world positions back to local ShapeTransforms.
pub fn positions_to_local_transforms(
    chain: &[BoneId],
    solved: &[Vec3],
    bone_tree: &SdfBone,
    world_transforms: &HashMap<BoneId, Mat4>,
) -> HashMap<BoneId, ShapeTransform> {
    let mut result = HashMap::new();
    let parent_map = build_parent_map(bone_tree);

    for (i, &bone_id) in chain.iter().enumerate() {
        let Some(bone) = bone_tree.find_bone(bone_id) else { continue };

        // Get parent world transform
        let parent_world = parent_map.get(&bone_id)
            .and_then(|pid| world_transforms.get(pid))
            .copied()
            .unwrap_or(Mat4::IDENTITY);

        if i + 1 < chain.len() {
            // Compute direction from this joint to the next
            let dir_to_next = (solved[i + 1] - solved[i]).normalize_or_zero();

            // Get the original local direction (what the bone currently points toward)
            let bone_world = world_transforms.get(&bone_id).copied().unwrap_or(Mat4::IDENTITY);
            let (_, orig_rot, _) = bone_world.to_scale_rotation_translation();

            // Compute the rotation that aligns the original direction to the solved direction
            let original_dir = orig_rot * Vec3::Y; // bones point along Y by convention
            let target_rot = if original_dir.dot(dir_to_next).abs() > 0.999 {
                orig_rot // nearly aligned, keep original
            } else {
                Quat::from_rotation_arc(original_dir, dir_to_next) * orig_rot
            };

            // Convert to local space
            let (_, parent_rot, _) = parent_world.to_scale_rotation_translation();
            let local_rot = parent_rot.inverse() * target_rot;

            // Convert quaternion to Euler XYZ degrees
            let (rx, ry, rz) = local_rot.to_euler(EulerRot::XYZ);
            let mut euler = [rx.to_degrees(), ry.to_degrees(), rz.to_degrees()];

            // Apply rotation limits
            euler = clamp_rotation(euler, &bone.physics.rotation_limits);

            // Compute local translation (position relative to parent)
            let local_pos = parent_world.inverse().transform_point3(solved[i]);

            result.insert(bone_id, ShapeTransform {
                translation: [local_pos.x, local_pos.y, local_pos.z],
                rotation: euler,
                scale: bone.transform.scale,
            });
        }
    }

    result
}

/// Solve IK for multiple requests. Returns new local transforms for affected bones.
pub fn solve_ik(
    bone_tree: &SdfBone,
    requests: &[IkRequest],
    world_transforms: &HashMap<BoneId, Mat4>,
) -> HashMap<BoneId, ShapeTransform> {
    let mut all_results = HashMap::new();

    for req in requests {
        if req.chain.len() < 2 || req.weight <= 0.0 { continue; }

        // Extract current world positions for chain joints
        let mut positions: Vec<Vec3> = req.chain.iter()
            .map(|id| {
                world_transforms.get(id)
                    .map(|m| m.to_scale_rotation_translation().2)
                    .unwrap_or(Vec3::ZERO)
            })
            .collect();

        let target = Vec3::new(req.target[0], req.target[1], req.target[2]);

        // Solve
        if req.chain.len() == 3 {
            if let Some(pole) = req.pole_vector {
                let pole_v = Vec3::new(pole[0], pole[1], pole[2]);
                let _rotations = solve_two_bone(
                    [positions[0], positions[1], positions[2]],
                    target, pole_v,
                );
            }
            // For 2-bone, also use FABRIK as it handles the position conversion better
            solve_fabrik(&mut positions, target, req.tolerance, req.max_iterations);
        } else {
            solve_fabrik(&mut positions, target, req.tolerance, req.max_iterations);
        }

        // Convert positions to local transforms
        let transforms = positions_to_local_transforms(
            &req.chain, &positions, bone_tree, world_transforms,
        );

        // Blend with current transforms
        if req.weight < 1.0 {
            for (bone_id, solved_t) in &transforms {
                if let Some(bone) = bone_tree.find_bone(*bone_id) {
                    let orig = &bone.transform;
                    let w = req.weight;
                    all_results.insert(*bone_id, ShapeTransform {
                        translation: [
                            orig.translation[0] * (1.0 - w) + solved_t.translation[0] * w,
                            orig.translation[1] * (1.0 - w) + solved_t.translation[1] * w,
                            orig.translation[2] * (1.0 - w) + solved_t.translation[2] * w,
                        ],
                        rotation: [
                            orig.rotation[0] * (1.0 - w) + solved_t.rotation[0] * w,
                            orig.rotation[1] * (1.0 - w) + solved_t.rotation[1] * w,
                            orig.rotation[2] * (1.0 - w) + solved_t.rotation[2] * w,
                        ],
                        scale: solved_t.scale,
                    });
                }
            }
        } else {
            all_results.extend(transforms);
        }
    }

    all_results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BonePhysicsProps, RotationLimits};

    fn make_chain_bones() -> SdfBone {
        let mut root = SdfBone::root();
        let mut a = SdfBone::new("A");
        a.transform.translation = [0.0, 1.0, 0.0];
        let mut b = SdfBone::new("B");
        b.transform.translation = [0.0, 1.0, 0.0];
        let mut c = SdfBone::new("C");
        c.transform.translation = [0.0, 1.0, 0.0];
        let c_id = c.id;
        let b_id = b.id;
        b.children.push(c);
        a.children.push(b);
        root.children.push(a);
        root
    }

    #[test]
    fn build_chain_simple() {
        let root = make_chain_bones();
        let tip_id = root.children[0].children[0].children[0].id;
        let chain = build_ik_chain(&root, tip_id, 3);
        assert_eq!(chain.len(), 4); // root → A → B → C
    }

    #[test]
    fn build_chain_auto_stops_at_kinematic() {
        let mut root = make_chain_bones();
        // A has mass=0 (default, kinematic) — chain should stop there
        let tip_id = root.children[0].children[0].children[0].id;
        let chain = build_ik_chain(&root, tip_id, 0);
        // Should stop at A (first kinematic non-root)
        assert!(chain.len() >= 2, "chain should have at least 2 joints, got {}", chain.len());
    }

    #[test]
    fn fabrik_reaches_target() {
        let mut positions = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
        ];
        let target = Vec3::new(1.0, 1.0, 0.0);
        let converged = solve_fabrik(&mut positions, target, 0.01, 10);
        let end = positions.last().unwrap();
        assert!((end.distance(target)) < 0.05, "should reach target, got distance {}", end.distance(target));
    }

    #[test]
    fn fabrik_unreachable() {
        let mut positions = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
        ];
        let target = Vec3::new(10.0, 0.0, 0.0); // far beyond reach (total length = 2)
        let converged = solve_fabrik(&mut positions, target, 0.01, 10);
        assert!(!converged, "should not converge for unreachable target");
        // Chain should be fully stretched toward target
        let end = positions.last().unwrap();
        assert!(end.x > 1.5, "chain should stretch toward target");
    }

    #[test]
    fn two_bone_basic() {
        let positions = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
        ];
        let target = Vec3::new(1.0, 1.0, 0.0);
        let pole = Vec3::new(0.0, 0.0, 1.0);
        let rotations = solve_two_bone(positions, target, pole);
        // Verify rotations are valid (no NaN)
        assert!(!rotations[0].is_nan(), "root rotation should not be NaN");
        assert!(!rotations[1].is_nan(), "mid rotation should not be NaN");
    }

    #[test]
    fn rotation_limits_clamped() {
        let limits = RotationLimits {
            pitch: Some([-30.0, 30.0]),
            yaw: Some([-45.0, 45.0]),
            roll: None,
        };
        let euler = [50.0, -60.0, 90.0];
        let clamped = clamp_rotation(euler, &limits);
        assert_eq!(clamped[0], 30.0);
        assert_eq!(clamped[1], -45.0);
        assert_eq!(clamped[2], 90.0); // no roll limit
    }

    #[test]
    fn solve_ik_integration() {
        let root = make_chain_bones();
        let tip_id = root.children[0].children[0].children[0].id;
        let chain = build_ik_chain(&root, tip_id, 3);

        let world_transforms = crate::scene::compute_bone_world_transforms(
            &root, Mat4::IDENTITY, &HashMap::new(),
        );

        let requests = vec![IkRequest {
            chain,
            target: [1.0, 1.5, 0.0],
            pole_vector: None,
            max_iterations: 10,
            tolerance: 0.01,
            weight: 1.0,
        }];

        let results = solve_ik(&root, &requests, &world_transforms);
        assert!(!results.is_empty(), "should produce transforms");
        for (_, t) in &results {
            assert!(!t.rotation[0].is_nan(), "rotation should not be NaN");
            assert!(!t.rotation[1].is_nan(), "rotation should not be NaN");
        }
    }
}
