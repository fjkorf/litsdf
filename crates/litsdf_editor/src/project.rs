use std::collections::HashMap;
use std::path::Path;

use egui_snarl::Snarl;
use serde::{Deserialize, Serialize};

use litsdf_core::models::{BoneId, ShapeId, SdfScene};
use crate::nodes::SdfNode;

/// A project file bundles a scene with its node graphs.
/// Backward compatible: old scene YAML files without graph sections load fine
/// via `#[serde(default)]`.
#[derive(Serialize, Deserialize)]
pub struct ProjectFile {
    pub scene: SdfScene,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub shape_graphs: HashMap<ShapeId, Snarl<SdfNode>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub bone_graphs: HashMap<BoneId, Snarl<SdfNode>>,
}

pub fn save_project(
    scene: &SdfScene,
    shape_graphs: &HashMap<ShapeId, Snarl<SdfNode>>,
    bone_graphs: &HashMap<BoneId, Snarl<SdfNode>>,
    path: &Path,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    // Filter out empty graphs
    let shape_graphs: HashMap<_, _> = shape_graphs.iter()
        .filter(|(_, snarl)| snarl.node_ids().next().is_some())
        .map(|(id, snarl)| (*id, snarl.clone()))
        .collect();
    let bone_graphs: HashMap<_, _> = bone_graphs.iter()
        .filter(|(_, snarl)| snarl.node_ids().next().is_some())
        .map(|(id, snarl)| (*id, snarl.clone()))
        .collect();
    let project = ProjectFile {
        scene: scene.clone(),
        shape_graphs,
        bone_graphs,
    };
    let yaml = serde_yaml::to_string(&project).map_err(|e| e.to_string())?;
    std::fs::write(path, yaml).map_err(|e| e.to_string())
}

pub fn load_project(path: &Path) -> Result<ProjectFile, String> {
    let yaml = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

    // Try loading as ProjectFile first (has `scene:` wrapper)
    if let Ok(project) = serde_yaml::from_str::<ProjectFile>(&yaml) {
        return Ok(project);
    }

    // Fall back to loading as plain SdfScene (old format without graphs)
    match litsdf_core::persistence::load_scene(path) {
        Ok(scene) => Ok(ProjectFile {
            scene,
            shape_graphs: HashMap::new(),
            bone_graphs: HashMap::new(),
        }),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use litsdf_core::models::{SdfScene, SdfBone, SdfShape, SdfPrimitive};
    use egui_snarl::{InPinId, OutPinId};

    #[test]
    fn project_round_trip_with_graphs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.yaml");

        let mut scene = SdfScene::new("Test");
        let mut bone = SdfBone::new("Arm");
        let shape = SdfShape::new("Ball", SdfPrimitive::Sphere { radius: 1.0 });
        let shape_id = shape.id;
        bone.shapes.push(shape);
        scene.root_bone.children.push(bone);

        // Create a shape graph
        let mut snarl = Snarl::new();
        let t = snarl.insert_node(egui::pos2(0.0, 0.0), SdfNode::Time);
        let osc = snarl.insert_node(egui::pos2(200.0, 0.0), SdfNode::SinOscillator {
            amplitude: 0.5, frequency: 1.0, phase: 0.0,
        });
        let out = snarl.insert_node(egui::pos2(400.0, 0.0), SdfNode::ShapeOutput);
        snarl.connect(OutPinId { node: t, output: 0 }, InPinId { node: osc, input: 3 });
        snarl.connect(OutPinId { node: osc, output: 0 }, InPinId { node: out, input: 1 });

        let mut shape_graphs = HashMap::new();
        shape_graphs.insert(shape_id, snarl);

        let bone_graphs = HashMap::new();

        save_project(&scene, &shape_graphs, &bone_graphs, &path).unwrap();

        let loaded = load_project(&path).unwrap();
        assert_eq!(loaded.scene.name, "Test");
        assert_eq!(loaded.shape_graphs.len(), 1);
        assert!(loaded.shape_graphs.contains_key(&shape_id));
        assert!(loaded.bone_graphs.is_empty());

        // Verify graph has nodes
        let graph = &loaded.shape_graphs[&shape_id];
        let node_count: usize = graph.node_ids().count();
        assert_eq!(node_count, 3); // Time, Oscillator, ShapeOutput
    }

    #[test]
    fn old_scene_loads_without_graphs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("old.yaml");

        // Save a plain scene (no graphs) using core's save
        let scene = SdfScene::new("Old Scene");
        litsdf_core::persistence::save_scene(&scene, &path).unwrap();

        // Load as project — should work with empty graphs
        let loaded = load_project(&path).unwrap();
        assert_eq!(loaded.scene.name, "Old Scene");
        assert!(loaded.shape_graphs.is_empty());
        assert!(loaded.bone_graphs.is_empty());
    }

    #[test]
    fn empty_graphs_not_serialized() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.yaml");

        let scene = SdfScene::new("Empty");
        let shape_graphs = HashMap::new();
        let bone_graphs = HashMap::new();

        save_project(&scene, &shape_graphs, &bone_graphs, &path).unwrap();

        let yaml = std::fs::read_to_string(&path).unwrap();
        assert!(!yaml.contains("shape_graphs"));
        assert!(!yaml.contains("bone_graphs"));
    }
}
