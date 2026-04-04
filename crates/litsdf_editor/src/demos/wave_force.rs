use std::collections::HashMap;
use egui_snarl::{InPinId, OutPinId, Snarl};
use litsdf_core::models::*;
use super::DemoResult;
use crate::nodes::SdfNode;

/// Demo: a sphere driven by oscillating force from the node graph.
/// Uses SinOscillator → BoneOutput.ForceY to create a bouncing ball.
pub fn create() -> DemoResult {
    let mut root = SdfBone::root();
    let mut bone_graphs = HashMap::new();

    // Ground reference (visual)
    let mut ground = SdfShape::new("Ground", SdfPrimitive::RoundBox {
        half_extents: [1.5, 0.05, 0.8], rounding: 0.02,
    });
    ground.transform.translation = [0.0, -0.5, 0.0];
    ground.material.color = [0.4, 0.4, 0.45];
    ground.material.metallic = 0.3;
    root.shapes.push(ground);

    // Physics ball bone
    let mut ball_bone = SdfBone::new("Ball");
    ball_bone.transform.translation = [0.0, 0.5, 0.0];
    ball_bone.physics = BonePhysicsProps { mass: 0.5, damping: 0.99, ..Default::default() };
    let ball_id = ball_bone.id;

    let mut ball = SdfShape::new("Ball", SdfPrimitive::Sphere { radius: 0.25 });
    ball.material.color = [0.3, 0.8, 0.4];
    ball.material.roughness = 0.3;
    ball.material.metallic = 0.4;
    ball_bone.shapes.push(ball);

    root.children.push(ball_bone);

    // Bone graph: Time → SinOscillator(amp=15, freq=0.5) → BoneOutput.ForceY (pin 8)
    // The oscillating force fights gravity, creating a bobbing motion driven by physics.
    let mut graph = Snarl::new();
    let t = graph.insert_node(egui::pos2(50.0, 150.0), SdfNode::Time);
    let osc = graph.insert_node(egui::pos2(250.0, 100.0), SdfNode::SinOscillator {
        amplitude: 15.0, frequency: 0.5, phase: 0.0,
    });
    let out = graph.insert_node(egui::pos2(500.0, 200.0), SdfNode::BoneOutput);

    graph.connect(OutPinId { node: t, output: 0 }, InPinId { node: osc, input: 3 }); // Time
    graph.connect(OutPinId { node: osc, output: 0 }, InPinId { node: out, input: 8 }); // Force Y

    bone_graphs.insert(ball_id, graph);

    DemoResult {
        scene: SdfScene {
            name: "Wave Force".into(),
            description: "A sphere driven by oscillating upward force from the node graph. The SinOscillator outputs to BoneOutput Force Y, fighting gravity to create physics-driven bouncing.".into(),
            root_bone: root,
            combination: CombinationOp::Union,
            light_dir: [0.6, 0.8, 0.4],
            settings: SceneSettings::default(),
        },
        shape_graphs: HashMap::new(),
        bone_graphs,
    }
}
