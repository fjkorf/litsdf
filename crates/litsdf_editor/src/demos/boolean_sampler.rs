use std::collections::HashMap;
use litsdf_core::models::*;
use super::DemoResult;

pub fn create() -> DemoResult {
    let mut root = SdfBone::root();

    // Group 1: Subtraction (sphere with box carved out)
    let mut bone1 = SdfBone::new("Subtracted");
    bone1.transform.translation = [-3.0, 0.0, 0.0];
    let mut base = SdfShape::new("Base", SdfPrimitive::Sphere { radius: 0.8 });
    base.material.color = [0.8, 0.3, 0.3];
    bone1.shapes.push(base);
    let mut carve = SdfShape::new("Carve", SdfPrimitive::Box { half_extents: [0.5, 0.5, 1.2] });
    carve.material.color = [0.8, 0.3, 0.3];
    carve.combination = CombinationOp::Subtraction;
    bone1.shapes.push(carve);
    root.children.push(bone1);

    // Group 2: SmoothIntersection
    let mut bone2 = SdfBone::new("Smooth Intersection");
    bone2.transform.translation = [-1.0, 0.0, 0.0];
    let mut a = SdfShape::new("Sphere", SdfPrimitive::Sphere { radius: 0.7 });
    a.material.color = [0.3, 0.7, 0.3];
    bone2.shapes.push(a);
    let mut b = SdfShape::new("Box", SdfPrimitive::Box { half_extents: [0.5, 0.5, 0.5] });
    b.material.color = [0.3, 0.9, 0.5];
    b.combination = CombinationOp::SmoothIntersection { k: 0.2 };
    bone2.shapes.push(b);
    root.children.push(bone2);

    // Group 3: ChamferUnion
    let mut bone3 = SdfBone::new("Chamfer Union");
    bone3.transform.translation = [1.0, 0.0, 0.0];
    let mut a = SdfShape::new("Block", SdfPrimitive::RoundBox { half_extents: [0.4, 0.4, 0.4], rounding: 0.02 });
    a.material.color = [0.3, 0.4, 0.8];
    a.material.metallic = 0.5;
    bone3.shapes.push(a);
    let mut b = SdfShape::new("Sphere", SdfPrimitive::Sphere { radius: 0.5 });
    b.material.color = [0.4, 0.5, 0.9];
    b.material.metallic = 0.5;
    b.combination = CombinationOp::ChamferUnion { k: 0.15 };
    b.transform.translation = [0.3, 0.3, 0.0];
    bone3.shapes.push(b);
    root.children.push(bone3);

    // Group 4: Bowl (SmoothSubtraction)
    let mut bone4 = SdfBone::new("Bowl");
    bone4.transform.translation = [3.0, 0.0, 0.0];
    let mut outer = SdfShape::new("Outer", SdfPrimitive::Ellipsoid { radii: [0.7, 0.5, 0.7] });
    outer.material.color = [0.7, 0.5, 0.3];
    outer.material.roughness = 0.8;
    bone4.shapes.push(outer);
    let mut inner = SdfShape::new("Inner", SdfPrimitive::Ellipsoid { radii: [0.6, 0.5, 0.6] });
    inner.material.color = [0.7, 0.5, 0.3];
    inner.transform.translation = [0.0, 0.15, 0.0];
    inner.combination = CombinationOp::SmoothSubtraction { k: 0.05 };
    bone4.shapes.push(inner);
    root.children.push(bone4);

    DemoResult {
        scene: SdfScene {
            name: "Boolean Sampler".into(),
            root_bone: root,
            combination: CombinationOp::Union,
            light_dir: [0.6, 0.8, 0.4],
            settings: SceneSettings::default(),
        },
        shape_graphs: HashMap::new(),
        bone_graphs: HashMap::new(),
    }
}
