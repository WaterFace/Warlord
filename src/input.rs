use bevy::prelude::*;
use leafwing_input_manager::{prelude::*, user_input::InputKind};

#[derive(Actionlike, Debug, PartialEq, Clone, Copy, Hash)]
pub enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    FireMainGun,
    Shield,
    DumpCargo,
}

pub fn default_input_map() -> InputMap<Action> {
    InputMap::new([
        (InputKind::Keyboard(KeyCode::W), Action::MoveUp),
        (InputKind::Keyboard(KeyCode::S), Action::MoveDown),
        (InputKind::Keyboard(KeyCode::A), Action::MoveLeft),
        (InputKind::Keyboard(KeyCode::D), Action::MoveRight),
        (InputKind::Mouse(MouseButton::Left), Action::FireMainGun),
        (InputKind::Mouse(MouseButton::Right), Action::Shield),
        (InputKind::Keyboard(KeyCode::F), Action::DumpCargo),
        // TODO: add gamepad inputs
    ])
}

#[derive(Actionlike, Debug, PartialEq, Clone, Copy, Hash)]
pub enum MenuAction {
    Menu,
}

pub fn default_menu_input_map() -> InputMap<MenuAction> {
    InputMap::new([(InputKind::Keyboard(KeyCode::Escape), MenuAction::Menu)])
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<Action>::default())
            .add_plugin(InputManagerPlugin::<MenuAction>::default());
    }
}
