use std::path::PathBuf;
use clap::Subcommand;
use litsdf_core::models::*;

#[derive(Subcommand)]
pub enum ShapeCmd {
    /// Add a shape to a bone
    Add {
        /// Scene file
        file: PathBuf,
        /// Bone name
        #[arg(long)]
        bone: String,
        /// Primitive type (Sphere, Box, RoundBox, Cylinder, CappedCone, Torus, Capsule, Plane, Ellipsoid)
        #[arg(long = "type")]
        prim_type: String,
        /// Shape name
        #[arg(long)]
        name: Option<String>,
        /// Primitive parameters a,b,c,d
        #[arg(long, value_parser = parse_vec4)]
        params: Option<[f32; 4]>,
    },
    /// Remove a shape
    Remove {
        /// Scene file
        file: PathBuf,
        /// Shape name
        #[arg(long)]
        name: String,
    },
    /// Set shape properties
    Set {
        /// Scene file
        file: PathBuf,
        /// Shape name
        #[arg(long)]
        name: String,
        /// Translation x,y,z
        #[arg(long, value_parser = parse_vec3)]
        translate: Option<[f32; 3]>,
        /// Rotation rx,ry,rz in degrees
        #[arg(long, value_parser = parse_vec3)]
        rotate: Option<[f32; 3]>,
        /// Uniform scale
        #[arg(long)]
        scale: Option<f32>,
        /// Color r,g,b (0-1)
        #[arg(long, value_parser = parse_vec3)]
        color: Option<[f32; 3]>,
        /// Roughness (0-1)
        #[arg(long)]
        roughness: Option<f32>,
        /// Metallic (0-1)
        #[arg(long)]
        metallic: Option<f32>,
        /// Fresnel/rim power
        #[arg(long)]
        fresnel: Option<f32>,
        /// Combination operation (Union, Intersection, Subtraction, SmoothUnion, SmoothIntersection, SmoothSubtraction)
        #[arg(long)]
        combine: Option<String>,
        /// Blend radius for smooth operations
        #[arg(long)]
        blend_k: Option<f32>,
    },
    /// Change primitive type and parameters
    SetType {
        /// Scene file
        file: PathBuf,
        /// Shape name
        #[arg(long)]
        name: String,
        /// Primitive type
        #[arg(long = "type")]
        prim_type: String,
        /// Primitive parameters a,b,c,d
        #[arg(long, value_parser = parse_vec4)]
        params: Option<[f32; 4]>,
    },
    /// Duplicate a shape
    Duplicate {
        /// Scene file
        file: PathBuf,
        /// Shape name
        #[arg(long)]
        name: String,
        /// Name for the copy
        #[arg(long)]
        r#as: Option<String>,
    },
    /// Move a shape to a different bone
    Reparent {
        /// Scene file
        file: PathBuf,
        /// Shape name
        #[arg(long)]
        name: String,
        /// Target bone name
        #[arg(long)]
        bone: String,
    },
    /// Set color mode and palette
    ColorMode {
        /// Scene file
        file: PathBuf,
        /// Shape name
        #[arg(long)]
        name: String,
        /// Mode: solid, palette, noise
        #[arg(long)]
        mode: String,
        /// Palette A (bias) r,g,b
        #[arg(long, value_parser = parse_vec3)]
        palette_a: Option<[f32; 3]>,
        /// Palette B (amplitude) r,g,b
        #[arg(long, value_parser = parse_vec3)]
        palette_b: Option<[f32; 3]>,
        /// Palette C (frequency) r,g,b
        #[arg(long, value_parser = parse_vec3)]
        palette_c: Option<[f32; 3]>,
        /// Palette D (phase) r,g,b
        #[arg(long, value_parser = parse_vec3)]
        palette_d: Option<[f32; 3]>,
    },
    /// Set noise parameters
    Noise {
        /// Scene file
        file: PathBuf,
        /// Shape name
        #[arg(long)]
        name: String,
        /// Noise amplitude
        #[arg(long)]
        amp: Option<f32>,
        /// Noise frequency
        #[arg(long)]
        freq: Option<f32>,
        /// Noise octaves
        #[arg(long)]
        oct: Option<u32>,
    },
    /// List all shapes
    List {
        /// Scene file
        file: PathBuf,
    },
}

fn parse_vec2(s: &str) -> Result<[f32; 2], String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 { return Err("expected 2 comma-separated values".into()); }
    Ok([
        parts[0].trim().parse().map_err(|e| format!("{e}"))?,
        parts[1].trim().parse().map_err(|e| format!("{e}"))?,
    ])
}

fn parse_vec3(s: &str) -> Result<[f32; 3], String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 3 { return Err("expected 3 comma-separated values".into()); }
    Ok([
        parts[0].trim().parse().map_err(|e| format!("{e}"))?,
        parts[1].trim().parse().map_err(|e| format!("{e}"))?,
        parts[2].trim().parse().map_err(|e| format!("{e}"))?,
    ])
}

fn parse_vec4(s: &str) -> Result<[f32; 4], String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 4 { return Err("expected 4 comma-separated values".into()); }
    Ok([
        parts[0].trim().parse().map_err(|e| format!("{e}"))?,
        parts[1].trim().parse().map_err(|e| format!("{e}"))?,
        parts[2].trim().parse().map_err(|e| format!("{e}"))?,
        parts[3].trim().parse().map_err(|e| format!("{e}"))?,
    ])
}

fn find_shape_mut<'a>(scene: &'a mut SdfScene, name: &str) -> Result<&'a mut SdfShape, String> {
    let shape_id = scene.root_bone.find_shape_by_name(name)
        .ok_or_else(|| format!("shape \"{}\" not found", name))?.0.id;
    scene.root_bone.find_shape_mut(shape_id)
        .map(|(s, _)| s)
        .ok_or_else(|| format!("shape \"{}\" not found", name))
}

fn parse_combo(name: &str, k: Option<f32>) -> Result<CombinationOp, String> {
    match name {
        "Union" => Ok(CombinationOp::Union),
        "Intersection" => Ok(CombinationOp::Intersection),
        "Subtraction" => Ok(CombinationOp::Subtraction),
        "SmoothUnion" => Ok(CombinationOp::SmoothUnion { k: k.unwrap_or(0.3) }),
        "SmoothIntersection" => Ok(CombinationOp::SmoothIntersection { k: k.unwrap_or(0.3) }),
        "SmoothSubtraction" => Ok(CombinationOp::SmoothSubtraction { k: k.unwrap_or(0.3) }),
        "ChamferUnion" => Ok(CombinationOp::ChamferUnion { k: k.unwrap_or(0.3) }),
        "ChamferIntersection" => Ok(CombinationOp::ChamferIntersection { k: k.unwrap_or(0.3) }),
        _ => Err(format!("unknown combination op \"{}\"", name)),
    }
}

fn color_mode_from_str(s: &str) -> Result<u32, String> {
    match s {
        "solid" => Ok(0),
        "palette" => Ok(1),
        "noise" => Ok(2),
        _ => Err(format!("unknown color mode \"{}\", expected: solid, palette, noise", s)),
    }
}

pub fn run(cmd: ShapeCmd) -> Result<(), String> {
    match cmd {
        ShapeCmd::Add { file, bone, prim_type, name, params } => {
            super::mutate(&file, |scene| {
                let bone_id = scene.root_bone.find_bone_by_name(&bone)
                    .ok_or_else(|| format!("bone \"{}\" not found", bone))?.id;
                let prim = SdfPrimitive::default_for(&prim_type);
                let shape_name = name.unwrap_or_else(|| prim_type.clone());
                let mut shape = SdfShape::new(&shape_name, prim);
                if let Some([a, b, c, d]) = params {
                    set_prim_params(&mut shape.primitive, a, b, c, d);
                }
                let shape_id = shape.id;
                let target = scene.root_bone.find_bone_mut(bone_id).unwrap();
                target.shapes.push(shape);
                Ok(format!("Added shape \"{}\" ({}) to bone \"{}\"", shape_name, shape_id.0, bone))
            })
        }
        ShapeCmd::Remove { file, name } => {
            super::mutate(&file, |scene| {
                let (shape, _) = scene.root_bone.find_shape_by_name(&name)
                    .ok_or_else(|| format!("shape \"{}\" not found", name))?;
                let shape_id = shape.id;
                scene.root_bone.remove_shape(shape_id);
                Ok(format!("Removed shape \"{}\"", name))
            })
        }
        ShapeCmd::Set { file, name, translate, rotate, scale, color, roughness, metallic, fresnel, combine, blend_k } => {
            super::mutate(&file, |scene| {
                let shape = find_shape_mut(scene, &name)?;
                if let Some(t) = translate { shape.transform.translation = t; }
                if let Some(r) = rotate { shape.transform.rotation = r; }
                if let Some(s) = scale { shape.transform.scale = s; }
                if let Some(c) = color { shape.material.color = c; }
                if let Some(r) = roughness { shape.material.roughness = r; }
                if let Some(m) = metallic { shape.material.metallic = m; }
                if let Some(f) = fresnel { shape.material.fresnel_power = f; }
                if let Some(c) = combine { shape.combination = parse_combo(&c, blend_k)?; }
                else if let Some(k) = blend_k {
                    match &mut shape.combination {
                        CombinationOp::SmoothUnion { k: kk } => *kk = k,
                        CombinationOp::SmoothIntersection { k: kk } => *kk = k,
                        CombinationOp::SmoothSubtraction { k: kk }
                        | CombinationOp::ChamferUnion { k: kk }
                        | CombinationOp::ChamferIntersection { k: kk } => *kk = k,
                        _ => return Err("blend-k requires a smooth/chamfer combination op".into()),
                    }
                }
                Ok(format!("Updated shape \"{}\"", name))
            })
        }
        ShapeCmd::SetType { file, name, prim_type, params } => {
            super::mutate(&file, |scene| {
                let shape = find_shape_mut(scene, &name)?;
                shape.primitive = SdfPrimitive::default_for(&prim_type);
                if let Some([a, b, c, d]) = params {
                    set_prim_params(&mut shape.primitive, a, b, c, d);
                }
                Ok(format!("Changed shape \"{}\" to {}", name, prim_type))
            })
        }
        ShapeCmd::Duplicate { file, name, r#as } => {
            super::mutate(&file, |scene| {
                let (shape, bone_id) = scene.root_bone.find_shape_by_name(&name)
                    .ok_or_else(|| format!("shape \"{}\" not found", name))?;
                let mut dup = shape.duplicate();
                if let Some(new_name) = &r#as {
                    dup.name = new_name.clone();
                }
                let dup_id = dup.id;
                let dup_name = dup.name.clone();
                let bone = scene.root_bone.find_bone_mut(bone_id).unwrap();
                bone.shapes.push(dup);
                Ok(format!("Duplicated shape \"{}\" as \"{}\" ({})", name, dup_name, dup_id.0))
            })
        }
        ShapeCmd::Reparent { file, name, bone } => {
            super::mutate(&file, |scene| {
                let (shape, _) = scene.root_bone.find_shape_by_name(&name)
                    .ok_or_else(|| format!("shape \"{}\" not found", name))?;
                let shape_id = shape.id;
                let target_id = scene.root_bone.find_bone_by_name(&bone)
                    .ok_or_else(|| format!("bone \"{}\" not found", bone))?.id;
                if !scene.root_bone.reparent_shape(shape_id, target_id) {
                    return Err(format!("failed to reparent shape \"{}\"", name));
                }
                Ok(format!("Moved shape \"{}\" to bone \"{}\"", name, bone))
            })
        }
        ShapeCmd::ColorMode { file, name, mode, palette_a, palette_b, palette_c, palette_d } => {
            super::mutate(&file, |scene| {
                let shape = find_shape_mut(scene, &name)?;
                shape.material.color_mode = color_mode_from_str(&mode)?;
                if let Some(a) = palette_a { shape.material.palette_a = a; }
                if let Some(b) = palette_b { shape.material.palette_b = b; }
                if let Some(c) = palette_c { shape.material.palette_c = c; }
                if let Some(d) = palette_d { shape.material.palette_d = d; }
                Ok(format!("Set color mode to \"{}\" on shape \"{}\"", mode, name))
            })
        }
        ShapeCmd::Noise { file, name, amp, freq, oct } => {
            super::mutate(&file, |scene| {
                let shape = find_shape_mut(scene, &name)?;
                if let Some(a) = amp { shape.material.noise_amplitude = a; }
                if let Some(f) = freq { shape.material.noise_frequency = f; }
                if let Some(o) = oct { shape.material.noise_octaves = o; }
                Ok(format!("Updated noise on shape \"{}\"", name))
            })
        }
        ShapeCmd::List { file } => {
            let scene = super::load(&file)?;
            list_shapes(&scene.root_bone, 0);
            Ok(())
        }
    }
}

fn list_shapes(bone: &SdfBone, depth: usize) {
    let indent = "  ".repeat(depth);
    for shape in &bone.shapes {
        println!("{}[{}] {} ({})", indent, shape.primitive.label(), shape.name, shape.id.0);
    }
    for child in &bone.children {
        println!("{}{}:", "  ".repeat(depth), child.name);
        list_shapes(child, depth + 1);
    }
}

fn set_prim_params(prim: &mut SdfPrimitive, a: f32, b: f32, c: f32, d: f32) {
    match prim {
        SdfPrimitive::Sphere { radius } => *radius = a,
        SdfPrimitive::Box { half_extents } => *half_extents = [a, b, c],
        SdfPrimitive::RoundBox { half_extents, rounding } => {
            *half_extents = [a, b, c]; *rounding = d;
        }
        SdfPrimitive::Cylinder { height, radius } => { *height = a; *radius = b; }
        SdfPrimitive::CappedCone { height, r1, r2 } => { *height = a; *r1 = b; *r2 = c; }
        SdfPrimitive::Torus { major_radius, minor_radius } => { *major_radius = a; *minor_radius = b; }
        SdfPrimitive::Capsule { radius, half_height } => { *radius = a; *half_height = b; }
        SdfPrimitive::Plane { normal, offset } => { *normal = [a, b, c]; *offset = d; }
        SdfPrimitive::Ellipsoid { radii } => *radii = [a, b, c],
        SdfPrimitive::Octahedron { size } => *size = a,
        SdfPrimitive::Pyramid { height, base } => { *height = a; *base = b; }
        SdfPrimitive::HexPrism { height, radius } => { *height = a; *radius = b; }
        SdfPrimitive::RoundCone { r1, r2, height } => { *r1 = a; *r2 = b; *height = c; }
    }
}
