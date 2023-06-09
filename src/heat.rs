use std::time::Duration;

use bevy::prelude::*;

use crate::state::GameState;

#[derive(Component, Debug)]
#[allow(dead_code)]
pub struct Heat {
    enabled: bool,
    current: f32,
    limit: f32,
    reaction_threshold: f32,
    threshold_visible: bool,
    decay_rate: f32,
    decay_timer: Timer,
}

#[allow(dead_code)]
impl Heat {
    pub fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    pub fn current(&self) -> f32 {
        self.current
    }
    pub fn limit(&self) -> f32 {
        self.limit
    }
    pub fn fraction(&self) -> f32 {
        self.current / self.limit
    }
    pub fn reaction_threshold(&self) -> f32 {
        self.reaction_threshold
    }
    pub fn threshold_visible(&self) -> bool {
        self.threshold_visible
    }
    pub fn set_threshold_visible(&mut self, visible: bool) {
        self.threshold_visible = visible;
    }
    pub fn can_react(&self) -> bool {
        self.fraction() > self.reaction_threshold
    }
    pub fn add(&mut self, heat: f32) {
        self.current = (self.current + heat).clamp(0.0, self.limit);
        self.decay_timer.reset();
    }
    pub fn tick(&mut self, dt: f32) {
        let leftover = dt - self.decay_timer.remaining_secs();
        self.decay_timer.tick(Duration::from_secs_f32(dt));
        if leftover > 0.0 && self.decay_timer.finished() {
            self.current -= self.decay_rate * leftover;
            self.current = self.current.clamp(0.0, self.limit);
        }
    }
}

fn tick_heat(mut query: Query<&mut Heat>, time: Res<Time>) {
    for mut heat in &mut query {
        heat.tick(time.delta_seconds());
    }
}

impl Default for Heat {
    fn default() -> Self {
        Self {
            enabled: false,
            current: 0.0,
            limit: 100.0,
            reaction_threshold: 0.75,
            threshold_visible: true,
            decay_rate: 25.0,
            decay_timer: Timer::from_seconds(1.5, TimerMode::Once),
        }
    }
}

pub struct HeatPlugin;

impl Plugin for HeatPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(tick_heat.in_set(OnUpdate(GameState::InGame)));
    }
}
