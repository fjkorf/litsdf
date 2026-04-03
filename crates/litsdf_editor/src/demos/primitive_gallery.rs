use std::collections::HashMap;
use litsdf_core::models::*;
use super::DemoResult;

pub fn create() -> DemoResult {
    let mut root = SdfBone::root();

    let primitives = [
        ("Sphere",    SdfPrimitive::Sphere { radius: 0.4 },                     [0.9, 0.3, 0.3]),
        ("Box",       SdfPrimitive::Box { half_extents: [0.35, 0.35, 0.35] },   [0.3, 0.9, 0.3]),
        ("RoundBox",  SdfPrimitive::RoundBox { half_extents: [0.3, 0.3, 0.3], rounding: 0.08 }, [0.3, 0.3, 0.9]),
        ("Cylinder",  SdfPrimitive::Cylinder { height: 0.4, radius: 0.25 },     [0.9, 0.9, 0.3]),
        ("CappedCone", SdfPrimitive::CappedCone { height: 0.5, r1: 0.35, r2: 0.1 }, [0.9, 0.5, 0.2]),
        ("Torus",     SdfPrimitive::Torus { major_radius: 0.35, minor_radius: 0.1 }, [0.5, 0.9, 0.9]),
        ("Capsule",   SdfPrimitive::Capsule { radius: 0.15, half_height: 0.3 }, [0.9, 0.3, 0.9]),
        ("Plane",     SdfPrimitive::Plane { normal: [0.0, 1.0, 0.0], offset: 0.0 }, [0.6, 0.6, 0.6]),
        ("Ellipsoid", SdfPrimitive::Ellipsoid { radii: [0.45, 0.3, 0.25] },     [0.4, 0.7, 0.4]),
        ("Octahedron", SdfPrimitive::Octahedron { size: 0.4 },                  [0.7, 0.4, 0.7]),
        ("Pyramid",   SdfPrimitive::Pyramid { height: 0.5, base: 0.6 },         [0.8, 0.6, 0.3]),
        ("HexPrism",  SdfPrimitive::HexPrism { height: 0.35, radius: 0.3 },     [0.3, 0.6, 0.8]),
        ("RoundCone", SdfPrimitive::RoundCone { r1: 0.3, r2: 0.1, height: 0.5 }, [0.6, 0.8, 0.5]),
    ];

    for (i, (name, prim, color)) in primitives.iter().enumerate() {
        let angle = (i as f32 / primitives.len() as f32) * std::f32::consts::TAU;
        let radius = 3.0;
        let x = angle.sin() * radius;
        let z = angle.cos() * radius;

        let mut shape = SdfShape::new(*name, prim.clone());
        shape.material.color = *color;
        shape.material.roughness = 0.5;
        shape.transform.translation = [x, 0.0, z];
        root.shapes.push(shape);
    }

    DemoResult {
        scene: SdfScene {
            name: "Primitive Gallery".into(),
            root_bone: root,
            combination: CombinationOp::Union,
            light_dir: [0.6, 0.8, 0.4],
            settings: SceneSettings::default(),
        },
        shape_graphs: HashMap::new(),
        bone_graphs: HashMap::new(),
    }
}
