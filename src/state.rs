use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, Hash, Eq, PartialEq)]
pub enum GameState {
    #[default]
    MainMenu,
    Intro,
    InGame,
    Paused,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>();
    }
}
