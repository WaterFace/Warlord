use bevy::{
    core_pipeline::{clear_color::ClearColorConfig, core_3d::Camera3dDepthLoadOp},
    prelude::*,
    reflect::TypeUuid,
    render::{
        camera::{self, CameraOutputMode, CameraRenderGraph, ScalingMode},
        render_resource::{
            AsBindGroup, AsBindGroupShaderType, BlendState, LoadOp, ShaderRef, ShaderType,
        },
        view::RenderLayers,
    },
};
use noisy_bevy::NoisyShaderPlugin;

use crate::camera::MainCamera;

#[derive(Bundle)]
pub struct StarfieldBundle {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StarfieldMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub render_layers: RenderLayers,
}

impl Default for StarfieldBundle {
    fn default() -> Self {
        Self {
            mesh: Handle::default(),
            material: Handle::default(),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            computed_visibility: ComputedVisibility::default(),
            render_layers: RenderLayers::layer(3),
        }
    }
}

#[derive(Bundle)]
pub struct StarfieldCameraBundle {
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
    pub render_layers: RenderLayers,
}

impl Default for StarfieldCameraBundle {
    fn default() -> Self {
        Self {
            camera: Camera {
                hdr: true,
                output_mode: CameraOutputMode::Write {
                    blend_state: Some(BlendState::ALPHA_BLENDING),
                    color_attachment_load_op: LoadOp::Load,
                },
                order: 1,
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
                depth_load_op: Camera3dDepthLoadOp::Load,
                ..Default::default()
            },
            tonemapping: Default::default(),
            dither: Default::default(),
            color_grading: Default::default(),
            render_layers: RenderLayers::layer(3),
        }
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "c58cc961-65cf-4eef-b3be-e12b99f55ec5"]
#[uniform(0, StarfieldMaterialUniform)]
pub struct StarfieldMaterial {
    // fields for the noise that forms the stars
    pub scale: f32,
    pub ramp_cutoff: f32,
    pub octaves: i32,
    pub lacunarity: f32,
    pub gain: f32,

    // fields for the brightness noise
    pub brightness_scale: f32,
    pub brightness_octaves: i32,
    pub brightness_lacunarity: f32,
    pub brightness_gain: f32,

    // scalar for the final brightness
    pub brightness: f32,

    // Parallax parameters
    pub parallax_factor: f32,
    pub camera_position: Vec2,
}

impl Material for StarfieldMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/starfield.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

#[derive(ShaderType)]
struct StarfieldMaterialUniform {
    // fields for the noise that forms the stars
    pub scale: f32,
    pub ramp_cutoff: f32,
    pub octaves: i32,
    pub lacunarity: f32,
    pub gain: f32,

    // fields for the brightness noise
    pub brightness_scale: f32,
    pub brightness_octaves: i32,
    pub brightness_lacunarity: f32,
    pub brightness_gain: f32,

    // scalar for the final brightness
    pub brightness: f32,

    // parallax parameters
    pub parallax_factor: f32,
    pub camera_position: Vec2,
}

impl AsBindGroupShaderType<StarfieldMaterialUniform> for StarfieldMaterial {
    fn as_bind_group_shader_type(
        &self,
        _images: &bevy::render::render_asset::RenderAssets<Image>,
    ) -> StarfieldMaterialUniform {
        StarfieldMaterialUniform {
            scale: self.scale,
            ramp_cutoff: self.ramp_cutoff,
            octaves: self.octaves,
            lacunarity: self.lacunarity,
            gain: self.gain,

            brightness_scale: self.brightness_scale,
            brightness_octaves: self.brightness_octaves,
            brightness_lacunarity: self.brightness_lacunarity,
            brightness_gain: self.brightness_gain,

            brightness: self.brightness,

            parallax_factor: self.parallax_factor,
            camera_position: self.camera_position,
        }
    }
}

fn update_starfield(
    camera_query: Query<&GlobalTransform, (With<MainCamera>, Without<Handle<StarfieldMaterial>>)>,
    mut starfields: ResMut<Assets<StarfieldMaterial>>,
) {
    let main_camera = camera_query.single();
    for mut starfield in starfields.iter_mut() {
        starfield.1.camera_position = main_camera.translation().truncate();
    }
}

pub struct StarfieldShaderPlugin;

impl Plugin for StarfieldShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(NoisyShaderPlugin)
            .add_plugin(MaterialPlugin::<StarfieldMaterial>::default())
            .add_system(update_starfield);
    }
}
