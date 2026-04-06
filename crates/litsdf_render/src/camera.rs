use bevy::ecs::message::MessageReader;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy_egui::input::EguiWantsInput;

#[derive(Component)]
pub struct OrbitCamera {
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub target: Vec3,
    /// Set by editor to trigger a one-shot frame-selection move.
    pub frame_target: Option<Vec3>,
    /// Set by editor to toggle orthographic projection.
    pub toggle_ortho: bool,
    /// Viewport center offset [x, y] in normalized screen coords (-0.5 to 0.5).
    /// Used to shift the rendered view to match the visible area between UI panels.
    pub viewport_offset: [f32; 2],
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.5, 5.0).looking_at(Vec3::new(0.0, 0.8, 0.0), Vec3::Y),
        OrbitCamera {
            distance: 5.0,
            yaw: 0.0,
            pitch: 0.15,
            target: Vec3::new(0.0, 0.8, 0.0),
            frame_target: None,
            toggle_ortho: false,
            viewport_offset: [0.0, 0.0],
        },
        AmbientLight {
            color: Color::WHITE,
            brightness: 300.0,
            affects_lightmapped_meshes: true,
        },
    ));
}

pub fn orbit_camera(
    mut mouse_motion: MessageReader<MouseMotion>,
    mut scroll: MessageReader<MouseWheel>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut OrbitCamera, &mut Transform, &mut Projection)>,
    egui_wants: Option<Res<EguiWantsInput>>,
    drag_state: Option<Res<crate::picking::DragState>>,
) {
    // Don't orbit when egui wants the pointer
    if let Some(wants) = egui_wants {
        if wants.wants_pointer_input() {
            mouse_motion.clear();
            scroll.clear();
            return;
        }
    }

    // Don't orbit when a drag handle is active
    if drag_state.as_ref().is_some_and(|d| d.active) {
        mouse_motion.clear();
        return;
    }

    let (mut orbit, mut transform, mut projection) = match query.single_mut() {
        Ok(q) => q,
        Err(_) => return,
    };

    // Consume one-shot frame selection
    if let Some(target) = orbit.frame_target.take() {
        orbit.target = target;
        orbit.distance = 3.0;
    }

    // Consume ortho toggle
    if orbit.toggle_ortho {
        orbit.toggle_ortho = false;
        *projection = match *projection {
            Projection::Perspective(_) => {
                let mut ortho = OrthographicProjection::default_3d();
                ortho.scale = orbit.distance * 0.3;
                Projection::Orthographic(ortho)
            }
            _ => Projection::Perspective(PerspectiveProjection::default()),
        };
    }

    // Rotate on left mouse drag (suppressed when drag handle is active)
    if mouse_button.pressed(MouseButton::Left) {
        for ev in mouse_motion.read() {
            orbit.yaw -= ev.delta.x * 0.005;
            orbit.pitch -= ev.delta.y * 0.005;
            orbit.pitch = orbit.pitch.clamp(-1.5, 1.5);
        }
    } else if mouse_button.pressed(MouseButton::Middle) {
        // Pan on middle mouse drag
        let pan_speed = orbit.distance * 0.001;
        for ev in mouse_motion.read() {
            let right = transform.right();
            let up = transform.up();
            orbit.target -= right * ev.delta.x * pan_speed;
            orbit.target += up * ev.delta.y * pan_speed;
        }
    } else {
        mouse_motion.clear();
    }

    // Zoom on scroll
    for ev in scroll.read() {
        let amount = match ev.unit {
            MouseScrollUnit::Line => ev.y * 0.5,
            MouseScrollUnit::Pixel => ev.y * 0.005,
        };
        orbit.distance = (orbit.distance - amount).clamp(0.5, 50.0);
    }

    // Apply orbit to transform
    let rot = Quat::from_euler(EulerRot::YXZ, orbit.yaw, orbit.pitch, 0.0);
    let cam_offset = rot * Vec3::new(0.0, 0.0, orbit.distance);
    transform.translation = orbit.target + cam_offset;
    transform.look_at(orbit.target, Vec3::Y);

    // Shift camera laterally to keep the target centered in the visible viewport area.
    // viewport_offset is [-0.5, 0.5] where (0,0) = panels evenly distributed.
    let vp = orbit.viewport_offset;
    if vp[0].abs() > 0.001 || vp[1].abs() > 0.001 {
        let right = transform.right();
        let up = transform.up();
        // Scale by distance for consistent shift regardless of zoom level
        let shift = right * (-vp[0] * orbit.distance * 2.0)
                  + up * (vp[1] * orbit.distance * 2.0);
        transform.translation += shift;
    }
}
