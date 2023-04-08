use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::state::GameState;

#[derive(Debug, Default)]
pub struct PhysicsPlugin {
    pub debug: bool,
}

fn pause_physics(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.physics_pipeline_active = false;
    rapier_config.query_pipeline_active = false;
}

fn resume_physics(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.physics_pipeline_active = true;
    rapier_config.query_pipeline_active = true;
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
        app.add_system(pause_physics.in_schedule(OnExit(GameState::InGame)))
            .add_system(resume_physics.in_schedule(OnEnter(GameState::InGame)));
    }
}
