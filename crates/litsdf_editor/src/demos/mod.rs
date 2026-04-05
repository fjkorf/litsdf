mod primitive_gallery;
mod boolean_sampler;
mod modifier_parade;
mod mushroom_garden;
mod robot_friend;
mod abstract_sculpture;
mod hanging_chain;
mod pendulum;
mod damping_lab;
mod speed_glow;
mod wave_force;

use std::collections::HashMap;
use egui_snarl::Snarl;
use litsdf_core::models::{BoneId, ShapeId, SdfScene};
use crate::nodes::SdfNode;

pub struct DemoResult {
    pub scene: SdfScene,
    pub shape_graphs: HashMap<ShapeId, Snarl<SdfNode>>,
    pub bone_graphs: HashMap<BoneId, Snarl<SdfNode>>,
}

/// Resource for passing node graphs from main.rs to the editor on startup.
#[derive(bevy::prelude::Resource)]
pub struct PendingGraphs {
    pub shape_graphs: HashMap<ShapeId, Snarl<SdfNode>>,
    pub bone_graphs: HashMap<BoneId, Snarl<SdfNode>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoScene {
    PrimitiveGallery,
    BooleanSampler,
    ModifierParade,
    MushroomGarden,
    RobotFriend,
    AbstractSculpture,
    HangingChain,
    Pendulum,
    DampingLab,
    SpeedGlow,
    WaveForce,
}

impl DemoScene {
    pub fn label(&self) -> &'static str {
        match self {
            Self::PrimitiveGallery => "Primitive Gallery",
            Self::BooleanSampler => "Boolean Sampler",
            Self::ModifierParade => "Modifier Parade",
            Self::MushroomGarden => "Mushroom Garden",
            Self::RobotFriend => "Robot Friend",
            Self::AbstractSculpture => "Abstract Sculpture",
            Self::HangingChain => "Hanging Chain",
            Self::Pendulum => "Pendulum",
            Self::DampingLab => "Damping Lab",
            Self::SpeedGlow => "Speed Glow",
            Self::WaveForce => "Wave Force",
        }
    }

    pub fn all() -> &'static [DemoScene] {
        &[
            Self::PrimitiveGallery,
            Self::BooleanSampler,
            Self::ModifierParade,
            Self::MushroomGarden,
            Self::RobotFriend,
            Self::AbstractSculpture,
            Self::HangingChain,
            Self::Pendulum,
            Self::DampingLab,
            Self::SpeedGlow,
            Self::WaveForce,
        ]
    }
}

pub fn load_demo(demo: DemoScene) -> DemoResult {
    match demo {
        DemoScene::PrimitiveGallery => primitive_gallery::create(),
        DemoScene::BooleanSampler => boolean_sampler::create(),
        DemoScene::ModifierParade => modifier_parade::create(),
        DemoScene::MushroomGarden => mushroom_garden::create(),
        DemoScene::RobotFriend => robot_friend::create(),
        DemoScene::AbstractSculpture => abstract_sculpture::create(),
        DemoScene::HangingChain => hanging_chain::create(),
        DemoScene::Pendulum => pendulum::create(),
        DemoScene::DampingLab => damping_lab::create(),
        DemoScene::SpeedGlow => speed_glow::create(),
        DemoScene::WaveForce => wave_force::create(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_demos_construct() {
        for demo in DemoScene::all() {
            let result = load_demo(*demo);
            assert!(!result.scene.name.is_empty(), "{:?} has empty name", demo);
            assert!(result.scene.root_bone.shape_count() > 0, "{:?} has no shapes", demo);
        }
    }

    #[test]
    fn primitive_gallery_has_all_primitives() {
        let result = load_demo(DemoScene::PrimitiveGallery);
        assert_eq!(result.scene.root_bone.shape_count(), 13);
    }
}
