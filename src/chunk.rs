use crate::ecs::component::*;
use crate::mesh::MeshVertex;

pub fn make_block() -> MeshReference {
    let mut vertex_data: Vec<MeshVertex> = Vec::new();
    let mut index_data: Vec<u32> = Vec::new();

    let uvs = [
        [0.125, 1.0 - 0.9375],
        [0.1875, 1.0 - 0.9375],
        [0.125, 1.0 - 1.0],
        [0.1875, 1.0 - 1.0],
    ];

    let p0 = [-0.5, -0.5, 0.5];
    let p1 = [0.5, -0.5, 0.5];
    let p2 = [0.5, -0.5, -0.5];
    let p3 = [-0.5, -0.5, -0.5];
    let p4 = [-0.5, 0.5, 0.5];
    let p5 = [0.5, 0.5, 0.5];
    let p6 = [0.5, 0.5, -0.5];
    let p7 = [-0.5, 0.5, -0.5];

    let mut offset = 0;

    // TOP

    vertex_data.push(MeshVertex {
        position: p7.clone(),
        tex_coords: uvs[3].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    vertex_data.push(MeshVertex {
        position: p6.clone(),
        tex_coords: uvs[2].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    vertex_data.push(MeshVertex {
        position: p5.clone(),
        tex_coords: uvs[0].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    vertex_data.push(MeshVertex {
        position: p4.clone(),
        tex_coords: uvs[1].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    index_data.extend(
        [
            3 + offset,
            1 + offset,
            0 + offset,
            3 + offset,
            2 + offset,
            1 + offset,
        ]
        .iter(),
    );
    offset += 4;

    // Bottom

    vertex_data.push(MeshVertex {
        position: p0.clone(),
        tex_coords: uvs[3].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    vertex_data.push(MeshVertex {
        position: p1.clone(),
        tex_coords: uvs[2].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    vertex_data.push(MeshVertex {
        position: p2.clone(),
        tex_coords: uvs[0].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    vertex_data.push(MeshVertex {
        position: p3.clone(),
        tex_coords: uvs[1].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    index_data.extend(
        [
            3 + offset,
            1 + offset,
            0 + offset,
            3 + offset,
            2 + offset,
            1 + offset,
        ]
        .iter(),
    );
    offset += 4;

    // Left

    vertex_data.push(MeshVertex {
        position: p7.clone(),
        tex_coords: uvs[3].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    vertex_data.push(MeshVertex {
        position: p4.clone(),
        tex_coords: uvs[2].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    vertex_data.push(MeshVertex {
        position: p0.clone(),
        tex_coords: uvs[0].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    vertex_data.push(MeshVertex {
        position: p3.clone(),
        tex_coords: uvs[1].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    index_data.extend(
        [
            3 + offset,
            1 + offset,
            0 + offset,
            3 + offset,
            2 + offset,
            1 + offset,
        ]
        .iter(),
    );
    offset += 4;

    // RIGHT

    vertex_data.push(MeshVertex {
        position: p5.clone(),
        tex_coords: uvs[3].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    vertex_data.push(MeshVertex {
        position: p6.clone(),
        tex_coords: uvs[2].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    vertex_data.push(MeshVertex {
        position: p2.clone(),
        tex_coords: uvs[0].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    vertex_data.push(MeshVertex {
        position: p1.clone(),
        tex_coords: uvs[1].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    index_data.extend(
        [
            3 + offset,
            1 + offset,
            0 + offset,
            3 + offset,
            2 + offset,
            1 + offset,
        ]
        .iter(),
    );
    offset += 4;

    // Forward

    vertex_data.push(MeshVertex {
        position: p4.clone(),
        tex_coords: uvs[3].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    vertex_data.push(MeshVertex {
        position: p5.clone(),
        tex_coords: uvs[2].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    vertex_data.push(MeshVertex {
        position: p1.clone(),
        tex_coords: uvs[0].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    vertex_data.push(MeshVertex {
        position: p0.clone(),
        tex_coords: uvs[1].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    index_data.extend(
        [
            3 + offset,
            1 + offset,
            0 + offset,
            3 + offset,
            2 + offset,
            1 + offset,
        ]
        .iter(),
    );
    offset += 4;

    // back

    vertex_data.push(MeshVertex {
        position: p6.clone(),
        tex_coords: uvs[3].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    vertex_data.push(MeshVertex {
        position: p7.clone(),
        tex_coords: uvs[2].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    vertex_data.push(MeshVertex {
        position: p3.clone(),
        tex_coords: uvs[0].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    vertex_data.push(MeshVertex {
        position: p2.clone(),
        tex_coords: uvs[1].clone(),
        normal: [0.0, 1.0, 0.0],
    });

    index_data.extend(
        [
            3 + offset,
            1 + offset,
            0 + offset,
            3 + offset,
            2 + offset,
            1 + offset,
        ]
        .iter(),
    );
    offset += 4;

    MeshReference {
        idx: 0,
        vertex_data: vertex_data.into_boxed_slice(),
        index_data: index_data.into_boxed_slice(),
    }
}
