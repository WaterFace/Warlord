use std::time::Duration;

use bevy::{prelude::*, render::view::RenderLayers, sprite::Anchor};

use crate::{
    heat::Heat,
    inventory::{Inventory, Reagent},
    reaction::{Reaction, Reactions},
    shield::ShieldEmitter,
    sound::SoundEvent,
    ui::{CustomUICamera, EnabledControls},
    weapon::{CargoDumper, MainGun},
};

#[derive(States, Default, Debug, Clone, Hash, Eq, PartialEq)]
pub enum GameState {
    #[default]
    MainMenu,
    Intro,
    InGame,
    Outro,
    EndScreen,
    Paused,
}

#[derive(States, Default, Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum ProgressStages {
    #[default]
    None,
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

fn enter_gun_and_heat_stage(
    mut query: Query<(&mut Heat, &mut MainGun)>,
    mut enabled_controls: ResMut<EnabledControls>,
    mut sound_event_writer: EventWriter<SoundEvent>,
) {
    for (mut heat, mut main_gun) in &mut query {
        heat.set_enabled(true);
        // heat.set_threshold_visible(false);
        main_gun.enabled = true;
    }
    *enabled_controls |= EnabledControls::Shoot;
    sound_event_writer.send(SoundEvent::NextStage);
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

fn enter_collect_exotic_stage(
    mut query: Query<&mut Inventory>,
    mut sound_event_writer: EventWriter<SoundEvent>,
) {
    for mut inventory in &mut query {
        inventory
            .reagent_mut(Reagent::Exotic)
            .set_threshold(Some(0.9));
    }
    sound_event_writer.send(SoundEvent::NextStage);
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
    mut enabled_controls: ResMut<EnabledControls>,
    mut sound_event_writer: EventWriter<SoundEvent>,
) {
    for (mut inventory, mut shield_emitter, mut cargo_dumper) in &mut query {
        inventory
            .reagent_mut(Reagent::Strange)
            .set_threshold(Some(0.9));
        shield_emitter.enabled = true;
        cargo_dumper.enabled = true;
    }
    *enabled_controls |= EnabledControls::Dump | EnabledControls::Shield;
    sound_event_writer.send(SoundEvent::NextStage);
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

fn enter_continuum_stage(
    mut query: Query<&mut Inventory>,
    mut reactions: ResMut<Reactions>,
    mut sound_event_writer: EventWriter<SoundEvent>,
) {
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
    sound_event_writer.send(SoundEvent::NextStage);
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

#[derive(Component, Debug)]
pub struct FadeOut {
    timer: Timer,
}

fn enter_end_stage(mut commands: Commands, mut sound_event_writer: EventWriter<SoundEvent>) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                anchor: Anchor::Center,
                custom_size: Some(Vec2::new(10.0, 10.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 10.0),
            ..Default::default()
        },
        FadeOut {
            timer: Timer::from_seconds(5.0, TimerMode::Once),
        },
        RenderLayers::layer(1), // So the ui camera can see it
    ));
    sound_event_writer.send(SoundEvent::NextStage);
}

fn update_end_stage(
    mut query: Query<(&mut FadeOut, &mut Sprite)>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    ui_camera: Query<&Camera, With<CustomUICamera>>,
    time: Res<Time>,
) {
    if current_state.0 != GameState::InGame {
        return;
    }
    let Ok(ui_camera) = ui_camera.get_single() else {return;};
    let Some(size) = ui_camera.logical_viewport_size() else {return;};

    for (mut fadeout, mut sprite) in &mut query {
        fadeout
            .timer
            .tick(Duration::from_secs_f32(time.delta_seconds()));
        let a = fadeout.timer.percent();
        sprite.custom_size = Some(size);
        sprite.color = Color::BLACK.with_a(a);
        if fadeout.timer.finished() {
            next_state.set(GameState::Outro);
        }
    }
}

fn exit_end_stage(mut commands: Commands, query: Query<Entity, With<FadeOut>>) {
    for e in &query {
        commands.entity(e).despawn_recursive();
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

        app.add_system(enter_end_stage.in_schedule(OnEnter(ProgressStages::End)))
            .add_system(update_end_stage.in_set(OnUpdate(ProgressStages::End)))
            .add_system(exit_end_stage.in_schedule(OnExit(ProgressStages::End)));
    }
}
