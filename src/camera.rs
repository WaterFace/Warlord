use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub struct MainCamera;

#[derive(Component, Debug)]
pub struct SmoothFollow {
    pub target: Option<Entity>,
    pub offset: Vec3,
    pub focus_radius: f32,
}

impl Default for SmoothFollow {
    fn default() -> Self {
        Self {
            target: None,
            offset: Vec3::new(0.0, 0.0, 10.0),
            focus_radius: 100.0,
        }
    }
}

#[derive(Component, Debug, Default)]
pub struct FocusPoint {
    pub focus: Vec3,
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
            focus_point.focus
        } else {
            target_transform.translation
        };
        // Now `focus` holds the point we want the camera to follow, however we got it
        if smooth_follow.focus_radius > 0.0 {
            let dist = Vec3::distance(focus + smooth_follow.offset, transform.translation);
            if dist > smooth_follow.focus_radius {
                transform.translation = Vec3::lerp(
                    focus,
                    transform.translation,
                    smooth_follow.focus_radius / dist,
                );
            }
        } else {
            transform.translation = focus;
        }
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(follow_target);
    }
}
