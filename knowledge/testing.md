# Testing Strategy

## Unit Tests (76 tests, cargo test --workspace)

Tests are distributed across crates:
- `litsdf_core`: 39 tests (models, scene, persistence, physics)
- `litsdf_editor`: 17 tests (UI, undo, node graph, project persistence, presets, demo scenes)
- `litsdf_render`: 4 tests (shader struct size, codegen default scene, codegen empty scene, topology hash)
- `litsdf_cli`: 16 integration tests (CLI workflow end-to-end)

### Model Tests (models.rs)
- `yaml_round_trip` — Serialize default scene to YAML, deserialize, compare
- `shape_defaults` — Verify default sphere has expected radius, transform, color
- `compact_yaml_omits_defaults` — Default shape YAML has no transform/material/modifiers/combination
- `compact_yaml_keeps_non_defaults` — Changed values present, defaults still omitted
- `compact_yaml_round_trips` — Compact YAML → load → save produces identical output
- `bone_find_shape` — Find shapes by ID across nested bone tree
- `bone_remove_reparents` — Removing a bone reparents its shapes/children to parent
- `all_primitives_serialize` — All 8 SdfPrimitive variants round-trip through YAML
- `combination_ops_serialize` — All 6 CombinationOp variants round-trip
- `modifiers_serialize` — All ShapeModifier variants round-trip
- `bone_hierarchy_serializes` — Nested bone tree survives YAML round-trip
- `shape_duplicate` — Clone shape gets new ID, " Copy" suffix, preserves all properties
- `shape_reset_transform` — Reset restores ShapeTransform::default()
- `shape_clear_modifiers` — Empties modifier vec
- `bone_duplicate_deep` — Deep clone assigns fresh UUIDs to all bones/shapes, only top-level gets " Copy" suffix
- `bone_find_by_name` — Recursive name lookup traverses children, returns None for missing
- `bone_reparent_shape` — Extracts shape from one bone, adds to target bone
- `bone_reparent_bone` — Moves bone subtree under new parent
- `bone_reparent_bone_prevents_cycle` — Rejects self-reparent and descendant-to-ancestor cycles
- `bone_counts` — bone_count() and shape_count() recurse correctly
- `bone_reset_and_clear_animation` — Reset transform and clear bone animation
- `scene_new` — Empty scene with root bone and default light
- `scene_info` — SceneInfo struct with counts and animation flag
- `scene_tree_string` — ASCII tree contains bone and shape names

### Physics Tests (physics.rs)
- `zero_mass_no_offset` — Kinematic bone (mass 0) produces no physics offset
- `positive_mass_falls` — Dynamic bone falls downward under gravity
- `reset_zeroes_velocity` — reset_physics clears all velocities
- `damping_reduces_velocity` — Heavy damping (0.5) limits terminal velocity
- `collider_sphere` — Sphere shape → ColliderApprox::Sphere
- `collider_capsule` — Capsule shape → ColliderApprox::Capsule
- `collider_fallback_bounding` — No shapes → fallback sphere with min radius
- `damping_conversion` — damping_to_avian maps 0.95→~3, 0.70→~21

### Scene Tests (scene.rs)
- `flatten_default_scene` — Default scene flattens to correct shape count and types
- `bone_child_offsets_shape` — Shape on child bone at (2,0,0) appears at world (2,0,0)
- `shape_keeps_own_combine_across_bones` — Shapes use their own combine op, not bone's
- `nested_bone_transforms_compose` — Root + child(1,0,0) + grandchild(0,1,0) → shape at (1,1,0)
- `modifier_flags_encode` — Vec<ShapeModifier> → bitmask + parameter extraction

### Persistence Tests (persistence.rs)
- `save_and_load` — Save scene to tempdir, load back, compare fields
- `list_scenes_finds_yaml` — Write .yaml/.yml/.txt files, verify only YAML listed

### Undo Tests (undo.rs)
- `undo_single_mutation` — Add shape, undo, verify shape gone
- `undo_redo_cycle` — Add shape, undo, redo, verify shape back
- `undo_stack_limit` — Push 60 snapshots, verify only 50 stored
- `redo_cleared_on_new_mutation` — Undo, then new edit, verify redo stack empty
- `load_nonexistent_errors` — Loading missing file returns Err

### UI Tests (ui/mod.rs)
- `click_through_bone_to_shape_edit` — Select Arm → see Box → select Box → edit size → switch to Root → shape clears
- `no_dirty_when_no_change` — Select shape, sync without editing → dirty stays false
- `nested_bone_shape_selection` — Select Arm 1 → see Torus → edit major_radius → verify model

### Node Graph Evaluator Tests (nodes/eval.rs)
- `constant_to_output` — Constant(1.5) → ShapeOutput.ty → result.ty == 1.5
- `time_to_oscillator_to_output` — Time → SinOscillator → ShapeOutput.ty, verified at t=0 (→0) and t=0.25 (→1.0)
- `math_chain` — Constant(3) * Constant(2) → ShapeOutput.scale → result.scale == 6.0
- `unconnected_returns_none` — Empty ShapeOutput → all outputs are None

### Shader Struct Tests (shader.rs)
- `shader_shape_size_matches_wgsl` — Asserts ShaderShape is 256 bytes and SdfShaderParams is 8288 bytes

### Shader Codegen Tests (codegen.rs)
- `generate_default_scene` — Default scene generates WGSL with sdf_scene/sdf_scene_material functions referencing all 5 shapes
- `empty_scene_errors` — Empty scene returns descriptive error
- `topology_hash_changes_on_shape_add` — Hash changes when shapes are added (triggers recompilation)

### CLI Integration Tests (litsdf_cli/tests/cli_workflows.rs)

Each test creates a temp YAML file, runs CLI commands via the binary, and verifies results by loading the file with `litsdf_core::persistence`:

- `scene_new_and_info` — Create scene, verify info output
- `scene_rename` — Create then rename, verify new name
- `scene_light` — Set light direction, verify in loaded scene
- `scene_tree` — Build scene with bones/shapes, verify ASCII tree output
- `bone_add_rename_move_rotate` — Add bone with translate, rename, move, rotate
- `bone_duplicate_and_reparent` — Duplicate bone with shapes, reparent under another
- `bone_remove_reparents` — Remove bone, verify children/shapes reparented to parent
- `bone_remove_recursive` — Remove bone with --recursive, verify subtree deleted
- `bone_list` — List bones, verify output contains names
- `shape_add_set_and_remove` — Add shape, set properties (translate, color, roughness, metallic, combine), remove
- `shape_set_type` — Change primitive type from Sphere to Box
- `shape_duplicate_and_reparent` — Duplicate shape, reparent to different bone
- `shape_animate_and_clear` — Set move/spin animation, clear
- `shape_color_mode` — Set palette color mode with palette params
- `shape_noise` — Set noise amplitude, frequency, octaves
- `modifier_set_list_clear` — Set modifiers, list output, replace (no duplicates), clear
- `full_scene_construction` — End-to-end: build Character scene with 2 bones, 2 shapes, animation

## Screenshot Tests

### Single Frame (LITSDF_SCREENSHOT)

```sh
LITSDF_SCREENSHOT=tests/screenshots/test.png cargo run
```

Captures at frame 30 (configurable), exits 5 frames later. Implemented in `testing.rs`:
- `ScreenshotConfig` resource with path, capture_frame, exit_frame
- `auto_screenshot` system runs in Update, spawns `Screenshot::primary_window()` at the right frame
- `LITSDF_SCREENSHOT_FRAME=N` overrides the capture frame (default 30). Use higher values for physics demos.

### Multi-Step Sequence (LITSDF_TEST_SEQUENCE)

```sh
LITSDF_TEST_SEQUENCE=tests/screenshots/sequence cargo run
```

Steps through bone selection states, capturing a screenshot at each:
- Step 0: Initial state
- Step 1: Root selected
- Step 2: Arm selected
- Step 3: Deepest bone selected
- Step 4: Shape on current bone selected

Implemented in `testing.rs` as `TestSequence` resource + `test_sequence_system`.

### Integration Script (tests/screenshot_test.sh)

```sh
./tests/screenshot_test.sh
```

Runs:
1. Default scene screenshot
2. Nested scene (my_shape_2.yaml) screenshot (if file exists)
3. Programmatic multi-bone scene (written to tempfile) screenshot
4. Unit tests

Outputs to `tests/screenshots/test_*.png`.

### Scene Loading for Tests

```sh
LITSDF_SCENE=path.yaml cargo run
```

Loads a YAML scene file at startup via `persistence::load_scene()`. Combined with `LITSDF_SCREENSHOT` for visual regression testing of specific scenes.

## Writing New Tests

### Unit Test Pattern

```rust
#[test]
fn my_test() {
    // Build a scene
    let scene = make_test_scene(); // helper in ui/mod.rs tests
    let (mut ui, mut scene) = make_state(scene);

    // Set selection
    scene.selected_bone = Some(bone_id);

    // Populate UI (simulates frame start)
    populate_bone_shapes(&mut ui, &scene);
    populate_shape_properties(&mut ui, &scene);

    // Assert UI state
    assert_eq!(ui.md.state.param_a, expected_value);

    // Simulate slider edit
    ui.md.state.param_a = 2.0;

    // Sync back (simulates frame end)
    sync_shape_properties(&mut ui, &mut scene);

    // Assert model updated
    assert!(scene.dirty);
}
```

### Key: populate only writes on selection change

`populate_shape_properties` checks `prev_selected_shape`. On the FIRST call with a new selection, it writes model → UI. On subsequent calls with the same selection, it returns early (so slider edits aren't overwritten). To test editing, call populate twice: once to trigger the write, once to simulate the next frame.

### Screenshot Test Pattern

For visual regression, add to `tests/screenshot_test.sh`:

```sh
echo "[N/M] Description..."
LITSDF_SCENE="path.yaml" LITSDF_SCREENSHOT="$SCREENSHOTS/test_name.png" \
  cargo run --quiet 2>&1 | tail -1
```
