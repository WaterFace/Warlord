use bevy::prelude::*;

use crate::{collectible::CollectionEvent, state::GameState};

// KEEP THIS UPDATED:
pub const REAGENT_TYPES: usize = 2;

#[derive(Debug, Clone, Copy)]
pub enum Reagent {
    Minerals = 0,
    Exotic = 1,
}

impl TryFrom<usize> for Reagent {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Minerals),
            1 => Ok(Self::Exotic),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct ReagentEvent {
    pub reagent: Reagent,
    pub delta: f32,
}

#[derive(Component, Debug)]
pub struct InventoryEntry {
    current: f32,
    limit: f32,
    visible: bool,
    color: Color,
    name: String,
}

impl InventoryEntry {
    pub fn current(&self) -> f32 {
        self.current
    }

    pub fn limit(&self) -> f32 {
        self.limit
    }

    pub fn visibile(&self) -> bool {
        self.visible
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fraction(&self) -> f32 {
        self.current / self.limit
    }

    pub fn add(&mut self, amount: f32) {
        self.current += amount;
        self.current = self.current.clamp(0.0, self.limit);
    }
}

#[derive(Component, Debug)]
pub struct Inventory {
    reagents: [InventoryEntry; REAGENT_TYPES],
}

impl Inventory {
    pub fn reagent(&self, reagent: Reagent) -> &InventoryEntry {
        &self.reagents[reagent as usize]
    }
    pub fn reagent_mut(&mut self, reagent: Reagent) -> &mut InventoryEntry {
        &mut self.reagents[reagent as usize]
    }
    pub fn reagents(&self) -> impl Iterator<Item = (Reagent, &'_ InventoryEntry)> {
        self.reagents.iter().enumerate().map(|(i, e)| (TryInto::<Reagent>::try_into(i).expect("There should be the same number of entries in `reagents` as there are in the Reagent enum."), e))
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Inventory {
            reagents: [
                InventoryEntry {
                    current: 0.0,
                    limit: 10.0,
                    visible: true,
                    color: Color::CYAN,
                    name: "MINERALS".into(),
                },
                InventoryEntry {
                    current: 0.0,
                    limit: 25.0,
                    visible: false,
                    color: Color::rgb(1.0, 0.0, 1.0),
                    name: "EXOTIC MATTER".into(),
                },
            ],
        }
    }
}

fn handle_collection_event(
    mut reader: EventReader<CollectionEvent>,
    mut inventory_query: Query<&mut Inventory>,
    mut writer: EventWriter<ReagentEvent>,
) {
    for ev in reader.iter() {
        for mut inv in &mut inventory_query {
            debug!("Adding {:?} to reagent {:?}", ev.amount, ev.reagent);
            inv.reagent_mut(ev.reagent).add(ev.amount);
            writer.send(ReagentEvent {
                reagent: ev.reagent,
                delta: ev.amount,
            });
        }
    }
}

fn set_visibility(mut reader: EventReader<ReagentEvent>, mut query: Query<&mut Inventory>) {
    let Ok(mut inventory) = query.get_single_mut() else { return; };
    for ev in reader.iter() {
        if ev.delta > 0.0 {
            inventory.reagent_mut(ev.reagent).visible = true;
        }
    }
}

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ReagentEvent>().add_systems(
            (handle_collection_event, set_visibility).in_set(OnUpdate(GameState::InGame)),
        );
    }
}
