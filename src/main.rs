use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bytemuck::pod_align_to;

mod camera;
mod physics;
mod player;
mod starfield_image;

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn((
        Camera2dBundle {
            ..Default::default()
        },
        camera::MainCamera,
    ));

    let star_field = images.add(starfield_image::BasicStarField::default().build(Extent3d {
        width: 1024,
        height: 1024,
        ..Default::default()
    }));

    commands.spawn(SpriteBundle {
        texture: star_field,
        transform: Transform::from_scale(Vec3::new(2.5, 2.5, 1.0)),
        ..Default::default()
    });

    commands
        .spawn(player::PlayerBundle::default())
        .insert(SpriteBundle {
            texture: images.add(Image::new_fill(
                Extent3d {
                    width: 64,
                    height: 64,
                    ..Default::default()
                },
                TextureDimension::D2,
                pod_align_to(&Color::GRAY.as_rgba_f32()).1,
                TextureFormat::Rgba32Float,
            )),
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..Default::default()
        });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(physics::PhysicsPlugin::default())
        .add_plugin(player::PlayerPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup)
        .run();
}
