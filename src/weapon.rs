use std::{f32::consts::PI, time::Duration};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{heat::Heat, player::Player};

#[derive(Component, Debug)]
pub struct MainGun {
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
struct Slug {
    pub timer: Timer,
}

#[derive(Resource, Debug, Default)]
struct SlugVisuals {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

fn setup_slug(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
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

fn kill_slug(mut commands: Commands, query: Query<(Entity, &Slug)>) {
    for (e, slug) in &query {
        if slug.timer.finished() {
            commands.entity(e).despawn_recursive();
        }
    }
}

fn tick_gun_timer(mut query: Query<&mut MainGun>, time: Res<Time>) {
    for mut gun in &mut query {
        gun.delay_timer
            .tick(Duration::from_secs_f32(time.delta_seconds()));
    }
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
    )>,
    input: Res<Input<MouseButton>>,
    slug_visuals: Res<SlugVisuals>,
) {
    for (player, mut main_gun, mut heat, transform, mut ext_impulse, player_velocity) in
        &mut player_query
    {
        if !input.pressed(MouseButton::Left) {
            return;
        }
        if !main_gun.delay_timer.finished() {
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
                radius: 2.0,
                ..Default::default()
            },
            PbrBundle {
                transform: Transform::from_xyz(pos.x, pos.y, transform.translation().z)
                    .with_rotation(rot),
                mesh: slug_visuals.mesh.clone(),
                material: slug_visuals.material.clone(),
                ..Default::default()
            },
        ));

        ext_impulse.impulse += -facing_dir * main_gun.recoil;

        heat.add(main_gun.heat_generated);

        let delay = Duration::from_secs_f32(main_gun.fire_delay);
        main_gun.delay_timer.reset();
        main_gun.delay_timer.set_duration(delay);
    }
}

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_slug)
            .add_systems((tick_slug, kill_slug).chain())
            .add_systems((tick_gun_timer, fire_main_gun).chain());
    }
}
