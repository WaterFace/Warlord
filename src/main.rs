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
    mut starfields: ResMut<Assets<StarfieldMaterial>>,
) {
    let distant_stars = starfields.add(StarfieldMaterial {
        scale: 50.0,
        ramp_cutoff: 0.85,
        octaves: 1,
        lacunarity: 1.0,
        gain: 1.0,

        brightness_scale: 10.0,
        brightness_octaves: 8,
        brightness_lacunarity: 0.5,
        brightness_gain: 1.0,

        brightness: 10.0,

        parallax_factor: 0.25,
        camera_position: Vec2::ZERO,

        resolution: Vec2::ZERO,
    });

    commands.spawn(StarfieldBundle {
        mesh: meshes.add(shape::Quad::default().into()),
        material: distant_stars,
        transform: Transform::from_xyz(0.0, 0.0, -1.0),
        ..Default::default()
    });

    // commands.spawn(StarfieldBundle {
    //     mesh: meshes.add(shape::Quad::default().into()),
    //     material: mid_stars,
    //     transform: Transform::from_xyz(0.0, 0.0, -0.5),
    //     ..Default::default()
    // });

    // commands.spawn(StarfieldBundle {
    //     mesh: meshes.add(shape::Quad::default().into()),
    //     material: dust_mat,
    //     transform: Transform::from_xyz(0.0, 0.0, 2.0),
    //     ..Default::default()
    // });

    commands.spawn(StarfieldCameraBundle {
        ..Default::default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
        ..Default::default()
    });

    let player = commands
        .spawn(player::PlayerBundle {
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
