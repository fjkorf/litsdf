use std::path::PathBuf;
use clap::Subcommand;
use litsdf_core::models::*;

#[derive(Subcommand)]
pub enum ModifierCmd {
    /// Set a modifier on a shape (replaces existing of same type)
    Set {
        /// Scene file
        file: PathBuf,
        /// Shape name
        #[arg(long)]
        shape: String,
        /// Rounding radius
        #[arg(long)]
        rounding: Option<f32>,
        /// Onion shell thickness
        #[arg(long)]
        onion: Option<f32>,
        /// Twist amount
        #[arg(long)]
        twist: Option<f32>,
        /// Bend amount
        #[arg(long)]
        bend: Option<f32>,
        /// Elongation x,y,z
        #[arg(long, value_parser = parse_vec3)]
        elongate: Option<[f32; 3]>,
        /// Repetition period x,y,z
        #[arg(long, value_parser = parse_vec3)]
        repeat: Option<[f32; 3]>,
    },
    /// Clear all modifiers from a shape
    Clear {
        /// Scene file
        file: PathBuf,
        /// Shape name
        #[arg(long)]
        shape: String,
    },
    /// List modifiers on a shape
    List {
        /// Scene file
        file: PathBuf,
        /// Shape name
        #[arg(long)]
        shape: String,
    },
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

fn find_shape_mut<'a>(scene: &'a mut SdfScene, name: &str) -> Result<&'a mut SdfShape, String> {
    let shape_id = scene.root_bone.find_shape_by_name(name)
        .ok_or_else(|| format!("shape \"{}\" not found", name))?.0.id;
    scene.root_bone.find_shape_mut(shape_id)
        .map(|(s, _)| s)
        .ok_or_else(|| format!("shape \"{}\" not found", name))
}

/// Replace or add a modifier, removing any existing modifier of the same discriminant.
fn set_modifier(modifiers: &mut Vec<ShapeModifier>, new: ShapeModifier) {
    let disc = std::mem::discriminant(&new);
    modifiers.retain(|m| std::mem::discriminant(m) != disc);
    modifiers.push(new);
}

pub fn run(cmd: ModifierCmd) -> Result<(), String> {
    match cmd {
        ModifierCmd::Set { file, shape, rounding, onion, twist, bend, elongate, repeat } => {
            super::mutate(&file, |scene| {
                let s = find_shape_mut(scene, &shape)?;
                if let Some(r) = rounding { set_modifier(&mut s.modifiers, ShapeModifier::Rounding(r)); }
                if let Some(o) = onion { set_modifier(&mut s.modifiers, ShapeModifier::Onion(o)); }
                if let Some(t) = twist { set_modifier(&mut s.modifiers, ShapeModifier::Twist(t)); }
                if let Some(b) = bend { set_modifier(&mut s.modifiers, ShapeModifier::Bend(b)); }
                if let Some(e) = elongate { set_modifier(&mut s.modifiers, ShapeModifier::Elongation(e)); }
                if let Some(r) = repeat {
                    set_modifier(&mut s.modifiers, ShapeModifier::Repetition {
                        period: r,
                        count: [3, 3, 3], // default count
                    });
                }
                Ok(format!("Updated modifiers on shape \"{}\"", shape))
            })
        }
        ModifierCmd::Clear { file, shape } => {
            super::mutate(&file, |scene| {
                let s = find_shape_mut(scene, &shape)?;
                s.clear_modifiers();
                Ok(format!("Cleared all modifiers on shape \"{}\"", shape))
            })
        }
        ModifierCmd::List { file, shape } => {
            let scene = super::load(&file)?;
            let (s, _) = scene.root_bone.find_shape_by_name(&shape)
                .ok_or_else(|| format!("shape \"{}\" not found", shape))?;
            if s.modifiers.is_empty() {
                println!("No modifiers on shape \"{}\"", shape);
            } else {
                println!("Modifiers on shape \"{}\":", shape);
                for m in &s.modifiers {
                    match m {
                        ShapeModifier::Rounding(r) => println!("  Rounding: {r}"),
                        ShapeModifier::Onion(t) => println!("  Onion: {t}"),
                        ShapeModifier::Twist(a) => println!("  Twist: {a}"),
                        ShapeModifier::Bend(a) => println!("  Bend: {a}"),
                        ShapeModifier::Elongation(e) => println!("  Elongation: {:?}", e),
                        ShapeModifier::Repetition { period, count } => {
                            println!("  Repetition: period={:?} count={:?}", period, count);
                        }
                    }
                }
            }
            Ok(())
        }
    }
}
