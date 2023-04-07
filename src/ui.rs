use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{
        camera::CameraOutputMode,
        render_resource::{BlendState, LoadOp},
        view::RenderLayers,
    },
    sprite::Anchor,
    text::Text2dBounds,
};

use crate::{
    heat::Heat,
    inventory::{Inventory, Reagent},
    player::Player,
};

#[derive(Component, Debug, Default)]
pub struct CustomUICamera;

fn setup_ui_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
            },
            camera: Camera {
                order: 2, // Main camera has order 0 by default, higher orders render after (on top) of that
                output_mode: CameraOutputMode::Write {
                    // Gotta do this stuff to not clear the previous camera's work
                    blend_state: Some(BlendState::ALPHA_BLENDING),
                    color_attachment_load_op: LoadOp::Load,
                },
                ..Default::default()
            },
            ..Default::default()
        },
        RenderLayers::layer(1), // 0 is the main layer, 1 is the ui layer
        CustomUICamera,
    ));
}

#[derive(Component, Debug, Default)]
struct CurrentHeatBar {
    // gradient: Vec<(f32, Color)>,
}

#[derive(Component, Debug, Default)]
struct HeatBarAnchor;

fn setup_heat_display(mut commands: Commands, assets_server: Res<AssetServer>) {
    setup_ui_bar(
        &mut commands,
        &assets_server,
        HeatBarAnchor,
        CurrentHeatBar::default(),
        "HEAT",
        Color::RED,
        Color::WHITE,
        Some(0.75), // TODO: hook this up so it matches with the real threshold
    );
}

fn update_heat_bar(
    mut heat_bar_query: Query<&mut Transform, With<CurrentHeatBar>>,
    player_query: Query<&Heat, (With<Player>, Without<CurrentHeatBar>)>,
) {
    let player_heat = player_query.single();
    for mut transform in &mut heat_bar_query {
        transform.scale.x = player_heat.fraction();
    }
}

fn reposition_heat_bar(
    mut heat_bar_query: Query<&mut Transform, (With<HeatBarAnchor>, Without<CustomUICamera>)>,
    ui_camera: Query<&Camera, (Changed<Camera>, With<CustomUICamera>)>,
) {
    let Ok(ui_camera) = ui_camera.get_single() else {return;};
    let Some((top_left, _)) = ui_camera.logical_viewport_rect() else {return;};
    let Some(size) = ui_camera.logical_viewport_size() else {return;};
    let top_left = top_left + Vec2::new(-size.x / 2.0, size.y / 2.0);
    debug!("top left = {}", top_left);
    for mut transform in &mut heat_bar_query {
        transform.translation.x = top_left.x;
        transform.translation.y = top_left.y - BAR_PADDING;
    }
}

#[derive(Component, Debug)]
struct CurrentReagentBar {
    reagent: Reagent,
}

#[derive(Component, Debug)]
struct ReagentBarAnchor {
    reagent: Reagent,
}

const FONT_HEIGHT: f32 = 40.0;
const BAR_LENGTH: f32 = 250.0;
const BAR_PADDING: f32 = 4.0;

fn setup_reagent_bars(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    inventory_query: Query<&Inventory, Added<Inventory>>,
) {
    let Ok(inventory) = inventory_query.get_single() else { return; };
    for (reagent, entry) in inventory.reagents() {
        setup_ui_bar(
            &mut commands,
            &assets_server,
            ReagentBarAnchor { reagent },
            CurrentReagentBar { reagent },
            entry.name(),
            entry.color(),
            Color::WHITE,
            None,
        );
    }
    debug!("Finished setting up ui bars for reagents");
}

fn update_reagent_bar(
    mut reagent_bar_query: Query<(&mut Transform, &CurrentReagentBar)>,
    inventory_query: Query<&Inventory, (With<Player>, Without<CurrentReagentBar>)>,
) {
    let Ok(inventory) = inventory_query.get_single() else { return; };
    for (mut transform, CurrentReagentBar { reagent }) in &mut reagent_bar_query {
        transform.scale.x = inventory.reagent(*reagent).fraction();
    }
}

fn reposition_reagent_bar(
    mut reagent_bar_query: Query<(&mut Transform, &ReagentBarAnchor), Without<CustomUICamera>>,
    ui_camera: Query<&Camera, (Changed<Camera>, With<CustomUICamera>)>,
) {
    let Ok(ui_camera) = ui_camera.get_single() else {return;};
    let Some((top_left, _)) = ui_camera.logical_viewport_rect() else {return;};
    let Some(size) = ui_camera.logical_viewport_size() else {return;};
    let top_left = top_left + Vec2::new(-size.x / 2.0, size.y / 2.0);
    for (mut transform, ReagentBarAnchor { reagent }) in &mut reagent_bar_query {
        let i = *reagent as usize;
        transform.translation.x = top_left.x;
        transform.translation.y =
            top_left.y - BAR_PADDING - (i + 1) as f32 * (FONT_HEIGHT + BAR_PADDING);
    }
}

fn update_reagent_bar_visibility(
    mut reagent_bar_query: Query<(&mut Visibility, &ReagentBarAnchor)>,
    inventory_query: Query<&Inventory, (With<Player>, Without<ReagentBarAnchor>)>,
) {
    let Ok(inventory) = inventory_query.get_single() else { return; };

    for (mut visibility, ReagentBarAnchor { reagent }) in &mut reagent_bar_query {
        if inventory.reagent(*reagent).visibile() {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn setup_ui_bar<T: Component, U: Component>(
    commands: &mut Commands,
    assets_server: &AssetServer,
    anchor_component: T,
    current_component: U,
    label: &str,
    bar_color: Color,
    text_color: Color,
    threshold: Option<f32>,
) -> Entity {
    let font = assets_server.load("font/BebasNeueRegular.otf");

    // These are used to place the text properly
    // probably need to be tuned differently for different fonts
    const NUDGE_RIGHT: f32 = 5.0;
    const NUDGE_DOWN: f32 = 4.0;

    commands
        .spawn((
            SpatialBundle {
                visibility: Visibility::Visible,
                ..Default::default()
            },
            anchor_component,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text2dBundle {
                    text: Text::from_section(
                        label,
                        TextStyle {
                            font: font.clone(),
                            font_size: FONT_HEIGHT,
                            color: text_color,
                        },
                    ),
                    text_anchor: Anchor::TopLeft,
                    text_2d_bounds: Text2dBounds {
                        size: Vec2::new(BAR_LENGTH, 50.0),
                    },
                    transform: Transform::from_xyz(NUDGE_RIGHT, -NUDGE_DOWN, 2.0),
                    ..Default::default()
                },
                RenderLayers::layer(1),
            ));
            // Drop shadow
            parent.spawn((
                Text2dBundle {
                    text: Text::from_section(
                        label,
                        TextStyle {
                            font: font.clone(),
                            font_size: FONT_HEIGHT,
                            color: Color::BLACK,
                        },
                    ),
                    text_anchor: Anchor::TopLeft,
                    text_2d_bounds: Text2dBounds {
                        size: Vec2::new(BAR_LENGTH, 50.0),
                    },
                    transform: Transform::from_xyz(NUDGE_RIGHT + 2.0, -NUDGE_DOWN - 2.0, 1.9),
                    ..Default::default()
                },
                RenderLayers::layer(1),
            ));
            parent.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        anchor: Anchor::TopLeft,
                        color: bar_color,
                        custom_size: Some(Vec2::new(BAR_LENGTH, FONT_HEIGHT)),
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(0.0, 0.0, 1.0),
                    ..Default::default()
                },
                RenderLayers::layer(1),
                current_component,
            ));
            parent.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        anchor: Anchor::TopLeft,
                        color: Color::DARK_GRAY,
                        custom_size: Some(Vec2::new(BAR_LENGTH, FONT_HEIGHT)),
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(0.0, 0.0, 0.0),
                    ..Default::default()
                },
                RenderLayers::layer(1),
            ));
            if let Some(threshold) = threshold {
                parent.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            anchor: Anchor::TopLeft,
                            color: Color::YELLOW,
                            custom_size: Some(Vec2::new(2.0, FONT_HEIGHT)),
                            ..Default::default()
                        },
                        transform: Transform::from_xyz(BAR_LENGTH * threshold, 0.0, 2.0),
                        ..Default::default()
                    },
                    RenderLayers::layer(1),
                ));
            }
        })
        .id()
}

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_ui_camera)
            .add_startup_system(setup_heat_display)
            .add_system(update_heat_bar)
            .add_system(reposition_heat_bar)
            .add_system(setup_reagent_bars)
            .add_system(update_reagent_bar)
            .add_system(reposition_reagent_bar)
            .add_system(update_reagent_bar_visibility);
    }
}
