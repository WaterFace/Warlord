use bevy::{
    prelude::*,
    render::{
        camera::ScalingMode,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use bytemuck::pod_align_to;

mod camera;
mod parallax;
mod physics;
mod player;
mod starfield_image;

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let star_field = images.add(starfield_image::BasicStarField::default().build(Extent3d {
        width: 1024,
        height: 1024,
        ..Default::default()
    }));

    let star_field_mat = materials.add(StandardMaterial {
        base_color_texture: Some(star_field.clone()),
        emissive_texture: Some(star_field.clone()),
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(
            shape::Quad {
                size: Vec2::new(100.0, 100.0),
                ..Default::default()
            }
            .into(),
        ),
        material: star_field_mat,
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

    commands
        .spawn((
            Camera3dBundle {
                projection: Projection::Orthographic(OrthographicProjection {
                    scale: 10.0,
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
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(physics::PhysicsPlugin::default())
        .add_plugin(player::PlayerPlugin)
        .add_plugin(camera::CameraPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup)
        .run();
}
