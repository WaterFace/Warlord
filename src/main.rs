use bevy::{prelude::*, render::render_resource::Extent3d};
mod starfield_image;

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn(Camera2dBundle {
        ..Default::default()
    });

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
}
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup)
        .run();
}
