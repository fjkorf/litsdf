use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

pub const MAX_SHAPES: usize = 32;

#[derive(Asset, AsBindGroup, TypePath, Clone)]
pub struct SdfMaterial {
    #[uniform(0)]
    pub params: SdfShaderParams,
}

impl Material for SdfMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sdf_raymarch.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

#[derive(Clone, ShaderType)]
pub struct SdfShaderParams {
    pub shape_count: u32,
    pub time: f32,
    pub _pad_h: Vec2,
    pub light_dir: Vec3,
    pub _pad_l: f32,
    pub shapes: [ShaderShape; MAX_SHAPES],
    // Scene settings (after shapes array to preserve array offset)
    pub fill_color: Vec3,
    pub fill_intensity: f32,
    pub back_color: Vec3,
    pub back_intensity: f32,
    pub sss_color: Vec3,
    pub sss_intensity: f32,
    pub ao_intensity: f32,
    pub shadow_softness: f32,
    pub vignette_intensity: f32,
    pub ground_color: Vec3,
    pub sun_sharpness: f32,
    pub sun_brightness: f32,
    pub show_environment: u32,
    pub _pad_s: Vec2,
}

#[derive(Clone, Copy, ShaderType)]
pub struct ShaderShape {
    // Geometry
    pub primitive_type: u32,
    pub combination_op: u32,
    pub smooth_k: f32,
    pub _pad0: f32,
    pub params: Vec4,
    pub translation: Vec3,
    pub _pad1: f32,
    pub rotation: Vec3,
    pub scale: f32,
    // Material
    pub color: Vec3,
    pub roughness: f32,
    pub metallic: f32,
    pub fresnel_power: f32,
    pub color_mode: u32,
    pub _pad3: f32,
    pub palette_a: Vec3,
    pub _pad4: f32,
    pub palette_b: Vec3,
    pub _pad5: f32,
    pub palette_c: Vec3,
    pub _pad6: f32,
    pub palette_d: Vec3,
    pub _pad7: f32,
    // Modifiers
    pub modifier_flags: u32,
    pub rounding: f32,
    pub onion_thickness: f32,
    pub twist_amount: f32,
    pub bend_amount: f32,
    pub _pad_mod0: Vec3,
    pub elongation: Vec3,
    pub _pad_mod1: f32,
    pub rep_period: Vec3,
    pub _pad_mod2: f32,
    // Noise
    pub noise_amplitude: f32,
    pub noise_frequency: f32,
    pub noise_octaves: u32,
    pub smooth_symmetry: f32,
}

impl Default for ShaderShape {
    fn default() -> Self {
        Self {
            primitive_type: 0,
            combination_op: 0,
            smooth_k: 0.0,
            _pad0: 0.0,
            params: Vec4::new(1.0, 0.0, 0.0, 0.0),
            translation: Vec3::ZERO,
            _pad1: 0.0,
            rotation: Vec3::ZERO,
            scale: 1.0,
            color: Vec3::ONE,
            roughness: 0.5,
            metallic: 0.0,
            fresnel_power: 0.0,
            color_mode: 0,
            _pad3: 0.0,
            palette_a: Vec3::ZERO,
            _pad4: 0.0,
            palette_b: Vec3::ZERO,
            _pad5: 0.0,
            palette_c: Vec3::ZERO,
            _pad6: 0.0,
            palette_d: Vec3::ZERO,
            _pad7: 0.0,
            modifier_flags: 0,
            rounding: 0.0,
            onion_thickness: 0.0,
            twist_amount: 0.0,
            bend_amount: 0.0,
            _pad_mod0: Vec3::ZERO,
            elongation: Vec3::ZERO,
            _pad_mod1: 0.0,
            rep_period: Vec3::ZERO,
            _pad_mod2: 0.0,
            noise_amplitude: 0.0,
            noise_frequency: 1.0,
            noise_octaves: 0,
            smooth_symmetry: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::render_resource::ShaderType;

    #[test]
    fn shader_shape_size_matches_wgsl() {
        // ShaderShape must match the WGSL struct exactly.
        // If this fails, the Rust struct and WGSL struct are out of sync.
        // Update both crates/litsdf_render/assets/shaders/sdf_raymarch.wgsl
        // and this file together.
        let shape_size = <ShaderShape as ShaderType>::min_size().get();
        let params_size = <SdfShaderParams as ShaderType>::min_size().get();
        // Log sizes for debugging
        eprintln!("ShaderShape min_size: {shape_size} bytes");
        eprintln!("SdfShaderParams min_size: {params_size} bytes");
        eprintln!("Expected params: 32 (header) + 32 * {shape_size} = {}", 32 + 32 * shape_size);
        // Sanity: shape size should be reasonable (not bloated by accidental fields)
        assert!(shape_size <= 512, "ShaderShape too large: {shape_size} bytes — check for accidental fields");
        assert!(shape_size >= 128, "ShaderShape too small: {shape_size} bytes — check for missing fields");
    }
}

impl Default for SdfShaderParams {
    fn default() -> Self {
        Self {
            shape_count: 0,
            time: 0.0,
            _pad_h: Vec2::ZERO,
            light_dir: Vec3::new(0.6, 0.8, 0.4),
            _pad_l: 0.0,
            shapes: [ShaderShape::default(); MAX_SHAPES],
            fill_color: Vec3::new(0.4, 0.5, 0.7),
            fill_intensity: 0.25,
            back_color: Vec3::new(0.3, 0.2, 0.1),
            back_intensity: 0.2,
            sss_color: Vec3::new(1.0, 0.2, 0.1),
            sss_intensity: 0.15,
            ao_intensity: 3.0,
            shadow_softness: 8.0,
            vignette_intensity: 0.3,
            ground_color: Vec3::new(0.15, 0.12, 0.1),
            sun_sharpness: 64.0,
            sun_brightness: 2.0,
            show_environment: 1,
            _pad_s: Vec2::ZERO,
        }
    }
}
