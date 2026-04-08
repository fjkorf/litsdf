use std::collections::HashMap;
use egui_snarl::{InPinId, OutPinId, Snarl};
use litsdf_core::models::*;
use super::DemoResult;
use crate::nodes::SdfNode;

/// Build a foot IK graph: RaycastDown → ground Y → BoneOutput IK Target.
fn foot_ik_graph(foot_x_offset: f32) -> Snarl<SdfNode> {
    let mut g = Snarl::new();

    // World position for current X/Z
    let world_pos = g.insert_node(egui::pos2(50.0, 50.0), SdfNode::BoneWorldPosition);
    // Raycast down for ground Y
    let ray = g.insert_node(egui::pos2(50.0, 200.0), SdfNode::RaycastDown);
    // Negate ray distance to get ground Y (world_pos.Y - ray.distance)
    let negate = g.insert_node(egui::pos2(250.0, 200.0), SdfNode::Negate);
    let add_y = g.insert_node(egui::pos2(400.0, 150.0), SdfNode::Add);

    // X offset constant for stride position
    let x_offset = g.insert_node(egui::pos2(50.0, 350.0), SdfNode::Constant { value: foot_x_offset });
    let add_x = g.insert_node(egui::pos2(250.0, 350.0), SdfNode::Add);

    // IK weight
    let weight = g.insert_node(egui::pos2(400.0, 400.0), SdfNode::Constant { value: 1.0 });

    let out = g.insert_node(egui::pos2(600.0, 200.0), SdfNode::BoneOutput);

    // Ground Y: world_pos.Y + (-ray_distance) = world_pos.Y - ground_distance
    g.connect(OutPinId { node: ray, output: 0 }, InPinId { node: negate, input: 0 }); // -distance
    g.connect(OutPinId { node: world_pos, output: 1 }, InPinId { node: add_y, input: 0 }); // pos.Y
    g.connect(OutPinId { node: negate, output: 0 }, InPinId { node: add_y, input: 1 });    // -dist

    // Foot X: world_pos.X + offset
    g.connect(OutPinId { node: world_pos, output: 0 }, InPinId { node: add_x, input: 0 }); // pos.X
    g.connect(OutPinId { node: x_offset, output: 0 }, InPinId { node: add_x, input: 1 }); // offset

    // Connect to BoneOutput IK Target pins (13=X, 14=Y, 15=Z, 16=weight)
    g.connect(OutPinId { node: add_x, output: 0 }, InPinId { node: out, input: 13 });  // IK Target X
    g.connect(OutPinId { node: add_y, output: 0 }, InPinId { node: out, input: 14 });   // IK Target Y
    g.connect(OutPinId { node: world_pos, output: 2 }, InPinId { node: out, input: 15 }); // IK Target Z (pass through)
    g.connect(OutPinId { node: weight, output: 0 }, InPinId { node: out, input: 16 });  // IK Weight

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

    // Body bone (kinematic, above platform)
    let mut body = SdfBone::new("Body");
    body.transform.translation = [0.0, 1.0, 0.0];

    let mut torso = SdfShape::new("Torso", SdfPrimitive::RoundBox {
        half_extents: [0.2, 0.3, 0.15], rounding: 0.05,
    });
    torso.material.color = [0.3, 0.6, 0.9];
    torso.material.roughness = 0.4;
    body.shapes.push(torso);

    // Left leg: Hip → Knee → Foot (IK effector)
    let mut l_hip = SdfBone::new("L_Hip");
    l_hip.transform.translation = [0.15, -0.3, 0.0];
    let mut l_hip_shape = SdfShape::new("L_Thigh", SdfPrimitive::Capsule { radius: 0.06, half_height: 0.2 });
    l_hip_shape.material.color = [0.3, 0.5, 0.8];
    l_hip.shapes.push(l_hip_shape);

    let mut l_knee = SdfBone::new("L_Knee");
    l_knee.transform.translation = [0.0, -0.5, 0.0];
    let mut l_shin = SdfShape::new("L_Shin", SdfPrimitive::Capsule { radius: 0.05, half_height: 0.2 });
    l_shin.material.color = [0.3, 0.5, 0.8];
    l_knee.shapes.push(l_shin);

    let mut l_foot = SdfBone::new("L_Foot");
    l_foot.transform.translation = [0.0, -0.5, 0.0];
    l_foot.physics.ik_chain_length = 2; // analytical 2-bone
    let mut l_foot_shape = SdfShape::new("L_Foot", SdfPrimitive::Sphere { radius: 0.06 });
    l_foot_shape.material.color = [0.4, 0.4, 0.45];
    l_foot.shapes.push(l_foot_shape);
    let l_foot_id = l_foot.id;

    l_knee.children.push(l_foot);
    l_hip.children.push(l_knee);

    // Right leg
    let mut r_hip = SdfBone::new("R_Hip");
    r_hip.transform.translation = [-0.15, -0.3, 0.0];
    let mut r_hip_shape = SdfShape::new("R_Thigh", SdfPrimitive::Capsule { radius: 0.06, half_height: 0.2 });
    r_hip_shape.material.color = [0.3, 0.5, 0.8];
    r_hip.shapes.push(r_hip_shape);

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

    // Foot IK graphs: target the ground below each foot
    bone_graphs.insert(l_foot_id, foot_ik_graph(0.0));
    bone_graphs.insert(r_foot_id, foot_ik_graph(0.0));

    DemoResult {
        scene: SdfScene {
            name: "IK Walker".into(),
            description: "A bipedal character with 2-bone IK legs. Each foot uses RaycastDown to find the ground and BoneOutput IK Target pins to solve foot placement. The FABRIK solver adjusts knee and hip angles automatically.".into(),
            root_bone: root,
            combination: CombinationOp::Union,
            light_dir: [0.6, 0.8, 0.4],
            settings: SceneSettings::default(),
        },
        shape_graphs: HashMap::new(),
        bone_graphs,
    }
}
