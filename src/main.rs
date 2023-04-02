use bevy::{
    prelude::*,
    render::{camera::ScalingMode, render_resource::Extent3d},
};

mod camera;
mod parallax;
mod physics;
mod player;
mod rock;
mod starfield_image;

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
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(physics::PhysicsPlugin::default())
        .add_plugin(player::PlayerPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(parallax::ParallaxPlugin)
        .add_plugin(rock::RockPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup)
        .run();
}
