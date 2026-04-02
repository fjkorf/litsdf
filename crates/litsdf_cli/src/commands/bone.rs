use std::path::PathBuf;
use clap::Subcommand;
use litsdf_core::models::{SdfBone, BoneId};

#[derive(Subcommand)]
pub enum BoneCmd {
    /// Add a new bone
    Add {
        /// Scene file
        file: PathBuf,
        /// Parent bone name
        #[arg(long, default_value = "Root")]
        parent: String,
        /// Bone name
        #[arg(long)]
        name: String,
        /// Translation x,y,z
        #[arg(long, value_parser = parse_vec3)]
        translate: Option<[f32; 3]>,
    },
    /// Remove a bone (reparents children and shapes to parent)
    Remove {
        /// Scene file
        file: PathBuf,
        /// Bone name
        #[arg(long)]
        name: String,
        /// Also delete all children and shapes recursively
        #[arg(long)]
        recursive: bool,
    },
    /// Rename a bone
    Rename {
        /// Scene file
        file: PathBuf,
        /// Current bone name
        #[arg(long)]
        name: String,
        /// New name
        #[arg(long)]
        to: String,
    },
    /// Set bone translation
    Move {
        /// Scene file
        file: PathBuf,
        /// Bone name
        #[arg(long)]
        name: String,
        /// Translation x,y,z
        #[arg(long, value_parser = parse_vec3)]
        translate: [f32; 3],
    },
    /// Set bone rotation (degrees)
    Rotate {
        /// Scene file
        file: PathBuf,
        /// Bone name
        #[arg(long)]
        name: String,
        /// Rotation rx,ry,rz in degrees
        #[arg(long, value_parser = parse_vec3)]
        rotation: [f32; 3],
    },
    /// Deep-duplicate a bone with all its children and shapes
    Duplicate {
        /// Scene file
        file: PathBuf,
        /// Bone name to duplicate
        #[arg(long)]
        name: String,
        /// Name for the copy
        #[arg(long)]
        r#as: Option<String>,
    },
    /// Move a bone to a new parent
    Reparent {
        /// Scene file
        file: PathBuf,
        /// Bone name to move
        #[arg(long)]
        name: String,
        /// New parent bone name
        #[arg(long)]
        parent: String,
    },
    /// List all bones
    List {
        /// Scene file
        file: PathBuf,
    },
}

fn parse_vec3(s: &str) -> Result<[f32; 3], String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 3 {
        return Err("expected 3 comma-separated values (e.g. 1.0,2.0,3.0)".into());
    }
    Ok([
        parts[0].trim().parse().map_err(|e| format!("{e}"))?,
        parts[1].trim().parse().map_err(|e| format!("{e}"))?,
        parts[2].trim().parse().map_err(|e| format!("{e}"))?,
    ])
}

fn parse_vec2(s: &str) -> Result<[f32; 2], String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err("expected 2 comma-separated values (e.g. 0.3,0.5)".into());
    }
    Ok([
        parts[0].trim().parse().map_err(|e| format!("{e}"))?,
        parts[1].trim().parse().map_err(|e| format!("{e}"))?,
    ])
}

fn find_bone_id(root: &SdfBone, name: &str) -> Result<BoneId, String> {
    root.find_bone_by_name(name)
        .map(|b| b.id)
        .ok_or_else(|| format!("bone \"{}\" not found", name))
}

pub fn run(cmd: BoneCmd) -> Result<(), String> {
    match cmd {
        BoneCmd::Add { file, parent, name, translate } => {
            super::mutate(&file, |scene| {
                let parent_id = find_bone_id(&scene.root_bone, &parent)?;
                let mut bone = SdfBone::new(&name);
                if let Some(t) = translate {
                    bone.transform.translation = t;
                }
                let bone_id = bone.id;
                let parent_bone = scene.root_bone.find_bone_mut(parent_id)
                    .ok_or_else(|| format!("parent bone \"{}\" not found", parent))?;
                parent_bone.children.push(bone);
                Ok(format!("Added bone \"{}\" ({})", name, bone_id.0))
            })
        }
        BoneCmd::Remove { file, name, recursive } => {
            super::mutate(&file, |scene| {
                let bone_id = find_bone_id(&scene.root_bone, &name)?;
                if bone_id.is_root() {
                    return Err("cannot remove root bone".into());
                }
                if recursive {
                    // Extract and discard the entire subtree
                    let removed = scene.root_bone.extract_bone(bone_id)
                        .ok_or_else(|| format!("bone \"{}\" not found", name))?;
                    let count = 1 + removed.bone_count();
                    let shapes = removed.shape_count();
                    Ok(format!("Removed bone \"{}\" and {} children, {} shapes", name, count - 1, shapes))
                } else {
                    scene.root_bone.remove_bone(bone_id);
                    Ok(format!("Removed bone \"{}\" (children and shapes reparented)", name))
                }
            })
        }
        BoneCmd::Rename { file, name, to } => {
            super::mutate(&file, |scene| {
                let bone_id = find_bone_id(&scene.root_bone, &name)?;
                let bone = scene.root_bone.find_bone_mut(bone_id).unwrap();
                bone.name = to.clone();
                Ok(format!("Renamed bone \"{}\" → \"{}\"", name, to))
            })
        }
        BoneCmd::Move { file, name, translate } => {
            super::mutate(&file, |scene| {
                let bone_id = find_bone_id(&scene.root_bone, &name)?;
                let bone = scene.root_bone.find_bone_mut(bone_id).unwrap();
                bone.transform.translation = translate;
                Ok(format!("Moved bone \"{}\" to {:?}", name, translate))
            })
        }
        BoneCmd::Rotate { file, name, rotation } => {
            super::mutate(&file, |scene| {
                let bone_id = find_bone_id(&scene.root_bone, &name)?;
                let bone = scene.root_bone.find_bone_mut(bone_id).unwrap();
                bone.transform.rotation = rotation;
                Ok(format!("Rotated bone \"{}\" to {:?}", name, rotation))
            })
        }
        BoneCmd::Duplicate { file, name, r#as } => {
            super::mutate(&file, |scene| {
                let bone_id = find_bone_id(&scene.root_bone, &name)?;
                // Find the parent that contains this bone
                let bone = scene.root_bone.find_bone(bone_id)
                    .ok_or_else(|| format!("bone \"{}\" not found", name))?;
                let mut dup = bone.duplicate_deep();
                if let Some(new_name) = &r#as {
                    dup.name = new_name.clone();
                }
                let dup_id = dup.id;
                // Find parent: walk tree to find who has bone_id as a child
                let parent_id = find_parent_of_bone(&scene.root_bone, bone_id)
                    .ok_or_else(|| format!("could not find parent of bone \"{}\"", name))?;
                let parent = scene.root_bone.find_bone_mut(parent_id).unwrap();
                parent.children.push(dup);
                Ok(format!("Duplicated bone \"{}\" as \"{}\" ({})",
                    name, r#as.as_deref().unwrap_or(&format!("{} Copy", name)), dup_id.0))
            })
        }
        BoneCmd::Reparent { file, name, parent } => {
            super::mutate(&file, |scene| {
                let bone_id = find_bone_id(&scene.root_bone, &name)?;
                let target_id = find_bone_id(&scene.root_bone, &parent)?;
                if !scene.root_bone.reparent_bone(bone_id, target_id) {
                    return Err(format!("cannot reparent \"{}\" under \"{}\" (would create cycle)", name, parent));
                }
                Ok(format!("Reparented bone \"{}\" under \"{}\"", name, parent))
            })
        }
        BoneCmd::List { file } => {
            let scene = super::load(&file)?;
            list_bones(&scene.root_bone, 0);
            Ok(())
        }
    }
}

fn list_bones(bone: &SdfBone, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{}{} ({})", indent, bone.name, bone.id.0);
    for child in &bone.children {
        list_bones(child, depth + 1);
    }
}

fn find_parent_of_bone(bone: &SdfBone, target: BoneId) -> Option<BoneId> {
    for child in &bone.children {
        if child.id == target { return Some(bone.id); }
        if let Some(id) = find_parent_of_bone(child, target) { return Some(id); }
    }
    None
}
