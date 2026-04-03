use litsdf_core::models::{BoneId, ShapeId, SdfBone};
use super::helpers::PRIM_NAMES;

/// Drag-and-drop payload for reparenting.
#[derive(Clone)]
pub enum DragPayload {
    Shape(ShapeId),
    Bone(BoneId),
}

/// Selection action returned by the tree renderer.
pub enum TreeAction {
    None,
    SelectBone(BoneId),
    SelectShape(ShapeId, BoneId),
}

/// Identifies an item being renamed inline.
#[derive(Clone, PartialEq)]
pub enum RenameTarget {
    Bone(BoneId),
    Shape(ShapeId),
}

/// Context menu action returned by the tree renderer.
pub enum ContextAction {
    None,
    AddChildBone(BoneId),
    AddShapeToBone(BoneId, String),
    DuplicateBone(BoneId),
    DuplicateShape(ShapeId),
    DeleteBone(BoneId),
    DeleteBoneRecursive(BoneId),
    DeleteShape(ShapeId),
    ToggleBoneVisibility(BoneId),
    ToggleShapeVisibility(ShapeId),
    ReparentBone { bone: BoneId, new_parent: BoneId },
    ReparentShape { shape: ShapeId, new_bone: BoneId },
    RenameBone(BoneId, String),
    RenameShape(ShapeId, String),
}

pub struct TreeResult {
    pub action: TreeAction,
    pub context: ContextAction,
}

/// Flat list of (BoneId, name) for reparent submenus.
fn collect_bone_list(bone: &SdfBone, out: &mut Vec<(BoneId, String)>) {
    out.push((bone.id, bone.name.clone()));
    for child in &bone.children {
        collect_bone_list(child, out);
    }
}

/// Renders the bone tree recursively using egui CollapsingHeader.
pub fn render_bone_tree(
    ui: &mut egui::Ui,
    bone: &SdfBone,
    selected_bone: Option<BoneId>,
    selected_shape: Option<ShapeId>,
    rename_state: &mut Option<(RenameTarget, String)>,
) -> TreeResult {
    let mut result = TreeResult {
        action: TreeAction::None,
        context: ContextAction::None,
    };
    let mut bone_list = Vec::new();
    collect_bone_list(bone, &mut bone_list);
    render_bone_recursive(ui, bone, selected_bone, selected_shape, &bone_list, rename_state, &mut result);
    result
}

fn render_bone_recursive(
    ui: &mut egui::Ui,
    bone: &SdfBone,
    selected_bone: Option<BoneId>,
    selected_shape: Option<ShapeId>,
    bone_list: &[(BoneId, String)],
    rename_state: &mut Option<(RenameTarget, String)>,
    result: &mut TreeResult,
) {
    let bone_selected = selected_bone == Some(bone.id);
    let bone_label = if bone_selected {
        format!("▸ {}", bone.name)
    } else {
        bone.name.clone()
    };

    let id = ui.make_persistent_id(bone.id.0);
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
        .show_header(ui, |ui| {
            let vis_text = if bone.visible { "👁" } else { "  " };
            if ui.small_button(vis_text).clicked() {
                result.context = ContextAction::ToggleBoneVisibility(bone.id);
            }

            // Inline rename or normal label
            let is_renaming = matches!(rename_state, Some((RenameTarget::Bone(id), _)) if *id == bone.id);
            let response = if is_renaming {
                let text = &mut rename_state.as_mut().unwrap().1;
                let r = ui.text_edit_singleline(text);
                if r.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let new_name = text.clone();
                    result.context = ContextAction::RenameBone(bone.id, new_name);
                    *rename_state = None;
                }
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    *rename_state = None;
                }
                r
            } else {
                let r = ui.selectable_label(bone_selected, &bone_label);
                if r.double_clicked() && !bone.id.is_root() {
                    *rename_state = Some((RenameTarget::Bone(bone.id), bone.name.clone()));
                }
                if r.clicked() {
                    result.action = TreeAction::SelectBone(bone.id);
                }
                r
            };

            // Drag source: non-root bones can be dragged
            if !bone.id.is_root() && response.dragged() {
                response.dnd_set_drag_payload(DragPayload::Bone(bone.id));
            }

            // Drop target: accept bone or shape drops
            if response.dnd_hover_payload::<DragPayload>().is_some() {
                // Visual feedback while hovering
                let stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(0x40, 0xa0, 0xff));
                ui.painter().rect_stroke(response.rect, egui::CornerRadius::same(2), stroke, egui::StrokeKind::Outside);
            }
            if let Some(payload) = response.dnd_release_payload::<DragPayload>() {
                match payload.as_ref() {
                    DragPayload::Shape(shape_id) => {
                        result.context = ContextAction::ReparentShape { shape: *shape_id, new_bone: bone.id };
                    }
                    DragPayload::Bone(dragged_id) => {
                        if *dragged_id != bone.id {
                            result.context = ContextAction::ReparentBone { bone: *dragged_id, new_parent: bone.id };
                        }
                    }
                }
            }

            response.context_menu(|ui| {
                if ui.button("Add Child Bone").clicked() {
                    result.context = ContextAction::AddChildBone(bone.id);
                    ui.close();
                }
                ui.menu_button("Add Shape", |ui| {
                    for name in PRIM_NAMES {
                        if ui.button(*name).clicked() {
                            result.context = ContextAction::AddShapeToBone(bone.id, name.to_string());
                            ui.close();
                        }
                    }
                });
                if !bone.id.is_root() {
                    ui.separator();
                    if ui.button("Duplicate").clicked() {
                        result.context = ContextAction::DuplicateBone(bone.id);
                        ui.close();
                    }
                    // Reparent submenu — list all bones except self and descendants
                    ui.menu_button("Reparent to", |ui| {
                        for (target_id, target_name) in bone_list {
                            if *target_id == bone.id { continue; }
                            // Skip descendants (simple check: if bone contains target, skip)
                            if bone.find_bone(*target_id).is_some() { continue; }
                            if ui.button(target_name).clicked() {
                                result.context = ContextAction::ReparentBone {
                                    bone: bone.id,
                                    new_parent: *target_id,
                                };
                                ui.close();
                            }
                        }
                    });
                    ui.separator();
                    if ui.button("Delete").clicked() {
                        result.context = ContextAction::DeleteBone(bone.id);
                        ui.close();
                    }
                    if ui.button("Delete with Contents").clicked() {
                        result.context = ContextAction::DeleteBoneRecursive(bone.id);
                        ui.close();
                    }
                }
            });
        })
        .body(|ui| {
            for shape in &bone.shapes {
                let shape_selected = selected_shape == Some(shape.id);
                let shape_label = if shape_selected {
                    format!("  ▸ {}", shape.name)
                } else {
                    format!("  {}", shape.name)
                };

                ui.horizontal(|ui| {
                    let vis_text = if shape.visible { "👁" } else { "  " };
                    if ui.small_button(vis_text).clicked() {
                        result.context = ContextAction::ToggleShapeVisibility(shape.id);
                    }

                    let is_shape_renaming = matches!(rename_state, Some((RenameTarget::Shape(id), _)) if *id == shape.id);
                    let response = if is_shape_renaming {
                        let text = &mut rename_state.as_mut().unwrap().1;
                        let r = ui.text_edit_singleline(text);
                        if r.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            let new_name = text.clone();
                            result.context = ContextAction::RenameShape(shape.id, new_name);
                            *rename_state = None;
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            *rename_state = None;
                        }
                        r
                    } else {
                        let r = ui.selectable_label(shape_selected, &shape_label);
                        if r.double_clicked() {
                            *rename_state = Some((RenameTarget::Shape(shape.id), shape.name.clone()));
                        }
                        if r.clicked() {
                            result.action = TreeAction::SelectShape(shape.id, bone.id);
                        }
                        r
                    };

                    // Drag source: shapes can be dragged to other bones
                    if response.dragged() {
                        response.dnd_set_drag_payload(DragPayload::Shape(shape.id));
                    }

                    response.context_menu(|ui| {
                        if ui.button("Duplicate").clicked() {
                            result.context = ContextAction::DuplicateShape(shape.id);
                            ui.close();
                        }
                        // Move to Bone submenu
                        ui.menu_button("Move to Bone", |ui| {
                            for (target_id, target_name) in bone_list {
                                if *target_id == bone.id { continue; } // already on this bone
                                if ui.button(target_name).clicked() {
                                    result.context = ContextAction::ReparentShape {
                                        shape: shape.id,
                                        new_bone: *target_id,
                                    };
                                    ui.close();
                                }
                            }
                        });
                        ui.separator();
                        if ui.button("Delete").clicked() {
                            result.context = ContextAction::DeleteShape(shape.id);
                            ui.close();
                        }
                    });
                });
            }

            for child in &bone.children {
                render_bone_recursive(ui, child, selected_bone, selected_shape, bone_list, rename_state, result);
            }
        });
}
