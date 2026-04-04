use std::collections::HashMap;
use litsdf_core::models::*;
use super::DemoResult;

pub fn create() -> DemoResult {
    let mut root = SdfBone::root();

    // Ground with noise displacement and gradient snow
    let mut ground = SdfShape::new("Ground", SdfPrimitive::RoundBox {
        half_extents: [3.0, 0.3, 2.5], rounding: 0.2,
    });
    ground.transform.translation = [0.0, -0.5, 0.0];
    ground.material.color = [0.35, 0.45, 0.25];
    ground.material.roughness = 0.85;
    ground.material.noise_amplitude = 0.04;
    ground.material.noise_frequency = 3.0;
    ground.material.noise_octaves = 2;
    ground.material.color_mode = 5; // Gradient Snow
    root.shapes.push(ground);

    // Mushroom 1: Cosine Palette cap
    let mut m1 = SdfBone::new("Mushroom 1");
    m1.transform.translation = [-1.0, 0.0, 0.0];
    let mut stem1 = SdfShape::new("Stem", SdfPrimitive::Cylinder { height: 0.4, radius: 0.12 });
    stem1.material.color = [0.85, 0.8, 0.7];
    stem1.material.roughness = 0.7;
    stem1.material.color_mode = 2; // Noise Tint
    stem1.material.noise_frequency = 6.0;
    m1.shapes.push(stem1);
    let mut cap1 = SdfShape::new("Cap", SdfPrimitive::Ellipsoid { radii: [0.45, 0.25, 0.45] });
    cap1.transform.translation = [0.0, 0.5, 0.0];
    cap1.material.color_mode = 1; // Cosine Palette
    cap1.material.palette_a = [0.5, 0.3, 0.2];
    cap1.material.palette_b = [0.5, 0.3, 0.3];
    cap1.material.palette_c = [1.0, 1.0, 1.0];
    cap1.material.palette_d = [0.0, 0.1, 0.2];
    cap1.material.roughness = 0.4;
    cap1.combination = CombinationOp::SmoothUnion { k: 0.15 };
    m1.shapes.push(cap1);
    root.children.push(m1);

    // Mushroom 2: Cellular cap
    let mut m2 = SdfBone::new("Mushroom 2");
    m2.transform.translation = [1.0, 0.0, 0.5];
    let mut stem2 = SdfShape::new("Stem", SdfPrimitive::Cylinder { height: 0.3, radius: 0.1 });
    stem2.material.color = [0.8, 0.75, 0.65];
    stem2.material.roughness = 0.7;
    m2.shapes.push(stem2);
    let mut cap2 = SdfShape::new("Cap", SdfPrimitive::Ellipsoid { radii: [0.35, 0.2, 0.35] });
    cap2.transform.translation = [0.0, 0.4, 0.0];
    cap2.material.color = [0.9, 0.4, 0.3];
    cap2.material.color_mode = 3; // Cellular
    cap2.material.noise_frequency = 5.0;
    cap2.material.roughness = 0.5;
    cap2.combination = CombinationOp::SmoothUnion { k: 0.12 };
    m2.shapes.push(cap2);
    root.children.push(m2);

    // Stone with ridged noise
    let mut stone = SdfShape::new("Stone", SdfPrimitive::RoundBox {
        half_extents: [0.3, 0.25, 0.35], rounding: 0.15,
    });
    stone.transform.translation = [2.0, -0.2, -0.5];
    stone.material.color = [0.5, 0.45, 0.4];
    stone.material.roughness = 0.9;
    stone.material.color_mode = 4; // Ridged
    stone.material.noise_frequency = 4.0;
    stone.material.noise_octaves = 3;
    stone.material.noise_amplitude = 0.03;
    root.shapes.push(stone);

    // Moss blob with smooth symmetry
    let mut moss = SdfShape::new("Moss", SdfPrimitive::Ellipsoid { radii: [0.25, 0.15, 0.2] });
    moss.transform.translation = [-2.0, -0.3, 0.8];
    moss.material.color = [0.2, 0.5, 0.15];
    moss.material.roughness = 0.8;
    moss.material.smooth_symmetry = 0.02;
    moss.material.noise_amplitude = 0.02;
    moss.material.noise_frequency = 5.0;
    moss.material.noise_octaves = 2;
    root.shapes.push(moss);

    DemoResult {
        scene: SdfScene {
            name: "Mushroom Garden".into(),
            description: "Stylized mushrooms using Cosine Palette, Cellular, Ridged, and Gradient Snow color modes with noise displacement.".into(),
            root_bone: root,
            combination: CombinationOp::Union,
            light_dir: [0.5, 0.9, 0.3],
            settings: SceneSettings::default(),
        },
        shape_graphs: HashMap::new(),
        bone_graphs: HashMap::new(),
    }
}
