use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_rapier2d::prelude::{Collider, RigidBody, Velocity};
use rand::distributions::uniform::SampleUniform;
use rand::Rng;

use crate::camera::MainCamera;
use crate::collectible::{Collectible, CollectibleBundle, MineralAppearance};
use crate::inventory::Reagent;

#[derive(Component, Debug, Default)]
pub struct Rock;

#[derive(Component, Debug)]
pub struct RockSpawner {
    /// The number of rocks in a cluster is drawn randomly from this range
    pub min_cluster_size: u32,
    pub max_cluster_size: u32,
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
    number_of_rocks: u32,
    centre_of_region: Vec2,
    chance_of_mineral: f32,
}

#[derive(Component, Default, Debug)]
struct RotatingRock {
    angvel: Vec3,
}

fn rotate_rocks(mut query: Query<(&mut Transform, &RotatingRock)>, time: Res<Time>) {
    for (mut transform, rotating_rock) in &mut query {
        let rot = Quat::from_euler(
            EulerRot::YZX,
            rotating_rock.angvel.y * time.delta_seconds(),
            rotating_rock.angvel.z * time.delta_seconds(),
            rotating_rock.angvel.x * time.delta_seconds(),
        );
        transform.rotate(rot);
    }
}

#[derive(Resource, Debug)]
struct RockLimit {
    current: u32,
    limit: u32,
}

impl Default for RockLimit {
    fn default() -> Self {
        Self {
            current: 0,
            limit: 150,
        }
    }
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

fn cull_far_away_entities(
    mut commands: Commands,
    query: Query<(Entity, &Cull, &GlobalTransform), Without<MainCamera>>,
    camera_query: Query<&GlobalTransform, With<MainCamera>>,
    mut rock_limit: ResMut<RockLimit>,
) {
    let main_camera = camera_query.single();
    for (e, cull, transform) in &query {
        let dist2 = Vec2::distance_squared(
            transform.translation().truncate(),
            main_camera.translation().truncate(),
        );
        if dist2 > cull.max_distance * cull.max_distance {
            commands.entity(e).despawn_recursive();
            rock_limit.current -= 1;
            debug!("Despawned entity {e:?}");
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
                chance_of_mineral: 0.05,
            });
        }
    }
}

fn spawn_first_cluster(mut writer: EventWriter<SpawnEvent>) {
    writer.send(SpawnEvent {
        number_of_rocks: 50,
        centre_of_region: Vec2::ZERO,
        chance_of_mineral: 0.05,
    });
}

fn spawn_rocks(
    mut commands: Commands,
    mut reader: EventReader<SpawnEvent>,
    rock_appearance: Res<RockAppearance>,
    mineral_appearance: Res<MineralAppearance>,
    mut rock_limit: ResMut<RockLimit>,
) {
    for SpawnEvent {
        number_of_rocks,
        centre_of_region,
        chance_of_mineral,
    } in reader.iter()
    {
        debug!("Trying to spawn a cluster of rocks at {centre_of_region:?} with {number_of_rocks} rocks.");
        if number_of_rocks + rock_limit.current > rock_limit.limit {
            debug!("Couldn't spawn {} rocks. There are currently {} rocks and that would exceed the limit of {}", number_of_rocks, rock_limit.current, rock_limit.limit);
            return;
        } else {
            rock_limit.current += number_of_rocks;
        }
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

            let velocity =
                Velocity::linear(Vec2::new(random_range(-1.0, 1.0), random_range(-1.0, 1.0)));

            // Spawn the visual component separately, so it can rotate in 3d
            // without interference from rapier
            let angvel = Vec3::new(
                random_range(-PI, PI),
                random_range(-PI, PI),
                random_range(-PI, PI),
            );
            let roll = random_range(0.0, 1.0);
            if roll > *chance_of_mineral {
                let rock_visuals = commands
                    .spawn((
                        RotatingRock { angvel },
                        PbrBundle {
                            mesh: rock_appearance.mesh.clone(),
                            material: rock_appearance.material.clone(),
                            visibility: Visibility::Visible,
                            ..Default::default()
                        },
                    ))
                    .id();

                commands
                    .spawn((
                        Rock,
                        RigidBody::Dynamic,
                        Collider::ball(f32::sqrt(3.0 / 4.0)),
                        velocity,
                        Cull::default(),
                        transform,
                        GlobalTransform::from(transform),
                        Visibility::Visible,
                        ComputedVisibility::default(),
                    ))
                    .add_child(rock_visuals);
            } else {
                debug!("Mineral spawned!");
                let mineral_visuals = commands
                    .spawn((
                        RotatingRock { angvel },
                        PbrBundle {
                            mesh: mineral_appearance.mesh.clone(),
                            material: mineral_appearance.material.clone(),
                            visibility: Visibility::Visible,
                            ..Default::default()
                        },
                    ))
                    .id();

                commands
                    .spawn(CollectibleBundle {
                        transform,
                        velocity,
                        collectible: Collectible::CollectibleReagent {
                            reagent: Reagent::Minerals,
                            amount: 1.0,
                        },
                        ..Default::default()
                    })
                    .add_child(mineral_visuals);
            }
        }
    }
}

fn setup_rock_appearance(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let side_length = 1.0;
    let rock_mat = materials.add(Color::DARK_GRAY.into());
    let rock_mesh = meshes.add(shape::Cube { size: side_length }.into());

    commands.insert_resource(RockAppearance {
        mesh: rock_mesh,
        material: rock_mat,
    });
}

pub struct RockPlugin;

impl Plugin for RockPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RockLimit::default()) // TODO: make the limit configurable from outside the plugin
            .add_startup_system(setup_rock_appearance)
            .add_startup_system(spawn_first_cluster)
            .add_event::<SpawnEvent>()
            .add_system(spawn_rocks_tick)
            .add_system(spawn_rocks)
            .add_system(cull_far_away_entities)
            .add_system(rotate_rocks);
    }
}
