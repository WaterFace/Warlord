use bevy::prelude::*;

#[derive(Debug)]
pub enum SoundEvent {
    ButtonClick,
    RockDestroyed { relative_pos: Vec3 },
    Collected,
    NextStage,
    CannonFire { direction: f32 },
    ShieldTransmute { relative_pos: Vec3 },
    RockCollision,
}

#[derive(Resource)]
struct VirtualListener {
    transform: Transform,
}

#[derive(Resource, Debug)]
pub struct VolumeSettings {
    pub sound_effects: f32,
    pub music: f32,
    pub mute: bool,
}

impl Default for VolumeSettings {
    fn default() -> Self {
        Self {
            sound_effects: 0.5,
            music: 0.5,
            mute: false,
        }
    }
}

#[derive(Resource)]
struct BackgroundMusic {
    handle: Handle<AudioSink>,
}

fn start_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    audio_sinks: Res<Assets<AudioSink>>,
    volume: Res<VolumeSettings>,
) {
    let music = asset_server.load("sound/bgm.mp3");
    let weak_handle = audio.play_with_settings(
        music,
        PlaybackSettings {
            repeat: true,
            volume: if volume.mute {
                0.0
            } else {
                2.0 * volume.sound_effects
            },
            speed: 1.0,
        },
    );
    let strong_handle = audio_sinks.get_handle(weak_handle);

    commands.insert_resource(BackgroundMusic {
        handle: strong_handle,
    });
}

fn set_music_volume(
    volume: Res<VolumeSettings>,
    audio_sinks: Res<Assets<AudioSink>>,
    background_music: Res<BackgroundMusic>,
) {
    if volume.is_changed() {
        let Some(sink) = audio_sinks.get(&background_music.handle) else {debug!("Couldn't find background music!"); return};
        if volume.mute {
            sink.set_volume(0.0);
        } else {
            sink.set_volume(2.0 * volume.music);
        }
    }
}

fn handle_sound_events(
    mut reader: EventReader<SoundEvent>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    listener: Res<VirtualListener>,
    volume: Res<VolumeSettings>,
) {
    for ev in reader.iter() {
        let settings = PlaybackSettings {
            repeat: false,
            volume: if volume.mute {
                0.0
            } else {
                2.0 * volume.sound_effects
            },
            speed: 1.0,
        };
        match ev {
            SoundEvent::ButtonClick => {
                let sound = asset_server.load("sound/buttonclick.mp3");
                audio.play_with_settings(sound, settings);
            }
            SoundEvent::CannonFire { direction } => {
                let pos = (Vec2::from_angle(*direction), 0.0).into();
                let sound = asset_server.load("sound/cannon.mp3");
                audio.play_spatial_with_settings(sound, settings, listener.transform, 1.0, pos);
            }
            SoundEvent::RockDestroyed { relative_pos } => {
                let sound = asset_server.load("sound/rock.mp3");
                audio.play_spatial_with_settings(
                    sound,
                    settings,
                    listener.transform,
                    1.0,
                    relative_pos.normalize_or_zero(),
                );
            }
            SoundEvent::Collected => {
                let sound = asset_server.load("sound/collect.mp3");
                audio.play_with_settings(sound, settings);
            }
            SoundEvent::NextStage => {
                let sound = asset_server.load("sound/nextstage.mp3");
                audio.play_with_settings(sound, settings);
            }
            SoundEvent::ShieldTransmute { relative_pos } => {
                let sound = asset_server.load("sound/transmute.mp3");
                audio.play_spatial_with_settings(
                    sound,
                    settings,
                    listener.transform,
                    1.0,
                    relative_pos.normalize_or_zero(),
                );
            }
            SoundEvent::RockCollision => {
                let sound = asset_server.load("sound/hitrock.mp3");
                audio.play_with_settings(sound, settings);
            }
        }
    }
}

fn setup_sound(mut commands: Commands) {
    let transform = Transform::from_xyz(0.0, 0.0, 0.0).looking_to(Vec3::Y, Vec3::Z);

    commands.insert_resource(VirtualListener { transform });
}

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SoundEvent>()
            .insert_resource(VolumeSettings::default())
            .add_startup_system(setup_sound)
            .add_startup_system(start_music)
            .add_system(set_music_volume)
            .add_system(handle_sound_events);
    }
}
