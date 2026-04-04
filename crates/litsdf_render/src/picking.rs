use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::input::EguiWantsInput;

use crate::camera::OrbitCamera;
use litsdf_core::models::{BoneId, SdfBone, SdfPrimitive, SdfScene, ShapeId, ShapeModifier};
use crate::scene_sync::SdfSceneState;
use litsdf_core::scene::compute_bone_world_transforms;

const MAX_STEPS: u32 = 64;
const MAX_DIST: f32 = 100.0;
const EPSILON: f32 = 0.005;
const CLICK_THRESHOLD: f32 = 3.0;

// ── Gizmo mode ─────────────────────────────────────────────────

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum GizmoMode {
    #[default]
    Translate,
    Rotate,
    Elongation,
    Repetition,
}

impl GizmoMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Translate => "Translate (G)",
            Self::Rotate => "Rotate (R)",
            Self::Elongation => "Elongation (E)",
            Self::Repetition => "Repetition (P)",
        }
    }
}

// ── Evaluate single shape at world point ────────────────────────

struct WorldShape {
    shape_id: ShapeId,
    bone_id: BoneId,
    translation: Vec3,
    rotation: Vec3,
    scale: f32,
    primitive: SdfPrimitive,
}

fn eval_world_shape(p: Vec3, s: &WorldShape) -> f32 {
    let q = litsdf_core::sdf::rotate_point((p - s.translation) / s.scale, -s.rotation);
    let d = litsdf_core::sdf::eval_primitive(q, &s.primitive);
    d * s.scale
}

fn collect_world_shapes(bone: &SdfBone, world_transforms: &HashMap<BoneId, Mat4>, out: &mut Vec<WorldShape>) {
    let bone_world = world_transforms[&bone.id];
    for shape in &bone.shapes {
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
        out.push(WorldShape {
            shape_id: shape.id, bone_id: bone.id,
            translation, rotation: Vec3::new(euler.0, euler.1, euler.2),
            scale: scale.x, primitive: shape.primitive.clone(),
        });
    }
    for child in &bone.children {
        collect_world_shapes(child, world_transforms, out);
    }
}

// ── Ray march and pick ──────────────────────────────────────────

fn sdf_scene(p: Vec3, shapes: &[WorldShape]) -> f32 {
    let mut d = MAX_DIST;
    for s in shapes { d = d.min(eval_world_shape(p, s)); }
    d
}

pub fn pick_shape(ray: Ray3d, scene: &SdfScene) -> Option<(ShapeId, BoneId)> {
    let overrides = HashMap::new();
    let world_transforms = compute_bone_world_transforms(&scene.root_bone, Mat4::IDENTITY, &overrides);
    let mut shapes = Vec::new();
    collect_world_shapes(&scene.root_bone, &world_transforms, &mut shapes);
    if shapes.is_empty() { return None; }

    let mut t = 0.0_f32;
    for _ in 0..MAX_STEPS {
        let p = ray.origin + *ray.direction * t;
        let d = sdf_scene(p, &shapes);
        if d < EPSILON {
            let mut best = None;
            let mut best_d = f32::MAX;
            for s in &shapes {
                let sd = eval_world_shape(p, s).abs();
                if sd < best_d { best_d = sd; best = Some((s.shape_id, s.bone_id)); }
            }
            return best;
        }
        t += d;
        if t > MAX_DIST { break; }
    }
    None
}

// ── Bevy systems ───────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct ClickTracker { press_pos: Option<Vec2> }

#[derive(Resource, Default)]
pub struct DragState {
    pub active: bool,
    pub axis: Vec3,         // local axis direction in world space (for projection)
    pub axis_index: usize,  // 0=X, 1=Y, 2=Z (which component to modify)
    pub start_world_pos: Vec3,
    pub start_value: [f32; 3],
    pub start_cursor: Vec2,
    pub screen_axis: Vec2,
}

pub fn pick_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<OrbitCamera>>,
    mut scene: ResMut<SdfSceneState>,
    egui_wants: Option<Res<EguiWantsInput>>,
    mut tracker: ResMut<ClickTracker>,
) {
    if let Some(wants) = &egui_wants {
        if wants.wants_pointer_input() { return; }
    }
    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, cam_transform)) = camera.single() else { return };

    if mouse.just_pressed(MouseButton::Left) {
        tracker.press_pos = Some(cursor_pos);
    }
    if mouse.just_released(MouseButton::Left) {
        if let Some(press_pos) = tracker.press_pos.take() {
            if (cursor_pos - press_pos).length() < CLICK_THRESHOLD {
                if let Ok(ray) = camera.viewport_to_world(cam_transform, cursor_pos) {
                    if let Some((shape_id, bone_id)) = pick_shape(ray, &scene.scene) {
                        scene.selected_shape = Some(shape_id);
                        scene.selected_bone = Some(bone_id);
                    } else {
                        scene.selected_shape = None;
                    }
                }
            }
        }
    }
}

// ── Local axes helper ───────────────────────────────────────────

/// Get the local coordinate axes for the selected shape/bone in world space.
/// These are the parent bone's world rotation applied to X/Y/Z unit vectors.
fn get_local_axes(scene: &SdfSceneState) -> [Vec3; 3] {
    let overrides = HashMap::new();
    let world_transforms = compute_bone_world_transforms(&scene.scene.root_bone, Mat4::IDENTITY, &overrides);

    let bone_id = if let Some(shape_id) = scene.selected_shape {
        scene.scene.root_bone.find_shape(shape_id).map(|(_, bid)| bid)
    } else {
        // For bone selection, use the bone's PARENT rotation
        // (the bone's own transform is what we're editing, so the frame is the parent's)
        scene.selected_bone.and_then(|bid| {
            find_parent_bone_id(&scene.scene.root_bone, bid)
        })
    };

    if let Some(bid) = bone_id {
        if let Some(&bone_world) = world_transforms.get(&bid) {
            let (_, rotation, _) = bone_world.to_scale_rotation_translation();
            return [
                rotation * Vec3::X,
                rotation * Vec3::Y,
                rotation * Vec3::Z,
            ];
        }
    }
    [Vec3::X, Vec3::Y, Vec3::Z] // fallback to world axes
}

fn find_parent_bone_id(bone: &litsdf_core::models::SdfBone, target: litsdf_core::models::BoneId) -> Option<litsdf_core::models::BoneId> {
    for child in &bone.children {
        if child.id == target { return Some(bone.id); }
        if let Some(id) = find_parent_bone_id(child, target) { return Some(id); }
    }
    None
}

// ── Drag handle system ──────────────────────────────────────────

const HANDLE_LENGTH: f32 = 0.8;
const HANDLE_PICK_RADIUS: f32 = 0.08;

pub fn draw_handles(
    mut gizmos: Gizmos,
    scene: Res<SdfSceneState>,
    drag: Res<DragState>,
    mode: Res<GizmoMode>,
) {
    let Some(pos) = get_selected_world_pos(&scene) else { return };

    let local = get_local_axes(&scene);
    let axes = [
        (local[0], Color::srgb(1.0, 0.2, 0.2)),
        (local[1], Color::srgb(0.2, 1.0, 0.2)),
        (local[2], Color::srgb(0.2, 0.2, 1.0)),
    ];

    match *mode {
        GizmoMode::Translate => {
            // Arrow lines + sphere tips
            for (axis, color) in &axes {
                let tip = pos + *axis * HANDLE_LENGTH;
                gizmos.line(pos, tip, *color);
                gizmos.sphere(Isometry3d::from_translation(tip), HANDLE_PICK_RADIUS, *color);
            }
        }
        GizmoMode::Rotate => {
            // Circular arcs per axis (approximated with line segments)
            let segments = 32;
            let radius = HANDLE_LENGTH * 0.8;
            for (axis, color) in &axes {
                let (perp1, perp2) = perpendicular_pair(*axis);
                for i in 0..segments {
                    let a0 = (i as f32 / segments as f32) * std::f32::consts::TAU;
                    let a1 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
                    let p0 = pos + (perp1 * a0.cos() + perp2 * a0.sin()) * radius;
                    let p1 = pos + (perp1 * a1.cos() + perp2 * a1.sin()) * radius;
                    gizmos.line(p0, p1, *color);
                }
            }
        }
        GizmoMode::Elongation => {
            // Double-headed thick lines with diamond tips
            for (axis, color) in &axes {
                let tip = pos + *axis * HANDLE_LENGTH * 0.6;
                let neg_tip = pos - *axis * HANDLE_LENGTH * 0.6;
                gizmos.line(neg_tip, tip, *color);
                gizmos.sphere(Isometry3d::from_translation(tip), HANDLE_PICK_RADIUS * 0.7, *color);
                gizmos.sphere(Isometry3d::from_translation(neg_tip), HANDLE_PICK_RADIUS * 0.7, *color);
            }
        }
        GizmoMode::Repetition => {
            // Dotted lines with cube indicators
            for (axis, color) in &axes {
                let tip = pos + *axis * HANDLE_LENGTH;
                gizmos.line(pos, tip, *color);
                // Draw small crosses at tip to indicate grid
                let (p1, p2) = perpendicular_pair(*axis);
                let s = HANDLE_PICK_RADIUS * 0.5;
                gizmos.line(tip - p1 * s, tip + p1 * s, *color);
                gizmos.line(tip - p2 * s, tip + p2 * s, *color);
            }
        }
    }
}

fn perpendicular_pair(axis: Vec3) -> (Vec3, Vec3) {
    let up = if axis.y.abs() > 0.9 { Vec3::X } else { Vec3::Y };
    let p1 = axis.cross(up).normalize();
    let p2 = axis.cross(p1).normalize();
    (p1, p2)
}

pub fn drag_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<OrbitCamera>>,
    mut scene: ResMut<SdfSceneState>,
    mut drag: ResMut<DragState>,
    egui_wants: Option<Res<EguiWantsInput>>,
    mode: Res<GizmoMode>,
) {
    if let Some(wants) = &egui_wants {
        if wants.wants_pointer_input() { return; }
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera_comp, cam_transform)) = camera.single() else { return };

    if mouse.just_pressed(MouseButton::Left) && !drag.active {
        if let Some(world_pos) = get_selected_world_pos(&scene) {
            if let Ok(ray) = camera_comp.viewport_to_world(cam_transform, cursor_pos) {
                let handle_len = match *mode {
                    GizmoMode::Elongation => HANDLE_LENGTH * 0.6,
                    _ => HANDLE_LENGTH,
                };
                let local = get_local_axes(&scene);
                for (i, axis) in local.iter().enumerate() {
                    let tip = world_pos + *axis * handle_len;
                    let to_tip = tip - ray.origin;
                    let proj = to_tip.dot(*ray.direction);
                    if proj > 0.0 {
                        let closest = ray.origin + *ray.direction * proj;
                        if (closest - tip).length() < HANDLE_PICK_RADIUS * 3.0 {
                            let screen_axis = project_axis_to_screen(camera_comp, cam_transform, world_pos, *axis);
                            let start_value = get_mode_value(&scene, &mode);
                            drag.active = true;
                            drag.axis = *axis;
                            drag.axis_index = i;
                            drag.start_world_pos = world_pos;
                            drag.start_value = start_value;
                            drag.start_cursor = cursor_pos;
                            drag.screen_axis = screen_axis;
                            break;
                        }
                    }
                }
            }
        }
    }

    if drag.active && mouse.pressed(MouseButton::Left) {
        let delta_screen = cursor_pos - drag.start_cursor;
        let proj = delta_screen.dot(drag.screen_axis);
        let cam_dist = (cam_transform.translation() - drag.start_world_pos).length();
        let world_delta = proj * cam_dist * 0.002;

        let mut new_val = drag.start_value;
        let axis_idx = drag.axis_index;

        match *mode {
            GizmoMode::Translate => {
                new_val[axis_idx] += world_delta;
                set_selected_translation(&mut scene, new_val);
            }
            GizmoMode::Rotate => {
                // Scale rotation more aggressively (degrees)
                new_val[axis_idx] += world_delta * 50.0;
                set_selected_rotation(&mut scene, new_val);
            }
            GizmoMode::Elongation => {
                new_val[axis_idx] = (new_val[axis_idx] + world_delta).max(0.0);
                set_selected_elongation(&mut scene, new_val);
            }
            GizmoMode::Repetition => {
                new_val[axis_idx] = (new_val[axis_idx] + world_delta).max(0.1);
                set_selected_repetition(&mut scene, new_val);
            }
        }
        scene.dirty = true;
    }

    if mouse.just_released(MouseButton::Left) {
        drag.active = false;
    }
}

// ── Property accessors per mode ─────────────────────────────────

fn get_mode_value(scene: &SdfSceneState, mode: &GizmoMode) -> [f32; 3] {
    // Shape selected — use shape transform/modifiers
    if let Some(shape_id) = scene.selected_shape {
        if let Some((shape, _)) = scene.scene.root_bone.find_shape(shape_id) {
            return match mode {
                GizmoMode::Translate => shape.transform.translation,
                GizmoMode::Rotate => shape.transform.rotation,
                GizmoMode::Elongation => get_modifier_vec3(&shape.modifiers, |m| matches!(m, ShapeModifier::Elongation(_))),
                GizmoMode::Repetition => get_modifier_vec3(&shape.modifiers, |m| matches!(m, ShapeModifier::Repetition { .. })),
            };
        }
    }
    // Bone selected (no shape) — use bone transform (translate/rotate only)
    if let Some(bone_id) = scene.selected_bone {
        if let Some(bone) = scene.scene.root_bone.find_bone(bone_id) {
            return match mode {
                GizmoMode::Translate => bone.transform.translation,
                GizmoMode::Rotate => bone.transform.rotation,
                _ => [0.0; 3], // bones don't have modifiers
            };
        }
    }
    [0.0; 3]
}

fn get_modifier_vec3(modifiers: &[ShapeModifier], pred: impl Fn(&ShapeModifier) -> bool) -> [f32; 3] {
    for m in modifiers {
        if pred(m) {
            return match m {
                ShapeModifier::Elongation(v) => *v,
                ShapeModifier::Repetition { period, .. } => *period,
                _ => [0.0; 3],
            };
        }
    }
    [0.0; 3]
}

pub fn get_selected_world_pos(scene: &SdfSceneState) -> Option<Vec3> {
    let overrides = HashMap::new();
    let world_transforms = compute_bone_world_transforms(&scene.scene.root_bone, Mat4::IDENTITY, &overrides);

    if let Some(shape_id) = scene.selected_shape {
        if let Some((shape, bone_id)) = scene.scene.root_bone.find_shape(shape_id) {
            if let Some(&bone_world) = world_transforms.get(&bone_id) {
                let st = shape.transform.translation;
                return Some(bone_world.transform_point3(Vec3::new(st[0], st[1], st[2])));
            }
        }
    }

    // Bone selected (no shape) — return bone's world position
    if let Some(bone_id) = scene.selected_bone {
        if !bone_id.is_root() {
            if let Some(&bone_world) = world_transforms.get(&bone_id) {
                let (_, _, translation) = bone_world.to_scale_rotation_translation();
                return Some(translation);
            }
        }
    }

    None
}

fn set_selected_translation(scene: &mut SdfSceneState, trans: [f32; 3]) {
    if let Some(shape_id) = scene.selected_shape {
        if let Some((shape, _)) = scene.scene.root_bone.find_shape_mut(shape_id) {
            shape.transform.translation = trans;
            return;
        }
    }
    // Bone selected (no shape)
    if let Some(bone_id) = scene.selected_bone {
        if let Some(bone) = scene.scene.root_bone.find_bone_mut(bone_id) {
            bone.transform.translation = trans;
        }
    }
}

fn set_selected_rotation(scene: &mut SdfSceneState, rot: [f32; 3]) {
    if let Some(shape_id) = scene.selected_shape {
        if let Some((shape, _)) = scene.scene.root_bone.find_shape_mut(shape_id) {
            shape.transform.rotation = rot;
            return;
        }
    }
    // Bone selected (no shape)
    if let Some(bone_id) = scene.selected_bone {
        if let Some(bone) = scene.scene.root_bone.find_bone_mut(bone_id) {
            bone.transform.rotation = rot;
        }
    }
}

fn set_selected_elongation(scene: &mut SdfSceneState, elong: [f32; 3]) {
    if let Some(shape_id) = scene.selected_shape {
        if let Some((shape, _)) = scene.scene.root_bone.find_shape_mut(shape_id) {
            // Update or add Elongation modifier
            let mut found = false;
            for m in &mut shape.modifiers {
                if let ShapeModifier::Elongation(v) = m {
                    *v = elong;
                    found = true;
                    break;
                }
            }
            if !found && (elong[0] > 0.0 || elong[1] > 0.0 || elong[2] > 0.0) {
                shape.modifiers.push(ShapeModifier::Elongation(elong));
            }
        }
    }
}

fn set_selected_repetition(scene: &mut SdfSceneState, period: [f32; 3]) {
    if let Some(shape_id) = scene.selected_shape {
        if let Some((shape, _)) = scene.scene.root_bone.find_shape_mut(shape_id) {
            let mut found = false;
            for m in &mut shape.modifiers {
                if let ShapeModifier::Repetition { period: p, .. } = m {
                    *p = period;
                    found = true;
                    break;
                }
            }
            if !found && (period[0] > 0.0 || period[1] > 0.0 || period[2] > 0.0) {
                shape.modifiers.push(ShapeModifier::Repetition { period, count: [3, 3, 3] });
            }
        }
    }
}

fn project_axis_to_screen(camera: &Camera, transform: &GlobalTransform, origin: Vec3, axis: Vec3) -> Vec2 {
    let p0 = camera.world_to_viewport(transform, origin);
    let p1 = camera.world_to_viewport(transform, origin + axis);
    match (p0, p1) {
        (Ok(a), Ok(b)) => (b - a).normalize_or_zero(),
        _ => Vec2::ZERO,
    }
}
