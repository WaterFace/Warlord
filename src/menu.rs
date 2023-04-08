use bevy::prelude::*;

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
const PRESSED_COLOR: Color = Color::OLIVE;
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

fn setup_menu_buttons(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::width(Val::Percent(100.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                gap: Size::all(Val::Px(4.0)),
                ..default()
            },
            background_color: Color::rgba(0.9, 0.9, 0.9, 0.5).into(),
            ..default()
        })
        .with_children(|parent| {
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
            let exit_button = MenuButton {
                event: Some(MenuEvent::Exit),
                hover_color: Color::RED,
                pressed_color: Color::ORANGE_RED,
                ..Default::default()
            };
            add_menu_button(parent, &assets_server, "QUIT", exit_button);
        });
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

fn process_menu_event(mut reader: EventReader<MenuEvent>) {
    for ev in reader.iter() {
        debug!("Recieved {ev:?} event");
    }
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MenuEvent>()
            .add_startup_system(setup_menu_buttons)
            .add_system(handle_button_interaction)
            .add_system(process_menu_event);
    }
}
