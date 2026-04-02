use bevy::prelude::*;

use litsdf_core::models::SdfScene;
use litsdf_render::scene_sync::SdfSceneState;

const MAX_UNDO: usize = 50;

#[derive(Resource)]
pub struct UndoHistory {
    undo_stack: Vec<SdfScene>,
    redo_stack: Vec<SdfScene>,
}

impl Default for UndoHistory {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}

impl UndoHistory {
    pub fn push(&mut self, scene: SdfScene) {
        self.redo_stack.clear();
        self.undo_stack.push(scene);
        if self.undo_stack.len() > MAX_UNDO {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self, current: &SdfScene) -> Option<SdfScene> {
        let prev = self.undo_stack.pop()?;
        self.redo_stack.push(current.clone());
        Some(prev)
    }

    pub fn redo(&mut self, current: &SdfScene) -> Option<SdfScene> {
        let next = self.redo_stack.pop()?;
        self.undo_stack.push(current.clone());
        Some(next)
    }

    pub fn undo_len(&self) -> usize { self.undo_stack.len() }
    pub fn redo_len(&self) -> usize { self.redo_stack.len() }
}

pub fn undo_redo_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut scene: ResMut<SdfSceneState>,
    mut history: ResMut<UndoHistory>,
) {
    let cmd = keys.pressed(KeyCode::SuperLeft) || keys.pressed(KeyCode::SuperRight)
        || keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    if cmd && keys.just_pressed(KeyCode::KeyZ) {
        if shift {
            if let Some(next) = history.redo(&scene.scene) {
                scene.scene = next;
                scene.selected_shape = None;
                scene.selected_bone = None;
                scene.dirty = true;
            }
        } else {
            if let Some(prev) = history.undo(&scene.scene) {
                scene.scene = prev;
                scene.selected_shape = None;
                scene.selected_bone = None;
                scene.dirty = true;
            }
        }
    }
}

/// Call this before a mutation to snapshot the current scene.
/// Returns the cloned scene to pass to UndoHistory::push after mutation.
pub fn snapshot_before_mutation(scene: &SdfScene) -> SdfScene {
    scene.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use litsdf_core::models::{CombinationOp, SceneSettings, SdfBone, SdfPrimitive, SdfShape};

    fn make_scene(n_shapes: usize) -> SdfScene {
        let mut root = SdfBone::root();
        for i in 0..n_shapes {
            root.shapes.push(SdfShape::new(format!("Shape{i}"), SdfPrimitive::Sphere { radius: 1.0 }));
        }
        SdfScene { name: "test".into(), root_bone: root, combination: CombinationOp::Union, light_dir: [0.6, 0.8, 0.4], settings: SceneSettings::default() }
    }

    #[test]
    fn undo_single_mutation() {
        let mut history = UndoHistory::default();
        let scene1 = make_scene(1);
        let scene2 = make_scene(2);

        history.push(scene1.clone()); // snapshot before adding shape2
        // scene is now scene2
        let restored = history.undo(&scene2).unwrap();
        assert_eq!(restored.root_bone.shapes.len(), 1);
    }

    #[test]
    fn undo_redo_cycle() {
        let mut history = UndoHistory::default();
        let scene1 = make_scene(1);
        let scene2 = make_scene(2);

        history.push(scene1.clone());
        let restored = history.undo(&scene2).unwrap();
        assert_eq!(restored.root_bone.shapes.len(), 1);

        let redone = history.redo(&restored).unwrap();
        assert_eq!(redone.root_bone.shapes.len(), 2);
    }

    #[test]
    fn undo_stack_limit() {
        let mut history = UndoHistory::default();
        for i in 0..60 {
            history.push(make_scene(i));
        }
        assert_eq!(history.undo_len(), MAX_UNDO);
    }

    #[test]
    fn redo_cleared_on_new_mutation() {
        let mut history = UndoHistory::default();
        let scene1 = make_scene(1);
        let scene2 = make_scene(2);
        let scene3 = make_scene(3);

        history.push(scene1.clone());
        let _restored = history.undo(&scene2).unwrap();
        assert_eq!(history.redo_len(), 1);

        // New mutation clears redo
        history.push(scene3);
        assert_eq!(history.redo_len(), 0);
    }
}
