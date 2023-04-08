use std::{f32::consts::PI, time::Duration};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    player::PlayerMoveEvent,
    rock::RockDestroyed,
    state::GameState,
    util::{random_direction, random_in_circle, random_range},
    weapon::{FireMainGunEvent, SlugDecayedEvent},
};

#[derive(Component, Default, Clone)]
pub struct Particle {
    pub lifetime_timer: Timer,
}

#[derive(Bundle, Default, Clone)]
pub struct ParticleBundle {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,

    pub velocity: Velocity,
    pub rigid_body: RigidBody,

    pub particle: Particle,
}

#[derive(Resource, Default, Debug)]
struct ParticleHandles {
    pub player_move_particle_mat: Handle<StandardMaterial>,
    pub player_move_particle_mesh: Handle<Mesh>,

    pub fire_main_gun_particle_mat: Handle<StandardMaterial>,
    pub fire_main_gun_particle_mesh: Handle<Mesh>,

    pub slug_decayed_particle_mat: Handle<StandardMaterial>,
    pub slug_decayed_particle_mesh: Handle<Mesh>,

    pub rock_destroyed_particle_mat: Handle<StandardMaterial>,
    pub rock_destroyed_particle_mesh: Handle<Mesh>,
}

fn setup_particle_handles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player_move_particle_mat = materials.add(StandardMaterial {
        base_color: Color::PURPLE,
        emissive: Color::PURPLE,
        ..Default::default()
    });
    let player_move_particle_mesh = meshes.add(
        shape::Icosphere {
            subdivisions: 0,
            radius: 0.2,
        }
        .try_into()
        .unwrap(),
    );

    let fire_main_gun_particle_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE * 15.0,
        emissive: Color::RED * 15.0,
        ..Default::default()
    });
    let fire_main_gun_particle_mesh = meshes.add(
        shape::Quad {
            size: Vec2::splat(0.1),
            ..Default::default()
        }
        .into(),
    );

    let slug_decayed_particle_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE * 15.0,
        emissive: Color::RED * 15.0,
        ..Default::default()
    });
    let slug_decayed_particle_mesh = meshes.add(
        shape::Quad {
            size: Vec2::splat(0.1),
            ..Default::default()
        }
        .into(),
    );

    let rock_destroyed_particle_mat = materials.add(StandardMaterial {
        base_color: Color::GRAY,
        ..Default::default()
    });
    let rock_destroyed_particle_mesh = meshes.add(
        shape::Quad {
            size: Vec2::splat(0.5),
            ..Default::default()
        }
        .into(),
    );

    commands.insert_resource(ParticleHandles {
        player_move_particle_mat,
        player_move_particle_mesh,

        fire_main_gun_particle_mat,
        fire_main_gun_particle_mesh,

        slug_decayed_particle_mat,
        slug_decayed_particle_mesh,

        rock_destroyed_particle_mat,
        rock_destroyed_particle_mesh,
    });
}

fn spawn_player_move_particles(
    mut commands: Commands,
    mut reader: EventReader<PlayerMoveEvent>,
    mut bundle: Local<Option<ParticleBundle>>,
    handles: Res<ParticleHandles>,
    mut cooldown: Local<Timer>,
    time: Res<Time>,
) {
    let bundle = match bundle.as_ref() {
        Some(b) => b.clone(),
        None => {
            let b = ParticleBundle {
                mesh: handles.player_move_particle_mesh.clone(),
                material: handles.player_move_particle_mat.clone(),
                ..Default::default()
            };
            *bundle = Some(b.clone());
            b
        }
    };
    if cooldown.duration().is_zero() {
        *cooldown = Timer::from_seconds(1.0 / 7.0, TimerMode::Repeating);
    }

    cooldown.tick(Duration::from_secs_f32(time.delta_seconds()));
    if cooldown.just_finished() {
        let Some(ev) = reader.iter().next() else { return };
        const RADIUS: f32 = 1.0;
        let pt = random_in_circle(RADIUS);
        let pos = ev.position + Vec3::new(pt.x, pt.y, 0.0);
        let vel = random_direction() * 0.3;
        let velocity = Velocity::linear(vel);
        let scale = random_range(0.95, 1.05);

        commands.spawn(ParticleBundle {
            particle: Particle {
                lifetime_timer: Timer::from_seconds(0.5, TimerMode::Once),
            },
            velocity,
            transform: Transform::from_translation(pos).with_scale(Vec3::splat(scale)),
            ..bundle
        });
    }
}

fn spawn_fire_main_gun_particles(
    mut commands: Commands,
    mut reader: EventReader<FireMainGunEvent>,
    mut bundle: Local<Option<ParticleBundle>>,
    handles: Res<ParticleHandles>,
) {
    let bundle = match bundle.as_ref() {
        Some(b) => b.clone(),
        None => {
            let b = ParticleBundle {
                mesh: handles.fire_main_gun_particle_mesh.clone(),
                material: handles.fire_main_gun_particle_mat.clone(),
                ..Default::default()
            };
            *bundle = Some(b.clone());
            b
        }
    };

    let Some(ev) = reader.iter().next() else { return };
    const NUM_PARTICLES: u32 = 15;
    for i in 0..NUM_PARTICLES {
        let pos = ev.position;
        let vel = Vec2::from_angle((i as f32 / NUM_PARTICLES as f32) * PI * 2.0) * 5.0;
        let spin = random_range(-PI, PI);
        let velocity = Velocity {
            linvel: vel,
            angvel: spin,
        };
        let scale = random_range(0.95, 1.05);

        commands.spawn(ParticleBundle {
            particle: Particle {
                lifetime_timer: Timer::from_seconds(0.5, TimerMode::Once),
            },
            velocity,
            transform: Transform::from_translation(pos).with_scale(Vec3::splat(scale)),
            ..bundle.clone()
        });
    }
}

fn spawn_slug_decayed_gun_particles(
    mut commands: Commands,
    mut reader: EventReader<SlugDecayedEvent>,
    mut bundle: Local<Option<ParticleBundle>>,
    handles: Res<ParticleHandles>,
) {
    let bundle = match bundle.as_ref() {
        Some(b) => b.clone(),
        None => {
            let b = ParticleBundle {
                mesh: handles.slug_decayed_particle_mesh.clone(),
                material: handles.slug_decayed_particle_mat.clone(),
                ..Default::default()
            };
            *bundle = Some(b.clone());
            b
        }
    };

    let Some(ev) = reader.iter().next() else { return };
    const NUM_PARTICLES_1: u32 = 16;
    for _ in 0..NUM_PARTICLES_1 {
        let pos = ev.position;
        let vel = ev.velocity / 3.0 + random_direction() * 2.0;
        let spin = random_range(-PI, PI);
        let velocity = Velocity {
            linvel: vel,
            angvel: spin,
        };
        let scale = random_range(0.95, 1.05);

        commands.spawn(ParticleBundle {
            particle: Particle {
                lifetime_timer: Timer::from_seconds(0.5, TimerMode::Once),
            },
            velocity,
            transform: Transform::from_translation(pos).with_scale(Vec3::splat(scale)),
            ..bundle.clone()
        });
    }

    const NUM_PARTICLES_2: u32 = 10;
    for _ in 0..NUM_PARTICLES_2 {
        let pos = ev.position;
        let vel = random_direction() * 2.0;
        let spin = random_range(-PI, PI);
        let velocity = Velocity {
            linvel: vel,
            angvel: spin,
        };
        let scale = random_range(0.95, 1.05);

        commands.spawn(ParticleBundle {
            particle: Particle {
                lifetime_timer: Timer::from_seconds(0.5, TimerMode::Once),
            },
            velocity,
            transform: Transform::from_translation(pos).with_scale(Vec3::splat(scale)),
            ..bundle.clone()
        });
    }
}

fn spawn_rock_destroyed_particles(
    mut commands: Commands,
    mut reader: EventReader<RockDestroyed>,
    mut bundle: Local<Option<ParticleBundle>>,
    handles: Res<ParticleHandles>,
) {
    let bundle = match bundle.as_ref() {
        Some(b) => b.clone(),
        None => {
            let b = ParticleBundle {
                mesh: handles.rock_destroyed_particle_mesh.clone(),
                material: handles.rock_destroyed_particle_mat.clone(),
                ..Default::default()
            };
            *bundle = Some(b.clone());
            b
        }
    };

    let Some(ev) = reader.iter().next() else { return };
    const NUM_PARTICLES: u32 = 8;
    for _ in 0..NUM_PARTICLES {
        let pos = ev.position;
        let vel = random_direction() * 5.0;
        let spin = random_range(-PI, PI);
        let velocity = Velocity {
            linvel: vel,
            angvel: spin,
        };
        let scale = random_range(0.95, 1.05);

        commands.spawn(ParticleBundle {
            particle: Particle {
                lifetime_timer: Timer::from_seconds(0.75, TimerMode::Once),
            },
            velocity,
            transform: Transform::from_translation(pos).with_scale(Vec3::splat(scale)),
            ..bundle.clone()
        });
    }
}

fn tick_particles(mut query: Query<&mut Particle>, time: Res<Time>) {
    for mut p in &mut query {
        p.lifetime_timer
            .tick(Duration::from_secs_f32(time.delta_seconds()));
    }
}

fn cull_particles(mut commands: Commands, query: Query<(Entity, &Particle)>) {
    for (e, p) in &query {
        if p.lifetime_timer.finished() {
            commands.entity(e).despawn_recursive();
        }
    }
}

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_particle_handles).add_systems(
            (
                spawn_player_move_particles,
                spawn_fire_main_gun_particles,
                spawn_slug_decayed_gun_particles,
                spawn_rock_destroyed_particles,
                tick_particles,
                cull_particles,
            )
                .in_set(OnUpdate(GameState::InGame)),
        );
    }
}
