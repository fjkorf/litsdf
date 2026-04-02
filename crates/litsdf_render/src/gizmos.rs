use std::collections::HashMap;
use bevy::prelude::*;

use crate::camera::OrbitCamera;
use litsdf_core::models::SdfBone;
use crate::scene_sync::SdfSceneState;
use litsdf_core::scene::compute_bone_world_transforms;

pub fn draw_bone_gizmos(
    mut gizmos: Gizmos,
    state: Res<SdfSceneState>,
) {
    if !state.show_bone_gizmos {
        return;
    }

    let overrides = HashMap::new();
    let world_transforms = compute_bone_world_transforms(&state.scene.root_bone, Mat4::IDENTITY, &overrides);
    draw_bone_recursive(&mut gizmos, &state.scene.root_bone, &world_transforms, None);
}

fn draw_bone_recursive(
    gizmos: &mut Gizmos,
    bone: &SdfBone,
    world_transforms: &std::collections::HashMap<litsdf_core::models::BoneId, Mat4>,
    parent_pos: Option<Vec3>,
) {
    let world = world_transforms[&bone.id];
    let pos = world.transform_point3(Vec3::ZERO);

    // Line from parent to this bone
    if let Some(parent) = parent_pos {
        gizmos.line(parent, pos, Color::srgb(0.9, 0.9, 0.2));
    }

    // Small axes at bone position
    let size = 0.15;
    let right = world.transform_vector3(Vec3::X * size);
    let up = world.transform_vector3(Vec3::Y * size);
    let forward = world.transform_vector3(Vec3::Z * size);
    gizmos.line(pos, pos + right, Color::srgb(1.0, 0.2, 0.2));
    gizmos.line(pos, pos + up, Color::srgb(0.2, 1.0, 0.2));
    gizmos.line(pos, pos + forward, Color::srgb(0.2, 0.2, 1.0));

    for child in &bone.children {
        draw_bone_recursive(gizmos, child, world_transforms, Some(pos));
    }
}

// ── Compass gizmo (egui overlay) ────────────────────────────────

pub fn draw_compass(
    mut contexts: bevy_egui::EguiContexts,
    camera: Query<&Transform, With<OrbitCamera>>,
) {
    let Ok(ctx) = contexts.ctx_mut().map(|c| c.clone()) else { return };
    let Ok(cam) = camera.single() else { return };

    let view_rot = cam.rotation.inverse();
    let x = project_axis(view_rot, Vec3::X);
    let y = project_axis(view_rot, Vec3::Y);
    let z = project_axis(view_rot, Vec3::Z);

    let compass_pos = egui::pos2(250.0, 680.0);
    let len = 28.0;

    egui::Area::new(egui::Id::new("compass"))
        .fixed_pos(compass_pos)
        .interactable(false)
        .show(&ctx, |ui| {
            let (_, rect) = ui.allocate_space(egui::vec2(80.0, 80.0));
            let center = rect.center();
            let painter = ui.painter();

            // Draw axes
            let axes = [
                (x, egui::Color32::from_rgb(220, 60, 60), "X"),
                (y, egui::Color32::from_rgb(60, 200, 60), "Y"),
                (z, egui::Color32::from_rgb(60, 100, 220), "Z"),
            ];

            // Sort by depth (draw furthest first)
            let mut sorted: Vec<_> = axes.iter().enumerate().collect();
            let depths = [
                (view_rot * Vec3::X).z,
                (view_rot * Vec3::Y).z,
                (view_rot * Vec3::Z).z,
            ];
            sorted.sort_by(|a, b| depths[a.0].partial_cmp(&depths[b.0]).unwrap());

            for (i, (dir, color, label)) in sorted {
                let tip = center + *dir * len;
                painter.arrow(center, *dir * len, egui::Stroke::new(2.0, *color));
                painter.text(
                    tip + *dir * 6.0,
                    egui::Align2::CENTER_CENTER,
                    *label,
                    egui::FontId::proportional(11.0),
                    *color,
                );
            }

            // Center dot
            painter.circle_filled(center, 2.0, egui::Color32::WHITE);
        });
}

fn project_axis(view_rot: Quat, axis: Vec3) -> egui::Vec2 {
    let rotated = view_rot * axis;
    egui::vec2(rotated.x, -rotated.y)
}
