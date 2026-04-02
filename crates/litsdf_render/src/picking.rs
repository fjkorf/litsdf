use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::input::EguiWantsInput;

use crate::camera::OrbitCamera;
use litsdf_core::models::{BoneId, SdfBone, SdfPrimitive, SdfScene, ShapeId};
use crate::scene_sync::SdfSceneState;
use litsdf_core::scene::compute_bone_world_transforms;

const MAX_STEPS: u32 = 64;
const MAX_DIST: f32 = 100.0;
const EPSILON: f32 = 0.005;
const CLICK_THRESHOLD: f32 = 3.0; // pixels — less than this counts as click, not drag

// ── SDF primitives in Rust (ported from sdf_raymarch.wgsl) ──────

fn sd_sphere(p: Vec3, r: f32) -> f32 {
    p.length() - r
}

fn sd_box(p: Vec3, b: Vec3) -> f32 {
    let q = p.abs() - b;
    q.max(Vec3::ZERO).length() + q.x.max(q.y.max(q.z)).min(0.0)
}

fn sd_round_box(p: Vec3, b: Vec3, r: f32) -> f32 {
    let q = p.abs() - b + Vec3::splat(r);
    q.max(Vec3::ZERO).length() + q.x.max(q.y.max(q.z)).min(0.0) - r
}

fn sd_cylinder(p: Vec3, h: f32, r: f32) -> f32 {
    let d = Vec2::new(Vec2::new(p.x, p.z).length() - r, p.y.abs() - h);
    d.x.max(d.y).min(0.0) + d.max(Vec2::ZERO).length()
}

fn sd_capped_cone(p: Vec3, h: f32, r1: f32, r2: f32) -> f32 {
    let q = Vec2::new(Vec2::new(p.x, p.z).length(), p.y);
    let k1 = Vec2::new(r2, h);
    let k2 = Vec2::new(r2 - r1, 2.0 * h);
    let ca = Vec2::new(
        q.x - q.x.min(if q.y < 0.0 { r1 } else { r2 }),
        q.y.abs() - h,
    );
    let cb = q - k1 + k2 * ((k1 - q).dot(k2) / k2.dot(k2)).clamp(0.0, 1.0);
    let s = if cb.x < 0.0 && ca.y < 0.0 { -1.0 } else { 1.0 };
    s * ca.dot(ca).min(cb.dot(cb)).sqrt()
}

fn sd_torus(p: Vec3, major: f32, minor: f32) -> f32 {
    let q = Vec2::new(Vec2::new(p.x, p.z).length() - major, p.y);
    q.length() - minor
}

fn sd_capsule(p: Vec3, r: f32, h: f32) -> f32 {
    let mut q = p;
    q.y -= q.y.clamp(-h, h);
    q.length() - r
}

fn sd_plane(p: Vec3, n: Vec3, d: f32) -> f32 {
    p.dot(n.normalize()) + d
}

fn sd_ellipsoid(p: Vec3, r: Vec3) -> f32 {
    let k0 = (p / r).length();
    let k1 = (p / (r * r)).length();
    if k1 == 0.0 { return 0.0; }
    k0 * (k0 - 1.0) / k1
}

// ── Rotation (matching shader) ──────────────────────────────────

fn rotate_point(p: Vec3, euler: Vec3) -> Vec3 {
    let (cx, sx) = (euler.x.cos(), euler.x.sin());
    let (cy, sy) = (euler.y.cos(), euler.y.sin());
    let (cz, sz) = (euler.z.cos(), euler.z.sin());
    let mut q = p;
    q = Vec3::new(q.x * cz - q.y * sz, q.x * sz + q.y * cz, q.z);
    q = Vec3::new(q.x, q.y * cx - q.z * sx, q.y * sx + q.z * cx);
    q = Vec3::new(q.x * cy + q.z * sy, q.y, -q.x * sy + q.z * cy);
    q
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
    let q = rotate_point((p - s.translation) / s.scale, -s.rotation);
    let d = litsdf_core::sdf::eval_primitive(q, &s.primitive);
    d * s.scale
}

// ── Flatten scene to world shapes ───────────────────────────────

fn collect_world_shapes(bone: &SdfBone, world_transforms: &HashMap<BoneId, Mat4>, out: &mut Vec<WorldShape>) {
    let bone_world = world_transforms[&bone.id];
    for shape in &bone.shapes {
        let st = shape.transform.translation;
        let sr = shape.transform.rotation;
        let shape_local = Mat4::from_scale_rotation_translation(
            Vec3::splat(shape.transform.scale),
            Quat::from_euler(EulerRot::XYZ, sr[0].to_radians(), sr[1].to_radians(), sr[2].to_radians()),
            Vec3::new(st[0], st[1], st[2]),
        );
        let world_mat = bone_world * shape_local;
        let (scale, rotation, translation) = world_mat.to_scale_rotation_translation();
        let euler = rotation.to_euler(EulerRot::XYZ);

        out.push(WorldShape {
            shape_id: shape.id,
            bone_id: bone.id,
            translation,
            rotation: Vec3::new(euler.0, euler.1, euler.2),
            scale: scale.x,
            primitive: shape.primitive.clone(),
        });
    }
    for child in &bone.children {
        collect_world_shapes(child, world_transforms, out);
    }
}

// ── Ray march and pick ──────────────────────────────────────────

/// Combined scene SDF for ray marching (uses union of all shapes).
fn sdf_scene(p: Vec3, shapes: &[WorldShape]) -> f32 {
    let mut d = MAX_DIST;
    for s in shapes {
        d = d.min(eval_world_shape(p, s));
    }
    d
}

/// Ray march to find a hit point, then determine which shape is closest.
pub fn pick_shape(ray: Ray3d, scene: &SdfScene) -> Option<(ShapeId, BoneId)> {
    let overrides = std::collections::HashMap::new();
    let world_transforms = compute_bone_world_transforms(&scene.root_bone, Mat4::IDENTITY, &overrides);
    let mut shapes = Vec::new();
    collect_world_shapes(&scene.root_bone, &world_transforms, &mut shapes);

    if shapes.is_empty() {
        return None;
    }

    // Ray march using union of all shapes
    let origin = ray.origin;
    let dir = *ray.direction;
    let mut t = 0.0f32;
    for _ in 0..MAX_STEPS {
        let p = origin + dir * t;
        let d = sdf_scene(p, &shapes);
        if d < EPSILON {
            // Hit — find which individual shape is closest
            let mut best_id = None;
            let mut best_dist = f32::MAX;
            for s in &shapes {
                let sd = eval_world_shape(p, s).abs();
                if sd < best_dist {
                    best_dist = sd;
                    best_id = Some((s.shape_id, s.bone_id));
                }
            }
            return best_id;
        }
        t += d;
        if t > MAX_DIST {
            break;
        }
    }

    None
}

// ── Bevy system ─────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct ClickTracker {
    press_pos: Option<Vec2>,
}

#[derive(Resource, Default)]
pub struct DragState {
    pub active: bool,
    pub axis: Vec3,
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
    // Skip if egui wants input
    if let Some(wants) = &egui_wants {
        if wants.wants_pointer_input() {
            return;
        }
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, cam_transform)) = camera.single() else { return };

    // Track press position for click vs drag detection
    if mouse.just_pressed(MouseButton::Left) {
        tracker.press_pos = Some(cursor_pos);
    }

    if mouse.just_released(MouseButton::Left) {
        if let Some(press_pos) = tracker.press_pos.take() {
            let drag_dist = (cursor_pos - press_pos).length();
            if drag_dist < CLICK_THRESHOLD {
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

// ── Drag handle system ──────────────────────────────────────────

const HANDLE_LENGTH: f32 = 0.8;
const HANDLE_PICK_RADIUS: f32 = 0.08;

/// Draw translation handles at selected shape/bone position.
pub fn draw_handles(
    mut gizmos: Gizmos,
    scene: Res<SdfSceneState>,
    drag: Res<DragState>,
) {
    let Some(pos) = get_selected_world_pos(&scene) else { return };

    let axes = [
        (Vec3::X, Color::srgb(1.0, 0.2, 0.2)),
        (Vec3::Y, Color::srgb(0.2, 1.0, 0.2)),
        (Vec3::Z, Color::srgb(0.2, 0.2, 1.0)),
    ];

    for (axis, color) in &axes {
        let width = if drag.active && drag.axis == *axis { 3.0 } else { 1.5 };
        let tip = pos + *axis * HANDLE_LENGTH;
        gizmos.line(pos, tip, *color);
        gizmos.sphere(Isometry3d::from_translation(tip), HANDLE_PICK_RADIUS, *color);
    }
}

/// Handle drag interaction.
pub fn drag_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<OrbitCamera>>,
    mut scene: ResMut<SdfSceneState>,
    mut drag: ResMut<DragState>,
    egui_wants: Option<Res<EguiWantsInput>>,
) {
    if let Some(wants) = &egui_wants {
        if wants.wants_pointer_input() { return; }
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera_comp, cam_transform)) = camera.single() else { return };

    if mouse.just_pressed(MouseButton::Left) && !drag.active {
        // Test if cursor is near a handle
        if let Some(world_pos) = get_selected_world_pos(&scene) {
            if let Ok(ray) = camera_comp.viewport_to_world(cam_transform, cursor_pos) {
                let axes = [Vec3::X, Vec3::Y, Vec3::Z];
                for axis in &axes {
                    let tip = world_pos + *axis * HANDLE_LENGTH;
                    // Simple sphere test on handle tip
                    let to_tip = tip - ray.origin;
                    let proj = to_tip.dot(*ray.direction);
                    if proj > 0.0 {
                        let closest = ray.origin + *ray.direction * proj;
                        let dist = (closest - tip).length();
                        if dist < HANDLE_PICK_RADIUS * 3.0 {
                            // Project axis to screen space for drag direction
                            let screen_axis = project_axis_to_screen(
                                camera_comp, cam_transform, world_pos, *axis,
                            );
                            let trans = get_selected_translation(&scene);
                            drag.active = true;
                            drag.axis = *axis;
                            drag.start_world_pos = world_pos;
                            drag.start_value = trans;
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
        // Scale by distance for consistent feel
        let cam_dist = (cam_transform.translation() - drag.start_world_pos).length();
        let world_delta = proj * cam_dist * 0.002;

        let mut new_trans = drag.start_value;
        if drag.axis == Vec3::X { new_trans[0] += world_delta; }
        if drag.axis == Vec3::Y { new_trans[1] += world_delta; }
        if drag.axis == Vec3::Z { new_trans[2] += world_delta; }

        set_selected_translation(&mut scene, new_trans);
        scene.dirty = true;
    }

    if mouse.just_released(MouseButton::Left) {
        drag.active = false;
    }
}

pub fn get_selected_world_pos(scene: &SdfSceneState) -> Option<Vec3> {
    if let Some(shape_id) = scene.selected_shape {
        if let Some((shape, bone_id)) = scene.scene.root_bone.find_shape(shape_id) {
            let overrides = std::collections::HashMap::new();
            let world_transforms = compute_bone_world_transforms(&scene.scene.root_bone, Mat4::IDENTITY, &overrides);
            if let Some(&bone_world) = world_transforms.get(&bone_id) {
                let st = shape.transform.translation;
                let pos = bone_world.transform_point3(Vec3::new(st[0], st[1], st[2]));
                return Some(pos);
            }
        }
    }
    None
}

fn get_selected_translation(scene: &SdfSceneState) -> [f32; 3] {
    if let Some(shape_id) = scene.selected_shape {
        if let Some((shape, _)) = scene.scene.root_bone.find_shape(shape_id) {
            return shape.transform.translation;
        }
    }
    [0.0, 0.0, 0.0]
}

fn set_selected_translation(scene: &mut SdfSceneState, trans: [f32; 3]) {
    if let Some(shape_id) = scene.selected_shape {
        if let Some((shape, _)) = scene.scene.root_bone.find_shape_mut(shape_id) {
            shape.transform.translation = trans;
        }
    }
}

fn project_axis_to_screen(
    camera: &Camera,
    transform: &GlobalTransform,
    origin: Vec3,
    axis: Vec3,
) -> Vec2 {
    let p0 = camera.world_to_viewport(transform, origin);
    let p1 = camera.world_to_viewport(transform, origin + axis);
    match (p0, p1) {
        (Ok(a), Ok(b)) => (b - a).normalize_or_zero(),
        _ => Vec2::ZERO,
    }
}
