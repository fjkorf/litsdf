use std::collections::HashMap;
use egui_snarl::{InPinId, OutPinId, Snarl};
use litsdf_core::models::*;
use super::DemoResult;
use crate::nodes::SdfNode;

/// Demo: a falling sphere whose color shifts from blue (still) to red (fast).
/// Uses BoneSpeed physics input node → Remap → ShapeOutput.Red/Blue.
pub fn create() -> DemoResult {
    let mut root = SdfBone::root();
    let mut shape_graphs = HashMap::new();

    // Anchor point (visual only)
    let mut anchor = SdfShape::new("Anchor", SdfPrimitive::Sphere { radius: 0.1 });
    anchor.material.color = [0.5, 0.5, 0.5];
    anchor.transform.translation = [0.0, 1.5, 0.0];
    root.shapes.push(anchor);

    // Physics ball on a child bone
    let mut arm = SdfBone::new("Arm");
    arm.transform.translation = [0.0, 1.5, 0.0];

    let mut ball_bone = SdfBone::new("Ball");
    ball_bone.transform.translation = [0.8, 0.0, 0.0]; // offset for pendulum swing
    ball_bone.physics = BonePhysicsProps { mass: 1.0, damping: 0.995, ..Default::default() };

    let mut ball = SdfShape::new("Glow Ball", SdfPrimitive::Sphere { radius: 0.3 });
    ball.material.color = [0.2, 0.3, 0.9]; // start blue
    ball.material.roughness = 0.2;
    ball.material.metallic = 0.5;
    let ball_id = ball.id;
    ball_bone.shapes.push(ball);

    arm.children.push(ball_bone);
    root.children.push(arm);

    // Shape graph: BoneSpeed → Remap(0..3 → 0..1) → Red + inverted Blue
    // This makes the ball glow red when moving fast, blue when still.
    let mut graph = Snarl::new();
    let speed = graph.insert_node(egui::pos2(50.0, 100.0), SdfNode::BoneSpeed);
    let remap = graph.insert_node(egui::pos2(250.0, 100.0), SdfNode::Remap {
        in_min: 0.0, in_max: 3.0, out_min: 0.0, out_max: 1.0,
    });
    let negate = graph.insert_node(egui::pos2(450.0, 200.0), SdfNode::Negate);
    let add_one = graph.insert_node(egui::pos2(450.0, 300.0), SdfNode::Add);
    let one = graph.insert_node(egui::pos2(300.0, 350.0), SdfNode::Constant { value: 1.0 });
    let out = graph.insert_node(egui::pos2(650.0, 150.0), SdfNode::ShapeOutput);

    // BoneSpeed → Remap → Red (pin 7)
    graph.connect(OutPinId { node: speed, output: 0 }, InPinId { node: remap, input: 0 });
    graph.connect(OutPinId { node: remap, output: 0 }, InPinId { node: out, input: 7 }); // Red

    // Remap → Negate → Add(1.0) → Blue (pin 9) — inverted: fast=0, slow=1
    graph.connect(OutPinId { node: remap, output: 0 }, InPinId { node: negate, input: 0 });
    graph.connect(OutPinId { node: negate, output: 0 }, InPinId { node: add_one, input: 0 });
    graph.connect(OutPinId { node: one, output: 0 }, InPinId { node: add_one, input: 1 });
    graph.connect(OutPinId { node: add_one, output: 0 }, InPinId { node: out, input: 9 }); // Blue

    shape_graphs.insert(ball_id, graph);

    DemoResult {
        scene: SdfScene {
            name: "Speed Glow".into(),
            description: "A pendulum ball that glows red when moving fast and blue when still. Uses the BoneSpeed physics node to drive material color through the node graph.".into(),
            root_bone: root,
            combination: CombinationOp::Union,
            light_dir: [0.6, 0.8, 0.4],
            settings: SceneSettings::default(),
        },
        shape_graphs,
        bone_graphs: HashMap::new(),
    }
}
