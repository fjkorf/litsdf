use std::collections::HashMap;
use egui_snarl::{InPinId, OutPinId, Snarl};
use litsdf_core::models::*;
use super::DemoResult;
use crate::nodes::SdfNode;

/// Build a foot IK graph with oscillating stride.
/// phase: 0.0 for left leg, PI for right leg (alternating steps).
fn foot_ik_graph(stride_amp: f32, stride_freq: f32, phase: f32) -> Snarl<SdfNode> {
    let mut g = Snarl::new();

    // Time-driven stride oscillation for X target
    let time = g.insert_node(egui::pos2(50.0, 50.0), SdfNode::Time);
    let stride_osc = g.insert_node(egui::pos2(250.0, 50.0), SdfNode::SinOscillator {
        amplitude: stride_amp, frequency: stride_freq, phase,
    });
    g.connect(OutPinId { node: time, output: 0 }, InPinId { node: stride_osc, input: 3 }); // time

    // Step height: abs(sin) for foot lift during swing
    let step_osc = g.insert_node(egui::pos2(250.0, 150.0), SdfNode::SinOscillator {
        amplitude: 0.15, frequency: stride_freq, phase,
    });
    g.connect(OutPinId { node: time, output: 0 }, InPinId { node: step_osc, input: 3 });
    let step_abs = g.insert_node(egui::pos2(450.0, 150.0), SdfNode::Abs);
    g.connect(OutPinId { node: step_osc, output: 0 }, InPinId { node: step_abs, input: 0 });

    // Ground Y constant (platform at -0.5, surface at ~-0.35)
    let ground_y = g.insert_node(egui::pos2(250.0, 250.0), SdfNode::Constant { value: -0.35 });
    // Foot Y = ground_y + step_height
    let add_y = g.insert_node(egui::pos2(450.0, 250.0), SdfNode::Add);
    g.connect(OutPinId { node: ground_y, output: 0 }, InPinId { node: add_y, input: 0 });
    g.connect(OutPinId { node: step_abs, output: 0 }, InPinId { node: add_y, input: 1 });

    // IK weight
    let weight = g.insert_node(egui::pos2(450.0, 350.0), SdfNode::Constant { value: 1.0 });

    let out = g.insert_node(egui::pos2(650.0, 200.0), SdfNode::BoneOutput);

    // Connect: stride X → IK Target X (pin 13)
    g.connect(OutPinId { node: stride_osc, output: 0 }, InPinId { node: out, input: 13 });
    // Connect: foot Y → IK Target Y (pin 14)
    g.connect(OutPinId { node: add_y, output: 0 }, InPinId { node: out, input: 14 });
    // Z = 0 (no lateral motion, leave unconnected → 0)
    // Weight → pin 16
    g.connect(OutPinId { node: weight, output: 0 }, InPinId { node: out, input: 16 });

    g
}

/// Body bob graph: gentle vertical oscillation.
fn body_bob_graph() -> Snarl<SdfNode> {
    let mut g = Snarl::new();
    let time = g.insert_node(egui::pos2(50.0, 100.0), SdfNode::Time);
    let osc = g.insert_node(egui::pos2(250.0, 100.0), SdfNode::SinOscillator {
        amplitude: 0.05, frequency: 2.0, phase: 0.0, // double frequency = bob per step
    });
    let base_y = g.insert_node(egui::pos2(250.0, 200.0), SdfNode::Constant { value: 1.0 });
    let add = g.insert_node(egui::pos2(450.0, 150.0), SdfNode::Add);
    let out = g.insert_node(egui::pos2(650.0, 150.0), SdfNode::BoneOutput);

    g.connect(OutPinId { node: time, output: 0 }, InPinId { node: osc, input: 3 });
    g.connect(OutPinId { node: osc, output: 0 }, InPinId { node: add, input: 0 });
    g.connect(OutPinId { node: base_y, output: 0 }, InPinId { node: add, input: 1 });
    g.connect(OutPinId { node: add, output: 0 }, InPinId { node: out, input: 1 }); // Pos Y

    g
}

/// Demo: bipedal character with 2-bone IK legs on a platform.
pub fn create() -> DemoResult {
    let mut root = SdfBone::root();
    let mut bone_graphs = HashMap::new();

    // Platform
    let mut platform = SdfShape::new("Platform", SdfPrimitive::RoundBox {
        half_extents: [3.0, 0.15, 1.0], rounding: 0.05,
    });
    platform.transform.translation = [0.0, -0.5, 0.0];
    platform.material.color = [0.45, 0.45, 0.5];
    platform.material.roughness = 0.7;
    root.shapes.push(platform);

    // Body bone (animated bob)
    let mut body = SdfBone::new("Body");
    body.transform.translation = [0.0, 1.0, 0.0];
    // Give body tiny mass so demo auto-plays (has_physics_bones = true)
    body.physics.mass = 0.001;
    body.physics.damping = 0.99;
    let body_id = body.id;

    let mut torso = SdfShape::new("Torso", SdfPrimitive::RoundBox {
        half_extents: [0.2, 0.3, 0.15], rounding: 0.05,
    });
    torso.material.color = [0.3, 0.6, 0.9];
    torso.material.roughness = 0.4;
    body.shapes.push(torso);

    // Left leg: Hip → Knee → Foot (IK effector)
    let mut l_hip = SdfBone::new("L_Hip");
    l_hip.transform.translation = [0.15, -0.3, 0.0];
    let mut l_thigh = SdfShape::new("L_Thigh", SdfPrimitive::Capsule { radius: 0.06, half_height: 0.2 });
    l_thigh.material.color = [0.3, 0.5, 0.8];
    l_hip.shapes.push(l_thigh);

    let mut l_knee = SdfBone::new("L_Knee");
    l_knee.transform.translation = [0.0, -0.5, 0.0];
    let mut l_shin = SdfShape::new("L_Shin", SdfPrimitive::Capsule { radius: 0.05, half_height: 0.2 });
    l_shin.material.color = [0.3, 0.5, 0.8];
    l_knee.shapes.push(l_shin);

    let mut l_foot = SdfBone::new("L_Foot");
    l_foot.transform.translation = [0.0, -0.5, 0.0];
    l_foot.physics.ik_chain_length = 2;
    let mut l_foot_shape = SdfShape::new("L_Foot", SdfPrimitive::Sphere { radius: 0.06 });
    l_foot_shape.material.color = [0.4, 0.4, 0.45];
    l_foot.shapes.push(l_foot_shape);
    let l_foot_id = l_foot.id;

    l_knee.children.push(l_foot);
    l_hip.children.push(l_knee);

    // Right leg
    let mut r_hip = SdfBone::new("R_Hip");
    r_hip.transform.translation = [-0.15, -0.3, 0.0];
    let mut r_thigh = SdfShape::new("R_Thigh", SdfPrimitive::Capsule { radius: 0.06, half_height: 0.2 });
    r_thigh.material.color = [0.3, 0.5, 0.8];
    r_hip.shapes.push(r_thigh);

    let mut r_knee = SdfBone::new("R_Knee");
    r_knee.transform.translation = [0.0, -0.5, 0.0];
    let mut r_shin = SdfShape::new("R_Shin", SdfPrimitive::Capsule { radius: 0.05, half_height: 0.2 });
    r_shin.material.color = [0.3, 0.5, 0.8];
    r_knee.shapes.push(r_shin);

    let mut r_foot = SdfBone::new("R_Foot");
    r_foot.transform.translation = [0.0, -0.5, 0.0];
    r_foot.physics.ik_chain_length = 2;
    let mut r_foot_shape = SdfShape::new("R_Foot", SdfPrimitive::Sphere { radius: 0.06 });
    r_foot_shape.material.color = [0.4, 0.4, 0.45];
    r_foot.shapes.push(r_foot_shape);
    let r_foot_id = r_foot.id;

    r_knee.children.push(r_foot);
    r_hip.children.push(r_knee);

    body.children.push(l_hip);
    body.children.push(r_hip);
    root.children.push(body);

    // Body bob animation
    bone_graphs.insert(body_id, body_bob_graph());
    // Foot IK with alternating stride (phase offset PI for opposite legs)
    bone_graphs.insert(l_foot_id, foot_ik_graph(0.3, 1.0, 0.0));
    bone_graphs.insert(r_foot_id, foot_ik_graph(0.3, 1.0, std::f32::consts::PI));

    DemoResult {
        scene: SdfScene {
            name: "IK Walker".into(),
            description: "A bipedal character with 2-bone IK legs. The body bobs vertically while feet follow oscillating IK targets with alternating stride. The FABRIK solver adjusts knee and hip rotations each frame.".into(),
            root_bone: root,
            combination: CombinationOp::Union,
            light_dir: [0.6, 0.8, 0.4],
            settings: SceneSettings::default(),
        },
        shape_graphs: HashMap::new(),
        bone_graphs,
    }
}
