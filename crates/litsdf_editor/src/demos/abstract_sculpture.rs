use std::collections::HashMap;
use litsdf_core::models::*;
use super::DemoResult;
use crate::nodes;

pub fn create() -> DemoResult {
    let mut root = SdfBone::root();
    let mut shape_graphs = HashMap::new();
    let mut bone_graphs = HashMap::new();

    // Main form bone (animated spin)
    let mut form = SdfBone::new("Form");
    let form_id = form.id;

    // Torus ring
    let mut ring = SdfShape::new("Ring", SdfPrimitive::Torus {
        major_radius: 0.8, minor_radius: 0.15,
    });
    ring.material.color = [0.7, 0.4, 0.2];
    ring.material.roughness = 0.3;
    ring.material.metallic = 0.4;
    let ring_id = ring.id;
    form.shapes.push(ring);

    // Crystal intersected with ring
    let mut crystal = SdfShape::new("Crystal", SdfPrimitive::Octahedron { size: 0.6 });
    crystal.material.color = [0.3, 0.5, 0.8];
    crystal.material.roughness = 0.15;
    crystal.material.metallic = 0.7;
    crystal.material.fresnel_power = 2.0;
    crystal.combination = CombinationOp::SmoothIntersection { k: 0.15 };
    form.shapes.push(crystal);

    // Twisted spire
    let mut spire = SdfShape::new("Spire", SdfPrimitive::Pyramid { height: 1.2, base: 0.4 });
    spire.material.color = [0.5, 0.3, 0.6];
    spire.material.roughness = 0.5;
    spire.modifiers.push(ShapeModifier::Twist(2.5));
    spire.combination = CombinationOp::SmoothUnion { k: 0.2 };
    form.shapes.push(spire);

    root.children.push(form);

    // Animate: slow bone spin
    bone_graphs.insert(form_id, nodes::bone_spin_preset(8.0));

    // Animate: color cycle on ring
    shape_graphs.insert(ring_id, nodes::color_cycle_preset(0.2));

    // Custom scene settings: dramatic lighting
    let settings = SceneSettings {
        fill_color: [0.5, 0.4, 0.6],
        fill_intensity: 0.3,
        back_color: [0.4, 0.2, 0.1],
        back_intensity: 0.25,
        sss_color: [0.8, 0.3, 0.2],
        sss_intensity: 0.2,
        ao_intensity: 4.0,
        shadow_softness: 12.0,
        vignette_intensity: 0.4,
    };

    DemoResult {
        scene: SdfScene {
            name: "Abstract Sculpture".into(),
            root_bone: root,
            combination: CombinationOp::Union,
            light_dir: [0.4, 0.7, 0.6],
            settings,
        },
        shape_graphs,
        bone_graphs,
    }
}
