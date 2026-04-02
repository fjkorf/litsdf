# Bone Hierarchy System

## Overview

Bones are spatial organizers for SDF shapes. They form a tree where each bone has a local transform (relative to parent). Moving a parent bone moves all its children and their shapes.

Bones are NOT Bevy entities — they're a pure data model in `SdfBone`. Transform propagation is computed CPU-side via matrix multiplication in `compute_bone_world_transforms()`.

## Data Model

```rust
pub struct SdfBone {
    pub id: BoneId,           // UUID, root uses Uuid::nil()
    pub name: String,
    pub transform: ShapeTransform,  // local, relative to parent
    pub visible: bool,              // default true, skips GPU flatten when false
    pub children: Vec<SdfBone>,     // recursive tree
    pub shapes: Vec<SdfShape>,      // shapes on this bone (each also has visible)
}
```

The root bone:
- ID: `BoneId(Uuid::nil())` — well-known, stable
- Name: "Root"
- Transform: identity (always at origin)
- Cannot be deleted, renamed, or moved
- Always exists in every scene

## World Transform Resolution

`compute_bone_world_transforms()` walks the tree:

```
Root (identity)
├── Arm (translate 1.2, 0, 0) → world = Root * Arm = (1.2, 0, 0)
│   └── Hand (translate 0, 1, 0) → world = Arm_world * Hand = (1.2, 1, 0)
```

For each shape, the final world position is: `bone_world_matrix * shape_local_matrix`.

The matrix is decomposed back to (translation, euler_rotation, scale) for the shader's flat `ShaderShape` format.

## Flattening for GPU

`flatten_bone_tree()` does depth-first traversal:
1. Skip bone entirely if `bone.visible == false` (entire subtree hidden)
2. For each shape, skip if `shape.visible == false`
3. Emit visible shapes with world-resolved transforms
4. Recurse into children
5. Each shape keeps its own `combination` op
6. First shape overall gets Union (ignored by shader)

The result is a flat `Vec<ShaderShape>` — the shader doesn't know about bones.

## Serialization (YAML)

Bones serialize naturally as nested YAML:

```yaml
root_bone:
  id: 00000000-0000-0000-0000-000000000000
  name: Root
  transform:
    translation: [0.0, 0.0, 0.0]
    rotation: [0.0, 0.0, 0.0]
    scale: 1.0
  children:
    - id: abc...
      name: Arm
      transform:
        translation: [1.2, 0.0, 0.0]
        ...
      children: []
      shapes:
        - id: def...
          name: Box
          ...
  shapes:
    - id: ghi...
      name: Sphere
      ...
```

## Selection Model

Two-tier selection:
- `selected_bone: Option<BoneId>` — which bone is active in the tree
- `selected_shape: Option<ShapeId>` — which shape is being edited

Both can be active simultaneously. Selecting a bone shows its shapes in the properties panel. Clicking a shape in that list enables shape editing. Switching bones clears shape selection.

## Helper Methods on SdfBone

### Lookup
- `find_bone(id)` / `find_bone_mut(id)` — recursive lookup by BoneId
- `find_bone_by_name(name)` / `find_bone_by_name_mut(name)` — recursive lookup by name string
- `find_shape(id)` / `find_shape_mut(id)` — find shape anywhere in tree, returns (shape, parent_bone_id)
- `find_shape_by_name(name)` — lookup by name
- `all_shapes()` — flat iteration: Vec<(&SdfShape, BoneId)>

### Mutation
- `remove_shape(id)` — removes from anywhere in tree
- `remove_bone(id)` — removes bone, reparents its shapes and children to parent
- `extract_shape(id)` — removes and returns a shape (for reparenting)
- `extract_bone(id)` — removes and returns a bone subtree (for reparenting or recursive delete)
- `reparent_shape(shape_id, target_bone_id)` — move shape between bones
- `reparent_bone(bone_id, target_bone_id)` — move bone subtree under new parent (with cycle prevention)

### Cloning
- `duplicate_deep()` — recursive clone with fresh UUIDs for bone + all children + all shapes; only top-level bone gets " Copy" suffix

### Counts and Resets
- `bone_count()` — recursive count of descendant bones (not including self)
- `shape_count()` — recursive count of all shapes in subtree
- `reset_transform()` — set transform to default

## Helper Methods on SdfShape

- `duplicate()` — clone with new ShapeId and " Copy" name suffix
- `reset_transform()` — set transform to ShapeTransform::default()
- `clear_modifiers()` — empty modifier vec

## Helper Methods on SdfScene

- `SdfScene::new(name)` — empty scene with root bone and default light
- `info()` → `SceneInfo` — struct with name, bone_count, shape_count (Display impl)
- `tree_string()` → String — ASCII tree representation of full hierarchy

## Animation

Animation is handled externally via **node graphs** in the editor crate, not by fields on SdfBone or SdfShape. The `anim_*` triplet fields were removed. Node graphs (stored as `HashMap<ShapeId/BoneId, Snarl<SdfNode>>`) evaluate per-frame and override transform/material properties. See `knowledge/node-architecture.md` for details.

`compute_bone_world_transforms` accepts a `&HashMap<BoneId, ShapeTransform>` overrides parameter, allowing node graph evaluation to provide computed bone transforms. Pass an empty HashMap when no overrides are needed.
