use std::collections::HashMap;
use egui_snarl::{InPinId, OutPinId, Snarl};
use litsdf_core::models::*;
use super::DemoResult;
use crate::nodes::SdfNode;

/// Build a walker bone graph: walk when grounded, brake at edges.
fn walker_graph() -> Snarl<SdfNode> {
    let mut g = Snarl::new();

    let is_col = g.insert_node(egui::pos2(50.0, 50.0), SdfNode::IsColliding);
    let walk_force = g.insert_node(egui::pos2(50.0, 150.0), SdfNode::Constant { value: 3.0 });
    let walk_gate = g.insert_node(egui::pos2(250.0, 80.0), SdfNode::Gate);
    g.connect(OutPinId { node: walk_force, output: 0 }, InPinId { node: walk_gate, input: 0 });
    g.connect(OutPinId { node: is_col, output: 0 }, InPinId { node: walk_gate, input: 1 });

    let ray = g.insert_node(egui::pos2(50.0, 300.0), SdfNode::RaycastDown);
    let thresh = g.insert_node(egui::pos2(50.0, 430.0), SdfNode::Constant { value: 1.5 });
    let cmp = g.insert_node(egui::pos2(250.0, 350.0), SdfNode::Compare { mode: 0 });
    let brake = g.insert_node(egui::pos2(250.0, 450.0), SdfNode::Constant { value: -5.0 });
    let brake_gate = g.insert_node(egui::pos2(450.0, 400.0), SdfNode::Gate);
    g.connect(OutPinId { node: ray, output: 0 }, InPinId { node: cmp, input: 0 });
    g.connect(OutPinId { node: thresh, output: 0 }, InPinId { node: cmp, input: 1 });
    g.connect(OutPinId { node: brake, output: 0 }, InPinId { node: brake_gate, input: 0 });
    g.connect(OutPinId { node: cmp, output: 0 }, InPinId { node: brake_gate, input: 1 });

    let sum = g.insert_node(egui::pos2(600.0, 200.0), SdfNode::Add);
    let out = g.insert_node(egui::pos2(800.0, 200.0), SdfNode::BoneOutput);
    g.connect(OutPinId { node: walk_gate, output: 0 }, InPinId { node: sum, input: 0 });
    g.connect(OutPinId { node: brake_gate, output: 0 }, InPinId { node: sum, input: 1 });
    g.connect(OutPinId { node: sum, output: 0 }, InPinId { node: out, input: 7 }); // Force X

    g
}

/// Demo: three walkers on a platform with a gap. They march forward and fall in.
pub fn create() -> DemoResult {
    let mut root = SdfBone::root();
    let mut bone_graphs = HashMap::new();

    // Left platform
    let mut plat_left = SdfShape::new("Platform Left", SdfPrimitive::RoundBox {
        half_extents: [2.0, 0.15, 1.0], rounding: 0.05,
    });
    plat_left.transform.translation = [-2.5, -0.5, 0.0];
    plat_left.material.color = [0.45, 0.45, 0.5];
    plat_left.material.roughness = 0.7;
    root.shapes.push(plat_left);

    // Right platform (after the gap)
    let mut plat_right = SdfShape::new("Platform Right", SdfPrimitive::RoundBox {
        half_extents: [2.0, 0.15, 1.0], rounding: 0.05,
    });
    plat_right.transform.translation = [2.5, -0.5, 0.0];
    plat_right.material.color = [0.45, 0.45, 0.5];
    plat_right.material.roughness = 0.7;
    root.shapes.push(plat_right);

    // Three walkers at staggered X positions
    let colors: [[f32; 3]; 3] = [
        [0.9, 0.3, 0.3], // red
        [0.3, 0.8, 0.4], // green
        [0.3, 0.5, 0.9], // blue
    ];
    let x_starts = [-3.5, -3.0, -2.5];

    for (i, (&color, &x)) in colors.iter().zip(x_starts.iter()).enumerate() {
        let mut walker = SdfBone::new(format!("Walker {}", i + 1));
        walker.transform.translation = [x, 0.0, 0.0];
        walker.physics = BonePhysicsProps { mass: 0.3, damping: 0.98, ..Default::default() };
        let walker_id = walker.id;

        let mut body = SdfShape::new("Body", SdfPrimitive::Capsule { radius: 0.15, half_height: 0.1 });
        body.material.color = color;
        body.material.roughness = 0.4;
        walker.shapes.push(body);

        root.children.push(walker);
        bone_graphs.insert(walker_id, walker_graph());
    }

    DemoResult {
        scene: SdfScene {
            name: "Lemmings".into(),
            description: "Three walkers march across a platform with a gap. Each uses IsColliding and RaycastDown nodes to walk when grounded and brake at edges. Watch as they approach the gap — physics and node logic interact.".into(),
            root_bone: root,
            combination: CombinationOp::Union,
            light_dir: [0.6, 0.8, 0.4],
            settings: SceneSettings { ground_plane: true, ..SceneSettings::default() },
        },
        shape_graphs: HashMap::new(),
        bone_graphs,
    }
}
