use std::collections::HashMap;
use litsdf_core::models::*;
use super::DemoResult;

pub fn create() -> DemoResult {
    let mut root = SdfBone::root();

    // Anchor sphere on root (fixed, no physics)
    let mut anchor = SdfShape::new("Anchor", SdfPrimitive::Sphere { radius: 0.2 });
    anchor.material.color = [0.7, 0.7, 0.7];
    anchor.material.metallic = 0.8;
    anchor.material.roughness = 0.2;
    root.shapes.push(anchor);

    // Build chain bottom-up: innermost link first, then wrap outward
    let colors: [[f32; 3]; 5] = [
        [0.9, 0.5, 0.2],
        [0.8, 0.4, 0.2],
        [0.7, 0.35, 0.2],
        [0.6, 0.3, 0.2],
        [0.5, 0.25, 0.2],
    ];

    let mut chain: Option<SdfBone> = None;
    for i in (0..5).rev() {
        let mut link = SdfBone::new(format!("Link {}", i + 1));
        link.transform.translation = [0.4, 0.0, 0.0];
        link.physics = BonePhysicsProps { mass: 0.5, damping: 0.99, ..Default::default() };

        let mut capsule = SdfShape::new("Segment", SdfPrimitive::Capsule {
            radius: 0.08, half_height: 0.15,
        });
        capsule.material.color = colors[i];
        capsule.material.roughness = 0.6;
        link.shapes.push(capsule);

        if let Some(child) = chain.take() {
            link.children.push(child);
        }
        chain = Some(link);
    }

    root.children.push(chain.unwrap());

    DemoResult {
        scene: SdfScene {
            name: "Hanging Chain".into(),
            description: "A chain of 5 links starting horizontal, falling under gravity. Each bone has mass 0.5 and damping 0.99. Try changing mass or damping in the bone properties panel.".into(),
            root_bone: root,
            combination: CombinationOp::Union,
            light_dir: [0.6, 0.8, 0.4],
            settings: SceneSettings::default(),
        },
        shape_graphs: HashMap::new(),
        bone_graphs: HashMap::new(),
    }
}
