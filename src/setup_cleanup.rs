use bevy::prelude::*;

use crate::{
    camera::{MainCamera, MainCameraBundle, SmoothFollow},
    collectible::Collectible,
    particles::Particle,
    player::{self, Player},
    reaction::Reactions,
    rock::{Rock, RockLimit, RockSpawner},
    starfield_shader::{
        StarfieldBundle, StarfieldCamera, StarfieldCameraBundle, StarfieldMaterial, StarfieldMesh,
    },
    state::{GameState, ProgressStages},
    ui::CustomUICameraBundle,
    weapon::Slug,
};

fn setup_starfield(
    mut commands: Commands,
    mut starfields: ResMut<Assets<StarfieldMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let distant_stars = starfields.add(StarfieldMaterial {
        parallax_factor: 0.25,
        ..Default::default()
    });

    commands.spawn(StarfieldBundle {
        mesh: meshes.add(shape::Quad::default().into()),
        material: distant_stars,
        transform: Transform::from_xyz(0.0, 0.0, -1.0),
        ..Default::default()
    });

    commands.spawn(StarfieldCameraBundle {
        ..Default::default()
    });
}

fn cleanup_starfield(
    mut commands: Commands,
    query: Query<Entity, Or<(With<StarfieldMesh>, With<StarfieldCamera>)>>,
) {
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
}

fn setup_main_camera(mut commands: Commands) {
    commands.spawn(MainCameraBundle {
        ..Default::default()
    });
    // Don't need to cleanup, beacuse this should last for the liftime of the program
}

fn setup_player(
    mut commands: Commands,
    mut main_camera_query: Query<&mut SmoothFollow, With<MainCamera>>,
) {
    let player = commands
        .spawn(player::PlayerBundle {
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..Default::default()
        })
        .id();

    let Ok(mut smooth_follow) = main_camera_query.get_single_mut() else {
      error!("setup_player: Couldn't find main camera! Aborting.");
      panic!()
    };

    smooth_follow.target = Some(player);
}

fn cleanup_player(
    mut commands: Commands,
    query: Query<Entity, With<Player>>,
    mut main_camera_query: Query<&mut SmoothFollow, (With<MainCamera>, Without<Player>)>,
) {
    for e in &query {
        commands.entity(e).despawn_recursive();
    }

    let Ok(mut smooth_follow) = main_camera_query.get_single_mut() else {
      error!("cleanup_player: Couldn't find main camera! Aborting.");
      panic!()
    };

    smooth_follow.target = None;
}

fn cleanup_collectibles(mut commands: Commands, query: Query<Entity, With<Collectible>>) {
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
}

fn setup_rocks(mut commands: Commands) {
    commands.insert_resource(RockLimit::default());
    commands.spawn(RockSpawner::default());
}

fn cleanup_rocks(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Rock>, With<RockSpawner>)>>,
) {
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
}

fn cleanup_particles(mut commands: Commands, query: Query<Entity, With<Particle>>) {
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
}

fn setup_reactions(mut commands: Commands) {
    commands.insert_resource(Reactions::default());
    // No need to clean this up, this replaces the existing resource if it exists
}

fn setup_ui_camera(mut commands: Commands) {
    commands.spawn(CustomUICameraBundle::default());
    // Don't need to clean this up because it should live as long as the game does
}

// fn cleanup_ui_camera(mut commands: Commands, query: Query<Entity, With<CustomUICamera>>) {
//     for e in &query {
//         commands.entity(e).despawn_recursive();
//     }
// }

fn cleanup_weapons(mut commands: Commands, query: Query<Entity, With<Slug>>) {
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
}

fn setup_progress_stage(mut progress_stage: ResMut<NextState<ProgressStages>>) {
    progress_stage.set(ProgressStages::Exploration);
}

fn reset_progress_stage(mut progress_stage: ResMut<NextState<ProgressStages>>) {
    progress_stage.set(ProgressStages::None);
}

pub struct SetupCleanupPlugin;

impl Plugin for SetupCleanupPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_main_camera);
        app.add_system(reset_progress_stage.in_schedule(OnEnter(GameState::MainMenu)));
        app.add_systems(
            (
                setup_starfield,
                setup_rocks,
                setup_reactions,
                setup_player,
                setup_ui_camera,
                setup_progress_stage,
            )
                .in_schedule(OnExit(GameState::Intro)),
        );
        app.add_systems(
            (
                cleanup_starfield,
                cleanup_player,
                cleanup_collectibles,
                cleanup_rocks,
                cleanup_particles,
                cleanup_weapons,
            )
                .in_schedule(OnEnter(GameState::Outro)),
        );
    }
}
