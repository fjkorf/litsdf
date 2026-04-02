use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use litsdf_editor::testing::{self, ScreenshotConfig, TestSequence};
use litsdf_render::SdfRenderPlugin;
use litsdf_editor::SdfEditorPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "litsdf".into(),
            resolution: (1280, 720).into(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(EguiPlugin::default())
    .add_plugins(SdfRenderPlugin)
    .add_plugins(SdfEditorPlugin);

    // LITSDF_SCENE=path.yaml — load a scene file at startup
    if let Ok(path) = std::env::var("LITSDF_SCENE") {
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

    // LITSDF_SCREENSHOT=path.png — single screenshot and exit
    if let Ok(path) = std::env::var("LITSDF_SCREENSHOT") {
        app.insert_resource(ScreenshotConfig {
            path,
            capture_frame: 30,
            exit_frame: 35,
        })
        .add_systems(Update, testing::auto_screenshot);
    }

    // LITSDF_RENDER_SEQUENCE=dir,frames,fps — render animation to numbered PNGs
    if let Ok(spec) = std::env::var("LITSDF_RENDER_SEQUENCE") {
        let parts: Vec<&str> = spec.split(',').collect();
        let dir = parts.first().unwrap_or(&"frames").to_string();
        let total_frames = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(60);
        let fps = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(30.0);
        std::fs::create_dir_all(&dir).ok();
        app.insert_resource(testing::RenderSequence {
            dir,
            total_frames,
            fps,
            current_frame: 0,
            frames_per_capture: 2,
            internal_frame: 0,
        })
        .add_systems(Update, testing::render_sequence_system);
    }

    // LITSDF_TEST_SEQUENCE=dir — multi-step test with screenshots
    if let Ok(dir) = std::env::var("LITSDF_TEST_SEQUENCE") {
        app.insert_resource(TestSequence {
            dir,
            frame: 0,
            step: 0,
        })
        .add_systems(Update, testing::test_sequence_system);
    }

    app.run();
}
