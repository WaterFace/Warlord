use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use camera::{MainCameraBundle, SmoothFollow};
use starfield_shader::{StarfieldBundle, StarfieldCameraBundle, StarfieldMaterial};

mod camera;
mod heat;
mod physics;
mod player;
mod rock;
mod starfield_shader;
mod ui;
mod weapon;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut starfields: ResMut<Assets<StarfieldMaterial>>,
) {
    let starfield1_mat = starfields.add(StarfieldMaterial {
        scale: 300.0,
        ramp_cutoff: 0.9,
        octaves: 1,
        lacunarity: 1.0,
        gain: 1.0,

        brightness_scale: 10.0,
        brightness_octaves: 8,
        brightness_lacunarity: 0.5,
        brightness_gain: 1.0,

        brightness: 5.0,

        parallax_factor: 0.5,
        camera_position: Vec2::ZERO,
    });

    let dust_mat = starfields.add(StarfieldMaterial {
        scale: 5.0,
        ramp_cutoff: 0.0,
        octaves: 3,
        lacunarity: 2.1,
        gain: 0.5,

        brightness_scale: 30.0,
        brightness_octaves: 1,
        brightness_lacunarity: 2.5,
        brightness_gain: 1.0,

        brightness: 0.3,

        parallax_factor: 0.8,
        camera_position: Vec2::ZERO,
    });

    commands.spawn(StarfieldBundle {
        mesh: meshes.add(
            shape::Quad {
                size: Vec2::splat(100.0),
                ..Default::default()
            }
            .into(),
        ),
        material: starfield1_mat,
        transform: Transform::from_xyz(0.0, 0.0, -1.0),
        ..Default::default()
    });

    commands.spawn(StarfieldBundle {
        mesh: meshes.add(
            shape::Quad {
                size: Vec2::splat(100.0),
                ..Default::default()
            }
            .into(),
        ),
        material: dust_mat,
        transform: Transform::from_xyz(0.0, 0.0, 2.0),
        ..Default::default()
    });

    commands.spawn(StarfieldCameraBundle {
        ..Default::default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
        ..Default::default()
    });

    let player = commands
        .spawn(player::PlayerBundle::default())
        .insert(PbrBundle {
            mesh: meshes.add(
                shape::UVSphere {
                    radius: 1.0,
                    ..Default::default()
                }
                .into(),
            ),
            material: materials.add(Color::GRAY.into()),
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..Default::default()
        })
        .id();

    commands.spawn(MainCameraBundle {
        smooth_follow: SmoothFollow {
            target: Some(player),
            ..Default::default()
        },
        ..Default::default()
    });

    commands.spawn(rock::RockSpawner::default());
}

fn main() {
    let mut app = App::new();
    #[cfg(not(debug_assertions))]
    app.add_plugins(DefaultPlugins);
    #[cfg(debug_assertions)]
    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                filter: "error,warlord=debug".into(),
                level: Level::DEBUG,
            })
            .set(AssetPlugin {
                watch_for_changes: true,
                ..Default::default()
            }),
    );
    app.add_plugin(physics::PhysicsPlugin { debug: false })
        .add_plugin(starfield_shader::StarfieldShaderPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(rock::RockPlugin)
        .add_plugin(weapon::WeaponPlugin)
        .add_plugin(heat::HeatPlugin)
        .add_plugin(ui::UIPlugin)
        .add_startup_system(setup)
        .run();
}
