use std::collections::HashMap;
use litsdf_core::models::*;
use super::DemoResult;

pub fn create() -> DemoResult {
    let mut root = SdfBone::root();

    // Horizontal bar
    let mut bar = SdfShape::new("Bar", SdfPrimitive::RoundBox {
        half_extents: [2.0, 0.06, 0.15], rounding: 0.02,
    });
    bar.transform.translation = [0.0, 1.5, 0.0];
    bar.material.color = [0.5, 0.5, 0.55];
    bar.material.metallic = 0.7;
    bar.material.roughness = 0.3;
    root.shapes.push(bar);

    // Three chains with different damping values
    let configs: &[(&str, f32, f32, [f32; 3])] = &[
        ("Bouncy", -1.5, 0.998, [0.9, 0.5, 0.2]),  // orange, very low damping
        ("Normal",  0.0, 0.99,  [0.3, 0.5, 0.9]),  // blue, normal damping
        ("Heavy",   1.5, 0.95,  [0.3, 0.8, 0.4]),  // green, high damping
    ];

    for (label, x_offset, damping, color) in configs {
        // Anchor bone under the bar
        let mut anchor = SdfBone::new(format!("{label} Anchor"));
        anchor.transform.translation = [*x_offset, 1.5, 0.0];

        // Build 3-link chain bottom-up (first link angled for potential energy)
        let mut chain: Option<SdfBone> = None;
        for i in (0..3).rev() {
            let mut link = SdfBone::new(format!("{label} {}", i + 1));
            link.transform.translation = if i == 0 { [0.3, -0.4, 0.0] } else { [0.0, -0.5, 0.0] };
            link.physics = BonePhysicsProps { mass: 0.5, damping: *damping, ..Default::default() };

            let mut capsule = SdfShape::new("Link", SdfPrimitive::Capsule {
                radius: 0.06, half_height: 0.18,
            });
            capsule.material.color = *color;
            capsule.material.roughness = 0.5;
            link.shapes.push(capsule);

            if let Some(child) = chain.take() {
                link.children.push(child);
            }
            chain = Some(link);
        }

        anchor.children.push(chain.unwrap());
        root.children.push(anchor);
    }

    DemoResult {
        scene: SdfScene {
            name: "Damping Lab".into(),
            description: "Three chains with different damping start angled and swing under gravity. Bouncy (0.998, orange) oscillates longest, heavy (0.95, green) settles fastest.".into(),
            root_bone: root,
            combination: CombinationOp::Union,
            light_dir: [0.6, 0.8, 0.4],
            settings: SceneSettings::default(),
        },
        shape_graphs: HashMap::new(),
        bone_graphs: HashMap::new(),
    }
}
