pub mod camera;
pub mod codegen;
pub mod gizmos;
pub mod picking;
pub mod scene_sync;
pub mod shader;

use bevy::prelude::*;

pub struct SdfRenderPlugin;

impl Plugin for SdfRenderPlugin {
    fn build(&self, app: &mut App) {
        // Ensure the runtime shader exists before Bevy tries to load it
        codegen::ensure_runtime_shader();

        app.add_plugins(MaterialPlugin::<shader::SdfMaterial>::default())
            .init_resource::<scene_sync::SdfSceneState>()
            .init_resource::<picking::ClickTracker>()
            .init_resource::<picking::DragState>()
            .init_resource::<picking::GizmoMode>()
            .add_systems(Startup, (camera::setup_camera, scene_sync::setup_initial_scene))
            .add_systems(Update, (
                camera::orbit_camera,
                scene_sync::sync_scene_to_shader,
                gizmos::draw_bone_gizmos,
                picking::pick_system,
                picking::drag_system,
                picking::draw_handles,
            ));
    }
}
