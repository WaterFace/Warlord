use bevy::prelude::*;

use crate::collectible::CollectionEvent;

// KEEP THIS UPDATED:
const REAGENT_TYPES: usize = 1;

#[derive(Debug, Clone, Copy)]
pub enum Reagent {
    Minerals = 0,
}

#[derive(Component, Debug)]
pub struct InventoryEntry {
    current: f32,
    limit: f32,
}

impl InventoryEntry {
    pub fn current(&self) -> f32 {
        self.current
    }

    pub fn limit(&self) -> f32 {
        self.limit
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
}

impl Default for Inventory {
    fn default() -> Self {
        Inventory {
            reagents: [InventoryEntry {
                current: 0.0,
                limit: 10.0,
            }],
        }
    }
}

fn handle_collection_event(
    mut reader: EventReader<CollectionEvent>,
    mut inventory_query: Query<&mut Inventory>,
) {
    for ev in reader.iter() {
        for mut inv in &mut inventory_query {
            debug!("Adding {:?} to reagent {:?}", ev.amount, ev.reagent);
            inv.reagent_mut(ev.reagent).add(ev.amount);
        }
    }
}

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_collection_event);
    }
}
