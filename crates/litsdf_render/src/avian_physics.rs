//! Avian3D physics integration.
//!
//! Creates a shadow entity layer that mirrors the bone hierarchy as Bevy entities
//! with RigidBody, Collider, and Joint components. Avian simulates them each frame,
//! and transforms are synced back to the bone tree for shader submission.

use std::collections::HashMap;

use avian3d::prelude::*;
use bevy::prelude::*;

use litsdf_core::models::{BoneId, ColliderApprox, SdfBone};
use litsdf_core::physics::{approximate_collider, damping_to_avian};
use litsdf_core::scene::compute_bone_world_transforms;

use crate::scene_sync::SdfSceneState;

/// Marker component linking an Avian entity to a bone.
#[derive(Component)]
pub struct PhysicsBoneMarker(pub BoneId);

/// Marker for the ground plane entity.
#[derive(Component)]
pub struct PhysicsGroundPlane;

/// Tracks the Avian shadow entities.
#[derive(Resource, Default)]
pub struct AvianPhysicsState {
    pub entity_map: HashMap<BoneId, Entity>,
    pub last_topology_hash: u64,
    pub has_ground_plane: bool,
}

pub struct SdfPhysicsPlugin;

impl Plugin for SdfPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default())
            .init_resource::<AvianPhysicsState>()
            // Spawn/despawn in Update (commands apply before FixedPostUpdate)
            .add_systems(Update, sync_physics_pause)
            .add_systems(Update, check_and_spawn)
            // Avian runs in FixedPostUpdate — sync systems run there too
            .add_systems(FixedPostUpdate, apply_node_forces.before(PhysicsSystems::Prepare))
            .add_systems(FixedPostUpdate, sync_kinematic_to_avian.before(PhysicsSystems::Prepare))
            .add_systems(FixedPostUpdate, sync_avian_to_bones.after(PhysicsSystems::Writeback))
            .add_systems(FixedPostUpdate, collect_physics_readings.after(PhysicsSystems::Writeback));
    }
}

/// Pause/unpause Avian's physics time based on scene state.
fn sync_physics_pause(
    scene: Res<SdfSceneState>,
    mut physics_time: ResMut<Time<Physics>>,
) {
    if scene.physics_paused || !scene.use_avian {
        physics_time.pause();
    } else {
        physics_time.unpause();
    }
}

/// Check if physics entities need to be (re)spawned due to topology change.
fn check_and_spawn(
    mut commands: Commands,
    scene: Res<SdfSceneState>,
    mut state: ResMut<AvianPhysicsState>,
    markers: Query<Entity, With<PhysicsBoneMarker>>,
    ground_planes: Query<Entity, With<PhysicsGroundPlane>>,
) {
    if !scene.use_avian {
        if !state.entity_map.is_empty() {
            despawn_all(&mut commands, &mut state, &markers, &ground_planes);
        }
        return;
    }

    let hash = scene.topology_hash;
    if hash == state.last_topology_hash && !state.entity_map.is_empty() {
        // Check ground plane state
        let want_ground = scene.scene.settings.ground_plane;
        if want_ground && !state.has_ground_plane {
            spawn_ground_plane(&mut commands);
            state.has_ground_plane = true;
        } else if !want_ground && state.has_ground_plane {
            for entity in ground_planes.iter() {
                commands.entity(entity).despawn();
            }
            state.has_ground_plane = false;
        }
        return;
    }

    // Topology changed — despawn and respawn
    despawn_all(&mut commands, &mut state, &markers, &ground_planes);
    spawn_physics_entities(&mut commands, &scene, &mut state);
    state.last_topology_hash = hash;
}

fn despawn_all(
    commands: &mut Commands,
    state: &mut AvianPhysicsState,
    markers: &Query<Entity, With<PhysicsBoneMarker>>,
    ground_planes: &Query<Entity, With<PhysicsGroundPlane>>,
) {
    for entity in markers.iter() {
        commands.entity(entity).despawn();
    }
    for entity in ground_planes.iter() {
        commands.entity(entity).despawn();
    }
    state.entity_map.clear();
    state.has_ground_plane = false;
}

fn spawn_ground_plane(commands: &mut Commands) {
    commands.spawn((
        PhysicsGroundPlane,
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
        Position(Vec3::ZERO.into()),
    ));
}

fn spawn_physics_entities(
    commands: &mut Commands,
    scene: &SdfSceneState,
    state: &mut AvianPhysicsState,
) {
    let world_transforms = compute_bone_world_transforms(
        &scene.scene.root_bone,
        Mat4::IDENTITY,
        &HashMap::new(),
    );

    spawn_bone_entity(commands, &scene.scene.root_bone, &world_transforms, state, true);
    create_joints(commands, &scene.scene.root_bone, state);

    if scene.scene.settings.ground_plane {
        spawn_ground_plane(commands);
        state.has_ground_plane = true;
    }
}

fn spawn_bone_entity(
    commands: &mut Commands,
    bone: &SdfBone,
    world_transforms: &HashMap<BoneId, Mat4>,
    state: &mut AvianPhysicsState,
    is_root: bool,
) {
    let world_mat = world_transforms.get(&bone.id).copied().unwrap_or(Mat4::IDENTITY);
    let (_, rot, pos) = world_mat.to_scale_rotation_translation();

    let transform = Transform::from_rotation(rot).with_translation(pos);

    if is_root {
        let entity = commands.spawn((
            PhysicsBoneMarker(bone.id),
            RigidBody::Static,
            transform,
        )).id();
        state.entity_map.insert(bone.id, entity);
    } else if bone.physics.mass > 0.0 {
        let collider = to_avian_collider(&approximate_collider(bone));
        let entity = commands.spawn((
            PhysicsBoneMarker(bone.id),
            RigidBody::Dynamic,
            collider,
            Mass(bone.physics.mass),
            LinearDamping(damping_to_avian(bone.physics.damping)),
            AngularDamping(damping_to_avian(bone.physics.damping)),
            SleepingDisabled,
            transform,
        )).id();
        state.entity_map.insert(bone.id, entity);
    } else {
        let entity = commands.spawn((
            PhysicsBoneMarker(bone.id),
            RigidBody::Kinematic,
            transform,
        )).id();
        state.entity_map.insert(bone.id, entity);
    }

    for child in &bone.children {
        spawn_bone_entity(commands, child, world_transforms, state, false);
    }
}

fn create_joints(
    commands: &mut Commands,
    bone: &SdfBone,
    state: &AvianPhysicsState,
) {
    for child in &bone.children {
        // Only create joints when the child is dynamic (has physics)
        if child.physics.mass <= 0.0 && bone.physics.mass <= 0.0 {
            create_joints(commands, child, state);
            continue;
        }
        if let (Some(&parent_entity), Some(&child_entity)) =
            (state.entity_map.get(&bone.id), state.entity_map.get(&child.id))
        {
            let offset = Vec3::new(
                child.transform.translation[0],
                child.transform.translation[1],
                child.transform.translation[2],
            );
            let distance = offset.length().max(0.01);

            // DistanceJoint allows pendulum-like swinging at fixed distance
            let joint = DistanceJoint {
                body1: parent_entity,
                body2: child_entity,
                anchor1: JointAnchor::default(),
                anchor2: JointAnchor::default(),
                limits: DistanceLimit { min: distance, max: distance },
                compliance: 0.0,
            };

            commands.spawn(joint);
        }

        create_joints(commands, child, state);
    }
}

fn to_avian_collider(approx: &ColliderApprox) -> Collider {
    match approx {
        ColliderApprox::Sphere { radius } => Collider::sphere(*radius),
        ColliderApprox::Capsule { radius, half_height } => {
            Collider::capsule(*radius, (*half_height * 2.0))
        }
        ColliderApprox::Box { half_extents } => {
            Collider::cuboid(
                (half_extents[0] * 2.0),
                (half_extents[1] * 2.0),
                (half_extents[2] * 2.0),
            )
        }
    }
}

/// Write animated (kinematic) bone transforms into Avian entities.
fn sync_kinematic_to_avian(
    scene: Res<SdfSceneState>,
    state: Res<AvianPhysicsState>,
    mut positions: Query<&mut Position>,
    mut rotations: Query<&mut Rotation>,
) {
    if !scene.use_avian || state.entity_map.is_empty() || scene.physics_paused { return; }

    let world_transforms = compute_bone_world_transforms(
        &scene.scene.root_bone,
        Mat4::IDENTITY,
        &HashMap::new(),
    );

    sync_kinematic_bone(
        &scene.scene.root_bone, &world_transforms, &state,
        &mut positions, &mut rotations,
    );
}

fn sync_kinematic_bone(
    bone: &SdfBone,
    world_transforms: &HashMap<BoneId, Mat4>,
    state: &AvianPhysicsState,
    positions: &mut Query<&mut Position>,
    rotations: &mut Query<&mut Rotation>,
) {
    if bone.physics.mass == 0.0 || bone.id.is_root() {
        if let Some(&entity) = state.entity_map.get(&bone.id) {
            if let Some(mat) = world_transforms.get(&bone.id) {
                let (_, rot, pos) = mat.to_scale_rotation_translation();
                if let Ok(mut p) = positions.get_mut(entity) {
                    p.0 = pos.into();
                }
                if let Ok(mut r) = rotations.get_mut(entity) {
                    r.0 = rot.into();
                }
            }
        }
    }

    for child in &bone.children {
        sync_kinematic_bone(child, world_transforms, state, positions, rotations);
    }
}

/// Read Avian dynamic entity transforms back into the bone tree.
///
/// Uses Avian entity transforms as the authoritative parent world transforms
/// (not the bone tree) to avoid euler round-trip error accumulation in chains.
fn sync_avian_to_bones(
    mut scene: ResMut<SdfSceneState>,
    state: Res<AvianPhysicsState>,
    positions: Query<&Position>,
    rotations: Query<&Rotation>,
) {
    if !scene.use_avian || state.entity_map.is_empty() || scene.physics_paused { return; }

    // Build world transforms from Avian entities — these are authoritative
    // and don't suffer from euler round-trip error.
    let mut avian_world: HashMap<BoneId, Mat4> = HashMap::new();
    for (bone_id, entity) in &state.entity_map {
        if let (Ok(pos), Ok(rot)) = (positions.get(*entity), rotations.get(*entity)) {
            let p: Vec3 = pos.0.into();
            let r: Quat = rot.0.into();
            avian_world.insert(*bone_id, Mat4::from_rotation_translation(r, p));
        }
    }

    let mut updates: Vec<(BoneId, [f32; 3], [f32; 3])> = Vec::new();

    // Walk tree, passing parent bone ID so we can look up parent in avian_world
    collect_dynamic_updates(
        &scene.scene.root_bone,
        None, // root has no parent
        &avian_world,
        &mut updates,
    );

    if !updates.is_empty() {
        for (bone_id, translation, rotation) in &updates {
            if let Some(bone) = scene.scene.root_bone.find_bone_mut(*bone_id) {
                bone.transform.translation = *translation;
                bone.transform.rotation = *rotation;
            }
        }
        scene.dirty = true;
    }
}

fn collect_dynamic_updates(
    bone: &SdfBone,
    parent_id: Option<BoneId>,
    avian_world: &HashMap<BoneId, Mat4>,
    updates: &mut Vec<(BoneId, [f32; 3], [f32; 3])>,
) {
    if bone.physics.mass > 0.0 {
        if let Some(my_world) = avian_world.get(&bone.id) {
            // Use parent's Avian world transform (authoritative, no euler error)
            let parent_world = parent_id
                .and_then(|pid| avian_world.get(&pid))
                .copied()
                .unwrap_or(Mat4::IDENTITY);

            let local = parent_world.inverse() * *my_world;
            let (_, local_rot, local_pos) = local.to_scale_rotation_translation();
            let (rx, ry, rz) = local_rot.to_euler(EulerRot::XYZ);

            updates.push((
                bone.id,
                [local_pos.x, local_pos.y, local_pos.z],
                [rx.to_degrees(), ry.to_degrees(), rz.to_degrees()],
            ));
        }
    }

    for child in &bone.children {
        collect_dynamic_updates(child, Some(bone.id), avian_world, updates);
    }
}

/// Collect physics state from Avian entities into scene.physics_readings
/// so node graphs can read velocity, position, etc.
fn collect_physics_readings(
    mut scene: ResMut<SdfSceneState>,
    state: Res<AvianPhysicsState>,
    positions: Query<&Position>,
    linear_vels: Query<&LinearVelocity>,
    angular_vels: Query<&AngularVelocity>,
) {
    scene.physics_readings.clear();
    if !scene.use_avian { return; }

    for (bone_id, &entity) in &state.entity_map {
        let mut reading = crate::scene_sync::BonePhysicsReading::default();
        if let Ok(pos) = positions.get(entity) {
            let p: Vec3 = pos.0.into();
            reading.position = [p.x, p.y, p.z];
        }
        if let Ok(vel) = linear_vels.get(entity) {
            let v: Vec3 = vel.0.into();
            reading.linear_velocity = [v.x, v.y, v.z];
        }
        if let Ok(avel) = angular_vels.get(entity) {
            let a: Vec3 = avel.0.into();
            reading.angular_velocity = [a.x, a.y, a.z];
        }
        scene.physics_readings.insert(*bone_id, reading);
    }
}

/// Apply force/torque outputs from node graphs to Avian entities.
fn apply_node_forces(
    scene: Res<SdfSceneState>,
    state: Res<AvianPhysicsState>,
    mut linear_vels: Query<&mut LinearVelocity>,
    mut angular_vels: Query<&mut AngularVelocity>,
    masses: Query<&Mass>,
) {
    if !scene.use_avian || scene.physics_paused { return; }

    // Apply forces as velocity changes (F=ma → dv = F/m * dt, approximate with 1/60)
    let dt = 1.0 / 60.0f32;
    for (bone_id, outputs) in &scene.force_outputs {
        if let Some(&entity) = state.entity_map.get(bone_id) {
            let f = Vec3::new(outputs.force[0], outputs.force[1], outputs.force[2]);
            let t = Vec3::new(outputs.torque[0], outputs.torque[1], outputs.torque[2]);
            if f.length_squared() > 0.0 {
                let mass = masses.get(entity).map(|m| m.0).unwrap_or(1.0);
                if let Ok(mut vel) = linear_vels.get_mut(entity) {
                    let v: Vec3 = vel.0.into();
                    vel.0 = (v + f * dt / mass).into();
                }
            }
            if t.length_squared() > 0.0 {
                if let Ok(mut avel) = angular_vels.get_mut(entity) {
                    let a: Vec3 = avel.0.into();
                    avel.0 = (a + t * dt).into();
                }
            }
        }
    }
}
