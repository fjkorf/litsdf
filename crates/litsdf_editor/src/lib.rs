pub mod nodes;
pub mod project;
pub mod ui;
pub mod undo;
pub mod testing;

use bevy::prelude::*;

pub struct SdfEditorPlugin;

impl Plugin for SdfEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ui::EditorUi>()
            .init_resource::<undo::UndoHistory>()
            .add_systems(bevy_egui::EguiPrimaryContextPass, (
                ui::editor_ui,
                litsdf_render::gizmos::draw_compass,
            ));
    }
}
