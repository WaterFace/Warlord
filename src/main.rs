use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use camera::{MainCameraBundle, SmoothFollow};
use starfield_shader::{StarfieldBundle, StarfieldCameraBundle, StarfieldMaterial};

mod camera;
mod collectible;
mod heat;
mod inventory;
mod physics;
mod player;
mod reaction;
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
        .add_plugin(collectible::CollectiblePlugin)
        .add_plugin(inventory::InventoryPlugin)
        .add_plugin(reaction::ReactionPlugin)
        .add_startup_system(setup)
        .run();
}
