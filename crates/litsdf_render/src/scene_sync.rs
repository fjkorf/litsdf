use std::collections::HashMap;
use bevy::prelude::*;

use litsdf_core::models::{BoneId, ShapeId, SdfScene};
use litsdf_core::scene;
use crate::shader::{SdfMaterial, SdfShaderParams, ShaderShape, MAX_SHAPES};

#[derive(Resource)]
pub struct SdfSceneState {
    pub scene: SdfScene,
    pub selected_shape: Option<ShapeId>,
    pub selected_bone: Option<BoneId>,
    pub show_bone_gizmos: bool,
    pub dirty: bool,
    pub topology_hash: u64,
}

impl Default for SdfSceneState {
    fn default() -> Self {
        Self {
            scene: SdfScene::default_scene(),
            selected_shape: None,
            selected_bone: None,
            show_bone_gizmos: false,
            dirty: true,
            topology_hash: 0,
        }
    }
}

#[derive(Component)]
pub struct SdfBoundingEntity;

pub fn setup_initial_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SdfMaterial>>,
    state: Res<SdfSceneState>,
) {
    let params = build_shader_params(&state.scene);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(40.0, 40.0, 40.0))),
        MeshMaterial3d(materials.add(SdfMaterial { params })),
        SdfBoundingEntity,
    ));
}

pub fn sync_scene_to_shader(
    mut state: ResMut<SdfSceneState>,
    mut materials: ResMut<Assets<SdfMaterial>>,
    query: Query<&MeshMaterial3d<SdfMaterial>, With<SdfBoundingEntity>>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    if state.dirty {
        // Codegen disabled — using fallback loop-based shader
        // TODO: re-enable when codegen WGSL is validated via naga
        state.dirty = false;
        let mut params = build_shader_params(&state.scene);
        params.time = t;
        for mat_handle in query.iter() {
            if let Some(material) = materials.get_mut(mat_handle) {
                material.params = params.clone();
            }
        }
    } else {
        for mat_handle in query.iter() {
            if let Some(material) = materials.get_mut(mat_handle) {
                material.params.time = t;
            }
        }
    }
}

pub fn build_shader_params(scene_data: &SdfScene) -> SdfShaderParams {
    let overrides = HashMap::new();
    let world_transforms = scene::compute_bone_world_transforms(&scene_data.root_bone, Mat4::IDENTITY, &overrides);
    let mut flat = Vec::new();
    scene::flatten_bone_tree(&scene_data.root_bone, &world_transforms, &mut flat);

    let mut params = SdfShaderParams::default();
    let count = flat.len().min(MAX_SHAPES);
    params.shape_count = count as u32;
    params.light_dir = Vec3::from_array(scene_data.light_dir);
    let s = &scene_data.settings;
    params.fill_color = Vec3::from_array(s.fill_color);
    params.fill_intensity = s.fill_intensity;
    params.back_color = Vec3::from_array(s.back_color);
    params.back_intensity = s.back_intensity;
    params.sss_color = Vec3::from_array(s.sss_color);
    params.sss_intensity = s.sss_intensity;
    params.ao_intensity = s.ao_intensity;
    params.shadow_softness = s.shadow_softness;
    params.vignette_intensity = s.vignette_intensity;

    for (i, fs) in flat.iter().take(count).enumerate() {
        params.shapes[i] = ShaderShape {
            primitive_type: fs.primitive_type,
            combination_op: fs.combination_op,
            smooth_k: fs.smooth_k,
            _pad0: 0.0,
            params: fs.params,
            translation: fs.translation,
            _pad1: 0.0,
            rotation: fs.rotation,
            scale: fs.scale,
            color: fs.color,
            roughness: fs.roughness,
            metallic: fs.metallic,
            fresnel_power: fs.fresnel_power,
            color_mode: fs.color_mode,
            _pad3: 0.0,
            palette_a: fs.palette_a, _pad4: 0.0,
            palette_b: fs.palette_b, _pad5: 0.0,
            palette_c: fs.palette_c, _pad6: 0.0,
            palette_d: fs.palette_d, _pad7: 0.0,
            modifier_flags: fs.modifier_flags,
            rounding: fs.rounding,
            onion_thickness: fs.onion_thickness,
            twist_amount: fs.twist_amount,
            bend_amount: fs.bend_amount,
            _pad_mod0: Vec3::ZERO,
            elongation: fs.elongation, _pad_mod1: 0.0,
            rep_period: fs.rep_period, _pad_mod2: 0.0,
            noise_amplitude: fs.noise_amplitude,
            noise_frequency: fs.noise_frequency,
            noise_octaves: fs.noise_octaves,
            smooth_symmetry: fs.smooth_symmetry,
        };
    }

    params
}
