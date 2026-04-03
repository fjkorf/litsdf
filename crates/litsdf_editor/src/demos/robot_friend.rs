use std::collections::HashMap;
use litsdf_core::models::*;
use super::DemoResult;
use crate::nodes;

pub fn create() -> DemoResult {
    let mut root = SdfBone::root();
    let mut shape_graphs = HashMap::new();
    let mut bone_graphs = HashMap::new();

    // Body
    let mut body = SdfShape::new("Body", SdfPrimitive::RoundBox {
        half_extents: [0.5, 0.6, 0.35], rounding: 0.08,
    });
    body.material.color = [0.6, 0.6, 0.65];
    body.material.roughness = 0.4;
    body.material.metallic = 0.6;
    root.shapes.push(body);

    // Head
    let mut head_bone = SdfBone::new("Head");
    head_bone.transform.translation = [0.0, 1.1, 0.0];

    let mut skull = SdfShape::new("Skull", SdfPrimitive::RoundBox {
        half_extents: [0.35, 0.3, 0.3], rounding: 0.1,
    });
    skull.material.color = [0.65, 0.65, 0.7];
    skull.material.metallic = 0.5;
    skull.material.roughness = 0.35;
    skull.combination = CombinationOp::ChamferUnion { k: 0.15 };
    head_bone.shapes.push(skull);

    let mut left_eye = SdfShape::new("Left Eye", SdfPrimitive::Sphere { radius: 0.08 });
    left_eye.transform.translation = [0.12, 0.05, 0.28];
    left_eye.material.color = [0.2, 0.8, 0.9];
    left_eye.material.fresnel_power = 3.0;
    left_eye.material.roughness = 0.1;
    left_eye.combination = CombinationOp::SmoothUnion { k: 0.05 };
    head_bone.shapes.push(left_eye);

    let mut right_eye = SdfShape::new("Right Eye", SdfPrimitive::Sphere { radius: 0.08 });
    right_eye.transform.translation = [-0.12, 0.05, 0.28];
    right_eye.material.color = [0.2, 0.8, 0.9];
    right_eye.material.fresnel_power = 3.0;
    right_eye.material.roughness = 0.1;
    right_eye.combination = CombinationOp::SmoothUnion { k: 0.05 };
    head_bone.shapes.push(right_eye);

    let mut gem = SdfShape::new("Gem", SdfPrimitive::Octahedron { size: 0.12 });
    gem.transform.translation = [0.0, 0.3, 0.0];
    gem.material.color = [0.9, 0.2, 0.2];
    gem.material.metallic = 1.0;
    gem.material.roughness = 0.05;
    gem.combination = CombinationOp::SmoothUnion { k: 0.02 };
    head_bone.shapes.push(gem);

    root.children.push(head_bone);

    // Left arm (animated)
    let mut left_arm = SdfBone::new("Left Arm");
    left_arm.transform.translation = [0.7, 0.3, 0.0];
    let mut arm_shape = SdfShape::new("Arm", SdfPrimitive::Capsule { radius: 0.1, half_height: 0.4 });
    arm_shape.material.color = [0.55, 0.55, 0.6];
    arm_shape.material.metallic = 0.5;
    arm_shape.material.roughness = 0.4;
    left_arm.shapes.push(arm_shape);
    let left_arm_id = left_arm.id;
    root.children.push(left_arm);
    bone_graphs.insert(left_arm_id, nodes::bone_bob_preset(0.15, 0.8));

    // Right arm (animated with phase offset)
    let mut right_arm = SdfBone::new("Right Arm");
    right_arm.transform.translation = [-0.7, 0.3, 0.0];
    let mut arm_shape = SdfShape::new("Arm", SdfPrimitive::Capsule { radius: 0.1, half_height: 0.4 });
    arm_shape.material.color = [0.55, 0.55, 0.6];
    arm_shape.material.metallic = 0.5;
    arm_shape.material.roughness = 0.4;
    right_arm.shapes.push(arm_shape);
    let right_arm_id = right_arm.id;
    root.children.push(right_arm);

    // Phase-offset bob for right arm
    let mut bob_graph = egui_snarl::Snarl::new();
    let t = bob_graph.insert_node(egui::pos2(50.0, 150.0), nodes::SdfNode::Time);
    let osc = bob_graph.insert_node(egui::pos2(250.0, 100.0), nodes::SdfNode::SinOscillator {
        amplitude: 0.15, frequency: 0.8, phase: std::f32::consts::PI,
    });
    let out = bob_graph.insert_node(egui::pos2(500.0, 200.0), nodes::SdfNode::BoneOutput);
    bob_graph.connect(
        egui_snarl::OutPinId { node: t, output: 0 },
        egui_snarl::InPinId { node: osc, input: 3 },
    );
    bob_graph.connect(
        egui_snarl::OutPinId { node: osc, output: 0 },
        egui_snarl::InPinId { node: out, input: 1 },
    );
    bone_graphs.insert(right_arm_id, bob_graph);

    // Neck joint
    let mut neck = SdfShape::new("Neck", SdfPrimitive::Cylinder { height: 0.1, radius: 0.12 });
    neck.transform.translation = [0.0, 0.7, 0.0];
    neck.material.color = [0.5, 0.5, 0.55];
    neck.material.metallic = 0.7;
    neck.combination = CombinationOp::ChamferUnion { k: 0.08 };
    root.shapes.push(neck);

    DemoResult {
        scene: SdfScene {
            name: "Robot Friend".into(),
            root_bone: root,
            combination: CombinationOp::Union,
            light_dir: [0.5, 0.8, 0.5],
            settings: SceneSettings::default(),
        },
        shape_graphs,
        bone_graphs,
    }
}
