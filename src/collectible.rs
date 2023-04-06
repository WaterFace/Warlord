use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component, Debug, Default)]
pub struct Collectible;

#[derive(Bundle, Debug)]
pub struct CollectibleBundle {
    pub collectible: Collectible,

    pub transform: Transform,
    pub global_transform: Transform,

    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,

    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub sensor: Sensor,
    pub velocity: Velocity,
}

impl Default for CollectibleBundle {
    fn default() -> Self {
        CollectibleBundle {
            collectible: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            mesh: Default::default(),
            material: Default::default(),
            visibility: Visibility::Visible,
            computed_visibility: Default::default(),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::ball(0.5),
            sensor: Default::default(),
            velocity: Default::default(),
        }
    }
}

// TODO: move this somewhere else?
#[derive(Debug, Clone)]
pub struct Tetrahedron {
    pub size: f32,
}

impl Default for Tetrahedron {
    fn default() -> Self {
        Tetrahedron {
            // This side length corresponds to the tetrahedron
            // with points on the unit sphere
            size: f32::sqrt(8.0 / 3.0),
        }
    }
}

impl From<Tetrahedron> for Mesh {
    fn from(tet: Tetrahedron) -> Self {
        let v1: Vec3 = [f32::sqrt(8.0 / 9.0), 0.0, -1.0 / 3.0].into();
        let v2: Vec3 = [-f32::sqrt(2.0 / 9.0), f32::sqrt(2.0 / 3.0), -1.0 / 3.0].into();
        let v3: Vec3 = [-f32::sqrt(2.0 / 9.0), -f32::sqrt(2.0 / 3.0), -1.0 / 3.0].into();
        let v4: Vec3 = [0.0, 0.0, 1.0].into();

        let n1 = Vec3::cross(v3 - v4, v1 - v4).into();
        let n2 = Vec3::cross(v1 - v4, v2 - v4).into();
        let n3 = Vec3::cross(v2 - v4, v3 - v4).into();
        let n4 = Vec3::cross(v2 - v3, v1 - v3).into();

        let default_side_length = f32::sqrt(8.0 / 3.0);

        let v1 = (v1 * tet.size / default_side_length).into();
        let v2 = (v2 * tet.size / default_side_length).into();
        let v3 = (v3 * tet.size / default_side_length).into();
        let v4 = (v4 * tet.size / default_side_length).into();

        // TODO: uv coordinates

        let vertices = [
            (v1, n1, [0.0, 0.0]),
            (v4, n1, [0.0, 0.0]),
            (v3, n1, [0.0, 0.0]),
            (v1, n2, [0.0, 0.0]),
            (v2, n2, [0.0, 0.0]),
            (v4, n2, [0.0, 0.0]),
            (v2, n3, [0.0, 0.0]),
            (v3, n3, [0.0, 0.0]),
            (v4, n3, [0.0, 0.0]),
            (v1, n4, [0.0, 0.0]),
            (v3, n4, [0.0, 0.0]),
            (v2, n4, [0.0, 0.0]),
        ];

        let indices = bevy::render::mesh::Indices::U32(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

        let positions: Vec<[f32; 3]> = vertices.iter().map(|(p, _, _)| *p).collect();
        let normals: Vec<[f32; 3]> = vertices.iter().map(|(_, n, _)| *n).collect();
        let uvs: Vec<[f32; 2]> = vertices.iter().map(|(_, _, uv)| *uv).collect();

        let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}

pub struct CollectiblePlugin;

impl Plugin for CollectiblePlugin {
    fn build(&self, app: &mut App) {}
}
