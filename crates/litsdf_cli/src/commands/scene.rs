use std::path::PathBuf;
use clap::Subcommand;
use litsdf_core::models::SdfScene;
use litsdf_core::persistence;

#[derive(Subcommand)]
pub enum SceneCmd {
    /// Create a new empty scene
    New {
        /// Scene name
        name: String,
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Rename a scene
    Rename {
        /// Scene file
        file: PathBuf,
        /// New name
        name: String,
    },
    /// Set light direction
    Light {
        /// Scene file
        file: PathBuf,
        /// X component
        x: f32,
        /// Y component
        y: f32,
        /// Z component
        z: f32,
    },
    /// Show scene info
    Info {
        /// Scene file
        file: PathBuf,
    },
    /// Print scene tree structure
    Tree {
        /// Scene file
        file: PathBuf,
    },
}

pub fn run(cmd: SceneCmd) -> Result<(), String> {
    match cmd {
        SceneCmd::New { name, output } => {
            let scene = SdfScene::new(&name);
            persistence::save_scene(&scene, &output)?;
            println!("Created scene \"{}\" → {}", name, output.display());
            Ok(())
        }
        SceneCmd::Rename { file, name } => {
            super::mutate(&file, |scene| {
                scene.name = name.clone();
                Ok(format!("Renamed scene to \"{}\"", name))
            })
        }
        SceneCmd::Light { file, x, y, z } => {
            super::mutate(&file, |scene| {
                scene.light_dir = [x, y, z];
                Ok(format!("Light direction set to [{x}, {y}, {z}]"))
            })
        }
        SceneCmd::Info { file } => {
            let scene = super::load(&file)?;
            println!("{}", scene.info());
            Ok(())
        }
        SceneCmd::Tree { file } => {
            let scene = super::load(&file)?;
            print!("{}", scene.tree_string());
            Ok(())
        }
    }
}
