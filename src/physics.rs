use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Debug, Default)]
pub struct PhysicsPlugin {
    pub debug: bool,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RapierPhysicsPlugin::<NoUserData>::default());
        if self.debug {
            app.add_plugin(RapierDebugRenderPlugin::default());
        }
        app.insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            ..Default::default()
        });
    }
}
