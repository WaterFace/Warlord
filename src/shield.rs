use bevy::{prelude::*, render::render_resource::Face};
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{player::Player, state::GameState};

#[derive(Resource)]
struct ShieldVisuals {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[derive(Component, Default)]
pub struct Shield;

#[derive(Component)]
pub struct ShieldParent {
    shield: Entity,
}

#[derive(Bundle)]
pub struct ShieldBundle {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
    pub shield: Shield,
    pub collider: Collider,
    pub sensor: Sensor,
    pub additional_mass_properties: AdditionalMassProperties,
}

impl Default for ShieldBundle {
    fn default() -> Self {
        Self {
            mesh: Default::default(),
            material: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            computed_visibility: Default::default(),
            shield: Default::default(),
            collider: Collider::ball(2.5),
            sensor: Default::default(),
            additional_mass_properties: AdditionalMassProperties::Mass(0.0),
        }
    }
}

fn setup_shield_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(
        shape::UVSphere {
            radius: 2.5,
            ..Default::default()
        }
        .into(),
    );

    let material = materials.add(StandardMaterial {
        alpha_mode: AlphaMode::Blend,
        base_color: Color::rgba(0.1, 0.8, 0.8, 0.4),
        emissive: Color::rgb(0.1, 0.8, 0.8) * 3.0,
        double_sided: true,
        cull_mode: None,
        ..Default::default()
    });

    commands.insert_resource(ShieldVisuals { mesh, material });
}

fn spawn_despawn_shield(
    mut commands: Commands,
    player_query: Query<
        (
            Entity,
            &ActionState<crate::input::Action>,
            Option<&ShieldParent>,
        ),
        With<Player>,
    >,
    shield_query: Query<Entity, (With<Shield>, Without<Player>)>,
    shield_visuals: Res<ShieldVisuals>,
) {
    for (player_entity, action_state, maybe_shield_parent) in &player_query {
        if action_state.pressed(crate::input::Action::Shield) {
            if maybe_shield_parent.is_none()
                || shield_query
                    .get(maybe_shield_parent.unwrap().shield)
                    .is_err()
            {
                // Then there's no shield and we should spawn one
                let shield = commands
                    .spawn(ShieldBundle {
                        mesh: shield_visuals.mesh.clone(),
                        material: shield_visuals.material.clone(),
                        ..Default::default()
                    })
                    .id();
                commands
                    .entity(player_entity)
                    .insert(ShieldParent { shield })
                    .add_child(shield);
            }
        } else {
            // Then there shouldn't be a shield, and we should remove it if one exists
            if let Some(shield_parent) = maybe_shield_parent {
                commands.entity(shield_parent.shield).despawn_recursive();
                commands.entity(player_entity).remove::<ShieldParent>();
            }
        }
    }
}

pub struct ShieldPlugin;

impl Plugin for ShieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_shield_visuals)
            .add_system(spawn_despawn_shield.in_set(OnUpdate(GameState::InGame)));
    }
}
