use bevy::{
    core_pipeline::{
        bloom::BloomSettings, clear_color::ClearColorConfig, core_3d::Camera3dDepthLoadOp,
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        camera::{CameraOutputMode, CameraRenderGraph, ScalingMode},
        render_resource::{AsBindGroup, BlendState, LoadOp, ShaderRef, ShaderType},
        view::RenderLayers,
    },
};
use noisy_bevy::NoisyShaderPlugin;

use crate::{camera::MainCamera, state::GameState};

#[derive(Component, Debug, Default)]
pub struct StarfieldMesh;

#[derive(Bundle)]
pub struct StarfieldBundle {
    pub starfield_mesh: StarfieldMesh,
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
            starfield_mesh: StarfieldMesh,
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

#[derive(Component, Debug)]
pub struct StarfieldCamera;

#[derive(Bundle)]
pub struct StarfieldCameraBundle {
    pub starfield_camera: StarfieldCamera,
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
    pub bloom_settings: BloomSettings,
}

impl Default for StarfieldCameraBundle {
    fn default() -> Self {
        Self {
            starfield_camera: StarfieldCamera,
            camera: Camera {
                hdr: true,
                output_mode: CameraOutputMode::Write {
                    blend_state: Some(BlendState::ALPHA_BLENDING),
                    color_attachment_load_op: LoadOp::Load,
                },
                order: -1,
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
            bloom_settings: BloomSettings::default(),
        }
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone, ShaderType)]
#[uuid = "c58cc961-65cf-4eef-b3be-e12b99f55ec5"]
// #[uniform(0, StarfieldMaterialUniform)]
pub struct StarfieldMaterial {
    #[uniform(0)]
    pub camera_position: Vec3,
    #[uniform(0)]
    pub parallax_factor: f32,
    #[uniform(0)]
    pub resolution: Vec3,
    #[uniform(0)]
    pub time: f32,
}

impl Default for StarfieldMaterial {
    fn default() -> Self {
        Self {
            camera_position: Vec3::ZERO,
            parallax_factor: 1.0,
            resolution: Vec3::ZERO,
            time: 0.0,
        }
    }
}

impl Material for StarfieldMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/starfield.vert".into()
    }

    fn fragment_shader() -> ShaderRef {
        // "shaders/starfield.wgsl".into()
        "shaders/starfield.frag".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.vertex.entry_point = "main".into();
        descriptor.fragment.as_mut().unwrap().entry_point = "main".into();
        Ok(())
    }
}

// #[derive(ShaderType)]
// struct StarfieldMaterialUniform {
//     pub camera_position: Vec2,
//     // pub parallax_factor: f32,
//     pub resolution: Vec2,
//     // pub time: f32,
// }

// impl AsBindGroupShaderType<StarfieldMaterialUniform> for StarfieldMaterial {
//     fn as_bind_group_shader_type(
//         &self,
//         _images: &bevy::render::render_asset::RenderAssets<Image>,
//     ) -> StarfieldMaterialUniform {
//         StarfieldMaterialUniform {
//             // parallax_factor: self.parallax_factor,
//             // time: self.time,
//             camera_position: self.camera_position,
//             resolution: self.resolution,
//         }
//     }
// }

fn update_starfield_on_resize(
    starfield_camera_query: Query<&Projection, (With<StarfieldCamera>, Changed<Projection>)>,
    mut starfield_query: Query<&mut Transform, With<StarfieldMesh>>,
    mut starfields: ResMut<Assets<StarfieldMaterial>>,
) {
    let Ok(proj) = starfield_camera_query.get_single() else { return; };
    let Projection::Orthographic(proj) = proj else { return };
    let Rect { min, max } = proj.area;
    let size = Vec2::abs(max - min);

    for mut starfield in starfields.iter_mut() {
        starfield.1.resolution = (size, 0.0).into();
    }

    for mut starfield in &mut starfield_query {
        debug!("Resized starfield to {:?}", size);
        starfield.scale = Vec3::new(size.x, size.y, 1.0);
    }
}

fn update_starfield_time(mut starfields: ResMut<Assets<StarfieldMaterial>>, time: Res<Time>) {
    for mut starfield in starfields.iter_mut() {
        starfield.1.time = time.elapsed_seconds_wrapped();
    }
}

fn update_starfield_camera_position(
    main_camera_query: Query<
        &GlobalTransform,
        (With<MainCamera>, Without<Handle<StarfieldMaterial>>),
    >,
    mut starfields: ResMut<Assets<StarfieldMaterial>>,
) {
    let Ok(main_camera) = main_camera_query.get_single() else { return; };
    for mut starfield in starfields.iter_mut() {
        starfield.1.camera_position = main_camera.translation();
    }
}

pub struct StarfieldShaderPlugin;

impl Plugin for StarfieldShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(NoisyShaderPlugin)
            .add_plugin(MaterialPlugin::<StarfieldMaterial>::default())
            .add_systems(
                (
                    update_starfield_on_resize,
                    update_starfield_camera_position,
                    update_starfield_time,
                )
                    .in_set(OnUpdate(GameState::InGame)),
            );
    }
}
