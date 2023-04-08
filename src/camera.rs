use bevy::{
    core_pipeline::{
        bloom::BloomSettings, clear_color::ClearColorConfig, tonemapping::Tonemapping,
    },
    prelude::*,
    render::camera::{CameraRenderGraph, ScalingMode},
};

use crate::state::GameState;

#[derive(Component, Debug, Default)]
pub struct MainCamera;

#[derive(Bundle)]
pub struct MainCameraBundle {
    pub camera: Camera,
    pub camera_render_graph: bevy::render::camera::CameraRenderGraph,
    pub projection: Projection,
    pub visible_entities: bevy::render::view::VisibleEntities,
    pub frustum: bevy::render::primitives::Frustum,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub camera_3d: Camera3d,
    pub tonemapping: bevy::core_pipeline::tonemapping::Tonemapping,
    pub dither: bevy::core_pipeline::tonemapping::DebandDither,
    pub color_grading: bevy::render::view::ColorGrading,
    pub bloom_settings: BloomSettings,
    pub smooth_follow: SmoothFollow,
    pub main_camera: MainCamera,
}

impl Default for MainCameraBundle {
    fn default() -> Self {
        Self {
            camera: Camera {
                hdr: true,

                ..Default::default()
            },
            camera_render_graph: CameraRenderGraph::new(bevy::core_pipeline::core_3d::graph::NAME),
            projection: Projection::Orthographic(OrthographicProjection {
                scale: 15.0,
                scaling_mode: ScalingMode::FixedVertical(2.0),
                ..Default::default()
            }),
            visible_entities: Default::default(),
            frustum: Default::default(),
            transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_to(Vec3::NEG_Z, Vec3::Y),
            global_transform: Default::default(),
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::None,
                ..Default::default()
            },
            tonemapping: Tonemapping::AcesFitted,
            dither: Default::default(),
            color_grading: Default::default(),
            bloom_settings: BloomSettings {
                intensity: 0.5,
                ..Default::default()
            },
            smooth_follow: Default::default(),
            main_camera: Default::default(),
        }
    }
}

#[derive(Component, Debug)]
pub struct SmoothFollow {
    pub target: Option<Entity>,
    pub offset: Vec3,
    pub focus_radius: f32,
    pub focus_centering: f32,
}

impl Default for SmoothFollow {
    fn default() -> Self {
        Self {
            target: None,
            offset: Vec3::new(0.0, 0.0, 10.0),
            focus_radius: 1.0,
            focus_centering: 0.5,
        }
    }
}

#[derive(Component, Debug, Default)]
pub struct FocusPoint {
    pub offset: Vec3,
}

fn follow_target(
    mut query: Query<(&SmoothFollow, &mut Transform)>,
    target_query: Query<(&Transform, Option<&FocusPoint>), Without<SmoothFollow>>,
    time: Res<Time>,
) {
    for (smooth_follow, mut transform) in &mut query {
        // If the camera doesn't have a target, give up
        let Some(target_entity) = smooth_follow.target else {
            continue;
        };
        // if the target the camera is pointing to doesn't exist, give up
        // TODO: maybe clear the camera's focus in this case?
        let Ok((target_transform, focus_point)) = target_query.get(target_entity) else {
            continue;
        };

        let focus = if let Some(focus_point) = focus_point {
            target_transform.translation + focus_point.offset
        } else {
            target_transform.translation
        };
        // Now `focus` holds the point we want the camera to follow, however we got it
        if smooth_follow.focus_radius > 0.0 {
            let dist = Vec3::distance(focus + smooth_follow.offset, transform.translation);
            let mut t = 1.0;
            if dist > 0.01 && smooth_follow.focus_centering > 0.0 {
                t = f32::powf(1.0 - smooth_follow.focus_centering, time.delta_seconds());
            }
            if dist > smooth_follow.focus_radius {
                transform.translation = Vec3::lerp(
                    focus,
                    transform.translation,
                    smooth_follow.focus_radius / dist,
                );
                t = f32::min(t, smooth_follow.focus_radius / dist);
            }
            transform.translation =
                Vec3::lerp(focus + smooth_follow.offset, transform.translation, t);
        } else {
            transform.translation = focus + smooth_follow.offset;
        }
        // info!("Camera position: {:?}", transform.translation);
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(follow_target.in_set(OnUpdate(GameState::InGame)));
    }
}
