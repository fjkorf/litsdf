use std::collections::HashMap;
use egui_snarl::{NodeId, InPinId, OutPinId, Snarl};
use super::types::SdfNode;

/// Computed value at a pin.
#[derive(Debug, Clone, Copy)]
pub enum PinValue {
    Float(f32),
    Vec3([f32; 3]),
}

impl PinValue {
    pub fn as_float(self) -> f32 {
        match self {
            Self::Float(v) => v,
            Self::Vec3(v) => v[0], // take X component
        }
    }

    pub fn as_vec3(self) -> [f32; 3] {
        match self {
            Self::Float(v) => [v, v, v], // broadcast
            Self::Vec3(v) => v,
        }
    }
}

/// Result of evaluating a graph — the values at each ShapeOutput input pin.
#[derive(Debug, Default)]
pub struct ShapeOutputValues {
    pub tx: Option<f32>,
    pub ty: Option<f32>,
    pub tz: Option<f32>,
    pub rx: Option<f32>,
    pub ry: Option<f32>,
    pub rz: Option<f32>,
    pub scale: Option<f32>,
    pub red: Option<f32>,
    pub green: Option<f32>,
    pub blue: Option<f32>,
    pub roughness: Option<f32>,
    pub metallic: Option<f32>,
    pub fresnel: Option<f32>,
    pub noise_amp: Option<f32>,
    pub noise_freq: Option<f32>,
    pub noise_oct: Option<f32>,
    pub symmetry: Option<f32>,
    pub rounding: Option<f32>,
    pub onion: Option<f32>,
    pub twist: Option<f32>,
    pub bend: Option<f32>,
    pub elongate_x: Option<f32>,
    pub elongate_y: Option<f32>,
    pub elongate_z: Option<f32>,
    pub repeat_x: Option<f32>,
    pub repeat_y: Option<f32>,
    pub repeat_z: Option<f32>,
}

/// Values computed by a BoneOutput node.
#[derive(Debug, Default)]
pub struct BoneOutputValues {
    pub tx: Option<f32>,
    pub ty: Option<f32>,
    pub tz: Option<f32>,
    pub rx: Option<f32>,
    pub ry: Option<f32>,
    pub rz: Option<f32>,
    pub scale: Option<f32>,
}

/// Evaluate a bone graph and return BoneOutput values.
pub fn evaluate_bone_graph(snarl: &Snarl<SdfNode>, time: f32) -> BoneOutputValues {
    let mut cache: HashMap<OutPinId, PinValue> = HashMap::new();
    let mut result = BoneOutputValues::default();

    for (node_id, node_info) in snarl.nodes_ids_data() {
        let node = &node_info.value;
        if !matches!(node, SdfNode::BoneOutput) { continue; }

        for input_idx in 0..node.input_count() {
            let in_pin_id = InPinId { node: node_id, input: input_idx };
            let in_pin = snarl.in_pin(in_pin_id);

            if let Some(&remote) = in_pin.remotes.first() {
                let value = eval_output(snarl, remote, time, &mut cache);
                let f = value.as_float();
                match input_idx {
                    0 => result.tx = Some(f),
                    1 => result.ty = Some(f),
                    2 => result.tz = Some(f),
                    3 => result.rx = Some(f),
                    4 => result.ry = Some(f),
                    5 => result.rz = Some(f),
                    6 => result.scale = Some(f),
                    _ => {}
                }
            }
        }
    }

    result
}

/// Evaluate a node graph and return the ShapeOutput values.
pub fn evaluate_graph(snarl: &Snarl<SdfNode>, time: f32) -> ShapeOutputValues {
    let mut cache: HashMap<OutPinId, PinValue> = HashMap::new();
    let mut result = ShapeOutputValues::default();

    // Find all ShapeOutput nodes and evaluate their inputs
    for (node_id, node_info) in snarl.nodes_ids_data() {
        let node = &node_info.value;
        if !matches!(node, SdfNode::ShapeOutput) { continue; }

        for input_idx in 0..node.input_count() {
            let in_pin_id = InPinId { node: node_id, input: input_idx };
            let in_pin = snarl.in_pin(in_pin_id);

            if let Some(&remote) = in_pin.remotes.first() {
                let value = eval_output(snarl, remote, time, &mut cache);
                let f = value.as_float();
                match input_idx {
                    0 => result.tx = Some(f),
                    1 => result.ty = Some(f),
                    2 => result.tz = Some(f),
                    3 => result.rx = Some(f),
                    4 => result.ry = Some(f),
                    5 => result.rz = Some(f),
                    6 => result.scale = Some(f),
                    7 => result.red = Some(f),
                    8 => result.green = Some(f),
                    9 => result.blue = Some(f),
                    10 => result.roughness = Some(f),
                    11 => result.metallic = Some(f),
                    12 => result.fresnel = Some(f),
                    13 => result.noise_amp = Some(f),
                    14 => result.noise_freq = Some(f),
                    15 => result.noise_oct = Some(f),
                    16 => result.symmetry = Some(f),
                    17 => result.rounding = Some(f),
                    18 => result.onion = Some(f),
                    19 => result.twist = Some(f),
                    20 => result.bend = Some(f),
                    21 => result.elongate_x = Some(f),
                    22 => result.elongate_y = Some(f),
                    23 => result.elongate_z = Some(f),
                    24 => result.repeat_x = Some(f),
                    25 => result.repeat_y = Some(f),
                    26 => result.repeat_z = Some(f),
                    _ => {}
                }
            }
        }
    }

    result
}

/// Recursively evaluate an output pin, caching results.
fn eval_output(
    snarl: &Snarl<SdfNode>,
    pin: OutPinId,
    time: f32,
    cache: &mut HashMap<OutPinId, PinValue>,
) -> PinValue {
    if let Some(&cached) = cache.get(&pin) {
        return cached;
    }

    let node = &snarl[pin.node];
    let value = eval_node(snarl, pin.node, node, pin.output, time, cache);

    cache.insert(pin, value);
    value
}

/// Get the value of an input pin — either from a connected wire or from the node's default.
fn get_input(
    snarl: &Snarl<SdfNode>,
    node_id: NodeId,
    input_idx: usize,
    time: f32,
    cache: &mut HashMap<OutPinId, PinValue>,
) -> Option<PinValue> {
    let in_pin_id = InPinId { node: node_id, input: input_idx };
    let in_pin = snarl.in_pin(in_pin_id);

    if let Some(&remote) = in_pin.remotes.first() {
        Some(eval_output(snarl, remote, time, cache))
    } else {
        None
    }
}

fn get_input_float(
    snarl: &Snarl<SdfNode>,
    node_id: NodeId,
    input_idx: usize,
    default: f32,
    time: f32,
    cache: &mut HashMap<OutPinId, PinValue>,
) -> f32 {
    get_input(snarl, node_id, input_idx, time, cache)
        .map(|v| v.as_float())
        .unwrap_or(default)
}

/// Evaluate a single node's output.
fn eval_node(
    snarl: &Snarl<SdfNode>,
    node_id: NodeId,
    node: &SdfNode,
    output_idx: usize,
    time: f32,
    cache: &mut HashMap<OutPinId, PinValue>,
) -> PinValue {
    match node {
        SdfNode::Time => PinValue::Float(time),

        SdfNode::SinOscillator { amplitude, frequency, phase } => {
            let amp = get_input_float(snarl, node_id, 0, *amplitude, time, cache);
            let freq = get_input_float(snarl, node_id, 1, *frequency, time, cache);
            let ph = get_input_float(snarl, node_id, 2, *phase, time, cache);
            let t = get_input_float(snarl, node_id, 3, time, time, cache);
            let value = amp * (t * freq * std::f32::consts::TAU + ph).sin();
            PinValue::Float(value)
        }

        SdfNode::Constant { value } => PinValue::Float(*value),

        SdfNode::ConstantVec3 { value } => PinValue::Vec3(*value),

        SdfNode::Add => {
            let a = get_input_float(snarl, node_id, 0, 0.0, time, cache);
            let b = get_input_float(snarl, node_id, 1, 0.0, time, cache);
            PinValue::Float(a + b)
        }

        SdfNode::Multiply => {
            let a = get_input_float(snarl, node_id, 0, 0.0, time, cache);
            let b = get_input_float(snarl, node_id, 1, 1.0, time, cache);
            PinValue::Float(a * b)
        }

        SdfNode::Mix { factor } => {
            let a = get_input_float(snarl, node_id, 0, 0.0, time, cache);
            let b = get_input_float(snarl, node_id, 1, 0.0, time, cache);
            let f = get_input_float(snarl, node_id, 2, *factor, time, cache);
            PinValue::Float(a * (1.0 - f) + b * f)
        }

        SdfNode::Clamp { min, max } => {
            let v = get_input_float(snarl, node_id, 0, 0.0, time, cache);
            let lo = get_input_float(snarl, node_id, 1, *min, time, cache);
            let hi = get_input_float(snarl, node_id, 2, *max, time, cache);
            PinValue::Float(v.clamp(lo, hi))
        }

        SdfNode::Negate => {
            let v = get_input_float(snarl, node_id, 0, 0.0, time, cache);
            PinValue::Float(-v)
        }

        SdfNode::Vec3Compose => {
            let x = get_input_float(snarl, node_id, 0, 0.0, time, cache);
            let y = get_input_float(snarl, node_id, 1, 0.0, time, cache);
            let z = get_input_float(snarl, node_id, 2, 0.0, time, cache);
            PinValue::Vec3([x, y, z])
        }

        SdfNode::Vec3Decompose => {
            let v = get_input(snarl, node_id, 0, time, cache)
                .map(|v| v.as_vec3())
                .unwrap_or([0.0; 3]);
            match output_idx {
                0 => PinValue::Float(v[0]),
                1 => PinValue::Float(v[1]),
                2 => PinValue::Float(v[2]),
                _ => PinValue::Float(0.0),
            }
        }

        SdfNode::SquareWave { amplitude, frequency, phase } => {
            let amp = get_input_float(snarl, node_id, 0, *amplitude, time, cache);
            let freq = get_input_float(snarl, node_id, 1, *frequency, time, cache);
            let ph = get_input_float(snarl, node_id, 2, *phase, time, cache);
            let t = get_input_float(snarl, node_id, 3, time, time, cache);
            let phase_val = t * freq * std::f32::consts::TAU + ph;
            let value = amp * if phase_val.sin() >= 0.0 { 1.0 } else { -1.0 };
            PinValue::Float(value)
        }

        SdfNode::TriangleWave { amplitude, frequency, phase } => {
            let amp = get_input_float(snarl, node_id, 0, *amplitude, time, cache);
            let freq = get_input_float(snarl, node_id, 1, *frequency, time, cache);
            let ph = get_input_float(snarl, node_id, 2, *phase, time, cache);
            let t = get_input_float(snarl, node_id, 3, time, time, cache);
            let p = t * freq + ph / std::f32::consts::TAU;
            let value = amp * (2.0 * (2.0 * (p - (p + 0.5).floor())).abs() - 1.0);
            PinValue::Float(value)
        }

        SdfNode::SawtoothWave { amplitude, frequency, phase } => {
            let amp = get_input_float(snarl, node_id, 0, *amplitude, time, cache);
            let freq = get_input_float(snarl, node_id, 1, *frequency, time, cache);
            let ph = get_input_float(snarl, node_id, 2, *phase, time, cache);
            let t = get_input_float(snarl, node_id, 3, time, time, cache);
            let p = t * freq + ph / std::f32::consts::TAU;
            let value = amp * 2.0 * (p - (p + 0.5).floor());
            PinValue::Float(value)
        }

        SdfNode::EaseInOut { exponent } => {
            let v = get_input_float(snarl, node_id, 0, 0.0, time, cache);
            let exp = get_input_float(snarl, node_id, 1, *exponent, time, cache);
            let clamped = v.clamp(0.0, 1.0);
            let result = if clamped < 0.5 {
                0.5 * (2.0 * clamped).powf(exp)
            } else {
                1.0 - 0.5 * (2.0 * (1.0 - clamped)).powf(exp)
            };
            PinValue::Float(result)
        }

        SdfNode::Remap { in_min, in_max, out_min, out_max } => {
            let v = get_input_float(snarl, node_id, 0, 0.0, time, cache);
            let imin = get_input_float(snarl, node_id, 1, *in_min, time, cache);
            let imax = get_input_float(snarl, node_id, 2, *in_max, time, cache);
            let omin = get_input_float(snarl, node_id, 3, *out_min, time, cache);
            let omax = get_input_float(snarl, node_id, 4, *out_max, time, cache);
            let t_val = if (imax - imin).abs() < 1e-10 { 0.0 } else { (v - imin) / (imax - imin) };
            PinValue::Float(omin + t_val * (omax - omin))
        }

        SdfNode::Abs => {
            let v = get_input_float(snarl, node_id, 0, 0.0, time, cache);
            PinValue::Float(v.abs())
        }

        SdfNode::Modulo { divisor } => {
            let v = get_input_float(snarl, node_id, 0, 0.0, time, cache);
            let d = get_input_float(snarl, node_id, 1, *divisor, time, cache);
            let result = if d.abs() < 1e-10 { 0.0 } else { v % d };
            PinValue::Float(result)
        }

        SdfNode::CosinePalette => {
            let t_val = get_input_float(snarl, node_id, 0, 0.0, time, cache);
            let a = get_input(snarl, node_id, 1, time, cache)
                .map(|v| v.as_vec3()).unwrap_or([0.5, 0.5, 0.5]);
            let b = get_input(snarl, node_id, 2, time, cache)
                .map(|v| v.as_vec3()).unwrap_or([0.5, 0.5, 0.5]);
            let c = get_input(snarl, node_id, 3, time, cache)
                .map(|v| v.as_vec3()).unwrap_or([1.0, 1.0, 1.0]);
            let d = get_input(snarl, node_id, 4, time, cache)
                .map(|v| v.as_vec3()).unwrap_or([0.0, 0.33, 0.67]);
            let tau = std::f32::consts::TAU;
            let result = [
                a[0] + b[0] * (tau * (c[0] * t_val + d[0])).cos(),
                a[1] + b[1] * (tau * (c[1] * t_val + d[1])).cos(),
                a[2] + b[2] * (tau * (c[2] * t_val + d[2])).cos(),
            ];
            PinValue::Vec3(result)
        }

        SdfNode::ExpImpulse { k } => {
            let v = get_input_float(snarl, node_id, 0, 0.0, time, cache);
            let kk = get_input_float(snarl, node_id, 1, *k, time, cache);
            let result = if kk <= 0.0 || v <= 0.0 { 0.0 } else { kk * v * (-kk * v + 1.0).exp() };
            PinValue::Float(result)
        }

        SdfNode::SmoothStep { edge0, edge1 } => {
            let v = get_input_float(snarl, node_id, 0, 0.0, time, cache);
            let e0 = get_input_float(snarl, node_id, 1, *edge0, time, cache);
            let e1 = get_input_float(snarl, node_id, 2, *edge1, time, cache);
            let t_val = if (e1 - e0).abs() < 1e-10 { 0.0 } else { ((v - e0) / (e1 - e0)).clamp(0.0, 1.0) };
            PinValue::Float(t_val * t_val * (3.0 - 2.0 * t_val))
        }

        SdfNode::Noise1D { frequency } => {
            let t = get_input_float(snarl, node_id, 0, time, time, cache);
            let freq = get_input_float(snarl, node_id, 1, *frequency, time, cache);
            // Simple 1D hash-based noise
            let x = t * freq;
            let i = x.floor();
            let f = x - i;
            let f = f * f * (3.0 - 2.0 * f); // smoothstep
            let hash = |n: f32| -> f32 { ((n * 127.1).sin() * 43758.5453).fract() };
            let a = hash(i);
            let b = hash(i + 1.0);
            PinValue::Float(a + (b - a) * f)
        }

        SdfNode::ShapeOutput | SdfNode::BoneOutput => PinValue::Float(0.0), // sinks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_snarl::Snarl;

    #[test]
    fn constant_to_output() {
        let mut snarl = Snarl::new();
        let c = snarl.insert_node(egui::pos2(0.0, 0.0), SdfNode::Constant { value: 1.5 });
        let out = snarl.insert_node(egui::pos2(200.0, 0.0), SdfNode::ShapeOutput);
        snarl.connect(OutPinId { node: c, output: 0 }, InPinId { node: out, input: 1 }); // ty

        let result = evaluate_graph(&snarl, 0.0);
        assert!(result.tx.is_none());
        assert_eq!(result.ty, Some(1.5));
    }

    #[test]
    fn time_to_oscillator_to_output() {
        let mut snarl = Snarl::new();
        let t = snarl.insert_node(egui::pos2(0.0, 0.0), SdfNode::Time);
        let osc = snarl.insert_node(egui::pos2(200.0, 0.0), SdfNode::SinOscillator {
            amplitude: 1.0, frequency: 1.0, phase: 0.0,
        });
        let out = snarl.insert_node(egui::pos2(400.0, 0.0), SdfNode::ShapeOutput);

        // Time → Oscillator.time
        snarl.connect(OutPinId { node: t, output: 0 }, InPinId { node: osc, input: 3 });
        // Oscillator → Output.ty
        snarl.connect(OutPinId { node: osc, output: 0 }, InPinId { node: out, input: 1 });

        // At time=0, sin(0 * 1.0 * TAU + 0) = sin(0) = 0
        let result = evaluate_graph(&snarl, 0.0);
        assert!((result.ty.unwrap() - 0.0).abs() < 0.001);

        // At time=0.25, sin(0.25 * 1.0 * TAU) = sin(PI/2) = 1.0
        let result = evaluate_graph(&snarl, 0.25);
        assert!((result.ty.unwrap() - 1.0).abs() < 0.001);
    }

    #[test]
    fn math_chain() {
        let mut snarl = Snarl::new();
        let a = snarl.insert_node(egui::pos2(0.0, 0.0), SdfNode::Constant { value: 3.0 });
        let b = snarl.insert_node(egui::pos2(0.0, 100.0), SdfNode::Constant { value: 2.0 });
        let mul = snarl.insert_node(egui::pos2(200.0, 0.0), SdfNode::Multiply);
        let out = snarl.insert_node(egui::pos2(400.0, 0.0), SdfNode::ShapeOutput);

        snarl.connect(OutPinId { node: a, output: 0 }, InPinId { node: mul, input: 0 });
        snarl.connect(OutPinId { node: b, output: 0 }, InPinId { node: mul, input: 1 });
        snarl.connect(OutPinId { node: mul, output: 0 }, InPinId { node: out, input: 6 }); // scale

        let result = evaluate_graph(&snarl, 0.0);
        assert_eq!(result.scale, Some(6.0));
    }

    #[test]
    fn unconnected_returns_none() {
        let mut snarl = Snarl::new();
        snarl.insert_node(egui::pos2(0.0, 0.0), SdfNode::ShapeOutput);

        let result = evaluate_graph(&snarl, 0.0);
        assert!(result.tx.is_none());
        assert!(result.ty.is_none());
        assert!(result.scale.is_none());
    }
}
