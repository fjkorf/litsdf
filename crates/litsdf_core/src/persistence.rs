use std::path::{Path, PathBuf};

use crate::models::SdfScene;

pub fn scenes_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("litsdf")
        .join("scenes")
}

pub fn save_scene(scene: &SdfScene, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let yaml = serde_yaml::to_string(scene).map_err(|e| e.to_string())?;
    std::fs::write(path, yaml).map_err(|e| e.to_string())
}

pub fn load_scene(path: &Path) -> Result<SdfScene, String> {
    let yaml = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_yaml::from_str(&yaml).map_err(|e| {
        let msg = e.to_string();
        // Detect old scene files that have removed animation fields
        if msg.contains("anim_tx") || msg.contains("anim_ty") || msg.contains("anim_tz")
            || msg.contains("anim_rx") || msg.contains("anim_ry") || msg.contains("anim_rz")
            || msg.contains("anim_scale")
        {
            format!(
                "Scene file uses legacy animation fields (anim_tx, anim_ty, etc.) which have been \
                 removed. Animation is now done via node graphs in the editor.\n\
                 To migrate: remove all anim_* fields from the YAML file, then use the node editor \
                 to recreate animations.\n\
                 Original error: {msg}"
            )
        } else {
            msg
        }
    })
}

pub fn list_scenes(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml")
        })
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SdfScene;

    #[test]
    fn save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.yaml");
        let scene = SdfScene::default_scene();

        save_scene(&scene, &path).unwrap();
        let loaded = load_scene(&path).unwrap();

        assert_eq!(scene.name, loaded.name);
        assert_eq!(scene.root_bone.children.len(), loaded.root_bone.children.len());
        // Verify nested structure survived round-trip
        let orig_shapes = scene.root_bone.all_shapes();
        let loaded_shapes = loaded.root_bone.all_shapes();
        assert_eq!(orig_shapes.len(), loaded_shapes.len());
    }

    #[test]
    fn list_scenes_finds_yaml() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("scene1.yaml"), "").unwrap();
        std::fs::write(dir.path().join("scene2.yml"), "").unwrap();
        std::fs::write(dir.path().join("readme.txt"), "").unwrap();

        let names = list_scenes(dir.path());
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"scene1.yaml".to_string()));
        assert!(names.contains(&"scene2.yml".to_string()));
    }

    #[test]
    fn load_nonexistent_errors() {
        let result = load_scene(Path::new("/nonexistent/path.yaml"));
        assert!(result.is_err());
    }
}
