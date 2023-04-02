use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    window::{PrimaryWindow, WindowRef},
};
use bevy_rapier2d::prelude::*;
use bytemuck::pod_align_to;

use crate::camera::{FocusPoint, MainCamera};

#[derive(Bundle, Debug)]
pub struct PlayerBundle {
    pub player: Player,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub locked_axes: LockedAxes,
    pub velocity: Velocity,
    pub external_impulse: ExternalImpulse,
    pub focus_point: FocusPoint,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player: Player::default(),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::ball(1.0),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            velocity: Velocity::default(),
            external_impulse: ExternalImpulse::default(),
            focus_point: FocusPoint::default(),
        }
    }
}

#[derive(Component, Debug)]
pub struct Player {
    pub facing: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub rotation_speed: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            facing: 0.0,
            max_speed: 15.0,
            acceleration: 30.0,
            rotation_speed: 180f32.to_radians(),
        }
    }
}

fn rotate_player(
    mut query: Query<(&mut Player, &mut FocusPoint, &GlobalTransform)>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    other_window_query: Query<&Window, Without<PrimaryWindow>>,
    time: Res<Time>,
) {
    let (main_camera, camera_transform) = camera_query.single();
    let Ok((mut player, mut focus_point, player_transform)) = query.get_single_mut() else {
        info!("get_single_mut didn't find exactly 1!");
        return;
    };

    let Some(window) = (match main_camera.target {
        RenderTarget::Window(window_ref) => match window_ref {
            WindowRef::Primary => primary_window_query.get_single().ok(),
            WindowRef::Entity(e) => other_window_query.get(e).ok(),
        }
        _ => return,
    }) else {
        return;
    };

    let mut desired_rotation = player.facing;

    if let Some(cursor_position) = window.cursor_position() {
        if let Some(world_pos) = main_camera.viewport_to_world_2d(camera_transform, cursor_position)
        {
            let dir = world_pos - player_transform.translation().truncate();
            desired_rotation = f32::atan2(dir.y, dir.x);

            // update the focus point
            let world_pos = Vec3::new(world_pos.x, world_pos.y, player_transform.translation().z);
            focus_point.offset = (world_pos - player_transform.translation()) * 0.25;
        }
    }
    let diff = Vec2::angle_between(
        Vec2::from_angle(player.facing),
        Vec2::from_angle(desired_rotation),
    );
    let rotation_amount = f32::abs(diff) * player.rotation_speed;
    player.facing += diff.signum() * rotation_amount * time.delta_seconds();
}

fn player_friction(mut query: Query<(&Player, &Velocity, &mut ExternalImpulse)>, time: Res<Time>) {
    if let Ok((_player, velocity, mut ext_impulse)) = query.get_single_mut() {
        const MAX_DECELERATION: f32 = 2.0; // TODO: make this configuarable
        let speed = velocity.linvel.length();
        let dir = velocity.linvel.normalize_or_zero();
        let deceleration = f32::min(MAX_DECELERATION, speed);
        ext_impulse.impulse += -dir * deceleration * time.delta_seconds();
    } else {
        info!("get_single_mut didn't find exactly 1!")
    }
}

fn move_player(
    mut query: Query<(&Player, &Velocity, &mut ExternalImpulse)>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let mut desired_thrust = 0.0;
    // TODO: support analog input
    if input.pressed(KeyCode::W) {
        desired_thrust += 1.0;
    }
    if input.pressed(KeyCode::S) {
        desired_thrust -= 1.0;
    }

    if let Ok((player, velocity, mut ext_impulse)) = query.get_single_mut() {
        let direction = Vec2::new(f32::cos(player.facing), f32::sin(player.facing));
        let desired_velocity = direction * desired_thrust * player.max_speed;

        let accel_needed = desired_velocity - velocity.linvel;
        ext_impulse.impulse +=
            accel_needed.normalize_or_zero() * player.acceleration * time.delta_seconds();
    } else {
        info!("get_single_mut didn't find exactly 1!")
    }
}

#[derive(Component)]
struct DebugThingy;

#[allow(dead_code)]
fn debug_facing(
    mut commands: Commands,
    query: Query<(&Player, &GlobalTransform)>,
    mut debug_marker_query: Query<&mut Transform, With<DebugThingy>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(mut debug_marker_transform) = debug_marker_query.get_single_mut()  else {
        info!("debug marker not found. Creating one for next frame...");
        commands.spawn(SpriteBundle {
            texture: images.add(Image::new_fill(
                Extent3d {
                    width: 16,
                    height: 16,
                    ..Default::default()
                },
                TextureDimension::D2,
                pod_align_to(&Color::RED.as_rgba_f32()).1,
                TextureFormat::Rgba32Float,
            )),
            ..Default::default()
        }).insert(DebugThingy);
        return;
    };

    if let Ok((player, player_transform)) = query.get_single() {
        let dir = Vec3::new(
            f32::cos(player.facing),
            f32::sin(player.facing),
            player_transform.translation().z,
        );
        debug_marker_transform.translation = player_transform.translation() + dir * 100.0;
    } else {
        info!("get_single didn't find exactly 1!")
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((rotate_player, player_friction, move_player).chain());
        app.add_system(debug_facing);
    }
}
