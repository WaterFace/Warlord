use bevy::prelude::*;

use crate::camera::MainCamera;

#[derive(Component, Debug, Default)]
pub struct ParallaxLayer {
    pub grid_size: usize,
    pub tile_size: Vec2,
    pub image: Handle<Image>,
    pub parallax_factor: f32,
    tiles: Vec<Entity>,
}

impl ParallaxLayer {
    pub fn with_image(
        grid_size: usize,
        tile_size: Vec2,
        parallax_factor: f32,
        image: Handle<Image>,
    ) -> Self {
        Self {
            grid_size,
            tile_size,
            image,
            parallax_factor,
            tiles: vec![],
        }
    }
}

#[derive(Component)]
struct ParallaxTile {
    start_pos: Vec2,
}

fn add_parallax_layer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&mut ParallaxLayer, &Transform), Added<ParallaxLayer>>,
) {
    for (mut layer, layer_transform) in &mut query {
        let tile_mesh = meshes.add(
            shape::Quad {
                size: layer.tile_size,
                ..Default::default()
            }
            .into(),
        );
        let tile_mat = materials.add(StandardMaterial {
            base_color_texture: Some(layer.image.clone()),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        });

        // offset from the base position as a multiple of tile_size.x, and tile_size.y respectively
        let start = Vec2::splat(-(layer.grid_size as f32 / 2.0 - 0.5));

        for i in 0..layer.grid_size * layer.grid_size {
            let x = i % layer.grid_size;
            let y = i / layer.grid_size;

            let transform = Transform::from_xyz(
                (start.x + x as f32) * layer.tile_size.x,
                (start.y + y as f32) * layer.tile_size.y,
                layer_transform.translation.z,
            )
            .mul_transform(*layer_transform);

            let tile = commands
                .spawn((
                    ParallaxTile {
                        start_pos: transform.translation.truncate(),
                    },
                    PbrBundle {
                        material: tile_mat.clone(),
                        // material: materials.add(
                        //     Color::rgba(
                        //         x as f32 / (layer.grid_size) as f32,
                        //         y as f32 / (layer.grid_size) as f32,
                        //         i as f32 / (layer.grid_size * layer.grid_size) as f32,
                        //         1.0,
                        //     )
                        //     .into(),
                        // ),
                        mesh: tile_mesh.clone(),
                        transform,
                        ..Default::default()
                    },
                ))
                .id();

            layer.tiles.push(tile);
            info!("Added parallax tile {tile:?}");
        }
    }
}

fn apply_parallax(
    layer_query: Query<&ParallaxLayer>,
    mut tile_query: Query<(&mut Transform, &mut ParallaxTile), Without<MainCamera>>,
    camera_query: Query<&Transform, With<MainCamera>>,
) {
    let main_camera = camera_query.single();

    for layer in &layer_query {
        for tile_entity in &layer.tiles {
            let Ok((mut tile_transform, mut tile)) = tile_query.get_mut(*tile_entity) else {
                info!("Couldn't find a parallax tile: {tile_entity:?}");
                continue;
            };

            let real_displacement = main_camera.translation * (1.0 - layer.parallax_factor);
            let distance = main_camera.translation * layer.parallax_factor;

            tile_transform.translation.x = tile.start_pos.x + distance.x;
            tile_transform.translation.y = tile.start_pos.y + distance.y;

            if real_displacement.x > tile.start_pos.x + layer.tile_size.x / 2.0 {
                tile.start_pos.x += layer.tile_size.x * layer.grid_size as f32;
            } else if real_displacement.x < tile.start_pos.x - layer.tile_size.x / 2.0 {
                tile.start_pos.x -= layer.tile_size.x * layer.grid_size as f32;
            }
            if real_displacement.y > tile.start_pos.y + layer.tile_size.y / 2.0 {
                tile.start_pos.y += layer.tile_size.y * layer.grid_size as f32;
            } else if real_displacement.y < tile.start_pos.y - layer.tile_size.y / 2.0 {
                tile.start_pos.y -= layer.tile_size.y * layer.grid_size as f32;
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct ParallaxPlugin;

impl Plugin for ParallaxPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(add_parallax_layer)
            .add_system(apply_parallax);
    }
}
