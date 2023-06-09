use bevy::{app::AppExit, prelude::*, ui::FocusPolicy};
use leafwing_input_manager::{prelude::ActionState, InputManagerBundle};

use crate::{
    input::default_menu_input_map,
    sound::{SoundEvent, VolumeSettings},
    state::{GameState, ProgressStages},
    util::markup_to_text_sections,
};

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
    Continue,
    Resume,
    Settings,
    Exit,
    Restart,
}

#[derive(Component, Debug)]
pub struct SettingsButton {
    /// The event that will be sent when this button is pressed
    event: Option<SettingsMenuEvent>,
    base_color: Color,
    hover_color: Color,
    pressed_color: Color,
}

impl Default for SettingsButton {
    fn default() -> Self {
        SettingsButton {
            event: None,
            base_color: BASE_COLOR,
            hover_color: HOVER_COLOR,
            pressed_color: PRESSED_COLOR,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SettingsMenuEvent {
    SoundEffectVolume { delta: f32 },
    MusicVolume { delta: f32 },
    ToggleMute,
    CloseSettings,
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

fn add_settings_button(
    builder: &mut ChildBuilder,
    assets_server: &AssetServer,
    label: &str,
    settings_button: SettingsButton,
) -> Entity {
    let font = assets_server.load("font/BebasNeueRegular.otf");
    let mut text = Entity::PLACEHOLDER;
    builder
        .spawn((
            ButtonBundle {
                style: Style {
                    size: Size::new(Val::Auto, Val::Px(FONT_HEIGHT)),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    padding: UiRect {
                        top: Val::Px(15.0),
                        bottom: Val::Px(15.0),
                        right: Val::Px(15.0),
                        left: Val::Px(15.0),
                    },
                    ..default()
                },
                background_color: settings_button.base_color.into(),
                ..default()
            },
            settings_button,
        ))
        .with_children(|parent| {
            text = parent
                .spawn(TextBundle {
                    text: Text::from_section(
                        label,
                        TextStyle {
                            font: font.clone(),
                            font_size: FONT_HEIGHT,
                            color: TEXT_COLOR,
                        },
                    ),
                    ..Default::default()
                })
                .id();
        });

    return text;
}

#[derive(Component)]
struct PauseMenuRoot;

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
                z_index: ZIndex::Global(1),
                ..default()
            },
            PauseMenuRoot,
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

fn cleanup_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuRoot>>) {
    for e in &query {
        debug!("Cleaned up after pause menu");
        commands.entity(e).despawn_recursive();
    }
}

#[derive(Component)]
struct SettingsMenuRoot;

#[derive(Component)]
struct SoundEffectsVolumeDisplay;
#[derive(Component)]
struct MusicVolumeDisplay;
#[derive(Component)]
struct MuteDisplay;

fn setup_settings_menu(mut commands: Commands, assets_server: Res<AssetServer>) {
    const VOLUME_DELTA: f32 = 0.05;
    let font = assets_server.load("font/BebasNeueRegular.otf");
    let mut mute: Entity = Entity::PLACEHOLDER;

    let rect = UiRect::all(Val::Percent(30.0));
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    gap: Size::all(Val::Px(4.0)),
                    position_type: PositionType::Absolute,
                    padding: UiRect::all(Val::Px(15.0)),
                    position: rect,
                    ..default()
                },
                background_color: Color::rgb(0.4, 0.4, 0.4).into(),
                visibility: Visibility::Hidden,
                z_index: ZIndex::Global(2),
                focus_policy: FocusPolicy::Block,
                ..default()
            },
            SettingsMenuRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::width(Val::Percent(100.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Row,
                        gap: Size::all(Val::Px(4.0)),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Sound",
                            TextStyle {
                                font: font.clone(),
                                font_size: FONT_HEIGHT,
                                color: Color::WHITE,
                            },
                        ),
                        style: Style {
                            size: Size::width(Val::Px(100.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                    let minus_button = SettingsButton {
                        event: Some(SettingsMenuEvent::SoundEffectVolume {
                            delta: -VOLUME_DELTA,
                        }),
                        ..Default::default()
                    };
                    add_settings_button(parent, &assets_server, "-", minus_button);
                    parent.spawn((
                        TextBundle {
                            text: Text::from_section(
                                "",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: FONT_HEIGHT,
                                    color: Color::ORANGE_RED,
                                },
                            ),
                            ..Default::default()
                        },
                        SoundEffectsVolumeDisplay,
                    ));
                    let plus_button = SettingsButton {
                        event: Some(SettingsMenuEvent::SoundEffectVolume {
                            delta: VOLUME_DELTA,
                        }),
                        ..Default::default()
                    };
                    add_settings_button(parent, &assets_server, "+", plus_button);
                });
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::width(Val::Percent(100.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Row,
                        gap: Size::all(Val::Px(4.0)),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Music",
                            TextStyle {
                                font: font.clone(),
                                font_size: FONT_HEIGHT,
                                color: Color::WHITE,
                            },
                        ),
                        style: Style {
                            size: Size::width(Val::Px(100.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                    let minus_button = SettingsButton {
                        event: Some(SettingsMenuEvent::MusicVolume {
                            delta: -VOLUME_DELTA,
                        }),
                        ..Default::default()
                    };
                    add_settings_button(parent, &assets_server, "-", minus_button);
                    parent.spawn((
                        TextBundle {
                            text: Text::from_section(
                                "",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: FONT_HEIGHT,
                                    color: Color::ORANGE_RED,
                                },
                            ),
                            ..Default::default()
                        },
                        MusicVolumeDisplay,
                    ));
                    let plus_button = SettingsButton {
                        event: Some(SettingsMenuEvent::MusicVolume {
                            delta: VOLUME_DELTA,
                        }),
                        ..Default::default()
                    };
                    add_settings_button(parent, &assets_server, "+", plus_button);
                });
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::width(Val::Percent(100.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Row,
                        gap: Size::all(Val::Px(4.0)),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    let mute_button = SettingsButton {
                        event: Some(SettingsMenuEvent::ToggleMute),
                        ..Default::default()
                    };
                    mute = add_settings_button(parent, &assets_server, "MUTE", mute_button);
                });

            let back_button = SettingsButton {
                event: Some(SettingsMenuEvent::CloseSettings),
                ..Default::default()
            };
            add_settings_button(parent, &assets_server, "BACK", back_button);
        });
    commands.entity(mute).insert(MuteDisplay);
}

fn cleanup_settings_menu(mut commands: Commands, query: Query<Entity, With<SettingsMenuRoot>>) {
    for e in &query {
        debug!("Cleaned up after settings menu");
        commands.entity(e).despawn_recursive();
    }
}

fn update_settings_menu_displays(
    mut mute_query: Query<&mut Text, With<MuteDisplay>>,
    mut sound_effect_query: Query<
        &mut Text,
        (With<SoundEffectsVolumeDisplay>, Without<MuteDisplay>),
    >,
    mut music_query: Query<
        &mut Text,
        (
            With<MusicVolumeDisplay>,
            Without<SoundEffectsVolumeDisplay>,
            Without<MuteDisplay>,
        ),
    >,
    volume: Res<VolumeSettings>,
    asset_server: Res<AssetServer>,
) {
    if volume.is_changed() {
        let font = asset_server.load("font/BebasNeueRegular.otf");
        for mut text in &mut mute_query {
            if volume.mute {
                *text = Text::from_section(
                    "UNMUTE",
                    TextStyle {
                        font: font.clone(),
                        font_size: FONT_HEIGHT,
                        color: Color::WHITE,
                    },
                );
            } else {
                *text = Text::from_section(
                    "MUTE",
                    TextStyle {
                        font: font.clone(),
                        font_size: FONT_HEIGHT,
                        color: Color::WHITE,
                    },
                );
            }
        }

        for mut text in &mut sound_effect_query {
            let value = format!("{:.2}", volume.sound_effects * 100.0);
            *text = Text::from_section(
                value,
                TextStyle {
                    font: font.clone(),
                    font_size: FONT_HEIGHT,
                    color: Color::ORANGE_RED,
                },
            );
        }

        for mut text in &mut music_query {
            let value = format!("{:.2}", volume.music * 100.0);
            *text = Text::from_section(
                value,
                TextStyle {
                    font: font.clone(),
                    font_size: FONT_HEIGHT,
                    color: Color::ORANGE_RED,
                },
            );
        }
    }
}

#[derive(Component)]
struct MainMenuRoot;

fn setup_main_menu(mut commands: Commands, assets_server: Res<AssetServer>) {
    let font = assets_server.load("font/BebasNeueRegular.otf");
    debug!("Setting up the main menu");
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
                z_index: ZIndex::Global(0),
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
        debug!("Cleaned up after main menu");
        commands.entity(e).despawn_recursive();
    }
}

#[derive(Component)]
struct IntroMenuRoot;

fn setup_intro_menu(mut commands: Commands, assets_server: Res<AssetServer>) {
    let font = assets_server.load("font/BebasNeueRegular.otf");

    let story = include_str!("story.txt");
    let text = markup_to_text_sections(story, font.clone(), 30.0, Color::ORANGE_RED, TEXT_COLOR);

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::width(Val::Percent(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    gap: Size::all(Val::Px(15.0)),
                    ..default()
                },
                background_color: Color::BLACK.into(),
                ..default()
            },
            IntroMenuRoot,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_sections(text),
                ..Default::default()
            });
            let continue_button = MenuButton {
                event: Some(MenuEvent::Continue),
                ..Default::default()
            };
            add_menu_button(parent, &assets_server, "CONTINUE", continue_button);
        });
}

fn cleanup_intro_menu(mut commands: Commands, query: Query<Entity, With<IntroMenuRoot>>) {
    for e in &query {
        debug!("Cleaned up after intro");
        commands.entity(e).despawn_recursive();
    }
}

#[derive(Component)]
struct OutroMenuRoot;

fn setup_outro_menu(mut commands: Commands, assets_server: Res<AssetServer>) {
    let font = assets_server.load("font/BebasNeueRegular.otf");

    let outro = include_str!("outro.txt");
    let text = markup_to_text_sections(outro, font.clone(), 30.0, Color::ORANGE_RED, TEXT_COLOR);

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::width(Val::Percent(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    gap: Size::all(Val::Px(15.0)),
                    ..default()
                },
                background_color: Color::BLACK.into(),
                ..default()
            },
            OutroMenuRoot,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_sections(text),
                ..Default::default()
            });
            let continue_button = MenuButton {
                event: Some(MenuEvent::Continue),
                ..Default::default()
            };
            add_menu_button(parent, &assets_server, "CONTINUE", continue_button);
        });
}

fn cleanup_outro_menu(mut commands: Commands, query: Query<Entity, With<OutroMenuRoot>>) {
    for e in &query {
        debug!("Cleaned up after outro");
        commands.entity(e).despawn_recursive();
    }
}

#[derive(Component)]
struct EndScreenMenuRoot;

fn setup_endscreen_menu(mut commands: Commands, assets_server: Res<AssetServer>) {
    let font = assets_server.load("font/BebasNeueRegular.otf");
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::width(Val::Percent(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    gap: Size::all(Val::Px(15.0)),
                    ..default()
                },
                background_color: Color::BLACK.into(),
                ..default()
            },
            EndScreenMenuRoot,
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
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "Thank you for playing!",
                    TextStyle {
                        font: font.clone(),
                        font_size: 30.0,
                        color: Color::ORANGE_RED,
                    },
                ),
                ..Default::default()
            });
            let restart_button = MenuButton {
                event: Some(MenuEvent::Restart),
                ..Default::default()
            };
            add_menu_button(parent, &assets_server, "MAIN MENU", restart_button);
        });
}

fn cleanup_endscreen_menu(mut commands: Commands, query: Query<Entity, With<EndScreenMenuRoot>>) {
    for e in &query {
        debug!("Cleaned up after endscreen");
        commands.entity(e).despawn_recursive();
    }
}

fn handle_button_interaction(
    mut menu_button_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut settings_button_query: Query<
        (&Interaction, &SettingsButton, &mut BackgroundColor),
        (Changed<Interaction>, Without<MenuButton>),
    >,
    mut menu_event_writer: EventWriter<MenuEvent>,
    mut settings_event_writer: EventWriter<SettingsMenuEvent>,
    mut sound_event_writer: EventWriter<SoundEvent>,
) {
    for (interaction, menu_button, mut color) in &mut menu_button_query {
        match interaction {
            Interaction::Clicked => {
                *color = menu_button.pressed_color.into();
                if let Some(event) = menu_button.event {
                    menu_event_writer.send(event);
                }
                sound_event_writer.send(SoundEvent::ButtonClick);
            }
            Interaction::Hovered => {
                *color = menu_button.hover_color.into();
            }
            Interaction::None => {
                *color = menu_button.base_color.into();
            }
        }
    }

    for (interaction, settings_button, mut color) in &mut settings_button_query {
        match interaction {
            Interaction::Clicked => {
                *color = settings_button.pressed_color.into();
                if let Some(event) = settings_button.event {
                    settings_event_writer.send(event);
                }
                sound_event_writer.send(SoundEvent::ButtonClick);
            }
            Interaction::Hovered => {
                *color = settings_button.hover_color.into();
            }
            Interaction::None => {
                *color = settings_button.base_color.into();
            }
        }
    }
}

fn process_menu_event(
    mut reader: EventReader<MenuEvent>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut next_game_stage: ResMut<NextState<ProgressStages>>,
    mut next_settings_state: ResMut<NextState<SettingsState>>,
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
            MenuEvent::Settings => next_settings_state.set(SettingsState::InSettings),
            MenuEvent::Start => {
                if current_state.0 == GameState::MainMenu {
                    next_state.set(GameState::Intro);
                    next_game_stage.set(ProgressStages::default());
                }
            }
            MenuEvent::Continue => {
                if current_state.0 == GameState::Intro {
                    next_state.set(GameState::InGame)
                } else if current_state.0 == GameState::Outro {
                    next_state.set(GameState::EndScreen)
                }
            }
            MenuEvent::Restart => {
                if current_state.0 == GameState::EndScreen {
                    next_state.set(GameState::MainMenu)
                }
            }
        }
    }
}

fn process_settings_menu_event(
    mut reader: EventReader<SettingsMenuEvent>,
    mut next_settings_state: ResMut<NextState<SettingsState>>,
    mut volume: ResMut<VolumeSettings>,
) {
    for ev in reader.iter() {
        match ev {
            SettingsMenuEvent::CloseSettings => {
                next_settings_state.set(SettingsState::None);
            }
            SettingsMenuEvent::ToggleMute => {
                volume.mute = !volume.mute;
            }
            SettingsMenuEvent::SoundEffectVolume { delta } => {
                volume.sound_effects = (volume.sound_effects + delta).clamp(0.0, 1.0);
            }
            SettingsMenuEvent::MusicVolume { delta } => {
                volume.music = (volume.music + delta).clamp(0.0, 1.0);
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
    current_settings_state: Res<State<SettingsState>>,
    mut next_settings_state: ResMut<NextState<SettingsState>>,
) {
    let action_state = query.single();
    if action_state.just_pressed(crate::input::MenuAction::Menu) {
        match current_settings_state.0 {
            SettingsState::InSettings => {
                next_settings_state.set(SettingsState::None);
                return;
            }
            SettingsState::None => (),
        }
        match current_state.0 {
            GameState::MainMenu => {
                // Do nothing
            }
            GameState::Intro => {
                // Do nothing
            }
            GameState::Outro => {
                // Do Nothing
            }
            GameState::EndScreen => {
                // Do Nothing
            }
            GameState::InGame => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::InGame),
        }
    }
}

fn hide_pause_menu(mut query: Query<&mut Visibility, With<PauseMenuRoot>>) {
    for mut visibility in &mut query {
        *visibility = Visibility::Hidden;
    }
}

fn show_pause_menu(mut query: Query<&mut Visibility, With<PauseMenuRoot>>) {
    for mut visibility in &mut query {
        *visibility = Visibility::Visible;
    }
}

fn hide_settings_menu(mut query: Query<&mut Visibility, With<SettingsMenuRoot>>) {
    for mut visibility in &mut query {
        *visibility = Visibility::Hidden;
    }
}

fn show_settings_menu(mut query: Query<&mut Visibility, With<SettingsMenuRoot>>) {
    for mut visibility in &mut query {
        *visibility = Visibility::Visible;
    }
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum SettingsState {
    #[default]
    None,
    InSettings,
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MenuEvent>()
            .add_event::<SettingsMenuEvent>()
            .add_state::<SettingsState>()
            .add_startup_system(setup_menu_controller)
            .add_system(handle_button_interaction)
            .add_system(process_menu_event)
            .add_system(handle_menu_input)
            .add_system(hide_pause_menu.in_schedule(OnEnter(GameState::InGame)))
            .add_system(show_pause_menu.in_schedule(OnEnter(GameState::Paused)))
            .add_system(hide_settings_menu.in_schedule(OnEnter(SettingsState::None)))
            .add_system(show_settings_menu.in_schedule(OnEnter(SettingsState::InSettings)))
            .add_system(setup_settings_menu.in_schedule(OnEnter(GameState::MainMenu)))
            .add_system(cleanup_settings_menu.in_schedule(OnEnter(GameState::Outro)))
            .add_system(setup_main_menu.in_schedule(OnEnter(GameState::MainMenu)))
            .add_system(cleanup_main_menu.in_schedule(OnExit(GameState::MainMenu)))
            .add_system(setup_intro_menu.in_schedule(OnEnter(GameState::Intro)))
            .add_system(cleanup_intro_menu.in_schedule(OnExit(GameState::Intro)))
            .add_system(setup_pause_menu.in_schedule(OnExit(GameState::Intro)))
            .add_system(cleanup_pause_menu.in_schedule(OnEnter(GameState::Outro)))
            .add_system(setup_outro_menu.in_schedule(OnEnter(GameState::Outro)))
            .add_system(cleanup_outro_menu.in_schedule(OnExit(GameState::Outro)))
            .add_system(setup_endscreen_menu.in_schedule(OnEnter(GameState::EndScreen)))
            .add_system(cleanup_endscreen_menu.in_schedule(OnExit(GameState::EndScreen)))
            .add_system(process_settings_menu_event.in_set(OnUpdate(SettingsState::InSettings)))
            .add_system(update_settings_menu_displays.in_set(OnUpdate(SettingsState::InSettings)));
    }
}
