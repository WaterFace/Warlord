use bevy::prelude::*;
use leafwing_input_manager::{prelude::*, user_input::InputKind};

#[derive(Actionlike, Debug, PartialEq, Clone, Copy, Hash)]
pub enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    FireMainGun,
    FireTorpedo,
    DumpCargo,
}

pub fn default_input_map() -> InputMap<Action> {
    InputMap::new([
        (InputKind::Keyboard(KeyCode::W), Action::MoveUp),
        (InputKind::Keyboard(KeyCode::S), Action::MoveDown),
        (InputKind::Keyboard(KeyCode::A), Action::MoveLeft),
        (InputKind::Keyboard(KeyCode::D), Action::MoveRight),
        (InputKind::Mouse(MouseButton::Left), Action::FireMainGun),
        (InputKind::Mouse(MouseButton::Right), Action::FireTorpedo),
        (InputKind::Keyboard(KeyCode::F), Action::DumpCargo),
        // TODO: add gamepad inputs
    ])
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<Action>::default());
    }
}
