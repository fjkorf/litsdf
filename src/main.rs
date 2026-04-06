use std::collections::HashMap;
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
    .add_plugins(litsdf_render::avian_physics::SdfPhysicsPlugin)
    .add_plugins(SdfEditorPlugin);

    // LITSDF_DEMO=name — load a demo scene at startup
    if let Ok(demo_name) = std::env::var("LITSDF_DEMO") {
        let demo = match demo_name.to_lowercase().as_str() {
            "gallery" | "primitive" => litsdf_editor::demos::DemoScene::PrimitiveGallery,
            "boolean" => litsdf_editor::demos::DemoScene::BooleanSampler,
            "modifier" => litsdf_editor::demos::DemoScene::ModifierParade,
            "mushroom" => litsdf_editor::demos::DemoScene::MushroomGarden,
            "robot" => litsdf_editor::demos::DemoScene::RobotFriend,
            "sculpture" | "abstract" => litsdf_editor::demos::DemoScene::AbstractSculpture,
            "chain" | "hanging" => litsdf_editor::demos::DemoScene::HangingChain,
            "pendulum" => litsdf_editor::demos::DemoScene::Pendulum,
            "damping" => litsdf_editor::demos::DemoScene::DampingLab,
            "speed" | "glow" => litsdf_editor::demos::DemoScene::SpeedGlow,
            "wave" | "force" => litsdf_editor::demos::DemoScene::WaveForce,
            "walker" => litsdf_editor::demos::DemoScene::Walker,
            "lemmings" => litsdf_editor::demos::DemoScene::Lemmings,
            _ => {
                eprintln!("Unknown demo: {demo_name}. Options: gallery, boolean, modifier, mushroom, robot, sculpture, chain, pendulum, damping, speed, wave");
                litsdf_editor::demos::DemoScene::PrimitiveGallery
            }
        };
        let result = litsdf_editor::demos::load_demo(demo);
        let has_physics = litsdf_core::models::SdfBone::has_physics_bones(&result.scene.root_bone);
        app.insert_resource(litsdf_editor::demos::PendingGraphs {
            shape_graphs: result.shape_graphs,
            bone_graphs: result.bone_graphs,
        });
        app.insert_resource(litsdf_render::scene_sync::SdfSceneState {
            scene: result.scene,
            selected_shape: None,
            selected_bone: None,
            show_bone_gizmos: false,
            dirty: true, topology_hash: 0, use_avian: true,
            physics_readings: HashMap::new(), force_outputs: HashMap::new(),
            physics_paused: !has_physics,
        });
        // Note: demo node graphs are loaded by the editor on first frame via the demo menu mechanism
    }

    // LITSDF_SCENE=path.yaml — load a scene file at startup
    if let Ok(path) = std::env::var("LITSDF_SCENE") {
        match litsdf_core::persistence::load_scene(std::path::Path::new(&path)) {
            Ok(scene) => {
                app.insert_resource(litsdf_render::scene_sync::SdfSceneState {
                    scene,
                    selected_shape: None,
                    selected_bone: None,
                    show_bone_gizmos: false,
                    dirty: true, topology_hash: 0, use_avian: true, physics_readings: HashMap::new(), force_outputs: HashMap::new(), physics_paused: true,
                });
            }
            Err(e) => eprintln!("Failed to load scene: {e}"),
        }
    }

    // LITSDF_SCREENSHOT=path.png — single screenshot and exit
    // LITSDF_SCREENSHOT_FRAME=N — capture at frame N (default 30)
    if let Ok(path) = std::env::var("LITSDF_SCREENSHOT") {
        let frame = std::env::var("LITSDF_SCREENSHOT_FRAME")
            .ok().and_then(|s| s.parse().ok()).unwrap_or(30u32);
        app.insert_resource(ScreenshotConfig {
            path,
            capture_frame: frame,
            exit_frame: frame + 5,
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
