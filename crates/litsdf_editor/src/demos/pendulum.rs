use std::collections::HashMap;
use litsdf_core::models::*;
use super::DemoResult;
use crate::nodes;

pub fn create() -> DemoResult {
    let mut root = SdfBone::root();
    let mut bone_graphs = HashMap::new();

    // Mount point on root
    let mut mount = SdfShape::new("Mount", SdfPrimitive::RoundBox {
        half_extents: [0.3, 0.1, 0.2], rounding: 0.03,
    });
    mount.transform.translation = [0.0, 1.0, 0.0];
    mount.material.color = [0.5, 0.5, 0.55];
    mount.material.metallic = 0.6;
    mount.material.roughness = 0.3;
    root.shapes.push(mount);

    // Animated arm bone (bobs up and down)
    let mut arm = SdfBone::new("Arm");
    arm.transform.translation = [0.0, 1.0, 0.0];
    let mut rod = SdfShape::new("Rod", SdfPrimitive::Capsule {
        radius: 0.04, half_height: 0.3,
    });
    rod.material.color = [0.6, 0.6, 0.6];
    rod.material.metallic = 0.7;
    arm.shapes.push(rod);
    let arm_id = arm.id;

    // Physics weight child
    let mut weight = SdfBone::new("Weight");
    weight.transform.translation = [0.6, -0.5, 0.0];
    weight.physics = BonePhysicsProps { mass: 1.0, damping: 0.99, ..Default::default() };
    let mut ball = SdfShape::new("Ball", SdfPrimitive::Sphere { radius: 0.25 });
    ball.material.color = [0.9, 0.3, 0.2];
    ball.material.roughness = 0.4;
    ball.material.metallic = 0.3;
    weight.shapes.push(ball);

    arm.children.push(weight);
    root.children.push(arm);

    // Animate the arm with a slow vertical bob
    bone_graphs.insert(arm_id, nodes::bone_bob_preset(0.3, 0.4));

    DemoResult {
        scene: SdfScene {
            name: "Pendulum".into(),
            description: "An animated arm bobs vertically while a weighted sphere swings below. The weight starts displaced sideways, demonstrating physics and animation blending.".into(),
            root_bone: root,
            combination: CombinationOp::Union,
            light_dir: [0.6, 0.8, 0.4],
            settings: SceneSettings::default(),
        },
        shape_graphs: HashMap::new(),
        bone_graphs,
    }
}
