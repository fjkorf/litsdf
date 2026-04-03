use litsdf_render::scene_sync::SdfSceneState;

fn rgba_to_f32(c: [u8; 4]) -> [f32; 3] {
    [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0]
}

use super::helpers::{PRIM_NAMES, index_to_combo, prim_params, prim_to_index, set_prim_params};
use super::EditorUi;

pub fn sync_shape_properties(ui: &mut EditorUi, scene: &mut SdfSceneState) {
    let Some(sel_id) = scene.selected_shape else { return };
    let Some((shape, _)) = scene.scene.root_bone.find_shape_mut(sel_id) else { return };

    if ui.md.state.prim_type != prim_to_index(&shape.primitive) {
        let type_name = PRIM_NAMES[ui.md.state.prim_type];
        shape.primitive = litsdf_core::models::SdfPrimitive::default_for(type_name);
        shape.name = type_name.to_string();
        let (a, b, c, d) = prim_params(&shape.primitive);
        ui.md.state.param_a = a;
        ui.md.state.param_b = b;
        ui.md.state.param_c = c;
        ui.md.state.param_d = d;
        scene.dirty = true;
        return;
    }

    let mut changed = false;
    let nt = [ui.md.state.tx as f32, ui.md.state.ty as f32, ui.md.state.tz as f32];
    let nr = [ui.md.state.rx as f32, ui.md.state.ry as f32, ui.md.state.rz as f32];
    let ns = ui.md.state.uniform_scale as f32;
    if shape.transform.translation != nt || shape.transform.rotation != nr || shape.transform.scale != ns {
        shape.transform.translation = nt;
        shape.transform.rotation = nr;
        shape.transform.scale = ns;
        changed = true;
    }

    let (oa, ob, oc, od) = prim_params(&shape.primitive);
    if oa != ui.md.state.param_a || ob != ui.md.state.param_b || oc != ui.md.state.param_c || od != ui.md.state.param_d {
        set_prim_params(
            &mut shape.primitive,
            ui.md.state.param_a as f32,
            ui.md.state.param_b as f32,
            ui.md.state.param_c as f32,
            ui.md.state.param_d as f32,
        );
        changed = true;
    }

    let nc = [
        ui.md.state.shape_color[0] as f32 / 255.0,
        ui.md.state.shape_color[1] as f32 / 255.0,
        ui.md.state.shape_color[2] as f32 / 255.0,
    ];
    let new_fresnel = ui.md.state.fresnel_power as f32;
    let new_color_mode = ui.md.state.color_mode as u32;
    let new_pa = rgba_to_f32(ui.md.state.palette_a_color);
    let new_pb = rgba_to_f32(ui.md.state.palette_b_color);
    let new_pc = rgba_to_f32(ui.md.state.palette_c_color);
    let new_pd = rgba_to_f32(ui.md.state.palette_d_color);

    if shape.material.color != nc
        || shape.material.roughness != ui.md.state.roughness as f32
        || shape.material.metallic != ui.md.state.metallic as f32
        || shape.material.fresnel_power != new_fresnel
        || shape.material.color_mode != new_color_mode
        || shape.material.palette_a != new_pa
        || shape.material.palette_b != new_pb
        || shape.material.palette_c != new_pc
        || shape.material.palette_d != new_pd
    {
        shape.material.color = nc;
        shape.material.roughness = ui.md.state.roughness as f32;
        shape.material.metallic = ui.md.state.metallic as f32;
        shape.material.fresnel_power = new_fresnel;
        shape.material.color_mode = new_color_mode;
        shape.material.palette_a = new_pa;
        shape.material.palette_b = new_pb;
        shape.material.palette_c = new_pc;
        shape.material.palette_d = new_pd;
        changed = true;
    }

    // Noise
    let new_noise_amp = ui.md.state.noise_amp as f32;
    let new_noise_freq = ui.md.state.noise_freq as f32;
    let new_noise_oct = ui.md.state.noise_oct as u32;
    if shape.material.noise_amplitude != new_noise_amp
        || shape.material.noise_frequency != new_noise_freq
        || shape.material.noise_octaves != new_noise_oct
    {
        shape.material.noise_amplitude = new_noise_amp;
        shape.material.noise_frequency = new_noise_freq;
        shape.material.noise_octaves = new_noise_oct;
        changed = true;
    }

    // Smooth symmetry
    let new_sym = ui.md.state.smooth_sym as f32;
    if shape.material.smooth_symmetry != new_sym {
        shape.material.smooth_symmetry = new_sym;
        changed = true;
    }

    // Modifiers — rebuild from slider values
    let new_mods = build_modifiers(
        ui.md.state.mod_rounding as f32,
        ui.md.state.mod_onion as f32,
        ui.md.state.mod_twist as f32,
        ui.md.state.mod_bend as f32,
        [ui.md.state.mod_elong_x as f32, ui.md.state.mod_elong_y as f32, ui.md.state.mod_elong_z as f32],
        [ui.md.state.mod_rep_x as f32, ui.md.state.mod_rep_y as f32, ui.md.state.mod_rep_z as f32],
    );
    if shape.modifiers != new_mods {
        shape.modifiers = new_mods;
        changed = true;
    }

    let combo = index_to_combo(ui.md.state.combo_op, ui.md.state.smooth_k as f32);
    if shape.combination != combo {
        shape.combination = combo;
        changed = true;
    }

    if changed {
        scene.dirty = true;
    }
}

fn build_modifiers(rounding: f32, onion: f32, twist: f32, bend: f32, elongation: [f32; 3], rep_period: [f32; 3]) -> Vec<litsdf_core::models::ShapeModifier> {
    use litsdf_core::models::ShapeModifier;
    let mut mods = Vec::new();
    if rounding > 0.0 { mods.push(ShapeModifier::Rounding(rounding)); }
    if onion > 0.0 { mods.push(ShapeModifier::Onion(onion)); }
    if twist.abs() > 0.001 { mods.push(ShapeModifier::Twist(twist)); }
    if bend.abs() > 0.001 { mods.push(ShapeModifier::Bend(bend)); }
    if elongation != [0.0, 0.0, 0.0] { mods.push(ShapeModifier::Elongation(elongation)); }
    if rep_period != [0.0, 0.0, 0.0] { mods.push(ShapeModifier::Repetition { period: rep_period, count: [3, 3, 3] }); }
    mods
}

pub fn sync_bone_properties(ui: &mut EditorUi, scene: &mut SdfSceneState) {
    let Some(bone_id) = scene.selected_bone else { return };
    if bone_id.is_root() { return; }
    let Some(bone) = scene.scene.root_bone.find_bone_mut(bone_id) else { return };

    let nt = [ui.md.state.bone_tx as f32, ui.md.state.bone_ty as f32, ui.md.state.bone_tz as f32];
    let nr = [ui.md.state.bone_rx as f32, ui.md.state.bone_ry as f32, ui.md.state.bone_rz as f32];
    if bone.transform.translation != nt || bone.transform.rotation != nr {
        bone.transform.translation = nt;
        bone.transform.rotation = nr;
        scene.dirty = true;
    }
    let nn = ui.md.state.bone_name.clone();
    if bone.name != nn && !nn.is_empty() {
        bone.name = nn;
    }

}
