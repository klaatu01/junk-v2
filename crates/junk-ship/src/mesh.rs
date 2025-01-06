use bevy::{
    asset::RenderAssetUsages,
    math::{IVec2, UVec2},
};
use bevy_mesh::{Indices, Mesh, PrimitiveTopology};

pub struct MeshPart {
    pub position: IVec2,
    pub size: UVec2,
    pub uv_position: UVec2,
    pub uv_size: UVec2,
}

pub const MESH_MULTIPLIER: f32 = 16.0;

pub fn generate_mesh(mesh_parts: Vec<MeshPart>) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let mut vertex_count = 0u32;

    for part in mesh_parts.iter() {
        let x = part.position.x as f32 * MESH_MULTIPLIER;
        let y = part.position.y as f32 * MESH_MULTIPLIER;
        let width = part.size.x as f32 * MESH_MULTIPLIER;
        let height = part.size.y as f32 * MESH_MULTIPLIER;

        let bottom_left = [x, y, 0.0];
        let bottom_right = [x + width, y, 0.0];
        let top_right = [x + width, y + height, 0.0];
        let top_left = [x, y + height, 0.0];

        positions.extend_from_slice(&[bottom_left, bottom_right, top_right, top_left]);

        let u = part.uv_position.x as f32;
        let v = part.uv_position.y as f32;
        let uv_width = part.uv_size.x as f32;
        let uv_height = part.uv_size.y as f32;

        let uv_bottom_left = [u, v];
        let uv_bottom_right = [u + uv_width, v];
        let uv_top_right = [u + uv_width, v + uv_height];
        let uv_top_left = [u, v + uv_height];

        uvs.extend_from_slice(&[uv_bottom_left, uv_bottom_right, uv_top_right, uv_top_left]);

        indices.extend_from_slice(&[
            vertex_count,
            vertex_count + 1,
            vertex_count + 2,
            vertex_count + 2,
            vertex_count + 3,
            vertex_count,
        ]);

        vertex_count += 4;
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );

    mesh.insert_attribute(
        bevy::render::mesh::Mesh::ATTRIBUTE_POSITION,
        positions.clone(),
    );

    mesh.insert_attribute(bevy::render::mesh::Mesh::ATTRIBUTE_UV_0, uvs);

    mesh.insert_indices(Indices::U32(indices));

    let normals = positions
        .iter()
        .map(|_| [0.0, 0.0, 1.0])
        .collect::<Vec<_>>();
    mesh.insert_attribute(bevy::render::mesh::Mesh::ATTRIBUTE_NORMAL, normals);

    mesh
}
