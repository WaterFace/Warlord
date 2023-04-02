use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_rapier2d::prelude::{Collider, RigidBody};
use rand::distributions::uniform::SampleUniform;
use rand::Rng;

use crate::camera::MainCamera;

#[derive(Component, Debug, Default)]
pub struct Rock;

#[derive(Component, Debug)]
pub struct RockSpawner {
    /// The number of rocks in a cluster is drawn randomly from this range
    pub min_cluster_size: usize,
    pub max_cluster_size: usize,
    /// Clusters of rocks will spawn within this range of the main camera
    pub min_spawn_distance: f32,
    pub max_spawn_distance: f32,
    /// The RockSpawner tries to spawn rocks whenever this timer finishes
    pub spawn_timer: Timer,
}

impl Default for RockSpawner {
    fn default() -> Self {
        Self {
            min_cluster_size: 15,
            max_cluster_size: 25,
            min_spawn_distance: 35.0,
            max_spawn_distance: 50.0,
            spawn_timer: Timer::from_seconds(5.0, TimerMode::Repeating),
        }
    }
}

#[derive(Resource, Debug)]
struct RockAppearance {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

struct SpawnEvent {
    number_of_rocks: usize,
    centre_of_region: Vec2,
}

#[derive(Component, Debug)]
struct Cull {
    max_distance: f32,
}

impl Default for Cull {
    fn default() -> Self {
        Self { max_distance: 75.0 }
    }
}

fn cull_far_away_rocks(
    mut commands: Commands,
    query: Query<(Entity, &Cull, &GlobalTransform), Without<MainCamera>>,
    camera_query: Query<&GlobalTransform, With<MainCamera>>,
) {
    let main_camera = camera_query.single();
    for (e, cull, transform) in &query {
        let dist2 = Vec2::distance_squared(
            transform.translation().truncate(),
            main_camera.translation().truncate(),
        );
        if dist2 > cull.max_distance * cull.max_distance {
            commands.entity(e).despawn_recursive();
            info!("Despawned rock {e:?}");
        }
    }
}

fn random_direction() -> Vec2 {
    let mut rng = rand::thread_rng();
    let mut dir = Vec2::ZERO;
    while dir.length_squared() == 0.0 {
        dir = Vec2::new(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0));
    }
    dir.normalize()
}

fn random_range<T: SampleUniform + PartialOrd>(min: T, max: T) -> T {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}

fn spawn_rocks_tick(
    mut query: Query<&mut RockSpawner, Without<MainCamera>>,
    camera_query: Query<&GlobalTransform, With<MainCamera>>,
    time: Res<Time>,
    mut writer: EventWriter<SpawnEvent>,
) {
    let main_camera = camera_query.single();
    for mut spawner in &mut query {
        spawner
            .spawn_timer
            .tick(Duration::from_secs_f32(time.delta_seconds()));
        for _ in 0..spawner.spawn_timer.times_finished_this_tick() {
            let dir = random_direction();
            let dist = random_range(spawner.min_spawn_distance, spawner.max_spawn_distance);
            let num = random_range(spawner.min_cluster_size, spawner.max_cluster_size);
            writer.send(SpawnEvent {
                number_of_rocks: num,
                centre_of_region: dir * dist + main_camera.translation().truncate(),
            });
        }
    }
}

fn spawn_rocks(
    mut commands: Commands,
    mut reader: EventReader<SpawnEvent>,
    rock_appearance: Res<RockAppearance>,
) {
    for SpawnEvent {
        number_of_rocks,
        centre_of_region,
    } in reader.iter()
    {
        info!("Trying to spawn a cluster of rocks at {centre_of_region:?} with {number_of_rocks} rocks.");
        for _ in 0..*number_of_rocks {
            // Rocks are 1x1 cubes, so the total area of the rocks to be spawned is about
            // `number_of_rocks`. A circle of that area has the following radius.
            // Should tune this so rocks don't overlap too much
            let radius = 2.0 * f32::sqrt(*number_of_rocks as f32 * 4.0 / PI);
            let pos = loop {
                let x = random_range(-radius, radius);
                let y = random_range(-radius, radius);

                if x * x + y * y < radius * radius {
                    break Vec2::new(x, y);
                }
            };
            let rot = Quat::from_euler(
                EulerRot::XYZ,
                random_range(-PI, PI),
                random_range(-PI, PI),
                random_range(-PI, PI),
            );
            let transform =
                Transform::from_xyz(centre_of_region.x + pos.x, centre_of_region.y + pos.y, 3.0)
                    .with_rotation(rot);

            commands.spawn((
                Rock,
                RigidBody::Dynamic,
                Collider::cuboid(0.5, 0.5),
                Cull::default(),
                PbrBundle {
                    mesh: rock_appearance.mesh.clone(),
                    material: rock_appearance.material.clone(),
                    transform,
                    ..Default::default()
                },
            ));
        }
    }
}

fn setup_rock_appearance(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let rock_mat = materials.add(Color::DARK_GRAY.into());
    let rock_mesh = meshes.add(shape::Cube { size: 1.0 }.into());

    commands.insert_resource(RockAppearance {
        mesh: rock_mesh,
        material: rock_mat,
    });
}

pub struct RockPlugin;

impl Plugin for RockPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_rock_appearance)
            .add_event::<SpawnEvent>()
            .add_system(spawn_rocks_tick)
            .add_system(spawn_rocks)
            .add_system(cull_far_away_rocks);
    }
}
