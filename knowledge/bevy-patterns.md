# Bevy 0.18 Patterns Used in litsdf

## Plugin Pattern

litsdf registers everything via `SdfPlugin` in `lib.rs`:

```rust
impl Plugin for SdfPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<SdfMaterial>::default())
            .init_resource::<SdfSceneState>()
            .init_resource::<EditorUi>()
            .add_systems(Startup, (setup_camera, setup_initial_scene))
            .add_systems(Update, (orbit_camera, sync_scene_to_shader, draw_bone_gizmos))
            .add_systems(EguiPrimaryContextPass, editor_ui);
    }
}
```

## Schedule Ordering

- **Startup**: Run once — camera setup, spawn bounding geometry with SdfMaterial
- **Update**: Every frame — camera orbit, scene-to-shader sync, bone gizmos
- **EguiPrimaryContextPass**: Every frame — UI rendering (runs inside `egui::Context::run`)
- **PostUpdate** (Bevy internal): Transform propagation, render extraction

The UI system runs in `EguiPrimaryContextPass`, which executes during `PostUpdate` inside `ctx.run()`. This means `sync_scene_to_shader` (Update) runs BEFORE `editor_ui` (PostUpdate/EguiPrimaryContextPass). When the UI sets `dirty = true`, the shader sync happens next frame.

## Custom Material

```rust
#[derive(Asset, AsBindGroup, TypePath, Clone)]
pub struct SdfMaterial {
    #[uniform(0)]
    pub params: SdfShaderParams,
}

impl Material for SdfMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sdf_raymarch.wgsl".into()
    }

    fn specialize(...) -> Result<(), ...> {
        descriptor.primitive.cull_mode = None; // camera inside bounding box
        Ok(())
    }
}
```

Key points:
- `#[uniform(0)]` maps to `@group(3) @binding(0)` in WGSL (group 3, NOT 2)
- Inline data on the material struct — mutating via `materials.get_mut(handle)` triggers `AssetChanged`
- `specialize()` disables backface culling

## Resource Pattern

`SdfSceneState` and `EditorUi` are Bevy Resources (global singletons):

```rust
#[derive(Resource)]
pub struct SdfSceneState {
    pub scene: SdfScene,
    pub selected_shape: Option<ShapeId>,
    pub selected_bone: Option<BoneId>,
    pub show_bone_gizmos: bool,
    pub dirty: bool,
}
```

Systems access them via `Res<T>` (read) or `ResMut<T>` (write).

## Entity Spawning

The bounding geometry is spawned once in Startup:

```rust
commands.spawn((
    Mesh3d(meshes.add(Cuboid::new(40.0, 40.0, 40.0))),
    MeshMaterial3d(materials.add(SdfMaterial { params })),
    SdfBoundingEntity,  // marker component
));
```

`SdfBoundingEntity` is a marker component for querying the entity later during shader sync.

## Bevy 0.18 API Changes from Earlier Versions

| Old | New in 0.18 |
|-----|-------------|
| `EventReader<T>` | `MessageReader<T>` (events are messages) |
| `AmbientLight` resource | `AmbientLight` camera component |
| `Plane3d::new()` | `Plane3d::default().mesh().size(w, h)` |
| `EventWriter<AppExit>` | `MessageWriter<AppExit>` |
| `shadows_enabled` | `shadows_enabled` (PointLight field) |

## Gizmos

Bevy's built-in gizmo system draws debug lines:

```rust
fn draw_bone_gizmos(mut gizmos: Gizmos, state: Res<SdfSceneState>) {
    gizmos.line(parent_pos, child_pos, Color::srgb(0.9, 0.9, 0.2));
    gizmos.line(pos, pos + right * 0.15, Color::srgb(1.0, 0.2, 0.2)); // X axis
}
```

Gizmos render through Bevy's normal pipeline. They're visible where SDF shapes don't occlude them.

## bevy_egui Integration

bevy_egui provides `EguiContexts` for accessing egui's `Context`:

```rust
pub fn editor_ui(mut contexts: bevy_egui::EguiContexts, ...) {
    let ctx = contexts.ctx_mut().unwrap().clone();
    egui::SidePanel::left("tree").show(&ctx, |ui| { ... });
}
```

- Use `EguiWantsInput` resource to check if egui wants mouse input (for camera orbit conflict)
- UI systems MUST run in `EguiPrimaryContextPass` schedule
