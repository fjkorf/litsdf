use std::collections::HashMap;
use litsdf_core::models::*;
use super::DemoResult;

pub fn create() -> DemoResult {
    let mut root = SdfBone::root();

    let base_color = [0.5, 0.6, 0.8];
    let spacing = 2.0;

    // Plain
    let mut s = SdfShape::new("Plain", SdfPrimitive::HexPrism { height: 0.6, radius: 0.3 });
    s.material.color = base_color;
    s.transform.translation = [-5.0, 0.0, 0.0];
    root.shapes.push(s);

    // Rounded
    let mut s = SdfShape::new("Rounded", SdfPrimitive::HexPrism { height: 0.6, radius: 0.3 });
    s.material.color = [0.6, 0.8, 0.5];
    s.transform.translation = [-5.0 + spacing, 0.0, 0.0];
    s.modifiers.push(ShapeModifier::Rounding(0.1));
    root.shapes.push(s);

    // Hollow (Onion)
    let mut s = SdfShape::new("Hollow", SdfPrimitive::Sphere { radius: 0.5 });
    s.material.color = [0.8, 0.6, 0.5];
    s.transform.translation = [-5.0 + spacing * 2.0, 0.0, 0.0];
    s.modifiers.push(ShapeModifier::Onion(0.05));
    root.shapes.push(s);

    // Twisted
    let mut s = SdfShape::new("Twisted", SdfPrimitive::HexPrism { height: 0.8, radius: 0.25 });
    s.material.color = [0.8, 0.5, 0.8];
    s.transform.translation = [-5.0 + spacing * 3.0, 0.0, 0.0];
    s.modifiers.push(ShapeModifier::Twist(3.0));
    root.shapes.push(s);

    // Bent
    let mut s = SdfShape::new("Bent", SdfPrimitive::HexPrism { height: 0.8, radius: 0.25 });
    s.material.color = [0.5, 0.8, 0.8];
    s.transform.translation = [-5.0 + spacing * 4.0, 0.0, 0.0];
    s.modifiers.push(ShapeModifier::Bend(2.0));
    root.shapes.push(s);

    // Elongated
    let mut s = SdfShape::new("Elongated", SdfPrimitive::Sphere { radius: 0.3 });
    s.material.color = [0.8, 0.8, 0.5];
    s.transform.translation = [-5.0 + spacing * 5.0, 0.0, 0.0];
    s.modifiers.push(ShapeModifier::Elongation([0.5, 0.0, 0.0]));
    root.shapes.push(s);

    // Repeated
    let mut s = SdfShape::new("Repeated", SdfPrimitive::Sphere { radius: 0.2 });
    s.material.color = [0.7, 0.5, 0.5];
    s.transform.translation = [0.0, 0.0, 3.0];
    s.modifiers.push(ShapeModifier::Repetition { period: [1.5, 1.5, 1.5], count: [3, 3, 3] });
    root.shapes.push(s);

    DemoResult {
        scene: SdfScene {
            name: "Modifier Parade".into(),
            description: "Each shape has a different domain modifier applied: Rounding, Shell, Twist, Bend, Elongation, and Repetition.".into(),
            root_bone: root,
            combination: CombinationOp::Union,
            light_dir: [0.6, 0.8, 0.4],
            settings: SceneSettings::default(),
        },
        shape_graphs: HashMap::new(),
        bone_graphs: HashMap::new(),
    }
}
