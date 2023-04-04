use std::time::Duration;

use bevy::prelude::*;

#[derive(Component, Debug)]
#[allow(dead_code)]
pub struct Heat {
    current: f32,
    limit: f32,
    reaction_threshold: f32,
    decay_rate: f32,
    decay_timer: Timer,
}

#[allow(dead_code)]
impl Heat {
    pub fn current(&self) -> f32 {
        self.current
    }
    pub fn fraction(&self) -> f32 {
        self.current / self.limit
    }
    pub fn can_react(&self) -> bool {
        self.current > self.reaction_threshold
    }
    pub fn add(&mut self, heat: f32) {
        self.current = (self.current + heat).clamp(0.0, self.limit);
        if heat > 0.0 {
            self.decay_timer.reset();
        }
    }
    pub fn tick(&mut self, dt: f32) {
        let leftover = dt - self.decay_timer.remaining_secs();
        self.decay_timer.tick(Duration::from_secs_f32(dt));
        if leftover > 0.0 && self.decay_timer.finished() {
            self.add(-self.decay_rate * leftover);
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
            current: 0.0,
            limit: 100.0,
            reaction_threshold: 75.0,
            decay_rate: 25.0,
            decay_timer: Timer::from_seconds(1.5, TimerMode::Once),
        }
    }
}

pub struct HeatPlugin;

impl Plugin for HeatPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(tick_heat);
    }
}
