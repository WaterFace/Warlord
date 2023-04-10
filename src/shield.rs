use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    collectible::{Collectible, CollectibleBundle, StrangeMatterAppearance},
    inventory::Reagent,
    player::Player,
    rock::{Rock, RockDestroyed},
    sound::SoundEvent,
    state::GameState,
};

#[derive(Resource)]
struct ShieldVisuals {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[derive(Component, Debug, Default)]
pub struct ShieldEmitter {
    pub enabled: bool,
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
    pub active_events: ActiveEvents,
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
            active_events: ActiveEvents::COLLISION_EVENTS,
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
            &ShieldEmitter,
            &ActionState<crate::input::Action>,
            Option<&ShieldParent>,
        ),
        With<Player>,
    >,
    shield_query: Query<Entity, (With<Shield>, Without<Player>)>,
    shield_visuals: Res<ShieldVisuals>,
) {
    for (player_entity, shield_emitter, action_state, maybe_shield_parent) in &player_query {
        if action_state.pressed(crate::input::Action::Shield) {
            if shield_emitter.enabled
                && (maybe_shield_parent.is_none()
                    || shield_query
                        .get(maybe_shield_parent.unwrap().shield)
                        .is_err())
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

#[derive(Debug)]
pub enum ShieldCollision {
    Rock {
        entity: Entity,
        position: Vec3,
    },
    Collectible {
        entity: Entity,
        position: Vec3,
        reagent: Reagent,
        amount: f32,
    },
}

fn handle_collision(
    mut collisions: EventReader<CollisionEvent>,
    shield_query: Query<Entity, With<Shield>>,
    collectible_query: Query<(&Collectible, &Transform), Without<Player>>,
    rock_query: Query<(&Rock, &Transform), (Without<Player>, Without<Collectible>)>,
    mut writer: EventWriter<ShieldCollision>,
) {
    for ev in collisions.iter() {
        match ev {
            CollisionEvent::Started(e1, e2, _flags) => {
                if let Ok(_) = shield_query.get(*e1) {
                    if let Ok((collectible, transform)) = collectible_query.get(*e2) {
                        match collectible {
                          Collectible::CollectibleReagent { reagent, amount } => {
                            writer.send(ShieldCollision::Collectible {
                                entity: *e2,
                                position: transform.translation,
                                reagent: *reagent,
                                amount: *amount,
                            });
                          }
                          _ => warn!("Shield collided with a collectible with no associated Reagent. That's probably not intentional."),
                      }
                    } else if let Ok((_rock, transform)) = rock_query.get(*e2) {
                        writer.send(ShieldCollision::Rock {
                            entity: *e2,
                            position: transform.translation,
                        })
                    }
                } else if let Ok(_) = shield_query.get(*e2) {
                    if let Ok((collectible, transform)) = collectible_query.get(*e1) {
                        match collectible {
                          Collectible::CollectibleReagent { reagent, amount } => {
                            writer.send(ShieldCollision::Collectible {
                                entity: *e1,
                                position: transform.translation,
                                reagent: *reagent,
                                amount: *amount,
                            });
                          }
                          _ => warn!("Shield collided with a collectible with no associated Reagent. That's probably not intentional."),
                      }
                    } else if let Ok((_rock, transform)) = rock_query.get(*e1) {
                        writer.send(ShieldCollision::Rock {
                            entity: *e1,
                            position: transform.translation,
                        })
                    }
                }
            }
            _ => {}
        }
    }
}

fn handle_shield_collisions(
    mut commands: Commands,
    mut reader: EventReader<ShieldCollision>,
    mut rock_destroyed_writer: EventWriter<RockDestroyed>,
    mut sound_event_writer: EventWriter<SoundEvent>,
    player_query: Query<&Transform, With<Player>>,
    strange_matter_appearance: Res<StrangeMatterAppearance>,
) {
    for ev in reader.iter() {
        match ev {
            ShieldCollision::Rock { entity, position } => {
                rock_destroyed_writer.send(RockDestroyed {
                    entity: *entity,
                    position: *position,
                })
            }
            ShieldCollision::Collectible {
                entity,
                position,
                reagent,
                amount,
            } => {
                match reagent {
                    Reagent::Exotic => {
                        let transform = Transform::from_translation(*position);
                        commands.entity(*entity).despawn_recursive();
                        commands.spawn(CollectibleBundle {
                            transform,
                            mesh: strange_matter_appearance.mesh.clone(),
                            material: strange_matter_appearance.material.clone(),
                            collectible: Collectible::CollectibleReagent {
                                reagent: Reagent::Strange,
                                amount: *amount,
                            },
                            ..Default::default()
                        });
                        if let Ok(player_transform) = player_query.get_single() {
                            let diff = transform.translation - player_transform.translation;
                            sound_event_writer
                                .send(SoundEvent::ShieldTransmute { relative_pos: diff })
                        }
                    }
                    Reagent::Strange => {
                        // Do Nothing
                    }
                    _ => commands.entity(*entity).despawn_recursive(),
                }
            }
        }
    }
}

pub struct ShieldPlugin;

impl Plugin for ShieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShieldCollision>()
            .add_startup_system(setup_shield_visuals)
            .add_systems(
                (
                    spawn_despawn_shield,
                    handle_collision,
                    handle_shield_collisions,
                )
                    .in_set(OnUpdate(GameState::InGame)),
            );
    }
}
