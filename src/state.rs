use bevy::prelude::*;

use crate::{
    heat::Heat,
    inventory::{Inventory, Reagent},
    reaction::{Reaction, Reactions},
    shield::ShieldEmitter,
    weapon::{CargoDumper, MainGun},
};

#[derive(States, Default, Debug, Clone, Hash, Eq, PartialEq)]
pub enum GameState {
    #[default]
    MainMenu,
    Intro,
    InGame,
    Paused,
}

#[derive(States, Default, Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum ProgressStages {
    #[default]
    Exploration,
    GunAndHeat,
    CollectExotic,
    ShieldAndStrange,
    Continuum,
    End,
}

fn enter_exploration_stage(mut query: Query<&mut Inventory>) {
    for mut inventory in &mut query {
        inventory
            .reagent_mut(Reagent::Minerals)
            .set_threshold(Some(0.9));
    }
}

fn update_exploration_stage(
    query: Query<&Inventory>,
    mut stage: ResMut<NextState<ProgressStages>>,
) {
    for inventory in &query {
        let entry = inventory.reagent(Reagent::Minerals);
        debug_assert!(
            entry.threshold().is_some(),
            "Minerals threshold is unset! It should be set here"
        );
        if entry.fraction() >= entry.threshold().unwrap() {
            stage.set(ProgressStages::GunAndHeat);
        }
    }
}

fn exit_exploration_stage(mut query: Query<&mut Inventory>) {
    for mut inventory in &mut query {
        let entry = inventory.reagent_mut(Reagent::Minerals);
        entry.set_threshold(None);
        entry.add(-entry.current());
    }
}

fn enter_gun_and_heat_stage(mut query: Query<(&mut Heat, &mut MainGun)>) {
    for (mut heat, mut main_gun) in &mut query {
        heat.set_enabled(true);
        heat.set_threshold_visible(false);
        main_gun.enabled = true;
    }
}

fn update_gun_and_heat_stage(
    query: Query<&Inventory>,
    mut stage: ResMut<NextState<ProgressStages>>,
) {
    for inventory in &query {
        let entry = inventory.reagent(Reagent::Exotic);
        if entry.current() > 0.0 {
            stage.set(ProgressStages::CollectExotic);
        }
    }
}

fn exit_gun_and_heat_stage(mut query: Query<&mut Heat>) {
    for mut heat in &mut query {
        heat.set_threshold_visible(true);
    }
}

fn enter_collect_exotic_stage(mut query: Query<&mut Inventory>) {
    for mut inventory in &mut query {
        inventory
            .reagent_mut(Reagent::Exotic)
            .set_threshold(Some(0.9));
    }
}

fn update_collect_exotic_stage(
    query: Query<&Inventory>,
    mut stage: ResMut<NextState<ProgressStages>>,
) {
    for inventory in &query {
        let entry = inventory.reagent(Reagent::Exotic);
        debug_assert!(
            entry.threshold().is_some(),
            "Exotic threshold is unset! It should be set here"
        );
        if entry.fraction() >= entry.threshold().unwrap() {
            stage.set(ProgressStages::ShieldAndStrange);
        }
    }
}

fn exit_collect_exotic_stage(mut query: Query<&mut Inventory>) {
    for mut inventory in &mut query {
        inventory.reagent_mut(Reagent::Exotic).set_threshold(None);
    }
}

fn enter_shield_and_strange_stage(
    mut query: Query<(&mut Inventory, &mut ShieldEmitter, &mut CargoDumper)>,
) {
    for (mut inventory, mut shield_emitter, mut cargo_dumper) in &mut query {
        inventory
            .reagent_mut(Reagent::Strange)
            .set_threshold(Some(0.9));
        shield_emitter.enabled = true;
        cargo_dumper.enabled = true;
    }
}

fn update_shield_and_strange_stage(
    query: Query<&Inventory>,
    mut stage: ResMut<NextState<ProgressStages>>,
) {
    for inventory in &query {
        let entry = inventory.reagent(Reagent::Strange);
        debug_assert!(
            entry.threshold().is_some(),
            "Strange threshold is unset! It should be set here"
        );
        if entry.fraction() >= entry.threshold().unwrap() {
            stage.set(ProgressStages::Continuum);
        }
    }
}

fn exit_shield_and_strange_stage(mut query: Query<&mut Inventory>) {
    for mut inventory in &mut query {
        inventory.reagent_mut(Reagent::Strange).set_threshold(None);
    }
}

fn enter_continuum_stage(mut query: Query<&mut Inventory>, mut reactions: ResMut<Reactions>) {
    for mut inventory in &mut query {
        inventory
            .reagent_mut(Reagent::Continuum)
            .set_threshold(Some(0.99));
    }
    reactions.reactions.push(Reaction {
        reagent1: Reagent::Exotic,
        reagent2: Some(Reagent::Strange),
        needs_heat: true,
        rate: 1.0,
        result: Some(Reagent::Continuum),
    });
}

fn update_continuum_stage(query: Query<&Inventory>, mut stage: ResMut<NextState<ProgressStages>>) {
    for inventory in &query {
        let entry = inventory.reagent(Reagent::Continuum);
        debug_assert!(
            entry.threshold().is_some(),
            "Continuum threshold is unset! It should be set here"
        );
        if entry.fraction() >= entry.threshold().unwrap() {
            stage.set(ProgressStages::End);
        }
    }
}

fn exit_continuum_stage(mut query: Query<&mut Inventory>) {
    for mut inventory in &mut query {
        inventory
            .reagent_mut(Reagent::Continuum)
            .set_threshold(None);
    }
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>().add_state::<ProgressStages>();

        app.add_system(enter_exploration_stage.in_schedule(OnEnter(ProgressStages::Exploration)))
            .add_system(update_exploration_stage.in_set(OnUpdate(ProgressStages::Exploration)))
            .add_system(exit_exploration_stage.in_schedule(OnExit(ProgressStages::Exploration)));

        app.add_system(enter_gun_and_heat_stage.in_schedule(OnEnter(ProgressStages::GunAndHeat)))
            .add_system(update_gun_and_heat_stage.in_set(OnUpdate(ProgressStages::GunAndHeat)))
            .add_system(exit_gun_and_heat_stage.in_schedule(OnExit(ProgressStages::GunAndHeat)));

        app.add_system(
            enter_collect_exotic_stage.in_schedule(OnEnter(ProgressStages::CollectExotic)),
        )
        .add_system(update_collect_exotic_stage.in_set(OnUpdate(ProgressStages::CollectExotic)))
        .add_system(exit_collect_exotic_stage.in_schedule(OnExit(ProgressStages::CollectExotic)));

        app.add_system(
            enter_shield_and_strange_stage.in_schedule(OnEnter(ProgressStages::ShieldAndStrange)),
        )
        .add_system(
            update_shield_and_strange_stage.in_set(OnUpdate(ProgressStages::ShieldAndStrange)),
        )
        .add_system(
            exit_shield_and_strange_stage.in_schedule(OnExit(ProgressStages::ShieldAndStrange)),
        );

        app.add_system(enter_continuum_stage.in_schedule(OnEnter(ProgressStages::Continuum)))
            .add_system(update_continuum_stage.in_set(OnUpdate(ProgressStages::Continuum)))
            .add_system(exit_continuum_stage.in_schedule(OnExit(ProgressStages::Continuum)));
    }
}
