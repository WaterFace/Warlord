use bevy::{
    core_pipeline::{bloom::BloomSettings, clear_color::ClearColorConfig},
    log::{Level, LogPlugin},
    prelude::*,
    render::{camera::ScalingMode, render_resource::Extent3d},
};

mod camera;
mod heat;
mod parallax;
mod physics;
mod player;
mod rock;
mod starfield_image;
mod ui;
mod weapon;

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let star_field = images.add(
        starfield_image::BasicStarField {
            density: 0.001,
            ..Default::default()
        }
        .build(Extent3d {
            width: 1024,
            height: 1024,
            ..Default::default()
        }),
    );

    commands.spawn((
        Transform::from_xyz(0.0, 0.0, -20.0),
        GlobalTransform::default(),
        ComputedVisibility::default(),
        parallax::ParallaxLayer::with_image(3, Vec2::splat(100.0), 0.9, star_field.clone()),
    ));

    commands.spawn((
        Transform::from_xyz(0.0, 0.0, -30.0),
        GlobalTransform::default(),
        ComputedVisibility::default(),
        parallax::ParallaxLayer::with_image(3, Vec2::splat(70.0), 0.99, star_field.clone()),
    ));

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

    commands
        .spawn((
            Camera3dBundle {
                projection: Projection::Orthographic(OrthographicProjection {
                    scale: 15.0,
                    scaling_mode: ScalingMode::FixedVertical(2.0),
                    ..Default::default()
                }),
                transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_to(Vec3::NEG_Z, Vec3::Y),
                camera: Camera {
                    hdr: true,
                    ..Default::default()
                },
                camera_3d: Camera3d {
                    clear_color: ClearColorConfig::Custom(Color::BLACK),
                    ..Default::default()
                },
                ..Default::default()
            },
            BloomSettings {
                ..Default::default()
            },
            camera::MainCamera,
        ))
        .insert(camera::SmoothFollow {
            target: Some(player),
            ..Default::default()
        });

    commands.spawn(rock::RockSpawner::default());
}

fn main() {
    let mut app = App::new();
    #[cfg(not(debug_assertions))]
    app.add_plugins(DefaultPlugins);
    #[cfg(debug_assertions)]
    app.add_plugins(DefaultPlugins.set(LogPlugin {
        filter: "error,warlord=debug".into(),
        level: Level::DEBUG,
    }));
    app.add_plugin(physics::PhysicsPlugin { debug: false })
        .add_plugin(player::PlayerPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(parallax::ParallaxPlugin)
        .add_plugin(rock::RockPlugin)
        .add_plugin(weapon::WeaponPlugin)
        .add_plugin(heat::HeatPlugin)
        .add_plugin(ui::UIPlugin)
        .add_startup_system(setup)
        .run();
}
