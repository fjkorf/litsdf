# CLI Crate — `litsdf-cli`

## Overview

A command-line tool for manipulating litsdf YAML scene files without the editor GUI. Depends only on `litsdf_core` + `clap` (no Bevy). Enables scripting, batch processing, and programmatic scene construction.

**Status: Implemented.** All scene, bone, shape, and modifier commands are functional with 18 integration tests.

## Workspace Addition

```
crates/
  litsdf_cli/
    Cargo.toml     # depends on litsdf_core + clap
    src/
      main.rs
      commands/
        scene.rs
        bone.rs
        shape.rs
        modifier.rs
        render.rs
```

Dependencies: `litsdf_core`, `clap` (CLI argument parsing), `uuid` (for ID generation).

## Command Structure

```
litsdf-cli <COMMAND> <SUBCOMMAND> [OPTIONS]
```

### Scene Commands

```sh
# Create a new empty scene
litsdf-cli scene new "My Scene" -o scene.yaml

# Rename a scene
litsdf-cli scene rename scene.yaml "New Name"

# Set light direction
litsdf-cli scene light scene.yaml 0.6 0.8 0.4

# Show scene info (bone count, shape count, tree structure)
litsdf-cli scene info scene.yaml

# Dump full tree structure
litsdf-cli scene tree scene.yaml
```

### Bone Commands

```sh
# Add a bone (prints new bone ID)
litsdf-cli bone add scene.yaml --parent root --name "Arm" --translate 1.2,0,0

# Remove a bone (reparents children)
litsdf-cli bone remove scene.yaml --name "Arm"

# Remove a bone and all its contents
litsdf-cli bone remove scene.yaml --name "Arm" --recursive

# Rename a bone
litsdf-cli bone rename scene.yaml --name "Arm" --to "LeftArm"

# Move a bone
litsdf-cli bone move scene.yaml --name "Arm" --translate 2.0,0,0

# Rotate a bone
litsdf-cli bone rotate scene.yaml --name "Arm" --rotation 0,45,0

# Animate a bone
litsdf-cli bone animate scene.yaml --name "Arm" --bob 0.3,0.5 --sway 5,0.2

# Duplicate a bone (deep clone)
litsdf-cli bone duplicate scene.yaml --name "Arm" --as "RightArm"

# Reparent a bone
litsdf-cli bone reparent scene.yaml --name "Hand" --parent "LeftArm"

# List all bones
litsdf-cli bone list scene.yaml
```

### Shape Commands

```sh
# Add a shape to a bone (prints new shape ID)
litsdf-cli shape add scene.yaml --bone "Arm" --type Sphere --radius 0.5

# Remove a shape
litsdf-cli shape remove scene.yaml --name "MySphere"

# Set properties (multiple can be combined)
litsdf-cli shape set scene.yaml --name "MySphere" \
  --translate 0,1,0 \
  --rotate 0,45,0 \
  --scale 1.5 \
  --color 0.8,0.2,0.2 \
  --roughness 0.5 \
  --metallic 0.8 \
  --fresnel 2.0 \
  --combine SmoothUnion \
  --blend-k 0.3

# Set primitive type and params
litsdf-cli shape set-type scene.yaml --name "MySphere" --type Box --params 0.5,0.5,0.5,0

# Duplicate a shape
litsdf-cli shape duplicate scene.yaml --name "MySphere" --as "MySphere2"

# Move shape to different bone
litsdf-cli shape reparent scene.yaml --name "MySphere" --bone "Body"

# Export shape as standalone YAML
litsdf-cli shape export scene.yaml --name "MySphere"

# Import shape from standalone YAML
litsdf-cli shape import scene.yaml --bone "Arm" < shape.yaml

# Set color mode
litsdf-cli shape color-mode scene.yaml --name "MySphere" --mode palette \
  --palette-a 0.5,0.5,0.5 --palette-b 0.5,0.5,0.5

# Set noise
litsdf-cli shape noise scene.yaml --name "MySphere" --amp 0.05 --freq 4.0 --oct 3

# Animate a shape
litsdf-cli shape animate scene.yaml --name "MySphere" \
  --move-y 0.3,0.5 --spin-y 45,0.3 --pulse 0.1,0.5

# Clear animation
litsdf-cli shape animate scene.yaml --name "MySphere" --clear

# List all shapes
litsdf-cli shape list scene.yaml
```

### Modifier Commands

```sh
# Set a modifier (replaces existing of same type)
litsdf-cli modifier set scene.yaml --shape "MySphere" --rounding 0.1
litsdf-cli modifier set scene.yaml --shape "MySphere" --twist 2.0
litsdf-cli modifier set scene.yaml --shape "MySphere" --elongate 1,0.5,0
litsdf-cli modifier set scene.yaml --shape "MySphere" --repeat 2,2,2

# Clear all modifiers
litsdf-cli modifier clear scene.yaml --shape "MySphere"

# List modifiers on a shape
litsdf-cli modifier list scene.yaml --shape "MySphere"
```

### Render Commands

```sh
# Render scene to image (requires litsdf_render, so this would be a separate binary or feature flag)
litsdf-cli render scene.yaml --output render.png --size 1920x1080

# Render animation sequence
litsdf-cli render scene.yaml --output-dir frames/ --frames 60 --fps 30
```

Note: Render commands require Bevy/GPU, so they'd either be a feature flag or a separate binary (`litsdf-render-cli`). The core CLI (scene/bone/shape/modifier) is pure data manipulation and needs no GPU.

## Implementation Approach

### Shape/Bone Identification

Shapes and bones can be identified by:
1. **Name** (default, easiest for users): `--name "MySphere"`
2. **UUID** (precise, for scripts): `--id a0000000-...`
3. **Path** (hierarchical): `--path "Root/Arm/MySphere"`

Name-based lookup searches the entire tree. If ambiguous (multiple shapes with same name), error with "multiple matches found, use --id".

### File Mutation Pattern

All commands that modify a scene:
1. Load YAML from file
2. Apply operation to the in-memory `SdfScene`
3. Save back to the same file (or `-o output.yaml` for a different file)

This is atomic per command. No file locking needed.

### Core Functions Used

All operations are implemented in `litsdf_core`:

| CLI Operation | Core Function |
|--------------|--------------|
| Add bone | `SdfBone::new()` + `find_bone_mut().children.push()` |
| Remove bone | `SdfBone::remove_bone()` (reparent) or `extract_bone()` (recursive) |
| Add shape | `SdfShape::new()` + `find_bone_mut().shapes.push()` |
| Remove shape | `SdfBone::remove_shape()` |
| Find by name | `SdfBone::find_shape_by_name()` / `find_bone_by_name()` |
| Find by id | `SdfBone::find_shape()` / `find_bone()` |
| Duplicate shape | `SdfShape::duplicate()` — clone with new UUID + " Copy" suffix |
| Duplicate bone | `SdfBone::duplicate_deep()` — recursive clone with fresh UUIDs |
| Reparent shape | `SdfBone::reparent_shape()` — extract + add to target |
| Reparent bone | `SdfBone::reparent_bone()` — extract + add to target (with cycle check) |
| Scene info | `SdfScene::info()` → `SceneInfo` struct |
| Scene tree | `SdfScene::tree_string()` → ASCII tree |
| Save/Load | `persistence::save_scene/load_scene` |
| Reset transform | `SdfShape::reset_transform()` / `SdfBone::reset_transform()` |
| Clear animation | `SdfShape::clear_animation()` / `SdfBone::clear_animation()` |
| Clear modifiers | `SdfShape::clear_modifiers()` |
| Counts | `SdfBone::bone_count()` / `shape_count()` |

## Example Workflow: Scripted Scene Construction

```sh
#!/bin/bash
# Build the selfie girl scene programmatically

litsdf-cli scene new "Selfie Girl" -o girl.yaml

# Body
litsdf-cli bone add girl.yaml --parent root --name Body
litsdf-cli shape add girl.yaml --bone Body --type Ellipsoid --params 0.32,0.4,0.22 --name Torso
litsdf-cli shape set girl.yaml --name Torso --color 0.85,0.65,0.55 --roughness 0.75

# Head
litsdf-cli bone add girl.yaml --parent Body --name Head --translate 0,1.55,0
litsdf-cli bone animate girl.yaml --name Head --sway 4,0.12
litsdf-cli shape add girl.yaml --bone Head --type Ellipsoid --params 0.42,0.5,0.42 --name Cranium
litsdf-cli shape set girl.yaml --name Cranium --color 0.85,0.65,0.55 --fresnel 1.2

# Eyes
litsdf-cli shape add girl.yaml --bone Head --type Sphere --radius 0.09 --name LeftEye
litsdf-cli shape set girl.yaml --name LeftEye --translate 0.14,0.05,0.36 --color 0.95,0.95,0.98

echo "Scene built: $(litsdf-cli scene info girl.yaml)"
```

## Crate Dependencies

```toml
[package]
name = "litsdf-cli"
version = "0.1.0"
edition = "2024"

[dependencies]
litsdf_core = { path = "../litsdf_core" }
clap = { version = "4", features = ["derive"] }
uuid = { version = "1", features = ["v4"] }
```

No Bevy, no egui, no GPU. Pure data manipulation. Fast compilation, works on any platform.
