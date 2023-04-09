use std::f32::consts::PI;

use bevy::{
    prelude::*,
    render::camera::RenderTarget,
    window::{PrimaryWindow, WindowRef},
};
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{
    camera::{FocusPoint, MainCamera},
    heat::Heat,
    inventory::Inventory,
    shield::ShieldEmitter,
    state::GameState,
    weapon::{CargoDumper, MainGun},
};

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
    pub main_gun: MainGun,
    pub heat: Heat,
    pub inventory: Inventory,
    pub shield_emitter: ShieldEmitter,
    pub cargo_dumper: CargoDumper,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub action_state: ActionState<crate::input::Action>,
    pub input_map: InputMap<crate::input::Action>,
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
            main_gun: MainGun::default(),
            heat: Heat::default(),
            shield_emitter: ShieldEmitter::default(),
            cargo_dumper: CargoDumper::default(),
            inventory: Inventory::default(),
            visibility: Visibility::Visible,
            computed_visibility: ComputedVisibility::default(),
            action_state: ActionState::default(),
            input_map: crate::input::default_input_map(),
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
    let Ok((main_camera, camera_transform)) = camera_query.get_single() else { return };
    let Ok((mut player, mut focus_point, player_transform)) = query.get_single_mut() else { return };

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

pub struct PlayerMoveEvent {
    pub position: Vec3,
}

fn move_player(
    mut query: Query<(
        &Player,
        &Velocity,
        &mut ExternalImpulse,
        &ActionState<crate::input::Action>,
        &Transform,
    )>,
    time: Res<Time>,
    mut writer: EventWriter<PlayerMoveEvent>,
) {
    for (player, velocity, mut ext_impulse, action_state, transform) in &mut query {
        let mut desired_thrust = Vec2::ZERO;
        desired_thrust += Vec2::Y
            * action_state
                .value(crate::input::Action::MoveUp)
                .clamp(0.0, 1.0);
        desired_thrust += Vec2::NEG_Y
            * action_state
                .value(crate::input::Action::MoveDown)
                .clamp(0.0, 1.0);
        desired_thrust += Vec2::X
            * action_state
                .value(crate::input::Action::MoveRight)
                .clamp(0.0, 1.0);
        desired_thrust += Vec2::NEG_X
            * action_state
                .value(crate::input::Action::MoveLeft)
                .clamp(0.0, 1.0);
        desired_thrust = desired_thrust.normalize_or_zero();

        // let direction = Vec2::new(f32::cos(player.facing), f32::sin(player.facing));
        let desired_velocity = desired_thrust * player.max_speed;

        let accel_needed = desired_velocity - velocity.linvel;
        ext_impulse.impulse +=
            accel_needed.normalize_or_zero() * player.acceleration * time.delta_seconds();

        if desired_thrust.length_squared() > 0.0 {
            writer.send(PlayerMoveEvent {
                position: transform.translation,
            });
        }
    }
}

#[derive(Component, Debug)]
struct PlayerModel {
    pub base_angvel: Vec3,
    pub current_angvel: Vec3,
}

impl Default for PlayerModel {
    fn default() -> Self {
        Self {
            base_angvel: Vec3::new(3.5, 2.3, 1.2),
            current_angvel: Vec3::new(3.5, 2.3, 1.2),
        }
    }
}

#[derive(Resource, Debug, Default)]
struct PlayerModelHandles {
    pub body_mesh: Handle<Mesh>,
    pub body_mat: Handle<StandardMaterial>,

    pub light_mesh: Handle<Mesh>,
    pub light_mat: Handle<StandardMaterial>,
}

fn setup_player_model_handles(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let body_mat = materials.add(StandardMaterial {
        base_color: Color::rgb(0.2, 0.2, 0.2),
        metallic: 0.99,
        perceptual_roughness: 0.2,
        ..Default::default()
    });
    let body_mesh = meshes.add(
        shape::UVSphere {
            radius: 1.0,
            ..Default::default()
        }
        .into(),
    );

    let light_mesh = meshes.add(
        shape::UVSphere {
            radius: 0.1,
            sectors: 12,
            stacks: 8,
        }
        .into(),
    );
    let light_mat = materials.add(StandardMaterial {
        base_color: Color::RED,
        ..Default::default()
    });

    commands.insert_resource(PlayerModelHandles {
        body_mat,
        body_mesh,
        light_mat,
        light_mesh,
    });
}

#[derive(Component, Debug, Default)]
struct PlayerModelLight;

fn setup_player_model(
    mut commands: Commands,
    query: Query<Entity, Added<Player>>,
    handles: Res<PlayerModelHandles>,
) {
    let Ok(player) = query.get_single() else { return; };
    debug!("Player component added to entity {player:?}");

    debug!("Adding base model to player");
    commands.entity(player).with_children(|parent| {
        parent
            .spawn((
                PbrBundle {
                    mesh: handles.body_mesh.clone(),
                    material: handles.body_mat.clone(),
                    ..Default::default()
                },
                PlayerModel::default(),
            ))
            .with_children(|parent| {
                const NUM_LIGHTS: u32 = 50;
                let phi: f32 = PI * (f32::sqrt(5.0) - 1.0);

                for i in 0..NUM_LIGHTS {
                    let y = 1.0 - (i as f32 / (NUM_LIGHTS - 1) as f32) * 2.0;
                    let radius = f32::sqrt(1.0 - y * y);

                    let theta = phi * i as f32;

                    let x = f32::cos(theta) * radius;
                    let z = f32::sin(theta) * radius;

                    debug!("Adding light to player base model");
                    parent.spawn((
                        PbrBundle {
                            mesh: handles.light_mesh.clone(),
                            material: handles.light_mat.clone(),
                            transform: Transform::from_xyz(x, y, z),
                            ..Default::default()
                        },
                        PlayerModelLight,
                    ));
                }
            });
    });
}

fn rotate_player_model(mut query: Query<(&PlayerModel, &mut Transform)>, time: Res<Time>) {
    for (player_model, mut transform) in &mut query {
        let rot = Quat::from_euler(
            EulerRot::YZX,
            player_model.current_angvel.y * time.delta_seconds(),
            player_model.current_angvel.z * time.delta_seconds(),
            player_model.current_angvel.x * time.delta_seconds(),
        );

        transform.rotate(rot);
    }
}

fn player_model_heat_effect(
    heat_query: Query<&Heat, Without<PlayerModel>>,
    mut model_query: Query<&mut PlayerModel>,
    handles: Res<PlayerModelHandles>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(heat) = heat_query.get_single() else { return; };
    let Ok(mut player_model) = model_query.get_single_mut() else { return; };

    const BASE_COLOR: Color = Color::GRAY;
    const HOT_COLOR: Color = Color::rgb(15.0, 5.0, 1.0);

    const ROTATION_FACTOR: f32 = 5.0;

    let Some(mut light_mat) = materials.get_mut(&handles.light_mat) else { return; };
    let t = heat.fraction();

    light_mat.base_color = BASE_COLOR * (1.0 - t) + HOT_COLOR * t;

    player_model.current_angvel = player_model.base_angvel * (1.0 + t * ROTATION_FACTOR);
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerMoveEvent>()
            .add_startup_system(setup_player_model_handles)
            .add_systems(
                (rotate_player, player_friction, move_player)
                    .chain()
                    .in_set(OnUpdate(GameState::InGame)),
            )
            .add_systems(
                (
                    setup_player_model,
                    rotate_player_model,
                    player_model_heat_effect,
                )
                    .in_set(OnUpdate(GameState::InGame)),
            );
    }
}
