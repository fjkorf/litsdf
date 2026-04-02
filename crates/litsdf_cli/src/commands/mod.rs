pub mod scene;
pub mod bone;
pub mod shape;
pub mod modifier;

use std::path::Path;
use litsdf_core::models::SdfScene;
use litsdf_core::persistence;

/// Load a scene from a YAML file.
pub fn load(path: &Path) -> Result<SdfScene, String> {
    persistence::load_scene(path)
}

/// Save a scene to a YAML file.
pub fn save(scene: &SdfScene, path: &Path) -> Result<(), String> {
    persistence::save_scene(scene, path)
}

/// Load, apply a mutation, save back.
pub fn mutate(path: &Path, f: impl FnOnce(&mut SdfScene) -> Result<String, String>) -> Result<(), String> {
    let mut scene = load(path)?;
    let msg = f(&mut scene)?;
    save(&scene, path)?;
    println!("{msg}");
    Ok(())
}
