use std::collections::HashMap;
use egui_snarl::{InPinId, OutPinId, Snarl};
use litsdf_core::models::*;
use super::DemoResult;
use crate::nodes::SdfNode;

/// Demo: a single character that walks forward when grounded and stops at edges.
/// Uses IsColliding → Gate → Force X for movement, RaycastDown → Compare for edge detection.
pub fn create() -> DemoResult {
    let mut root = SdfBone::root();
    let mut bone_graphs = HashMap::new();

    // Platform
    let mut platform = SdfShape::new("Platform", SdfPrimitive::RoundBox {
        half_extents: [3.0, 0.15, 1.0], rounding: 0.05,
    });
    platform.transform.translation = [0.0, -0.5, 0.0];
    platform.material.color = [0.45, 0.45, 0.5];
    platform.material.metallic = 0.3;
    platform.material.roughness = 0.7;
    root.shapes.push(platform);

    // Walker character bone
    let mut walker = SdfBone::new("Walker");
    walker.transform.translation = [-2.0, 0.0, 0.0];
    walker.physics = BonePhysicsProps { mass: 0.5, damping: 0.98, ..Default::default() };
    let walker_id = walker.id;

    let mut body = SdfShape::new("Body", SdfPrimitive::Capsule { radius: 0.2, half_height: 0.15 });
    body.material.color = [0.3, 0.7, 0.9];
    body.material.roughness = 0.4;
    walker.shapes.push(body);

    root.children.push(walker);

    // Walker bone graph:
    // IsColliding → Gate(walkForce) → BoneOutput Force X   (walk when grounded)
    // RaycastDown.Distance → Compare(> 1.5) → Gate(brakeForce) → BoneOutput Force X (brake at edge)
    let mut graph = Snarl::new();

    // Walk force: IsColliding gates a constant forward force
    let is_col = graph.insert_node(egui::pos2(50.0, 50.0), SdfNode::IsColliding);
    let walk_force = graph.insert_node(egui::pos2(50.0, 150.0), SdfNode::Constant { value: 3.0 });
    let walk_gate = graph.insert_node(egui::pos2(250.0, 80.0), SdfNode::Gate);

    graph.connect(OutPinId { node: walk_force, output: 0 }, InPinId { node: walk_gate, input: 0 }); // Value
    graph.connect(OutPinId { node: is_col, output: 0 }, InPinId { node: walk_gate, input: 1 });     // Control

    // Edge detection: RaycastDown distance > 1.5 means edge ahead (no ground)
    // When at edge, apply reverse brake force
    let ray = graph.insert_node(egui::pos2(50.0, 300.0), SdfNode::RaycastDown);
    let edge_thresh = graph.insert_node(egui::pos2(50.0, 430.0), SdfNode::Constant { value: 1.5 });
    let edge_cmp = graph.insert_node(egui::pos2(250.0, 350.0), SdfNode::Compare { mode: 0 }); // GT
    let brake_force = graph.insert_node(egui::pos2(250.0, 450.0), SdfNode::Constant { value: -5.0 });
    let brake_gate = graph.insert_node(egui::pos2(450.0, 400.0), SdfNode::Gate);

    graph.connect(OutPinId { node: ray, output: 0 }, InPinId { node: edge_cmp, input: 0 });         // Distance → A
    graph.connect(OutPinId { node: edge_thresh, output: 0 }, InPinId { node: edge_cmp, input: 1 }); // 1.5 → B
    graph.connect(OutPinId { node: brake_force, output: 0 }, InPinId { node: brake_gate, input: 0 }); // Value
    graph.connect(OutPinId { node: edge_cmp, output: 0 }, InPinId { node: brake_gate, input: 1 });   // Control

    // Sum walk + brake → Force X
    let sum = graph.insert_node(egui::pos2(600.0, 200.0), SdfNode::Add);
    let out = graph.insert_node(egui::pos2(800.0, 200.0), SdfNode::BoneOutput);

    graph.connect(OutPinId { node: walk_gate, output: 0 }, InPinId { node: sum, input: 0 });
    graph.connect(OutPinId { node: brake_gate, output: 0 }, InPinId { node: sum, input: 1 });
    graph.connect(OutPinId { node: sum, output: 0 }, InPinId { node: out, input: 7 }); // Force X

    bone_graphs.insert(walker_id, graph);

    DemoResult {
        scene: SdfScene {
            name: "Walker".into(),
            description: "A character walks forward when grounded (IsColliding → Gate → Force X) and brakes at edges (RaycastDown → Compare → Gate). Open the Node Editor to see the logic graph.".into(),
            root_bone: root,
            combination: CombinationOp::Union,
            light_dir: [0.6, 0.8, 0.4],
            settings: SceneSettings::default(),
        },
        shape_graphs: HashMap::new(),
        bone_graphs,
    }
}
