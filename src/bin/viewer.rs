use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

use litsdf_render::SdfRenderPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "litsdf viewer".into(),
            resolution: (1280, 720).into(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(SdfRenderPlugin);

    // Load scene from CLI arg or env var
    let scene_path = std::env::args().nth(1)
        .or_else(|| std::env::var("LITSDF_SCENE").ok());

    if let Some(path) = scene_path {
        match litsdf_core::persistence::load_scene(std::path::Path::new(&path)) {
            Ok(scene) => {
                app.insert_resource(litsdf_render::scene_sync::SdfSceneState {
                    scene,
                    selected_shape: None,
                    selected_bone: None,
                    show_bone_gizmos: false,
                    dirty: true, topology_hash: 0,
                });
            }
            Err(e) => eprintln!("Failed to load scene: {e}"),
        }
    }

    // Optional screenshot
    if let Ok(path) = std::env::var("LITSDF_SCREENSHOT") {
        app.insert_resource(ViewerScreenshot { path, frame: 0 })
            .add_systems(Update, viewer_screenshot);
    }

    app.run();
}

#[derive(Resource)]
struct ViewerScreenshot {
    path: String,
    frame: u32,
}

fn viewer_screenshot(mut commands: Commands, mut config: ResMut<ViewerScreenshot>) {
    config.frame += 1;
    if config.frame == 30 {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(config.path.clone()));
    }
    if config.frame == 35 {
        std::process::exit(0);
    }
}
