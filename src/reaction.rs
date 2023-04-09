use bevy::prelude::*;

use crate::{
    heat::Heat,
    inventory::{Inventory, Reagent, ReagentEvent},
    state::GameState,
};

#[derive(Debug)]
pub struct Reaction {
    pub reagent1: Reagent,
    pub reagent2: Option<Reagent>,
    pub needs_heat: bool,
    pub rate: f32,
    pub result: Option<Reagent>,
}

impl Reaction {
    pub fn tick(
        &self,
        inventory: &mut Inventory,
        heat: &Heat,
        dt: f32,
        mut send: impl FnMut(ReagentEvent),
    ) {
        if self.needs_heat && !heat.can_react() {
            // The reaction needs heat, but we don't have enough
            return;
        }

        if let Some(reagent2) = self.reagent2 {
            // two-reagent reaction
            let mut amount_reacted = {
                let entry1 = inventory.reagent(self.reagent1);
                let entry2 = inventory.reagent(reagent2);
                entry1.current().min(entry2.current()).min(dt * self.rate)
            };
            if let Some(result) = self.result {
                // Will only react as long as there's space
                let result_entry = inventory.reagent_mut(result);
                amount_reacted = amount_reacted.min(result_entry.limit() - result_entry.current());
                result_entry.add(amount_reacted);
                send(ReagentEvent {
                    reagent: result,
                    delta: amount_reacted,
                });
            }
            {
                let entry1 = inventory.reagent_mut(self.reagent1);
                entry1.add(-amount_reacted);
                send(ReagentEvent {
                    reagent: self.reagent1,
                    delta: -amount_reacted,
                });
            }
            {
                let entry2 = inventory.reagent_mut(reagent2);
                entry2.add(-amount_reacted);
                send(ReagentEvent {
                    reagent: reagent2,
                    delta: -amount_reacted,
                });
            }
        } else {
            // one-reagent reaction
            let mut amount_reacted = {
                let entry = inventory.reagent(self.reagent1);
                entry.current().min(dt * self.rate)
            };
            if let Some(result) = self.result {
                // Will only react as long as there's space
                let result_entry = inventory.reagent_mut(result);
                amount_reacted = amount_reacted.min(result_entry.limit() - result_entry.current());
                result_entry.add(amount_reacted);
                send(ReagentEvent {
                    reagent: result,
                    delta: amount_reacted,
                });
            }

            let entry = inventory.reagent_mut(self.reagent1);
            entry.add(-amount_reacted);
            send(ReagentEvent {
                reagent: self.reagent1,
                delta: -amount_reacted,
            });
        }
    }
}

#[derive(Resource)]
pub struct Reactions {
    pub reactions: Vec<Reaction>,
}

impl Default for Reactions {
    fn default() -> Self {
        let reactions = vec![Reaction {
            reagent1: Reagent::Minerals,
            reagent2: None,
            needs_heat: true,
            rate: 0.5,
            result: Some(Reagent::Exotic),
        }];

        Reactions { reactions }
    }
}

fn perform_reactions(
    mut query: Query<(&mut Inventory, &Heat)>,
    reactions: Res<Reactions>,
    time: Res<Time>,
    mut writer: EventWriter<ReagentEvent>,
) {
    for (mut inventory, heat) in &mut query {
        for reaction in reactions.reactions.iter() {
            reaction.tick(&mut inventory, &heat, time.delta_seconds(), |ev| {
                writer.send(ev)
            });
        }
    }
}

pub struct ReactionPlugin;

impl Plugin for ReactionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(perform_reactions.in_set(OnUpdate(GameState::InGame)));
    }
}
