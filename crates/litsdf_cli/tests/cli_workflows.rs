use std::process::Command;
use tempfile::NamedTempFile;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_litsdf-cli"))
}

fn temp_yaml() -> NamedTempFile {
    NamedTempFile::with_suffix(".yaml").unwrap()
}

// ── Scene workflows ────────────────────────────────────────────

#[test]
fn scene_new_and_info() {
    let f = temp_yaml();
    let path = f.path().to_str().unwrap();

    let out = cli().args(["scene", "new", "Test Scene", "-o", path])
        .output().unwrap();
    assert!(out.status.success(), "scene new failed: {}", String::from_utf8_lossy(&out.stderr));

    let out = cli().args(["scene", "info", path])
        .output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Test Scene"));
    assert!(stdout.contains("0 bones"));
    assert!(stdout.contains("0 shapes"));
}

#[test]
fn scene_rename() {
    let f = temp_yaml();
    let path = f.path().to_str().unwrap();

    cli().args(["scene", "new", "Old", "-o", path]).output().unwrap();
    cli().args(["scene", "rename", path, "New Name"]).output().unwrap();

    let out = cli().args(["scene", "info", path]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("New Name"));
}

#[test]
fn scene_light() {
    let f = temp_yaml();
    let path = f.path().to_str().unwrap();

    cli().args(["scene", "new", "Test", "-o", path]).output().unwrap();
    let out = cli().args(["scene", "light", path, "1.0", "0.0", "0.0"])
        .output().unwrap();
    assert!(out.status.success());

    // Verify by loading the YAML
    let scene = litsdf_core::persistence::load_scene(f.path()).unwrap();
    assert_eq!(scene.light_dir, [1.0, 0.0, 0.0]);
}

#[test]
fn scene_tree() {
    let f = temp_yaml();
    let path = f.path().to_str().unwrap();

    cli().args(["scene", "new", "TreeTest", "-o", path]).output().unwrap();
    cli().args(["bone", "add", path, "--parent", "Root", "--name", "Arm"]).output().unwrap();
    cli().args(["shape", "add", path, "--bone", "Arm", "--type", "Sphere", "--name", "Ball"]).output().unwrap();

    let out = cli().args(["scene", "tree", path]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Scene: TreeTest"));
    assert!(stdout.contains("[Bone] Root"));
    assert!(stdout.contains("[Bone] Arm"));
    assert!(stdout.contains("(Sphere) Ball"));
}

// ── Bone workflows ─────────────────────────────────────────────

#[test]
fn bone_add_rename_move_rotate() {
    let f = temp_yaml();
    let path = f.path().to_str().unwrap();

    cli().args(["scene", "new", "Test", "-o", path]).output().unwrap();

    // Add bone with translation
    let out = cli().args(["bone", "add", path, "--parent", "Root", "--name", "Arm", "--translate", "1.0,0.0,0.0"])
        .output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));

    // Rename
    cli().args(["bone", "rename", path, "--name", "Arm", "--to", "LeftArm"]).output().unwrap();

    // Move
    cli().args(["bone", "move", path, "--name", "LeftArm", "--translate", "2.0,0.0,0.0"]).output().unwrap();

    // Rotate
    cli().args(["bone", "rotate", path, "--name", "LeftArm", "--rotation", "0.0,45.0,0.0"]).output().unwrap();

    let scene = litsdf_core::persistence::load_scene(f.path()).unwrap();
    let arm = scene.root_bone.find_bone_by_name("LeftArm").unwrap();
    assert_eq!(arm.transform.translation, [2.0, 0.0, 0.0]);
    assert_eq!(arm.transform.rotation, [0.0, 45.0, 0.0]);
}

#[test]
fn bone_duplicate_and_reparent() {
    let f = temp_yaml();
    let path = f.path().to_str().unwrap();

    cli().args(["scene", "new", "Test", "-o", path]).output().unwrap();
    cli().args(["bone", "add", path, "--parent", "Root", "--name", "Body"]).output().unwrap();
    cli().args(["bone", "add", path, "--parent", "Root", "--name", "Arm"]).output().unwrap();
    cli().args(["shape", "add", path, "--bone", "Arm", "--type", "Sphere", "--name", "Ball"]).output().unwrap();

    // Duplicate
    let out = cli().args(["bone", "duplicate", path, "--name", "Arm", "--as", "RightArm"])
        .output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));

    // Reparent under Body
    cli().args(["bone", "reparent", path, "--name", "RightArm", "--parent", "Body"]).output().unwrap();

    let scene = litsdf_core::persistence::load_scene(f.path()).unwrap();
    let body = scene.root_bone.find_bone_by_name("Body").unwrap();
    assert_eq!(body.children.len(), 1);
    assert_eq!(body.children[0].name, "RightArm");
    assert_eq!(body.children[0].shapes.len(), 1);
}

#[test]
fn bone_remove_reparents() {
    let f = temp_yaml();
    let path = f.path().to_str().unwrap();

    cli().args(["scene", "new", "Test", "-o", path]).output().unwrap();
    cli().args(["bone", "add", path, "--parent", "Root", "--name", "Arm"]).output().unwrap();
    cli().args(["bone", "add", path, "--parent", "Arm", "--name", "Hand"]).output().unwrap();
    cli().args(["shape", "add", path, "--bone", "Arm", "--type", "Sphere"]).output().unwrap();

    cli().args(["bone", "remove", path, "--name", "Arm"]).output().unwrap();

    let scene = litsdf_core::persistence::load_scene(f.path()).unwrap();
    // Hand should be reparented to Root, shape too
    assert!(scene.root_bone.find_bone_by_name("Hand").is_some());
    assert_eq!(scene.root_bone.shape_count(), 1);
}

#[test]
fn bone_remove_recursive() {
    let f = temp_yaml();
    let path = f.path().to_str().unwrap();

    cli().args(["scene", "new", "Test", "-o", path]).output().unwrap();
    cli().args(["bone", "add", path, "--parent", "Root", "--name", "Arm"]).output().unwrap();
    cli().args(["bone", "add", path, "--parent", "Arm", "--name", "Hand"]).output().unwrap();
    cli().args(["shape", "add", path, "--bone", "Hand", "--type", "Sphere"]).output().unwrap();

    cli().args(["bone", "remove", path, "--name", "Arm", "--recursive"]).output().unwrap();

    let scene = litsdf_core::persistence::load_scene(f.path()).unwrap();
    assert!(scene.root_bone.find_bone_by_name("Arm").is_none());
    assert!(scene.root_bone.find_bone_by_name("Hand").is_none());
    assert_eq!(scene.root_bone.shape_count(), 0);
}

#[test]
fn bone_list() {
    let f = temp_yaml();
    let path = f.path().to_str().unwrap();

    cli().args(["scene", "new", "Test", "-o", path]).output().unwrap();
    cli().args(["bone", "add", path, "--parent", "Root", "--name", "Arm"]).output().unwrap();

    let out = cli().args(["bone", "list", path]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Root"));
    assert!(stdout.contains("Arm"));
}

// ── Shape workflows ────────────────────────────────────────────

#[test]
fn shape_add_set_and_remove() {
    let f = temp_yaml();
    let path = f.path().to_str().unwrap();

    cli().args(["scene", "new", "Test", "-o", path]).output().unwrap();
    cli().args(["bone", "add", path, "--parent", "Root", "--name", "Body"]).output().unwrap();

    // Add shape
    cli().args(["shape", "add", path, "--bone", "Body", "--type", "Sphere", "--name", "Ball"]).output().unwrap();

    // Set properties
    cli().args(["shape", "set", path, "--name", "Ball",
        "--translate", "1.0,2.0,0.0",
        "--color", "0.8,0.2,0.2",
        "--roughness", "0.7",
        "--metallic", "0.9",
        "--combine", "SmoothUnion",
        "--blend-k", "0.5",
    ]).output().unwrap();

    let scene = litsdf_core::persistence::load_scene(f.path()).unwrap();
    let (shape, _) = scene.root_bone.find_shape_by_name("Ball").unwrap();
    assert_eq!(shape.transform.translation, [1.0, 2.0, 0.0]);
    assert_eq!(shape.material.color, [0.8, 0.2, 0.2]);
    assert_eq!(shape.material.roughness, 0.7);
    assert_eq!(shape.material.metallic, 0.9);
    assert!(matches!(shape.combination, litsdf_core::models::CombinationOp::SmoothUnion { k } if (k - 0.5).abs() < 0.001));

    // Remove
    cli().args(["shape", "remove", path, "--name", "Ball"]).output().unwrap();
    let scene = litsdf_core::persistence::load_scene(f.path()).unwrap();
    assert!(scene.root_bone.find_shape_by_name("Ball").is_none());
}

#[test]
fn shape_set_type() {
    let f = temp_yaml();
    let path = f.path().to_str().unwrap();

    cli().args(["scene", "new", "Test", "-o", path]).output().unwrap();
    cli().args(["bone", "add", path, "--parent", "Root", "--name", "Body"]).output().unwrap();
    cli().args(["shape", "add", path, "--bone", "Body", "--type", "Sphere", "--name", "S"]).output().unwrap();

    cli().args(["shape", "set-type", path, "--name", "S", "--type", "Box", "--params", "0.5,0.5,0.5,0.0"]).output().unwrap();

    let scene = litsdf_core::persistence::load_scene(f.path()).unwrap();
    let (shape, _) = scene.root_bone.find_shape_by_name("S").unwrap();
    assert!(matches!(shape.primitive, litsdf_core::models::SdfPrimitive::Box { .. }));
}

#[test]
fn shape_duplicate_and_reparent() {
    let f = temp_yaml();
    let path = f.path().to_str().unwrap();

    cli().args(["scene", "new", "Test", "-o", path]).output().unwrap();
    cli().args(["bone", "add", path, "--parent", "Root", "--name", "A"]).output().unwrap();
    cli().args(["bone", "add", path, "--parent", "Root", "--name", "B"]).output().unwrap();
    cli().args(["shape", "add", path, "--bone", "A", "--type", "Sphere", "--name", "Ball"]).output().unwrap();

    // Duplicate
    cli().args(["shape", "duplicate", path, "--name", "Ball", "--as", "Ball2"]).output().unwrap();

    let scene = litsdf_core::persistence::load_scene(f.path()).unwrap();
    assert!(scene.root_bone.find_shape_by_name("Ball2").is_some());

    // Reparent Ball2 to bone B
    cli().args(["shape", "reparent", path, "--name", "Ball2", "--bone", "B"]).output().unwrap();

    let scene = litsdf_core::persistence::load_scene(f.path()).unwrap();
    let (_, bone_id) = scene.root_bone.find_shape_by_name("Ball2").unwrap();
    let bone = scene.root_bone.find_bone(bone_id).unwrap();
    assert_eq!(bone.name, "B");
}

#[test]
fn shape_color_mode() {
    let f = temp_yaml();
    let path = f.path().to_str().unwrap();

    cli().args(["scene", "new", "Test", "-o", path]).output().unwrap();
    cli().args(["bone", "add", path, "--parent", "Root", "--name", "A"]).output().unwrap();
    cli().args(["shape", "add", path, "--bone", "A", "--type", "Sphere", "--name", "S"]).output().unwrap();

    cli().args(["shape", "color-mode", path, "--name", "S", "--mode", "palette",
        "--palette-a", "0.5,0.5,0.5", "--palette-b", "0.5,0.5,0.5"]).output().unwrap();

    let scene = litsdf_core::persistence::load_scene(f.path()).unwrap();
    let (shape, _) = scene.root_bone.find_shape_by_name("S").unwrap();
    assert_eq!(shape.material.color_mode, 1);
    assert_eq!(shape.material.palette_a, [0.5, 0.5, 0.5]);
}

#[test]
fn shape_noise() {
    let f = temp_yaml();
    let path = f.path().to_str().unwrap();

    cli().args(["scene", "new", "Test", "-o", path]).output().unwrap();
    cli().args(["bone", "add", path, "--parent", "Root", "--name", "A"]).output().unwrap();
    cli().args(["shape", "add", path, "--bone", "A", "--type", "Sphere", "--name", "S"]).output().unwrap();

    cli().args(["shape", "noise", path, "--name", "S", "--amp", "0.05", "--freq", "4.0", "--oct", "3"]).output().unwrap();

    let scene = litsdf_core::persistence::load_scene(f.path()).unwrap();
    let (shape, _) = scene.root_bone.find_shape_by_name("S").unwrap();
    assert_eq!(shape.material.noise_amplitude, 0.05);
    assert_eq!(shape.material.noise_frequency, 4.0);
    assert_eq!(shape.material.noise_octaves, 3);
}

// ── Modifier workflows ─────────────────────────────────────────

#[test]
fn modifier_set_list_clear() {
    let f = temp_yaml();
    let path = f.path().to_str().unwrap();

    cli().args(["scene", "new", "Test", "-o", path]).output().unwrap();
    cli().args(["bone", "add", path, "--parent", "Root", "--name", "A"]).output().unwrap();
    cli().args(["shape", "add", path, "--bone", "A", "--type", "Sphere", "--name", "S"]).output().unwrap();

    // Set modifiers
    cli().args(["modifier", "set", path, "--shape", "S", "--rounding", "0.1", "--twist", "2.0"]).output().unwrap();

    let scene = litsdf_core::persistence::load_scene(f.path()).unwrap();
    let (shape, _) = scene.root_bone.find_shape_by_name("S").unwrap();
    assert_eq!(shape.modifiers.len(), 2);

    // List
    let out = cli().args(["modifier", "list", path, "--shape", "S"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Rounding: 0.1"));
    assert!(stdout.contains("Twist: 2"));

    // Replace rounding (should not create duplicate)
    cli().args(["modifier", "set", path, "--shape", "S", "--rounding", "0.2"]).output().unwrap();
    let scene = litsdf_core::persistence::load_scene(f.path()).unwrap();
    let (shape, _) = scene.root_bone.find_shape_by_name("S").unwrap();
    assert_eq!(shape.modifiers.len(), 2); // still 2, not 3

    // Clear
    cli().args(["modifier", "clear", path, "--shape", "S"]).output().unwrap();
    let scene = litsdf_core::persistence::load_scene(f.path()).unwrap();
    let (shape, _) = scene.root_bone.find_shape_by_name("S").unwrap();
    assert!(shape.modifiers.is_empty());
}

// ── Full workflow: scripted scene construction ──────────────────

#[test]
fn full_scene_construction() {
    let f = temp_yaml();
    let path = f.path().to_str().unwrap();

    // Build a small scene from scratch
    cli().args(["scene", "new", "Character", "-o", path]).output().unwrap();
    cli().args(["bone", "add", path, "--parent", "Root", "--name", "Body"]).output().unwrap();
    cli().args(["bone", "add", path, "--parent", "Body", "--name", "Head", "--translate", "0.0,1.5,0.0"]).output().unwrap();
    cli().args(["shape", "add", path, "--bone", "Body", "--type", "Ellipsoid", "--name", "Torso", "--params", "0.3,0.4,0.2,0.0"]).output().unwrap();
    cli().args(["shape", "add", path, "--bone", "Head", "--type", "Sphere", "--name", "Cranium"]).output().unwrap();
    cli().args(["shape", "set", path, "--name", "Cranium", "--color", "0.85,0.65,0.55"]).output().unwrap();

    // Verify final state
    let scene = litsdf_core::persistence::load_scene(f.path()).unwrap();
    let info = scene.info();
    assert_eq!(info.name, "Character");
    assert_eq!(info.bone_count, 2); // Body + Head
    assert_eq!(info.shape_count, 2); // Torso + Cranium

    let head = scene.root_bone.find_bone_by_name("Head").unwrap();
    assert_eq!(head.transform.translation, [0.0, 1.5, 0.0]);

    let (cranium, _) = scene.root_bone.find_shape_by_name("Cranium").unwrap();
    assert_eq!(cranium.material.color, [0.85, 0.65, 0.55]);
}
