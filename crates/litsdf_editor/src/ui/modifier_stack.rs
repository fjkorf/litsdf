use litsdf_core::models::ShapeModifier;
use litsdf_render::scene_sync::SdfSceneState;

const MODIFIER_TYPES: &[&str] = &[
    "Rounding", "Shell", "Twist", "Bend", "Elongation", "Repetition",
];

/// Render the modifier stack as pure egui widgets.
/// Returns true if any modifier was changed (caller should set scene.dirty).
pub fn render_modifier_stack(ui: &mut egui::Ui, scene: &mut SdfSceneState) -> bool {
    let Some(shape_id) = scene.selected_shape else { return false };
    let Some((shape, _)) = scene.scene.root_bone.find_shape_mut(shape_id) else { return false };

    let mut changed = false;

    ui.horizontal(|ui| {
        ui.heading("Modifiers");
        ui.menu_button("Add ▾", |ui| {
            for name in MODIFIER_TYPES {
                // Only show types not already present
                let present = shape.modifiers.iter().any(|m| modifier_name(m) == *name);
                if !present {
                    if ui.button(*name).clicked() {
                        shape.modifiers.push(default_modifier(name));
                        changed = true;
                        ui.close();
                    }
                }
            }
            if shape.modifiers.len() == MODIFIER_TYPES.len() {
                ui.label("All modifiers added");
            }
        });
    });

    if shape.modifiers.is_empty() {
        ui.label("No modifiers. Use Add to apply domain transforms.");
        return changed;
    }

    let mut to_remove: Option<usize> = None;

    for (i, modifier) in shape.modifiers.iter_mut().enumerate() {
        let id = ui.make_persistent_id(format!("mod_{i}"));
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
            .show_header(ui, |ui| {
                ui.label(modifier_name(modifier));
                if ui.small_button("✕").clicked() {
                    to_remove = Some(i);
                }
            })
            .body(|ui| {
                if render_modifier_params(ui, modifier) {
                    changed = true;
                }
            });
    }

    if let Some(i) = to_remove {
        shape.modifiers.remove(i);
        changed = true;
    }

    if shape.modifiers.len() > 1 {
        if ui.small_button("Clear All").clicked() {
            shape.modifiers.clear();
            changed = true;
        }
    }

    changed
}

fn modifier_name(m: &ShapeModifier) -> &'static str {
    match m {
        ShapeModifier::Rounding(_) => "Rounding",
        ShapeModifier::Onion(_) => "Shell",
        ShapeModifier::Twist(_) => "Twist",
        ShapeModifier::Bend(_) => "Bend",
        ShapeModifier::Elongation(_) => "Elongation",
        ShapeModifier::Repetition { .. } => "Repetition",
    }
}

fn default_modifier(name: &str) -> ShapeModifier {
    match name {
        "Rounding" => ShapeModifier::Rounding(0.1),
        "Shell" => ShapeModifier::Onion(0.05),
        "Twist" => ShapeModifier::Twist(1.0),
        "Bend" => ShapeModifier::Bend(1.0),
        "Elongation" => ShapeModifier::Elongation([0.5, 0.0, 0.0]),
        "Repetition" => ShapeModifier::Repetition { period: [2.0, 2.0, 2.0], count: [3, 3, 3] },
        _ => ShapeModifier::Rounding(0.1),
    }
}

/// Render parameter sliders for a single modifier. Returns true if value changed.
fn render_modifier_params(ui: &mut egui::Ui, modifier: &mut ShapeModifier) -> bool {
    let mut changed = false;
    match modifier {
        ShapeModifier::Rounding(v) => {
            changed |= ui.add(egui::Slider::new(v, 0.0..=1.0).text("Edge radius").fixed_decimals(2)).changed();
        }
        ShapeModifier::Onion(v) => {
            changed |= ui.add(egui::Slider::new(v, 0.0..=0.5).text("Wall thickness").fixed_decimals(3)).changed();
        }
        ShapeModifier::Twist(v) => {
            changed |= ui.add(egui::Slider::new(v, -5.0..=5.0).text("Y-axis warp").fixed_decimals(1)).changed();
        }
        ShapeModifier::Bend(v) => {
            changed |= ui.add(egui::Slider::new(v, -5.0..=5.0).text("X-axis warp").fixed_decimals(1)).changed();
        }
        ShapeModifier::Elongation(v) => {
            changed |= ui.add(egui::Slider::new(&mut v[0], 0.0..=2.0).text("X").fixed_decimals(2)).changed();
            changed |= ui.add(egui::Slider::new(&mut v[1], 0.0..=2.0).text("Y").fixed_decimals(2)).changed();
            changed |= ui.add(egui::Slider::new(&mut v[2], 0.0..=2.0).text("Z").fixed_decimals(2)).changed();
        }
        ShapeModifier::Repetition { period, count } => {
            ui.label("Period:");
            changed |= ui.add(egui::Slider::new(&mut period[0], 0.1..=5.0).text("X").fixed_decimals(1)).changed();
            changed |= ui.add(egui::Slider::new(&mut period[1], 0.1..=5.0).text("Y").fixed_decimals(1)).changed();
            changed |= ui.add(egui::Slider::new(&mut period[2], 0.1..=5.0).text("Z").fixed_decimals(1)).changed();
            ui.label("Count:");
            let mut cx = count[0] as i32;
            let mut cy = count[1] as i32;
            let mut cz = count[2] as i32;
            changed |= ui.add(egui::Slider::new(&mut cx, 1..=10).text("X").integer()).changed();
            changed |= ui.add(egui::Slider::new(&mut cy, 1..=10).text("Y").integer()).changed();
            changed |= ui.add(egui::Slider::new(&mut cz, 1..=10).text("Z").integer()).changed();
            count[0] = cx as u32;
            count[1] = cy as u32;
            count[2] = cz as u32;
        }
    }
    changed
}
