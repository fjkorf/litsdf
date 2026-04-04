use litsdf_render::scene_sync::SdfSceneState;

use super::app;

fn f32_to_rgba(c: [f32; 3]) -> [u8; 4] {
    [(c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8, 255]
}
use super::helpers::{combo_smooth_k, combo_to_index, prim_params, prim_to_index};
use super::EditorUi;

pub fn populate_bone_shapes(ui: &mut EditorUi, scene: &SdfSceneState) {
    ui.md.state.bone_shapes.clear();
    ui.shape_order.clear();

    ui.md.state.has_bone_selection = scene.selected_bone.is_some();
    ui.md.state.no_selection = scene.selected_bone.is_none();

    let Some(bone_id) = scene.selected_bone else { return };
    let Some(bone) = scene.scene.root_bone.find_bone(bone_id) else { return };

    for shape in &bone.shapes {
        let mut row = app::Bone_shapesRow::default();
        let is_selected = scene.selected_shape == Some(shape.id);
        row.shape_name = if is_selected {
            format!("> {}", shape.name)
        } else {
            shape.name.clone()
        };
        ui.md.state.bone_shapes.push(row);
        ui.shape_order.push(shape.id);
    }
}

pub fn populate_shape_properties(ui: &mut EditorUi, scene: &SdfSceneState) {
    ui.md.state.has_selection = scene.selected_shape.is_some();

    let shape_changed = scene.selected_shape != ui.prev_selected_shape;
    ui.prev_selected_shape = scene.selected_shape;
    if !shape_changed { return; }

    let Some(sel_id) = scene.selected_shape else { return };
    let Some((shape, _)) = scene.scene.root_bone.find_shape(sel_id) else { return };

    ui.md.state.selected_shape_name = shape.name.clone();
    ui.md.state.prim_type = prim_to_index(&shape.primitive);
    let (a, b, c, d) = prim_params(&shape.primitive);
    ui.md.state.param_a = a;
    ui.md.state.param_b = b;
    ui.md.state.param_c = c;
    ui.md.state.param_d = d;
    ui.md.state.tx = shape.transform.translation[0] as f64;
    ui.md.state.ty = shape.transform.translation[1] as f64;
    ui.md.state.tz = shape.transform.translation[2] as f64;
    ui.md.state.rx = shape.transform.rotation[0] as f64;
    ui.md.state.ry = shape.transform.rotation[1] as f64;
    ui.md.state.rz = shape.transform.rotation[2] as f64;
    ui.md.state.uniform_scale = shape.transform.scale as f64;
    ui.md.state.shape_color = [
        (shape.material.color[0] * 255.0) as u8,
        (shape.material.color[1] * 255.0) as u8,
        (shape.material.color[2] * 255.0) as u8,
        255,
    ];
    ui.md.state.roughness = shape.material.roughness as f64;
    ui.md.state.metallic = shape.material.metallic as f64;
    ui.md.state.fresnel_power = shape.material.fresnel_power as f64;
    ui.md.state.color_mode = shape.material.color_mode as usize;
    ui.md.state.is_palette_mode = shape.material.color_mode == 1;
    ui.md.state.palette_a_color = f32_to_rgba(shape.material.palette_a);
    ui.md.state.palette_b_color = f32_to_rgba(shape.material.palette_b);
    ui.md.state.palette_c_color = f32_to_rgba(shape.material.palette_c);
    ui.md.state.palette_d_color = f32_to_rgba(shape.material.palette_d);
    // Noise
    ui.md.state.noise_amp = shape.material.noise_amplitude as f64;
    ui.md.state.noise_freq = shape.material.noise_frequency as f64;
    ui.md.state.noise_oct = shape.material.noise_octaves as f64;
    // Symmetry
    ui.md.state.smooth_sym = shape.material.smooth_symmetry as f64;
    // Modifiers are now edited directly via the egui modifier stack (modifier_stack.rs)
    // Combine
    ui.md.state.combo_op = combo_to_index(&shape.combination);
    ui.md.state.smooth_k = combo_smooth_k(&shape.combination) as f64;
}

pub fn populate_bone_properties(ui: &mut EditorUi, scene: &SdfSceneState) {
    let bone_changed = scene.selected_bone != ui.prev_selected_bone;
    ui.prev_selected_bone = scene.selected_bone;

    if let Some(bone_id) = scene.selected_bone {
        ui.md.state.bone_is_root = bone_id.is_root();
        ui.md.state.bone_editable = !bone_id.is_root();
    }

    if !bone_changed { return; }

    if let Some(bone_id) = scene.selected_bone {
        if let Some(bone) = scene.scene.root_bone.find_bone(bone_id) {
            ui.md.state.bone_name = bone.name.clone();
            ui.md.state.bone_tx = bone.transform.translation[0] as f64;
            ui.md.state.bone_ty = bone.transform.translation[1] as f64;
            ui.md.state.bone_tz = bone.transform.translation[2] as f64;
            ui.md.state.bone_rx = bone.transform.rotation[0] as f64;
            ui.md.state.bone_ry = bone.transform.rotation[1] as f64;
            ui.md.state.bone_rz = bone.transform.rotation[2] as f64;
            ui.md.state.bone_mass = bone.physics.mass as f64;
            ui.md.state.bone_damping = bone.physics.damping as f64;
        }
    }
}

pub fn populate_file_browser(ui: &mut EditorUi) {
    if !ui.md.state.show_file_browser { return; }
    let dir = litsdf_core::persistence::scenes_dir();
    let files = litsdf_core::persistence::list_scenes(&dir);
    ui.md.state.file_rows.clear();
    for name in files {
        let mut row = app::File_rowsRow::default();
        row.name = name;
        ui.md.state.file_rows.push(row);
    }
    while ui.prev_pick_file_counts.len() < ui.md.state.file_rows.len() {
        ui.prev_pick_file_counts.push(0);
    }
    ui.prev_pick_file_counts.truncate(ui.md.state.file_rows.len());
}
