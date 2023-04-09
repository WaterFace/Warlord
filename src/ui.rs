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
    state::{GameState, ProgressStages},
};

#[derive(Component, Debug, Default)]
pub struct CustomUICamera;

#[derive(Bundle)]
pub struct CustomUICameraBundle {
    pub custom_ui_camera: CustomUICamera,
    pub render_layers: RenderLayers,
    pub camera2d_bundle: Camera2dBundle,
}

impl Default for CustomUICameraBundle {
    fn default() -> Self {
        CustomUICameraBundle {
            custom_ui_camera: CustomUICamera,
            render_layers: RenderLayers::layer(1),
            camera2d_bundle: Camera2dBundle {
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
        }
    }
}

#[derive(Component, Debug, Default)]
struct CurrentHeatBar;

#[derive(Component, Debug, Default)]
struct HeatBarAnchor;

#[derive(Component, Debug, Default)]
struct HeatBarThreshold;

fn setup_heat_display(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    heat_query: Query<&Heat, Added<Heat>>,
) {
    let Ok(heat) = heat_query.get_single() else { return; };
    setup_ui_bar(
        &mut commands,
        &assets_server,
        HeatBarAnchor,
        CurrentHeatBar,
        HeatBarThreshold,
        "HEAT",
        Color::RED,
        Color::WHITE,
        Some(heat.reaction_threshold()),
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
    ui_camera: Query<&Camera, With<CustomUICamera>>,
) {
    let Ok(ui_camera) = ui_camera.get_single() else {return;};
    let Some((top_left, _)) = ui_camera.logical_viewport_rect() else {return;};
    let Some(size) = ui_camera.logical_viewport_size() else {return;};
    let top_left = top_left + Vec2::new(-size.x / 2.0, size.y / 2.0);
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

#[derive(Component, Debug)]
struct ReagentBarThreshold {
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
            ReagentBarThreshold { reagent },
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
    ui_camera: Query<&Camera, With<CustomUICamera>>,
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

fn update_heat_bar_visibility(
    mut heat_bar_query: Query<(&mut Visibility, &HeatBarAnchor)>,
    heat_query: Query<&Heat, (With<Player>, Without<HeatBarAnchor>)>,
) {
    let Ok(heat) = heat_query.get_single() else { return; };

    for (mut visibility, HeatBarAnchor) in &mut heat_bar_query {
        if heat.enabled() {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
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

fn update_heat_bar_threshold(
    mut heat_bar_query: Query<(&mut Visibility, &mut Transform, &HeatBarThreshold)>,
    heat_query: Query<&Heat, (With<Player>, Without<HeatBarThreshold>)>,
) {
    let Ok(heat) = heat_query.get_single() else { return; };

    for (mut visibility, mut transform, HeatBarThreshold) in &mut heat_bar_query {
        if heat.threshold_visible() {
            *visibility = Visibility::Inherited;
            transform.translation.x = BAR_LENGTH * heat.reaction_threshold();
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn update_reagent_bar_threshold(
    mut reagent_bar_query: Query<(&mut Visibility, &mut Transform, &ReagentBarThreshold)>,
    inventory_query: Query<&Inventory, (With<Player>, Without<ReagentBarThreshold>)>,
) {
    let Ok(inventory) = inventory_query.get_single() else { return; };

    for (mut visibility, mut transform, ReagentBarThreshold { reagent }) in &mut reagent_bar_query {
        if let Some(threshold) = inventory.reagent(*reagent).threshold() {
            *visibility = Visibility::Inherited;
            transform.translation.x = BAR_LENGTH * threshold;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

#[derive(Component, Debug, Default)]
struct HintMarker {
    stage: ProgressStages,
}

fn setup_all_hints(mut commands: Commands, asset_server: Res<AssetServer>) {
    const FONT_SIZE: f32 = 25.0;

    let text = include_str!("hints/0.txt");
    let font = asset_server.load("font/BebasNeueRegular.otf");
    let sections = crate::util::markup_to_text_sections(
        text,
        font,
        FONT_SIZE,
        Color::ORANGE_RED,
        Color::WHITE,
    );
    setup_hint(
        &mut commands,
        sections,
        HintMarker {
            stage: ProgressStages::Exploration,
        },
    );

    let text = include_str!("hints/1.txt");
    let font = asset_server.load("font/BebasNeueRegular.otf");
    let sections = crate::util::markup_to_text_sections(
        text,
        font,
        FONT_SIZE,
        Color::ORANGE_RED,
        Color::WHITE,
    );
    setup_hint(
        &mut commands,
        sections,
        HintMarker {
            stage: ProgressStages::GunAndHeat,
        },
    );

    let text = include_str!("hints/2.txt");
    let font = asset_server.load("font/BebasNeueRegular.otf");
    let sections = crate::util::markup_to_text_sections(
        text,
        font,
        FONT_SIZE,
        Color::ORANGE_RED,
        Color::WHITE,
    );
    setup_hint(
        &mut commands,
        sections,
        HintMarker {
            stage: ProgressStages::CollectExotic,
        },
    );

    let text = include_str!("hints/3.txt");
    let font = asset_server.load("font/BebasNeueRegular.otf");
    let sections = crate::util::markup_to_text_sections(
        text,
        font,
        FONT_SIZE,
        Color::ORANGE_RED,
        Color::WHITE,
    );
    setup_hint(
        &mut commands,
        sections,
        HintMarker {
            stage: ProgressStages::ShieldAndStrange,
        },
    );

    let text = include_str!("hints/4.txt");
    let font = asset_server.load("font/BebasNeueRegular.otf");
    let sections = crate::util::markup_to_text_sections(
        text,
        font,
        FONT_SIZE,
        Color::ORANGE_RED,
        Color::WHITE,
    );
    setup_hint(
        &mut commands,
        sections,
        HintMarker {
            stage: ProgressStages::Continuum,
        },
    );
}

fn cleanup_hints(mut commands: Commands, query: Query<Entity, With<HintMarker>>) {
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
}

fn display_correct_hint(
    mut query: Query<(&mut Visibility, &HintMarker)>,
    progress: Res<State<ProgressStages>>,
) {
    if !progress.is_changed() {
        return;
    }

    for (mut visibility, hint_marker) in &mut query {
        *visibility = Visibility::Hidden;

        if hint_marker.stage == progress.0 {
            *visibility = Visibility::Visible;
        }
    }
}

fn reposition_hints(
    mut hint_query: Query<&mut Transform, (With<HintAnchor>, Without<CustomUICamera>)>,
    ui_camera: Query<&Camera, With<CustomUICamera>>,
) {
    let Ok(ui_camera) = ui_camera.get_single() else {return;};
    let Some((top_left, _)) = ui_camera.logical_viewport_rect() else {return;};
    let Some(size) = ui_camera.logical_viewport_size() else {return;};
    let top_right = top_left + Vec2::new(size.x / 2.0, size.y / 2.0);
    for mut transform in &mut hint_query {
        transform.translation.x = top_right.x - BAR_PADDING;
        transform.translation.y = top_right.y - BAR_PADDING;
    }
}

#[derive(Component, Debug, Default)]
pub struct UIMarker;

#[derive(Component, Debug, Default)]
pub struct HintAnchor;

const HINT_WIDTH: f32 = 350.0;
const HINT_HEIGHT: f32 = 200.0;

// These are used to place the text properly
// probably need to be tuned differently for different fonts
const NUDGE_RIGHT: f32 = 5.0;
const NUDGE_DOWN: f32 = 4.0;

fn setup_hint<C: Component>(commands: &mut Commands, sections: Vec<TextSection>, marker: C) {
    commands
        .spawn((
            SpatialBundle {
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            UIMarker,
            HintAnchor,
            marker,
            RenderLayers::layer(1),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text2dBundle {
                    text: Text::from_sections(sections.clone()).with_alignment(TextAlignment::Left),
                    text_anchor: Anchor::TopRight,
                    text_2d_bounds: Text2dBounds {
                        size: Vec2::new(HINT_WIDTH, HINT_HEIGHT),
                    },
                    transform: Transform::from_xyz(-NUDGE_RIGHT, -NUDGE_DOWN, 2.0),
                    ..Default::default()
                },
                RenderLayers::layer(1),
            ));
            // background
            parent.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        anchor: Anchor::TopRight,
                        color: Color::rgba(0.3, 0.3, 0.3, 0.5),
                        custom_size: Some(Vec2::new(HINT_WIDTH + 15.0, HINT_HEIGHT + 15.0)),
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(0.0, 0.0, 1.0),
                    ..Default::default()
                },
                RenderLayers::layer(1),
            ));
        });
}

fn setup_ui_bar<T: Component, U: Component, V: Component>(
    commands: &mut Commands,
    assets_server: &AssetServer,
    anchor_component: T,
    current_component: U,
    threshold_component: V,
    label: &str,
    bar_color: Color,
    text_color: Color,
    threshold: Option<f32>,
) -> Entity {
    let font = assets_server.load("font/BebasNeueRegular.otf");

    commands
        .spawn((
            SpatialBundle {
                visibility: Visibility::Visible,
                ..Default::default()
            },
            anchor_component,
            UIMarker,
            RenderLayers::layer(1),
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
            let visibility = if let Some(_) = threshold {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            parent.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        anchor: Anchor::TopLeft,
                        color: Color::YELLOW,
                        custom_size: Some(Vec2::new(2.0, FONT_HEIGHT)),
                        ..Default::default()
                    },
                    visibility,
                    transform: Transform::from_xyz(
                        BAR_LENGTH * threshold.unwrap_or(f32::INFINITY),
                        0.0,
                        2.0,
                    ),
                    ..Default::default()
                },
                RenderLayers::layer(1),
                threshold_component,
            ));
        })
        .id()
}

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_all_hints.in_schedule(OnExit(GameState::Intro)));
        app.add_system(cleanup_hints.in_schedule(OnEnter(GameState::Outro)));
        app.add_systems(
            (
                setup_heat_display,
                reposition_heat_bar,
                reposition_reagent_bar,
                update_heat_bar,
                update_heat_bar_visibility,
                update_heat_bar_threshold,
                setup_reagent_bars,
                update_reagent_bar,
                update_reagent_bar_visibility,
                update_reagent_bar_threshold,
                display_correct_hint,
                reposition_hints,
            )
                .in_set(OnUpdate(GameState::InGame)),
        );
    }
}
