use egui_snarl::{InPinId, OutPinId, Snarl};
use super::types::SdfNode;

/// Create a starter graph: Time → SinOscillator(amp=0.3, freq=0.5) → ShapeOutput.ty
pub fn create_starter_graph() -> Snarl<SdfNode> {
    bob_preset(0.3, 0.5)
}

/// Vertical bobbing on Y axis.
pub fn bob_preset(amplitude: f32, frequency: f32) -> Snarl<SdfNode> {
    let mut s = Snarl::new();
    let t = s.insert_node(egui::pos2(50.0, 150.0), SdfNode::Time);
    let osc = s.insert_node(egui::pos2(250.0, 100.0), SdfNode::SinOscillator {
        amplitude, frequency, phase: 0.0,
    });
    let out = s.insert_node(egui::pos2(500.0, 200.0), SdfNode::ShapeOutput);
    s.connect(OutPinId { node: t, output: 0 }, InPinId { node: osc, input: 3 });
    s.connect(OutPinId { node: osc, output: 0 }, InPinId { node: out, input: 1 }); // ty
    s
}

/// Y-axis rotation.
pub fn spin_preset(degrees_per_second: f32) -> Snarl<SdfNode> {
    let mut s = Snarl::new();
    let t = s.insert_node(egui::pos2(50.0, 150.0), SdfNode::Time);
    let speed = s.insert_node(egui::pos2(50.0, 250.0), SdfNode::Constant { value: degrees_per_second });
    let mul = s.insert_node(egui::pos2(250.0, 200.0), SdfNode::Multiply);
    let out = s.insert_node(egui::pos2(500.0, 200.0), SdfNode::ShapeOutput);
    s.connect(OutPinId { node: t, output: 0 }, InPinId { node: mul, input: 0 });
    s.connect(OutPinId { node: speed, output: 0 }, InPinId { node: mul, input: 1 });
    s.connect(OutPinId { node: mul, output: 0 }, InPinId { node: out, input: 4 }); // ry
    s
}

/// Breathing scale animation (oscillates around 1.0).
pub fn pulse_preset(amplitude: f32, frequency: f32) -> Snarl<SdfNode> {
    let mut s = Snarl::new();
    let t = s.insert_node(egui::pos2(50.0, 150.0), SdfNode::Time);
    let osc = s.insert_node(egui::pos2(250.0, 100.0), SdfNode::SinOscillator {
        amplitude, frequency, phase: 0.0,
    });
    let base = s.insert_node(egui::pos2(250.0, 250.0), SdfNode::Constant { value: 1.0 });
    let add = s.insert_node(egui::pos2(450.0, 150.0), SdfNode::Add);
    let out = s.insert_node(egui::pos2(650.0, 200.0), SdfNode::ShapeOutput);
    s.connect(OutPinId { node: t, output: 0 }, InPinId { node: osc, input: 3 });
    s.connect(OutPinId { node: osc, output: 0 }, InPinId { node: add, input: 0 });
    s.connect(OutPinId { node: base, output: 0 }, InPinId { node: add, input: 1 });
    s.connect(OutPinId { node: add, output: 0 }, InPinId { node: out, input: 6 }); // scale
    s
}

/// Circular orbit in XZ plane.
pub fn orbit_preset(radius: f32, speed: f32) -> Snarl<SdfNode> {
    let mut s = Snarl::new();
    let t = s.insert_node(egui::pos2(50.0, 200.0), SdfNode::Time);
    let osc_x = s.insert_node(egui::pos2(250.0, 100.0), SdfNode::SinOscillator {
        amplitude: radius, frequency: speed, phase: 0.0,
    });
    let osc_z = s.insert_node(egui::pos2(250.0, 300.0), SdfNode::SinOscillator {
        amplitude: radius, frequency: speed, phase: std::f32::consts::FRAC_PI_2,
    });
    let out = s.insert_node(egui::pos2(500.0, 200.0), SdfNode::ShapeOutput);
    s.connect(OutPinId { node: t, output: 0 }, InPinId { node: osc_x, input: 3 });
    s.connect(OutPinId { node: t, output: 0 }, InPinId { node: osc_z, input: 3 });
    s.connect(OutPinId { node: osc_x, output: 0 }, InPinId { node: out, input: 0 }); // tx
    s.connect(OutPinId { node: osc_z, output: 0 }, InPinId { node: out, input: 2 }); // tz
    s
}

/// RGB color cycling with phase-offset oscillators.
pub fn color_cycle_preset(speed: f32) -> Snarl<SdfNode> {
    let pi3 = std::f32::consts::TAU / 3.0;
    let mut s = Snarl::new();
    let t = s.insert_node(egui::pos2(50.0, 200.0), SdfNode::Time);
    let osc_r = s.insert_node(egui::pos2(250.0, 50.0), SdfNode::SinOscillator {
        amplitude: 0.5, frequency: speed, phase: 0.0,
    });
    let osc_g = s.insert_node(egui::pos2(250.0, 200.0), SdfNode::SinOscillator {
        amplitude: 0.5, frequency: speed, phase: pi3,
    });
    let osc_b = s.insert_node(egui::pos2(250.0, 350.0), SdfNode::SinOscillator {
        amplitude: 0.5, frequency: speed, phase: pi3 * 2.0,
    });
    let base = s.insert_node(egui::pos2(450.0, 50.0), SdfNode::Constant { value: 0.5 });
    let add_r = s.insert_node(egui::pos2(450.0, 100.0), SdfNode::Add);
    let add_g = s.insert_node(egui::pos2(450.0, 250.0), SdfNode::Add);
    let add_b = s.insert_node(egui::pos2(450.0, 400.0), SdfNode::Add);
    let out = s.insert_node(egui::pos2(650.0, 200.0), SdfNode::ShapeOutput);
    // Time → all oscillators
    s.connect(OutPinId { node: t, output: 0 }, InPinId { node: osc_r, input: 3 });
    s.connect(OutPinId { node: t, output: 0 }, InPinId { node: osc_g, input: 3 });
    s.connect(OutPinId { node: t, output: 0 }, InPinId { node: osc_b, input: 3 });
    // Oscillators + 0.5 offset → colors in [0, 1]
    s.connect(OutPinId { node: osc_r, output: 0 }, InPinId { node: add_r, input: 0 });
    s.connect(OutPinId { node: base, output: 0 }, InPinId { node: add_r, input: 1 });
    s.connect(OutPinId { node: osc_g, output: 0 }, InPinId { node: add_g, input: 0 });
    s.connect(OutPinId { node: base, output: 0 }, InPinId { node: add_g, input: 1 });
    s.connect(OutPinId { node: osc_b, output: 0 }, InPinId { node: add_b, input: 0 });
    s.connect(OutPinId { node: base, output: 0 }, InPinId { node: add_b, input: 1 });
    // Adds → ShapeOutput color pins
    s.connect(OutPinId { node: add_r, output: 0 }, InPinId { node: out, input: 7 });
    s.connect(OutPinId { node: add_g, output: 0 }, InPinId { node: out, input: 8 });
    s.connect(OutPinId { node: add_b, output: 0 }, InPinId { node: out, input: 9 });
    s
}

/// Bone bob preset (same as shape bob but uses BoneOutput).
pub fn bone_bob_preset(amplitude: f32, frequency: f32) -> Snarl<SdfNode> {
    let mut s = Snarl::new();
    let t = s.insert_node(egui::pos2(50.0, 150.0), SdfNode::Time);
    let osc = s.insert_node(egui::pos2(250.0, 100.0), SdfNode::SinOscillator {
        amplitude, frequency, phase: 0.0,
    });
    let out = s.insert_node(egui::pos2(500.0, 200.0), SdfNode::BoneOutput);
    s.connect(OutPinId { node: t, output: 0 }, InPinId { node: osc, input: 3 });
    s.connect(OutPinId { node: osc, output: 0 }, InPinId { node: out, input: 1 }); // ty
    s
}

/// Bone spin preset (Y rotation).
pub fn bone_spin_preset(degrees_per_second: f32) -> Snarl<SdfNode> {
    let mut s = Snarl::new();
    let t = s.insert_node(egui::pos2(50.0, 150.0), SdfNode::Time);
    let speed = s.insert_node(egui::pos2(50.0, 250.0), SdfNode::Constant { value: degrees_per_second });
    let mul = s.insert_node(egui::pos2(250.0, 200.0), SdfNode::Multiply);
    let out = s.insert_node(egui::pos2(500.0, 200.0), SdfNode::BoneOutput);
    s.connect(OutPinId { node: t, output: 0 }, InPinId { node: mul, input: 0 });
    s.connect(OutPinId { node: speed, output: 0 }, InPinId { node: mul, input: 1 });
    s.connect(OutPinId { node: mul, output: 0 }, InPinId { node: out, input: 4 }); // ry
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::evaluate_graph;

    #[test]
    fn presets_produce_expected_values() {
        // Bob at t=0.25: amp * sin(0.25 * freq * TAU) = 0.3 * sin(0.25 * 0.5 * TAU) = 0.3 * sin(PI/4) ≈ 0.212
        let bob = bob_preset(0.3, 0.5);
        let result = evaluate_graph(&bob, 0.25);
        assert!(result.ty.is_some());
        let ty = result.ty.unwrap();
        assert!((ty - 0.3 * (0.25_f32 * 0.5 * std::f32::consts::TAU).sin()).abs() < 0.001);

        // Pulse at t=0: scale should be 1.0 (base) + 0.0 (sin(0)) = 1.0
        let pulse = pulse_preset(0.1, 1.0);
        let result = evaluate_graph(&pulse, 0.0);
        assert!((result.scale.unwrap() - 1.0).abs() < 0.001);

        // Spin: ry = time * speed
        let spin = spin_preset(90.0);
        let result = evaluate_graph(&spin, 2.0);
        assert!((result.ry.unwrap() - 180.0).abs() < 0.001);
    }
}
