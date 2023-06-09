use std::{f32::consts::PI, time::Duration};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    collectible::{Collectible, CollectibleBundle, ExoticMatter, ExoticMatterAppearance},
    heat::Heat,
    input::Action,
    inventory::{Inventory, Reagent},
    player::Player,
    rock::RotatingRock,
    sound::SoundEvent,
    state::GameState,
    util::{random_direction, random_range},
};

#[derive(Component, Debug)]
pub struct MainGun {
    pub enabled: bool,
    pub fire_delay: f32,
    pub delay_timer: Timer,
    pub recoil: f32,
    pub projectile_speed: f32,
    pub max_projectile_distance: f32,
    pub origin_distance: f32,
    pub heat_generated: f32,
}

impl Default for MainGun {
    fn default() -> Self {
        Self {
            enabled: false,
            fire_delay: 0.33,
            delay_timer: Timer::from_seconds(0.0, TimerMode::Once),
            recoil: 5.0,
            projectile_speed: 45.0,
            max_projectile_distance: 15.0,
            origin_distance: 1.5,
            heat_generated: 8.0,
        }
    }
}

#[derive(Component, Debug)]
pub struct Slug {
    pub timer: Timer,
}

#[derive(Resource, Debug, Default)]
struct SlugVisuals {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

fn setup_slug_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::ORANGE_RED * 5.0,
        emissive: Color::rgb(5.0, 5.0, 5.0),
        ..Default::default()
    });
    let mesh = meshes.add(
        shape::Capsule {
            depth: 0.5,
            radius: 0.1,
            ..Default::default()
        }
        .into(),
    );
    commands.insert_resource(SlugVisuals { material, mesh });
}

fn tick_slug(mut query: Query<&mut Slug>, time: Res<Time>) {
    for mut slug in &mut query {
        slug.timer
            .tick(Duration::from_secs_f32(time.delta_seconds()));
    }
}

pub struct SlugDecayedEvent {
    pub position: Vec3,
    pub velocity: Vec2,
}

fn kill_slug(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Velocity, &Slug)>,
    mut writer: EventWriter<SlugDecayedEvent>,
) {
    for (e, transform, velocity, slug) in &query {
        if slug.timer.finished() {
            commands.entity(e).despawn_recursive();
            writer.send(SlugDecayedEvent {
                position: transform.translation,
                velocity: velocity.linvel,
            });
        }
    }
}

fn tick_gun_timer(mut query: Query<&mut MainGun>, time: Res<Time>) {
    for mut gun in &mut query {
        gun.delay_timer
            .tick(Duration::from_secs_f32(time.delta_seconds()));
    }
}

pub struct FireMainGunEvent {
    pub position: Vec3,
    pub facing: f32,
}

fn fire_main_gun(
    mut commands: Commands,
    mut player_query: Query<(
        &Player,
        &mut MainGun,
        &mut Heat,
        &GlobalTransform,
        &mut ExternalImpulse,
        &Velocity,
        &ActionState<crate::input::Action>,
    )>,
    slug_visuals: Res<SlugVisuals>,
    mut gun_event_writer: EventWriter<FireMainGunEvent>,
    mut sound_event_writer: EventWriter<SoundEvent>,
) {
    for (
        player,
        mut main_gun,
        mut heat,
        transform,
        mut ext_impulse,
        player_velocity,
        action_state,
    ) in &mut player_query
    {
        if !main_gun.enabled {
            // main gun not enabled
            return;
        }
        if action_state.value(crate::input::Action::FireMainGun) <= 0.0 {
            // Not pressing the fire input
            return;
        }
        if !main_gun.delay_timer.finished() {
            // not ready to fire the next shot yet
            return;
        }
        if heat.limit() - heat.current() < main_gun.heat_generated {
            // prevent firing if we're overheated
            return;
        }

        let facing_dir = Vec2::from_angle(player.facing);
        let pos = transform.translation().truncate() + facing_dir * main_gun.origin_distance;
        let rot = Quat::from_rotation_z(PI / 2.0 + player.facing);

        let time_to_live = main_gun.max_projectile_distance / main_gun.projectile_speed;

        let velocity = facing_dir * main_gun.projectile_speed + player_velocity.linvel;

        commands.spawn((
            Slug {
                timer: Timer::from_seconds(time_to_live, TimerMode::Once),
            },
            Velocity::linear(velocity),
            RigidBody::Dynamic,
            AdditionalMassProperties::Mass(10.0),
            Collider::capsule_y(0.25, 0.1),
            Ccd::enabled(),
            PointLight {
                color: Color::ORANGE_RED,
                intensity: 4000.0,
                radius: 5.0,
                ..Default::default()
            },
            ActiveEvents::COLLISION_EVENTS,
            PbrBundle {
                transform: Transform::from_xyz(pos.x, pos.y, transform.translation().z)
                    .with_rotation(rot),
                mesh: slug_visuals.mesh.clone(),
                material: slug_visuals.material.clone(),
                ..Default::default()
            },
        ));

        gun_event_writer.send(FireMainGunEvent {
            position: Vec3::new(pos.x, pos.y, transform.translation().z),
            facing: player.facing,
        });

        sound_event_writer.send(SoundEvent::CannonFire {
            direction: player.facing,
        });

        ext_impulse.impulse += -facing_dir * main_gun.recoil;

        heat.add(main_gun.heat_generated);

        let delay = Duration::from_secs_f32(main_gun.fire_delay);
        main_gun.delay_timer.reset();
        main_gun.delay_timer.set_duration(delay);
    }
}

#[derive(Component, Debug, Default)]
pub struct CargoDumper {
    pub enabled: bool,
}

fn dump_cargo(
    mut commands: Commands,
    mut query: Query<(
        &Player,
        &CargoDumper,
        &Transform,
        &Velocity,
        &mut Inventory,
        &ActionState<Action>,
    )>,
    exotic_matter_appearance: Res<ExoticMatterAppearance>,
) {
    for (player, cargo_dumper, transform, velocity, mut inventory, action_state) in &mut query {
        if !cargo_dumper.enabled {
            continue;
        }
        if !action_state.just_pressed(Action::DumpCargo) {
            continue;
        }

        let amount = inventory.reagent(Reagent::Exotic).current();
        let num_chunks = amount as u32;
        let facing_dir = Vec2::from_angle(player.facing);
        let pos = transform.translation.truncate() + facing_dir * 3.0;
        if num_chunks > 0 {
            let amount_per_chunk = amount / num_chunks as f32;
            inventory.reagent_mut(Reagent::Exotic).add(-amount);

            for _ in 0..num_chunks {
                let linvel = facing_dir * 3.0 + velocity.linvel + random_direction() * 1.5;
                let angvel = Vec3::new(
                    random_range(-PI, PI),
                    random_range(-PI, PI),
                    random_range(-PI, PI),
                );
                commands
                    .spawn((
                        CollectibleBundle {
                            transform: Transform::from_xyz(pos.x, pos.y, transform.translation.z),
                            velocity: Velocity::linear(linvel),
                            collectible: Collectible::CollectibleReagent {
                                reagent: Reagent::Exotic,
                                amount: amount_per_chunk,
                            },
                            ..Default::default()
                        },
                        ExoticMatter::default(),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            RotatingRock { angvel },
                            PbrBundle {
                                mesh: exotic_matter_appearance.mesh.clone(),
                                material: exotic_matter_appearance.material.clone(),
                                visibility: Visibility::Visible,
                                ..Default::default()
                            },
                        ));
                    });
            }
        }
    }
}

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FireMainGunEvent>()
            .add_event::<SlugDecayedEvent>()
            .add_startup_system(setup_slug_visuals)
            .add_systems(
                (tick_slug, kill_slug)
                    .chain()
                    .in_set(OnUpdate(GameState::InGame)),
            )
            .add_systems(
                (tick_gun_timer, fire_main_gun)
                    .chain()
                    .in_set(OnUpdate(GameState::InGame)),
            )
            .add_system(dump_cargo.in_set(OnUpdate(GameState::InGame)));
    }
}
