use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

use litsdf_core::models::BoneId;
use litsdf_render::scene_sync::SdfSceneState;

#[derive(Resource)]
pub struct ScreenshotConfig {
    pub path: String,
    pub capture_frame: u32,
    pub exit_frame: u32,
}

pub fn auto_screenshot(
    mut commands: Commands,
    config: Res<ScreenshotConfig>,
    mut frame: Local<u32>,
) {
    *frame += 1;
    if *frame == config.capture_frame {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(config.path.clone()));
    }
    if *frame == config.exit_frame {
        std::process::exit(0);
    }
}

#[derive(Resource)]
pub struct TestSequence {
    pub dir: String,
    pub frame: u32,
    pub step: u32,
}

pub fn test_sequence_system(
    mut commands: Commands,
    mut seq: ResMut<TestSequence>,
    mut scene: ResMut<SdfSceneState>,
) {
    seq.frame += 1;
    let step_frame = 10 + seq.step * 8;

    if seq.frame == step_frame {
        match seq.step {
            0 => eprintln!("[test] step 0: initial state"),
            1 => {
                scene.selected_bone = Some(BoneId::root());
                scene.selected_shape = None;
                eprintln!("[test] step 1: Root bone selected");
            }
            2 => {
                let arm_id = scene.scene.root_bone.children.first().map(|b| b.id);
                scene.selected_bone = arm_id;
                scene.selected_shape = None;
                eprintln!("[test] step 2: Arm bone selected");
            }
            3 => {
                let arm1_id = scene.scene.root_bone.children.first()
                    .and_then(|arm| arm.children.first())
                    .map(|b| b.id);
                if let Some(id) = arm1_id {
                    scene.selected_bone = Some(id);
                    scene.selected_shape = None;
                }
                eprintln!("[test] step 3: deepest bone selected");
            }
            4 => {
                // Select a shape on the current bone
                if let Some(bone_id) = scene.selected_bone {
                    if let Some(bone) = scene.scene.root_bone.find_bone(bone_id) {
                        if let Some(shape) = bone.shapes.first() {
                            scene.selected_shape = Some(shape.id);
                        }
                    }
                }
                eprintln!("[test] step 4: shape selected");
            }
            _ => {
                eprintln!("[test] done — {} steps completed", seq.step);
                std::process::exit(0);
            }
        }
    }

    if seq.frame == step_frame + 2 {
        let path = format!("{}/step_{}.png", seq.dir, seq.step);
        eprintln!("[test] capturing {path}");
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
        seq.step += 1;
    }
}

/// Render a sequence of frames to numbered PNGs for video assembly.
/// Controlled by LITSDF_RENDER_SEQUENCE env var: "output_dir,total_frames,fps"
#[derive(Resource)]
pub struct RenderSequence {
    pub dir: String,
    pub total_frames: u32,
    pub fps: f32,
    pub current_frame: u32,
    pub frames_per_capture: u32, // render frames between captures (for settling)
    pub internal_frame: u32,
}

pub fn render_sequence_system(
    mut commands: Commands,
    mut seq: ResMut<RenderSequence>,
    mut scene: ResMut<SdfSceneState>,
) {
    seq.internal_frame += 1;

    // Wait a few frames at start for shader compilation
    if seq.internal_frame < 10 { return; }

    // Capture every N internal frames
    let capture_interval = seq.frames_per_capture.max(1);
    if (seq.internal_frame - 10) % capture_interval == 0 {
        if seq.current_frame < seq.total_frames {
            let path = format!("{}/frame_{:04}.png", seq.dir, seq.current_frame);
            eprintln!("[render] frame {}/{} → {}", seq.current_frame + 1, seq.total_frames, path);
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path));
            scene.dirty = true; // ensure shader updates with new time
            seq.current_frame += 1;
        } else {
            eprintln!("[render] sequence complete: {} frames in {}", seq.total_frames, seq.dir);
            std::process::exit(0);
        }
    }
}
