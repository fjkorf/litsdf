//! Shader code generation from SDF scene tree.
//!
//! Generates WGSL source by unrolling the scene's shape evaluation loop
//! into static code. The generated shader indexes the existing uniform
//! array — zero changes to CPU sync code.

use std::fmt::Write;
use bevy::prelude::*;
use litsdf_core::models::*;
use litsdf_core::scene;

/// Generate a complete WGSL shader for the given scene.
///
/// Returns the full shader source with the scene-specific `sdf_scene()`
/// and `sdf_scene_material()` functions inlined.
pub fn generate_shader(scene_data: &SdfScene) -> Result<String, String> {
    let overrides = std::collections::HashMap::new();
    let world_transforms = scene::compute_bone_world_transforms(
        &scene_data.root_bone,
        Mat4::IDENTITY,
        &overrides,
    );
    let mut flat = Vec::new();
    scene::flatten_bone_tree(&scene_data.root_bone, &world_transforms, &mut flat);

    let count = flat.len();
    if count == 0 {
        return Err("empty scene — no shapes to render".into());
    }

    let mut out = String::with_capacity(16000);
    out.push_str(PREAMBLE);
    out.push('\n');

    // Generate sdf_scene (distance only)
    write_sdf_scene(&mut out, count)?;

    // Generate sdf_scene_material (distance + material blending)
    write_sdf_scene_material(&mut out, count)?;

    out.push_str(POSTAMBLE);
    Ok(out)
}

fn write_sdf_scene(out: &mut String, count: usize) -> Result<(), String> {
    writeln!(out, "// --- Generated: distance-only scene evaluation ---").map_err(|e| e.to_string())?;
    writeln!(out, "fn sdf_scene(p: vec3<f32>) -> f32 {{").map_err(|e| e.to_string())?;
    writeln!(out, "    var d = eval_shape(p, params.shapes[0]);").map_err(|e| e.to_string())?;
    for i in 1..count {
        writeln!(out, "    {{").map_err(|e| e.to_string())?;
        writeln!(out, "        let s = params.shapes[{i}u];").map_err(|e| e.to_string())?;
        writeln!(out, "        let d_s = eval_shape(p, s);").map_err(|e| e.to_string())?;
        writeln!(out, "        d = combine_blend(d, d_s, s.combination_op, s.smooth_k).x;").map_err(|e| e.to_string())?;
        writeln!(out, "    }}").map_err(|e| e.to_string())?;
    }
    writeln!(out, "    return d;").map_err(|e| e.to_string())?;
    writeln!(out, "}}").map_err(|e| e.to_string())?;
    Ok(())
}

fn write_sdf_scene_material(out: &mut String, count: usize) -> Result<(), String> {
    writeln!(out, "").map_err(|e| e.to_string())?;
    writeln!(out, "// --- Generated: material scene evaluation ---").map_err(|e| e.to_string())?;
    writeln!(out, "fn sdf_scene_material(p: vec3<f32>) -> MatResult {{").map_err(|e| e.to_string())?;
    writeln!(out, "    let s0 = params.shapes[0];").map_err(|e| e.to_string())?;
    writeln!(out, "    var result = MatResult(").map_err(|e| e.to_string())?;
    writeln!(out, "        get_shape_color(s0, p),").map_err(|e| e.to_string())?;
    writeln!(out, "        s0.roughness,").map_err(|e| e.to_string())?;
    writeln!(out, "        s0.metallic,").map_err(|e| e.to_string())?;
    writeln!(out, "        s0.fresnel_power,").map_err(|e| e.to_string())?;
    writeln!(out, "        s0.color_mode,").map_err(|e| e.to_string())?;
    writeln!(out, "    );").map_err(|e| e.to_string())?;
    writeln!(out, "    var d = eval_shape(p, s0);").map_err(|e| e.to_string())?;

    for i in 1..count {
        writeln!(out, "    {{").map_err(|e| e.to_string())?;
        writeln!(out, "        let s = params.shapes[{i}u];").map_err(|e| e.to_string())?;
        writeln!(out, "        let d_s = eval_shape(p, s);").map_err(|e| e.to_string())?;
        writeln!(out, "        let blend = combine_blend(d, d_s, s.combination_op, s.smooth_k);").map_err(|e| e.to_string())?;
        writeln!(out, "        let t = blend.y;").map_err(|e| e.to_string())?;
        writeln!(out, "        d = blend.x;").map_err(|e| e.to_string())?;
        writeln!(out, "        result.color = mix(result.color, get_shape_color(s, p), t);").map_err(|e| e.to_string())?;
        writeln!(out, "        result.roughness = mix(result.roughness, s.roughness, t);").map_err(|e| e.to_string())?;
        writeln!(out, "        result.metallic = mix(result.metallic, s.metallic, t);").map_err(|e| e.to_string())?;
        writeln!(out, "        result.fresnel_power = mix(result.fresnel_power, s.fresnel_power, t);").map_err(|e| e.to_string())?;
        writeln!(out, "        if t > 0.5 {{ result.color_mode = s.color_mode; }}").map_err(|e| e.to_string())?;
        writeln!(out, "    }}").map_err(|e| e.to_string())?;
    }

    writeln!(out, "    return result;").map_err(|e| e.to_string())?;
    writeln!(out, "}}").map_err(|e| e.to_string())?;
    Ok(())
}

/// Fixed shader preamble: structs, noise, primitives, eval_shape, get_shape_color,
/// combine_blend, BRDF functions. Everything up to (but not including) sdf_scene().
const PREAMBLE: &str = include_str!("../assets/shaders/sdf_preamble.wgsl");

/// Fixed shader postamble: normals, shadows, AO, ray march, fragment shader.
/// Everything after sdf_scene_material().
const POSTAMBLE: &str = include_str!("../assets/shaders/sdf_postamble.wgsl");

/// Write the generated shader to the assets directory for Bevy hot-reload.
/// Returns Ok(true) if the shader was regenerated, Ok(false) if topology unchanged.
pub fn regenerate_if_changed(
    scene: &SdfScene,
    last_hash: &mut u64,
) -> Result<bool, String> {
    let hash = topology_hash(scene);
    if hash == *last_hash {
        return Ok(false);
    }

    let wgsl = generate_shader(scene)?;
    let path = std::path::Path::new("assets/shaders/sdf_raymarch.wgsl");
    std::fs::write(path, &wgsl).map_err(|e| format!("failed to write shader: {e}"))?;
    *last_hash = hash;
    Ok(true)
}

/// Compute a topology hash for change detection.
/// Only changes when shape count, types, combinations, or modifiers change.
pub fn topology_hash(scene: &SdfScene) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    let overrides = std::collections::HashMap::new();
    let world_transforms = scene::compute_bone_world_transforms(
        &scene.root_bone,
        Mat4::IDENTITY,
        &overrides,
    );
    let mut flat = Vec::new();
    scene::flatten_bone_tree(&scene.root_bone, &world_transforms, &mut flat);
    flat.len().hash(&mut hasher);
    for fs in &flat {
        fs.primitive_type.hash(&mut hasher);
        fs.combination_op.hash(&mut hasher);
        fs.modifier_flags.hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_default_scene() {
        let scene = SdfScene::default_scene();
        let wgsl = generate_shader(&scene).unwrap();
        assert!(wgsl.contains("fn sdf_scene("));
        assert!(wgsl.contains("fn sdf_scene_material("));
        assert!(wgsl.contains("params.shapes[0]"));
        // Scene has 5 shapes
        assert!(wgsl.contains("params.shapes[4u]"));
    }

    #[test]
    fn empty_scene_errors() {
        let scene = SdfScene::new("Empty");
        let result = generate_shader(&scene);
        assert!(result.is_err());
    }

    #[test]
    fn topology_hash_changes_on_shape_add() {
        let mut scene = SdfScene::new("Test");
        scene.root_bone.shapes.push(SdfShape::default_sphere());
        let h1 = topology_hash(&scene);
        scene.root_bone.shapes.push(SdfShape::new("Box", SdfPrimitive::Box { half_extents: [0.5; 3] }));
        let h2 = topology_hash(&scene);
        assert_ne!(h1, h2);
    }
}
