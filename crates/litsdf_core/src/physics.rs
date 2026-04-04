//! Simple bone physics simulation.
//!
//! Physics produces position offsets (not absolute transforms), making it trivial
//! to blend with animation: animation sets the base transform, physics adds
//! displacement on top.
//!
//! Semi-implicit Euler integration with fixed timestep for stability.

use std::collections::HashMap;

use crate::models::{BoneId, SdfBone};

const FIXED_DT: f32 = 1.0 / 60.0;

/// Runtime physics state per bone (not serialized).
#[derive(Debug, Clone)]
pub struct BonePhysicsState {
    pub velocity: [f32; 3],
    pub accumulator: f32,
}

impl Default for BonePhysicsState {
    fn default() -> Self {
        Self {
            velocity: [0.0, 0.0, 0.0],
            accumulator: 0.0,
        }
    }
}

/// Step physics for all dynamic bones (mass > 0).
/// Returns position offsets to add on top of base transforms.
pub fn step_physics(
    bone: &SdfBone,
    states: &mut HashMap<BoneId, BonePhysicsState>,
    gravity: f32,
    dt: f32,
) -> HashMap<BoneId, [f32; 3]> {
    let mut offsets = HashMap::new();
    step_bone(bone, states, gravity, dt, &mut offsets);
    offsets
}

fn step_bone(
    bone: &SdfBone,
    states: &mut HashMap<BoneId, BonePhysicsState>,
    gravity: f32,
    dt: f32,
    offsets: &mut HashMap<BoneId, [f32; 3]>,
) {
    if bone.physics.mass > 0.0 {
        let state = states.entry(bone.id).or_default();
        state.accumulator += dt;

        let damping = bone.physics.damping;
        let mut offset = [0.0f32; 3];

        // Fixed timestep integration
        while state.accumulator >= FIXED_DT {
            // Semi-implicit Euler: update velocity first, then position
            state.velocity[1] += gravity * FIXED_DT;

            // Apply damping
            state.velocity[0] *= damping;
            state.velocity[1] *= damping;
            state.velocity[2] *= damping;

            // Accumulate offset
            offset[0] += state.velocity[0] * FIXED_DT;
            offset[1] += state.velocity[1] * FIXED_DT;
            offset[2] += state.velocity[2] * FIXED_DT;

            state.accumulator -= FIXED_DT;
        }

        if offset != [0.0, 0.0, 0.0] {
            offsets.insert(bone.id, offset);
        }
    }

    for child in &bone.children {
        step_bone(child, states, gravity, dt, offsets);
    }
}

/// Approximate a physics collider from a bone's shapes.
pub fn approximate_collider(bone: &SdfBone) -> crate::models::ColliderApprox {
    use crate::models::{ColliderApprox, SdfPrimitive};

    if bone.shapes.len() == 1 {
        let shape = &bone.shapes[0];
        match &shape.primitive {
            SdfPrimitive::Sphere { radius } => {
                return ColliderApprox::Sphere { radius: *radius * shape.transform.scale };
            }
            SdfPrimitive::Capsule { radius, half_height } => {
                let s = shape.transform.scale;
                return ColliderApprox::Capsule { radius: *radius * s, half_height: *half_height * s };
            }
            SdfPrimitive::Box { half_extents } | SdfPrimitive::RoundBox { half_extents, .. } => {
                let s = shape.transform.scale;
                return ColliderApprox::Box {
                    half_extents: [half_extents[0] * s, half_extents[1] * s, half_extents[2] * s],
                };
            }
            SdfPrimitive::Cylinder { height, radius } => {
                let s = shape.transform.scale;
                return ColliderApprox::Capsule { radius: *radius * s, half_height: *height * 0.5 * s };
            }
            _ => {}
        }
    }

    // Fallback: bounding sphere from all shapes
    let mut max_r: f32 = 0.2; // minimum radius
    for shape in &bone.shapes {
        let s = shape.transform.scale;
        let t = shape.transform.translation;
        let offset = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt();
        let shape_r = match &shape.primitive {
            SdfPrimitive::Sphere { radius } => *radius * s,
            SdfPrimitive::Capsule { radius, half_height } => (*radius + *half_height) * s,
            SdfPrimitive::Box { half_extents } | SdfPrimitive::RoundBox { half_extents, .. } => {
                (half_extents[0].max(half_extents[1]).max(half_extents[2])) * s
            }
            SdfPrimitive::Torus { major_radius, minor_radius } => (*major_radius + *minor_radius) * s,
            _ => 0.3 * s,
        };
        max_r = max_r.max(offset + shape_r);
    }
    ColliderApprox::Sphere { radius: max_r }
}

/// Convert multiplicative per-step damping (0..1) to Avian's linear damping coefficient.
pub fn damping_to_avian(custom: f32) -> f32 {
    -60.0 * custom.max(0.001).ln()
}

/// Reset all physics velocities and accumulators to zero.
pub fn reset_physics(states: &mut HashMap<BoneId, BonePhysicsState>) {
    for state in states.values_mut() {
        state.velocity = [0.0, 0.0, 0.0];
        state.accumulator = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BonePhysicsProps, SdfBone};

    fn bone_with_mass(mass: f32) -> SdfBone {
        let mut bone = SdfBone::new("test");
        bone.physics = BonePhysicsProps { mass, damping: 1.0, ..Default::default() };
        bone
    }

    #[test]
    fn zero_mass_no_offset() {
        let bone = SdfBone::new("kinematic");
        let mut states = HashMap::new();
        let offsets = step_physics(&bone, &mut states, -9.81, 1.0 / 60.0);
        assert!(offsets.is_empty());
    }

    #[test]
    fn positive_mass_falls() {
        let bone = bone_with_mass(1.0);
        let mut states = HashMap::new();
        // Step for one fixed timestep
        let offsets = step_physics(&bone, &mut states, -9.81, 1.0 / 60.0);
        assert!(offsets.contains_key(&bone.id));
        let offset = offsets[&bone.id];
        assert!(offset[1] < 0.0, "should fall downward");
        assert_eq!(offset[0], 0.0);
        assert_eq!(offset[2], 0.0);
    }

    #[test]
    fn reset_zeroes_velocity() {
        let bone = bone_with_mass(1.0);
        let mut states = HashMap::new();
        step_physics(&bone, &mut states, -9.81, 0.1);
        assert!(states[&bone.id].velocity[1] < 0.0);
        reset_physics(&mut states);
        assert_eq!(states[&bone.id].velocity, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn damping_reduces_velocity() {
        let mut bone = bone_with_mass(1.0);
        bone.physics.damping = 0.5; // heavy damping
        let mut states = HashMap::new();
        // Run several frames
        for _ in 0..10 {
            step_physics(&bone, &mut states, -9.81, 1.0 / 60.0);
        }
        let v = states[&bone.id].velocity[1];
        // With 0.5 damping the velocity should be much less than free-fall
        assert!(v > -2.0, "heavy damping should limit velocity, got {v}");
    }

    #[test]
    fn collider_sphere() {
        use crate::models::{ColliderApprox, SdfPrimitive, SdfShape};
        let mut bone = SdfBone::new("test");
        bone.shapes.push(SdfShape::new("S", SdfPrimitive::Sphere { radius: 0.5 }));
        assert_eq!(approximate_collider(&bone), ColliderApprox::Sphere { radius: 0.5 });
    }

    #[test]
    fn collider_capsule() {
        use crate::models::{ColliderApprox, SdfPrimitive, SdfShape};
        let mut bone = SdfBone::new("test");
        bone.shapes.push(SdfShape::new("C", SdfPrimitive::Capsule { radius: 0.1, half_height: 0.3 }));
        assert_eq!(approximate_collider(&bone), ColliderApprox::Capsule { radius: 0.1, half_height: 0.3 });
    }

    #[test]
    fn collider_fallback_bounding() {
        use crate::models::ColliderApprox;
        // No shapes → fallback sphere with min radius 0.2
        let bone = SdfBone::new("empty");
        match approximate_collider(&bone) {
            ColliderApprox::Sphere { radius } => assert!((radius - 0.2).abs() < 0.01),
            _ => panic!("expected sphere fallback"),
        }
    }

    #[test]
    fn damping_conversion() {
        let avian = damping_to_avian(0.95);
        assert!(avian > 2.0 && avian < 4.0, "0.95 damping should map to ~3.08, got {avian}");
        let heavy = damping_to_avian(0.70);
        assert!(heavy > 20.0, "0.70 damping should map to ~21, got {heavy}");
    }
}
