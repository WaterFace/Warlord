use bevy::{app::AppExit, prelude::*};
use leafwing_input_manager::{prelude::ActionState, InputManagerBundle};

use crate::{input::default_menu_input_map, state::GameState};

#[derive(Component, Debug)]
pub struct MenuButton {
    /// The event that will be sent when this button is pressed
    event: Option<MenuEvent>,
    base_color: Color,
    hover_color: Color,
    pressed_color: Color,
}

impl Default for MenuButton {
    fn default() -> Self {
        MenuButton {
            event: None,
            base_color: BASE_COLOR,
            hover_color: HOVER_COLOR,
            pressed_color: PRESSED_COLOR,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MenuEvent {
    Start,
    Resume,
    Settings,
    Exit,
}

const BASE_COLOR: Color = Color::GRAY;
const HOVER_COLOR: Color = Color::DARK_GRAY;
const PRESSED_COLOR: Color = Color::ORANGE_RED;
const TEXT_COLOR: Color = Color::WHITE;

const FONT_HEIGHT: f32 = 50.0;
const BUTTON_WIDTH: f32 = 300.0;

fn add_menu_button(
    builder: &mut ChildBuilder,
    assets_server: &AssetServer,
    label: &str,
    menu_button: MenuButton,
) {
    let font = assets_server.load("font/BebasNeueRegular.otf");

    builder
        .spawn((
            ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(BUTTON_WIDTH), Val::Px(FONT_HEIGHT)),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    padding: UiRect {
                        top: Val::Px(15.0),
                        bottom: Val::Px(15.0),
                        ..Default::default()
                    },
                    ..default()
                },
                background_color: menu_button.base_color.into(),
                ..default()
            },
            menu_button,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(
                    label,
                    TextStyle {
                        font: font.clone(),
                        font_size: FONT_HEIGHT,
                        color: TEXT_COLOR,
                    },
                ),
                ..Default::default()
            });
        });
}

#[derive(Component)]
struct MenuRoot;

fn setup_pause_menu(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::width(Val::Percent(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    gap: Size::all(Val::Px(4.0)),
                    ..default()
                },
                background_color: Color::rgba(0.4, 0.4, 0.4, 0.5).into(),
                ..default()
            },
            MenuRoot,
        ))
        .with_children(|parent| {
            let resume_button = MenuButton {
                event: Some(MenuEvent::Resume),
                ..Default::default()
            };
            add_menu_button(parent, &assets_server, "RESUME", resume_button);
            let settings_button = MenuButton {
                event: Some(MenuEvent::Settings),
                ..Default::default()
            };
            add_menu_button(parent, &assets_server, "SETTINGS", settings_button);
            #[cfg(not(target_arch = "wasm32"))]
            {
                let exit_button = MenuButton {
                    event: Some(MenuEvent::Exit),
                    hover_color: Color::RED,
                    pressed_color: Color::ORANGE_RED,
                    ..Default::default()
                };
                add_menu_button(parent, &assets_server, "QUIT", exit_button);
            }
        });
}

#[derive(Component)]
struct MainMenuRoot;

fn setup_main_menu(mut commands: Commands, assets_server: Res<AssetServer>) {
    let font = assets_server.load("font/BebasNeueRegular.otf");

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::width(Val::Percent(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    gap: Size::all(Val::Px(4.0)),
                    ..default()
                },
                background_color: Color::BLACK.into(),
                ..default()
            },
            MainMenuRoot,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "WARLORD",
                    TextStyle {
                        font: font.clone(),
                        font_size: 120.0,
                        color: Color::ORANGE_RED,
                    },
                ),
                ..Default::default()
            });
            let start_button = MenuButton {
                event: Some(MenuEvent::Start),
                ..Default::default()
            };
            add_menu_button(parent, &assets_server, "START", start_button);
            let settings_button = MenuButton {
                event: Some(MenuEvent::Settings),
                ..Default::default()
            };
            add_menu_button(parent, &assets_server, "SETTINGS", settings_button);
            #[cfg(not(target_arch = "wasm32"))]
            {
                let exit_button = MenuButton {
                    event: Some(MenuEvent::Exit),
                    hover_color: Color::RED,
                    pressed_color: Color::BLACK,
                    ..Default::default()
                };
                add_menu_button(parent, &assets_server, "QUIT", exit_button);
            }
        });
}

fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuRoot>>) {
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
}

fn handle_button_interaction(
    mut query: Query<(&Interaction, &MenuButton, &mut BackgroundColor), Changed<Interaction>>,
    mut writer: EventWriter<MenuEvent>,
) {
    for (interaction, menu_button, mut color) in &mut query {
        match interaction {
            Interaction::Clicked => {
                *color = menu_button.pressed_color.into();
                if let Some(event) = menu_button.event {
                    writer.send(event);
                }
            }
            Interaction::Hovered => {
                *color = menu_button.hover_color.into();
            }
            Interaction::None => {
                *color = menu_button.base_color.into();
            }
        }
    }
}

fn process_menu_event(
    mut reader: EventReader<MenuEvent>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    for ev in reader.iter() {
        match ev {
            MenuEvent::Exit => {
                info!("Goodbye!");
                // TODO: ask for confirmation
                exit.send(AppExit)
            }
            MenuEvent::Resume => {
                if current_state.0 == GameState::Paused {
                    next_state.set(GameState::InGame)
                }
            }
            MenuEvent::Settings => debug!("MenuEvent::Settings recieved"),
            MenuEvent::Start => {
                if current_state.0 == GameState::MainMenu {
                    next_state.set(GameState::InGame)
                }
            }
        }
    }
}

#[derive(Component)]
struct MenuController;

fn setup_menu_controller(mut commands: Commands) {
    commands.spawn((
        InputManagerBundle {
            action_state: ActionState::default(),
            input_map: default_menu_input_map(),
        },
        MenuController,
    ));
}

fn handle_menu_input(
    query: Query<&ActionState<crate::input::MenuAction>, With<MenuController>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let action_state = query.single();
    if action_state.just_pressed(crate::input::MenuAction::Menu) {
        match current_state.0 {
            GameState::InGame => next_state.set(GameState::Paused),
            GameState::MainMenu => {
                // Do nothing
                // TODO: maybe exit the game?
            }
            GameState::Paused => next_state.set(GameState::InGame),
        }
    }
}

fn hide_menu(mut query: Query<&mut Visibility, With<MenuRoot>>) {
    for mut visibility in &mut query {
        *visibility = Visibility::Hidden;
    }
}

fn show_menu(mut query: Query<&mut Visibility, With<MenuRoot>>) {
    for mut visibility in &mut query {
        *visibility = Visibility::Visible;
    }
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MenuEvent>()
            .add_startup_system(setup_menu_controller)
            .add_system(handle_button_interaction)
            .add_system(process_menu_event)
            .add_system(handle_menu_input)
            .add_system(hide_menu.in_schedule(OnEnter(GameState::InGame)))
            .add_system(show_menu.in_schedule(OnEnter(GameState::Paused)))
            .add_system(setup_main_menu.in_schedule(OnEnter(GameState::MainMenu)))
            .add_system(cleanup_main_menu.in_schedule(OnExit(GameState::MainMenu)))
            .add_system(setup_pause_menu.in_schedule(OnExit(GameState::MainMenu)));
    }
}
