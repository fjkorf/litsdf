use litsdf_core::models::{BoneId, SdfPrimitive, SdfShape};
use litsdf_render::scene_sync::SdfSceneState;

use super::EditorUi;
use super::helpers::PRIM_NAMES;

pub fn handle_confirm_add(ui: &mut EditorUi, scene: &mut SdfSceneState) {
    if ui.md.state.on_confirm_add_count > ui.prev_on_confirm_add {
        let type_name = PRIM_NAMES[ui.md.state.new_shape_type];
        let prim = SdfPrimitive::default_for(type_name);
        let shape = SdfShape::new(type_name, prim);
        let new_id = shape.id;
        let target_bone = scene.selected_bone.unwrap_or(BoneId::root());
        if let Some(bone) = scene.scene.root_bone.find_bone_mut(target_bone) {
            bone.shapes.push(shape);
        }
        scene.selected_shape = Some(new_id);
        scene.dirty = true;
        ui.md.state.show_add_shape = false;
    }
    ui.prev_on_confirm_add = ui.md.state.on_confirm_add_count;
}

pub fn handle_delete_shape(ui: &mut EditorUi, scene: &mut SdfSceneState) {
    if ui.md.state.on_delete_shape_count > ui.prev_on_delete_shape {
        if let Some(sel_id) = scene.selected_shape {
            scene.scene.root_bone.remove_shape(sel_id);
            scene.selected_shape = None;
            scene.dirty = true;
        }
    }
    ui.prev_on_delete_shape = ui.md.state.on_delete_shape_count;
}

pub fn handle_edit_yaml(ui: &mut EditorUi, scene: &SdfSceneState) {
    if ui.md.state.on_edit_yaml_count > ui.prev_on_edit_yaml {
        if let Some(sel_id) = scene.selected_shape {
            if let Some((shape, _)) = scene.scene.root_bone.find_shape(sel_id) {
                ui.md.state.yaml_text = serde_yaml::to_string(shape).unwrap_or_default();
                ui.md.state.show_yaml_editor = true;
            }
        }
    }
    ui.prev_on_edit_yaml = ui.md.state.on_edit_yaml_count;
}

pub fn handle_apply_yaml(ui: &mut EditorUi, scene: &mut SdfSceneState) {
    if ui.md.state.on_apply_yaml_count > ui.prev_on_apply_yaml {
        if let Some(sel_id) = scene.selected_shape {
            match serde_yaml::from_str::<SdfShape>(&ui.md.state.yaml_text) {
                Ok(mut new_shape) => {
                    new_shape.id = sel_id; // preserve ID
                    if let Some((shape, _)) = scene.scene.root_bone.find_shape_mut(sel_id) {
                        *shape = new_shape;
                        scene.dirty = true;
                        ui.md.state.show_yaml_editor = false;
                    }
                }
                Err(e) => {
                    eprintln!("YAML parse error: {e}");
                }
            }
        }
    }
    ui.prev_on_apply_yaml = ui.md.state.on_apply_yaml_count;
}

pub fn handle_shape_selection(ui: &mut EditorUi, scene: &mut SdfSceneState) -> bool {
    let mut changed = false;
    for (i, row) in ui.md.state.bone_shapes.iter().enumerate() {
        if let Some(&id) = ui.shape_order.get(i) {
            if row.on_select_shape_count > ui.prev_shape_clicks.get(&id).copied().unwrap_or(0) {
                scene.selected_shape = Some(id);
                changed = true;
            }
        }
    }
    for (i, row) in ui.md.state.bone_shapes.iter().enumerate() {
        if let Some(&id) = ui.shape_order.get(i) {
            ui.prev_shape_clicks.insert(id, row.on_select_shape_count);
        }
    }
    changed
}

pub fn handle_reset_transform(ui: &mut EditorUi, scene: &mut SdfSceneState) {
    if ui.md.state.on_reset_transform_count > ui.prev_on_reset_transform {
        if let Some(sel_id) = scene.selected_shape {
            if let Some((shape, _)) = scene.scene.root_bone.find_shape_mut(sel_id) {
                shape.reset_transform();
                scene.dirty = true;
            }
        }
        // Force re-populate by clearing prev selection
        ui.prev_selected_shape = None;
    }
    ui.prev_on_reset_transform = ui.md.state.on_reset_transform_count;
}

pub fn handle_save_load(_ui: &mut EditorUi) {
    // Save/Load now triggered via File menu shortcuts (Cmd+S, Cmd+O)
}

pub fn handle_file_browser(ui: &mut EditorUi, scene: &mut SdfSceneState) {
    if ui.md.state.on_confirm_save_count > ui.prev_on_confirm_save {
        let filename = if ui.md.state.filename.is_empty() {
            "untitled.yaml".to_string()
        } else if !ui.md.state.filename.ends_with(".yaml") {
            format!("{}.yaml", ui.md.state.filename)
        } else {
            ui.md.state.filename.clone()
        };
        let path = litsdf_core::persistence::scenes_dir().join(&filename);
        if let Err(e) = crate::project::save_project(&scene.scene, &ui.node_graphs, &ui.bone_graphs, &path) {
            eprintln!("Save error: {e}");
        }
        ui.md.state.show_file_browser = false;
    }
    ui.prev_on_confirm_save = ui.md.state.on_confirm_save_count;

    for (i, row) in ui.md.state.file_rows.iter().enumerate() {
        let prev = ui.prev_pick_file_counts.get(i).copied().unwrap_or(0);
        if row.on_pick_file_count > prev {
            let path = litsdf_core::persistence::scenes_dir().join(&row.name);
            match crate::project::load_project(&path) {
                Ok(project) => {
                    scene.scene = project.scene;
                    ui.node_graphs = project.shape_graphs;
                    ui.bone_graphs = project.bone_graphs;
                    ui.show_description = !scene.scene.description.is_empty();
                    scene.selected_shape = None;
                    scene.selected_bone = None;
                    scene.dirty = true;
                    ui.md.state.show_file_browser = false;
                }
                Err(e) => eprintln!("Load error: {e}"),
            }
        }
    }
    for (i, row) in ui.md.state.file_rows.iter().enumerate() {
        if i < ui.prev_pick_file_counts.len() {
            ui.prev_pick_file_counts[i] = row.on_pick_file_count;
        }
    }
}
