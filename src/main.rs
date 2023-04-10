use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};

mod camera;
mod collectible;
mod heat;
mod input;
mod inventory;
mod menu;
mod particles;
mod physics;
mod player;
mod reaction;
mod rock;
mod setup_cleanup;
mod shield;
mod sound;
mod starfield_shader;
mod state;
mod ui;
mod util;
mod weapon;

fn setup(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
        ..Default::default()
    });
}

fn main() {
    let mut app = App::new();
    #[cfg(not(debug_assertions))]
    app.add_plugins(DefaultPlugins);
    #[cfg(debug_assertions)]
    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                filter: "error,warlord=debug".into(),
                level: Level::DEBUG,
                ..Default::default()
            })
            .set(AssetPlugin {
                watch_for_changes: true,
                ..Default::default()
            }),
    );
    app.add_plugin(state::StatePlugin)
        .add_plugin(setup_cleanup::SetupCleanupPlugin)
        .add_plugin(physics::PhysicsPlugin { debug: false })
        .add_plugin(starfield_shader::StarfieldShaderPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(rock::RockPlugin)
        .add_plugin(weapon::WeaponPlugin)
        .add_plugin(heat::HeatPlugin)
        .add_plugin(ui::UIPlugin)
        .add_plugin(collectible::CollectiblePlugin)
        .add_plugin(inventory::InventoryPlugin)
        .add_plugin(reaction::ReactionPlugin)
        .add_plugin(input::InputPlugin)
        .add_plugin(menu::MenuPlugin)
        .add_plugin(particles::ParticlePlugin)
        .add_plugin(shield::ShieldPlugin)
        .add_plugin(sound::SoundPlugin)
        .add_startup_system(setup)
        .run();
}
