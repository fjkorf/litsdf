use std::collections::HashMap;

use egui::{Color32, Ui};
use egui_snarl::{InPin, NodeId, OutPinId, OutPin, Snarl};
use egui_snarl::ui::{PinInfo, SnarlViewer};

use super::eval::PinValue;
use super::types::{PinType, SdfNode};

const FLOAT_COLOR: Color32 = Color32::from_rgb(0x60, 0xa0, 0xe0);
const VEC3_COLOR: Color32 = Color32::from_rgb(0xe0, 0xa0, 0x40);
const BOOL_COLOR: Color32 = Color32::from_rgb(0x60, 0xe0, 0x80);
const INT_COLOR: Color32 = Color32::from_rgb(0x60, 0xe0, 0xe0);
const OUTPUT_COLOR: Color32 = Color32::from_rgb(0x40, 0xe0, 0x60);

pub struct SdfNodeViewer {
    pub search_text: String,
    pub eval_cache: HashMap<OutPinId, PinValue>,
}

fn format_pin_value(v: &PinValue) -> String {
    match v {
        PinValue::Float(f) => format!("{f:.2}"),
        PinValue::Vec3(v) => format!("[{:.1},{:.1},{:.1}]", v[0], v[1], v[2]),
    }
}

#[allow(refining_impl_trait)]
impl SnarlViewer<SdfNode> for SdfNodeViewer {
    fn title(&mut self, node: &SdfNode) -> String {
        node.name().to_string()
    }

    fn header_frame(
        &mut self,
        default: egui::Frame,
        node: egui_snarl::NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        snarl: &Snarl<SdfNode>,
    ) -> egui::Frame {
        let color = match &snarl[node] {
            SdfNode::Time | SdfNode::Constant { .. } | SdfNode::ConstantVec3 { .. } =>
                Color32::from_rgb(0x2a, 0x6a, 0x3a), // green — generators
            SdfNode::SinOscillator { .. } | SdfNode::SquareWave { .. }
            | SdfNode::TriangleWave { .. } | SdfNode::SawtoothWave { .. } =>
                Color32::from_rgb(0x2a, 0x5a, 0x7a), // teal — oscillators
            SdfNode::Add | SdfNode::Multiply | SdfNode::Mix { .. } | SdfNode::Clamp { .. }
            | SdfNode::Negate | SdfNode::Abs | SdfNode::Modulo { .. }
            | SdfNode::EaseInOut { .. } | SdfNode::Remap { .. }
            | SdfNode::ExpImpulse { .. } | SdfNode::SmoothStep { .. } | SdfNode::Noise1D { .. } =>
                Color32::from_rgb(0x3a, 0x3a, 0x7a), // blue — math
            SdfNode::Vec3Compose | SdfNode::Vec3Decompose | SdfNode::CosinePalette =>
                Color32::from_rgb(0x6a, 0x4a, 0x2a), // amber — vec3/color
            SdfNode::Compare { .. } | SdfNode::Gate | SdfNode::BoolMath { .. } | SdfNode::StateVar { .. }
            | SdfNode::Expression { .. } =>
                Color32::from_rgb(0x6a, 0x2a, 0x6a), // purple — logic
            SdfNode::IsColliding | SdfNode::ContactNormal | SdfNode::RaycastDown
            | SdfNode::BoneVelocity | SdfNode::BoneAngularVelocity
            | SdfNode::BoneWorldPosition | SdfNode::BoneSpeed =>
                Color32::from_rgb(0x8a, 0x5a, 0x1a), // amber — physics/sensing
            SdfNode::ShapeOutput | SdfNode::BoneOutput =>
                Color32::from_rgb(0x7a, 0x3a, 0x2a), // red-orange — output
        };
        default.fill(color)
    }

    fn inputs(&mut self, node: &SdfNode) -> usize {
        node.input_count()
    }

    fn outputs(&mut self, node: &SdfNode) -> usize {
        node.output_count()
    }

    fn show_input(&mut self, pin: &InPin, ui: &mut Ui, snarl: &mut Snarl<SdfNode>) -> PinInfo {
        let node = &snarl[pin.id.node];
        let label = node.input_label(pin.id.input);
        let pin_type = node.input_type(pin.id.input);

        if pin.remotes.is_empty() {
            // No connection — show editable default value
            match &mut snarl[pin.id.node] {
                SdfNode::SinOscillator { amplitude, frequency, phase, .. } => {
                    match pin.id.input {
                        0 => { ui.add(egui::DragValue::new(amplitude).speed(0.01).prefix("Amp: ")); }
                        1 => { ui.add(egui::DragValue::new(frequency).speed(0.01).prefix("Freq: ")); }
                        2 => { ui.add(egui::DragValue::new(phase).speed(0.01).prefix("Phase: ")); }
                        3 => { ui.label("Time"); }
                        _ => { ui.label(label); }
                    }
                }
                SdfNode::Constant { value } if matches!(pin.id.input, 0) => {
                    // Constants have no inputs, but just in case
                    ui.add(egui::DragValue::new(value).speed(0.01));
                }
                SdfNode::SquareWave { amplitude, frequency, phase, .. }
                | SdfNode::TriangleWave { amplitude, frequency, phase, .. }
                | SdfNode::SawtoothWave { amplitude, frequency, phase, .. } => {
                    match pin.id.input {
                        0 => { ui.add(egui::DragValue::new(amplitude).speed(0.01).prefix("Amp: ")); }
                        1 => { ui.add(egui::DragValue::new(frequency).speed(0.01).prefix("Freq: ")); }
                        2 => { ui.add(egui::DragValue::new(phase).speed(0.01).prefix("Phase: ")); }
                        3 => { ui.label("Time"); }
                        _ => { ui.label(label); }
                    }
                }
                SdfNode::EaseInOut { exponent } if pin.id.input == 1 => {
                    ui.add(egui::DragValue::new(exponent).speed(0.01).prefix("Exp: "));
                }
                SdfNode::Remap { in_min, in_max, out_min, out_max, .. } => {
                    match pin.id.input {
                        0 => { ui.label(label); }
                        1 => { ui.add(egui::DragValue::new(in_min).speed(0.01).prefix("In Min: ")); }
                        2 => { ui.add(egui::DragValue::new(in_max).speed(0.01).prefix("In Max: ")); }
                        3 => { ui.add(egui::DragValue::new(out_min).speed(0.01).prefix("Out Min: ")); }
                        4 => { ui.add(egui::DragValue::new(out_max).speed(0.01).prefix("Out Max: ")); }
                        _ => { ui.label(label); }
                    }
                }
                SdfNode::Modulo { divisor, .. } if pin.id.input == 1 => {
                    ui.add(egui::DragValue::new(divisor).speed(0.01).prefix("Div: "));
                }
                SdfNode::ExpImpulse { k } if pin.id.input == 1 => {
                    ui.add(egui::DragValue::new(k).speed(0.1).prefix("K: "));
                }
                SdfNode::SmoothStep { edge0, edge1 } => {
                    match pin.id.input {
                        0 => { ui.label(label); }
                        1 => { ui.add(egui::DragValue::new(edge0).speed(0.01).prefix("E0: ")); }
                        2 => { ui.add(egui::DragValue::new(edge1).speed(0.01).prefix("E1: ")); }
                        _ => { ui.label(label); }
                    }
                }
                SdfNode::Noise1D { frequency } if pin.id.input == 1 => {
                    ui.add(egui::DragValue::new(frequency).speed(0.01).prefix("Freq: "));
                }
                SdfNode::Mix { factor, .. } if pin.id.input == 2 => {
                    ui.add(egui::DragValue::new(factor).speed(0.01).range(0.0..=1.0).prefix("Mix: "));
                }
                SdfNode::Clamp { min, max, .. } => {
                    match pin.id.input {
                        0 => { ui.label(label); }
                        1 => { ui.add(egui::DragValue::new(min).speed(0.01).prefix("Min: ")); }
                        2 => { ui.add(egui::DragValue::new(max).speed(0.01).prefix("Max: ")); }
                        _ => { ui.label(label); }
                    }
                }
                SdfNode::ShapeOutput => {
                    ui.label(label);
                }
                _ => {
                    ui.label(label);
                }
            }
        } else {
            // Connected — show label + current value preview
            ui.horizontal(|ui| {
                ui.label(label);
                if let Some(&remote) = pin.remotes.first() {
                    if let Some(val) = self.eval_cache.get(&remote) {
                        ui.weak(format_pin_value(val));
                    }
                }
            });
        }

        let color = match pin_type {
            PinType::Float => FLOAT_COLOR,
            PinType::Vec3 => VEC3_COLOR,
            PinType::Bool => BOOL_COLOR,
            PinType::Int => INT_COLOR,
        };
        let shape = if pin_type == PinType::Bool {
            PinInfo::triangle().with_fill(color)
        } else {
            PinInfo::circle().with_fill(color)
        };
        shape
    }

    fn show_output(&mut self, pin: &OutPin, ui: &mut Ui, snarl: &mut Snarl<SdfNode>) -> PinInfo {
        let node = &snarl[pin.id.node];
        let label = node.output_label(pin.id.output);
        let pin_type = node.output_type(pin.id.output);

        // Show editable value for source nodes
        match &mut snarl[pin.id.node] {
            SdfNode::Constant { value } => {
                ui.add(egui::DragValue::new(value).speed(0.01));
            }
            SdfNode::ConstantVec3 { value } => {
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut value[0]).speed(0.01).prefix("x:"));
                    ui.add(egui::DragValue::new(&mut value[1]).speed(0.01).prefix("y:"));
                    ui.add(egui::DragValue::new(&mut value[2]).speed(0.01).prefix("z:"));
                });
            }
            _ => {
                ui.label(label);
            }
        }

        let color = match pin_type {
            PinType::Float => FLOAT_COLOR,
            PinType::Vec3 => VEC3_COLOR,
            PinType::Bool => BOOL_COLOR,
            PinType::Int => INT_COLOR,
        };

        if matches!(snarl[pin.id.node], SdfNode::ShapeOutput) {
            PinInfo::circle().with_fill(OUTPUT_COLOR)
        } else if pin_type == PinType::Bool {
            PinInfo::triangle().with_fill(color)
        } else {
            PinInfo::circle().with_fill(color)
        }
    }

    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<SdfNode>) {
        let from_type = snarl[from.id.node].output_type(from.id.output);
        let to_type = snarl[to.id.node].input_type(to.id.input);

        // Type check with auto-coercion (Bool↔Float, Int↔Float)
        if !PinType::compatible(from_type, to_type) {
            return;
        }

        // Disconnect existing connections to this input (single connection per input)
        for &remote in &to.remotes {
            snarl.disconnect(remote, to.id);
        }

        snarl.connect(from.id, to.id);
    }

    fn has_body(&mut self, node: &SdfNode) -> bool {
        matches!(node, SdfNode::Compare { .. } | SdfNode::BoolMath { .. } | SdfNode::StateVar { .. } | SdfNode::Expression { .. })
    }

    fn show_body(&mut self, node: NodeId, _inputs: &[InPin], _outputs: &[OutPin], ui: &mut Ui, snarl: &mut Snarl<SdfNode>) {
        match &mut snarl[node] {
            SdfNode::Compare { mode } => {
                let labels = ["A > B", "A < B", "A == B", "A >= B", "A <= B"];
                let selected = labels.get(*mode as usize).unwrap_or(&"?");
                egui::ComboBox::from_id_salt(format!("cmp_{node:?}"))
                    .selected_text(*selected)
                    .width(80.0)
                    .show_ui(ui, |ui| {
                        for (i, label) in labels.iter().enumerate() {
                            ui.selectable_value(mode, i as u32, *label);
                        }
                    });
            }
            SdfNode::BoolMath { op } => {
                let labels = ["AND", "OR", "NOT"];
                let selected = labels.get(*op as usize).unwrap_or(&"?");
                egui::ComboBox::from_id_salt(format!("bool_{node:?}"))
                    .selected_text(*selected)
                    .width(60.0)
                    .show_ui(ui, |ui| {
                        for (i, label) in labels.iter().enumerate() {
                            ui.selectable_value(op, i as u32, *label);
                        }
                    });
            }
            SdfNode::StateVar { index } => {
                ui.add(egui::DragValue::new(index).range(0..=99).prefix("Var #"));
            }
            SdfNode::Expression { text, var_count } => {
                let mut edited = text.clone();
                if ui.text_edit_singleline(&mut edited).changed() {
                    // Re-parse to count variables
                    if let Ok(parsed) = crate::nodes::expression::parse(&edited) {
                        *var_count = parsed.variables.len() as u32;
                    }
                    *text = edited;
                }
            }
            _ => {}
        }
    }

    fn has_on_hover_popup(&mut self, _node: &SdfNode) -> bool { true }

    fn show_on_hover_popup(&mut self, node: NodeId, _inputs: &[InPin], _outputs: &[OutPin], ui: &mut Ui, snarl: &mut Snarl<SdfNode>) {
        ui.label(snarl[node].description());
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<SdfNode>) -> bool {
        true
    }

    fn show_graph_menu(
        &mut self,
        pos: egui::Pos2,
        ui: &mut Ui,
        snarl: &mut Snarl<SdfNode>,
    ) {
        ui.text_edit_singleline(&mut self.search_text);

        // When searching, show flat filtered list
        if !self.search_text.is_empty() {
            let filter = self.search_text.to_lowercase();
            let entries: &[(&str, fn() -> SdfNode)] = &[
                ("Time", || SdfNode::Time),
                ("Sin Oscillator", || SdfNode::SinOscillator { amplitude: 1.0, frequency: 1.0, phase: 0.0 }),
                ("Square Wave", || SdfNode::SquareWave { amplitude: 1.0, frequency: 1.0, phase: 0.0 }),
                ("Triangle Wave", || SdfNode::TriangleWave { amplitude: 1.0, frequency: 1.0, phase: 0.0 }),
                ("Sawtooth Wave", || SdfNode::SawtoothWave { amplitude: 1.0, frequency: 1.0, phase: 0.0 }),
                ("Constant", || SdfNode::Constant { value: 0.0 }),
                ("Constant Vec3", || SdfNode::ConstantVec3 { value: [0.0; 3] }),
                ("Add", || SdfNode::Add),
                ("Multiply", || SdfNode::Multiply),
                ("Mix", || SdfNode::Mix { factor: 0.5 }),
                ("Clamp", || SdfNode::Clamp { min: 0.0, max: 1.0 }),
                ("Negate", || SdfNode::Negate),
                ("Abs", || SdfNode::Abs),
                ("Modulo", || SdfNode::Modulo { divisor: 1.0 }),
                ("Ease In/Out", || SdfNode::EaseInOut { exponent: 2.0 }),
                ("Remap", || SdfNode::Remap { in_min: 0.0, in_max: 1.0, out_min: 0.0, out_max: 1.0 }),
                ("Exp Impulse", || SdfNode::ExpImpulse { k: 4.0 }),
                ("Smooth Step", || SdfNode::SmoothStep { edge0: 0.0, edge1: 1.0 }),
                ("Noise 1D", || SdfNode::Noise1D { frequency: 1.0 }),
                ("Vec3 Compose", || SdfNode::Vec3Compose),
                ("Vec3 Decompose", || SdfNode::Vec3Decompose),
                ("Cosine Palette", || SdfNode::CosinePalette),
                ("Compare", || SdfNode::Compare { mode: 0 }),
                ("Gate", || SdfNode::Gate),
                ("Bool Math", || SdfNode::BoolMath { op: 0 }),
                ("State Variable", || SdfNode::StateVar { index: 0 }),
                ("Is Colliding", || SdfNode::IsColliding),
                ("Contact Normal", || SdfNode::ContactNormal),
                ("Raycast Down", || SdfNode::RaycastDown),
                ("Bone Velocity", || SdfNode::BoneVelocity),
                ("Bone Angular Vel", || SdfNode::BoneAngularVelocity),
                ("Bone World Pos", || SdfNode::BoneWorldPosition),
                ("Bone Speed", || SdfNode::BoneSpeed),
                ("Expression", || SdfNode::Expression { text: String::new(), var_count: 0 }),
                ("Shape Output", || SdfNode::ShapeOutput),
                ("Bone Output", || SdfNode::BoneOutput),
            ];
            for (name, ctor) in entries {
                if name.to_lowercase().contains(&filter) {
                    if ui.button(*name).clicked() {
                        snarl.insert_node(pos, ctor());
                        self.search_text.clear();
                        ui.close();
                    }
                }
            }
            return;
        }

        // Default: categorized menu
        ui.separator();

        if ui.button("Time").clicked() {
            snarl.insert_node(pos, SdfNode::Time);
            ui.close();
        }
        ui.menu_button("Oscillators", |ui| {
            if ui.button("Sin").clicked() {
                snarl.insert_node(pos, SdfNode::SinOscillator {
                    amplitude: 1.0, frequency: 1.0, phase: 0.0,
                });
                ui.close();
            }
            if ui.button("Square").clicked() {
                snarl.insert_node(pos, SdfNode::SquareWave {
                    amplitude: 1.0, frequency: 1.0, phase: 0.0,
                });
                ui.close();
            }
            if ui.button("Triangle").clicked() {
                snarl.insert_node(pos, SdfNode::TriangleWave {
                    amplitude: 1.0, frequency: 1.0, phase: 0.0,
                });
                ui.close();
            }
            if ui.button("Sawtooth").clicked() {
                snarl.insert_node(pos, SdfNode::SawtoothWave {
                    amplitude: 1.0, frequency: 1.0, phase: 0.0,
                });
                ui.close();
            }
        });
        if ui.button("Constant").clicked() {
            snarl.insert_node(pos, SdfNode::Constant { value: 0.0 });
            ui.close();
        }
        if ui.button("Constant Vec3").clicked() {
            snarl.insert_node(pos, SdfNode::ConstantVec3 { value: [0.0; 3] });
            ui.close();
        }

        ui.separator();

        if ui.button("Add").clicked() {
            snarl.insert_node(pos, SdfNode::Add);
            ui.close();
        }
        if ui.button("Multiply").clicked() {
            snarl.insert_node(pos, SdfNode::Multiply);
            ui.close();
        }
        if ui.button("Mix").clicked() {
            snarl.insert_node(pos, SdfNode::Mix { factor: 0.5 });
            ui.close();
        }
        if ui.button("Clamp").clicked() {
            snarl.insert_node(pos, SdfNode::Clamp { min: 0.0, max: 1.0 });
            ui.close();
        }
        if ui.button("Negate").clicked() {
            snarl.insert_node(pos, SdfNode::Negate);
            ui.close();
        }
        if ui.button("Abs").clicked() {
            snarl.insert_node(pos, SdfNode::Abs);
            ui.close();
        }
        if ui.button("Modulo").clicked() {
            snarl.insert_node(pos, SdfNode::Modulo { divisor: 1.0 });
            ui.close();
        }
        if ui.button("Ease In/Out").clicked() {
            snarl.insert_node(pos, SdfNode::EaseInOut { exponent: 2.0 });
            ui.close();
        }
        if ui.button("Remap").clicked() {
            snarl.insert_node(pos, SdfNode::Remap { in_min: 0.0, in_max: 1.0, out_min: 0.0, out_max: 1.0 });
            ui.close();
        }
        if ui.button("Exp Impulse").clicked() {
            snarl.insert_node(pos, SdfNode::ExpImpulse { k: 4.0 });
            ui.close();
        }
        if ui.button("Smooth Step").clicked() {
            snarl.insert_node(pos, SdfNode::SmoothStep { edge0: 0.0, edge1: 1.0 });
            ui.close();
        }
        if ui.button("Noise 1D").clicked() {
            snarl.insert_node(pos, SdfNode::Noise1D { frequency: 1.0 });
            ui.close();
        }

        ui.separator();

        if ui.button("Vec3 Compose").clicked() {
            snarl.insert_node(pos, SdfNode::Vec3Compose);
            ui.close();
        }
        if ui.button("Vec3 Decompose").clicked() {
            snarl.insert_node(pos, SdfNode::Vec3Decompose);
            ui.close();
        }
        if ui.button("Cosine Palette").clicked() {
            snarl.insert_node(pos, SdfNode::CosinePalette);
            ui.close();
        }

        ui.separator();

        ui.menu_button("Logic", |ui| {
            if ui.button("Compare").clicked() {
                snarl.insert_node(pos, SdfNode::Compare { mode: 0 });
                ui.close();
            }
            if ui.button("Gate").clicked() {
                snarl.insert_node(pos, SdfNode::Gate);
                ui.close();
            }
            if ui.button("Bool Math").clicked() {
                snarl.insert_node(pos, SdfNode::BoolMath { op: 0 });
                ui.close();
            }
            if ui.button("State Variable").clicked() {
                snarl.insert_node(pos, SdfNode::StateVar { index: 0 });
                ui.close();
            }
        });
        ui.menu_button("Sensing", |ui| {
            if ui.button("Is Colliding").clicked() {
                snarl.insert_node(pos, SdfNode::IsColliding);
                ui.close();
            }
            if ui.button("Contact Normal").clicked() {
                snarl.insert_node(pos, SdfNode::ContactNormal);
                ui.close();
            }
            if ui.button("Raycast Down").clicked() {
                snarl.insert_node(pos, SdfNode::RaycastDown);
                ui.close();
            }
            if ui.button("Bone Velocity").clicked() {
                snarl.insert_node(pos, SdfNode::BoneVelocity);
                ui.close();
            }
            if ui.button("Bone Angular Vel").clicked() {
                snarl.insert_node(pos, SdfNode::BoneAngularVelocity);
                ui.close();
            }
            if ui.button("Bone World Pos").clicked() {
                snarl.insert_node(pos, SdfNode::BoneWorldPosition);
                ui.close();
            }
            if ui.button("Bone Speed").clicked() {
                snarl.insert_node(pos, SdfNode::BoneSpeed);
                ui.close();
            }
        });

        if ui.button("Expression").clicked() {
            snarl.insert_node(pos, SdfNode::Expression { text: String::new(), var_count: 0 });
            ui.close();
        }

        ui.separator();

        if ui.button("Shape Output").clicked() {
            snarl.insert_node(pos, SdfNode::ShapeOutput);
            ui.close();
        }
        if ui.button("Bone Output").clicked() {
            snarl.insert_node(pos, SdfNode::BoneOutput);
            ui.close();
        }
    }

    fn has_node_menu(&mut self, _node: &SdfNode) -> bool {
        true
    }

    fn show_node_menu(
        &mut self,
        node: egui_snarl::NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<SdfNode>,
    ) {
        if ui.button("Delete").clicked() {
            snarl.remove_node(node);
            ui.close();
        }
    }
}
