use crate::ecs::component::*;
use crate::mesh::MeshVertex;

bitflags! {
    pub struct Sides: u32 {
        const TOP       = 0b00000001;
        const BOTTOM    = 0b00000010;
        const LEFT      = 0b00000100;
        const RIGHT     = 0b00001000;
        const FORWARD   = 0b00010000;
        const BACKWARD  = 0b00100000;
    }
}

const CUBE_COORDINATES: [[f32; 3]; 8] = [
    [-0.5, -0.5, 0.5],
    [0.5, -0.5, 0.5],
    [0.5, -0.5, -0.5],
    [-0.5, -0.5, -0.5],
    [-0.5, 0.5, 0.5],
    [0.5, 0.5, 0.5],
    [0.5, 0.5, -0.5],
    [-0.5, 0.5, -0.5],
];

const UVS: [[f32; 2]; 4] = [
    [0.125, 1.0 - 0.9375],
    [0.1875, 1.0 - 0.9375],
    [0.125, 1.0 - 1.0],
    [0.1875, 1.0 - 1.0],
];

const QUAD_UV_ORDER: [u32; 4] = [3, 2, 0, 1];

enum CubeSide {
    Top,
    Bottom,
    Left,
    Right,
    Forward,
    Backward,
}

pub fn make_block() -> MeshReference {
    let mut vertex_data: Vec<MeshVertex> = Vec::new();
    let mut index_data: Vec<u32> = Vec::new();

    let offset = 0;

    // TOP

    let (verticies, indices, offset) = generate_quad(CubeSide::Top, offset);
    vertex_data.extend(verticies.iter());
    index_data.extend(indices.iter());

    let (verticies, indices, offset) = generate_quad(CubeSide::Bottom, offset);
    vertex_data.extend(verticies.iter());
    index_data.extend(indices.iter());

    let (verticies, indices, offset) = generate_quad(CubeSide::Left, offset);
    vertex_data.extend(verticies.iter());
    index_data.extend(indices.iter());

    let (verticies, indices, offset) = generate_quad(CubeSide::Right, offset);
    vertex_data.extend(verticies.iter());
    index_data.extend(indices.iter());

    let (verticies, indices, offset) = generate_quad(CubeSide::Forward, offset);
    vertex_data.extend(verticies.iter());
    index_data.extend(indices.iter());

    let (verticies, indices, _) = generate_quad(CubeSide::Backward, offset);
    vertex_data.extend(verticies.iter());
    index_data.extend(indices.iter());

    MeshReference {
        idx: 0,
        vertex_data: vertex_data.into_boxed_slice(),
        index_data: index_data.into_boxed_slice(),
    }
}

fn generate_quad(side: CubeSide, index_offset: u32) -> ([MeshVertex; 4], [u32; 6], u32) {
    let vertex_idx = match side {
        CubeSide::Top => [7, 6, 5, 4],
        CubeSide::Bottom => [0, 1, 2, 3],
        CubeSide::Left => [7, 4, 0, 3],
        CubeSide::Right => [5, 6, 2, 1],
        CubeSide::Forward => [4, 5, 1, 0],
        CubeSide::Backward => [6, 7, 3, 2],
    };

    (
        [
            MeshVertex {
                position: CUBE_COORDINATES[vertex_idx[0]].clone(),
                tex_coords: UVS[3].clone(),
                normal: [0.0, 1.0, 0.0],
            },
            MeshVertex {
                position: CUBE_COORDINATES[vertex_idx[1]].clone(),
                tex_coords: UVS[2].clone(),
                normal: [0.0, 1.0, 0.0],
            },
            MeshVertex {
                position: CUBE_COORDINATES[vertex_idx[2]].clone(),
                tex_coords: UVS[0].clone(),
                normal: [0.0, 1.0, 0.0],
            },
            MeshVertex {
                position: CUBE_COORDINATES[vertex_idx[3]].clone(),
                tex_coords: UVS[1].clone(),
                normal: [0.0, 1.0, 0.0],
            },
        ],
        [
            3 + index_offset,
            1 + index_offset,
            0 + index_offset,
            3 + index_offset,
            2 + index_offset,
            1 + index_offset,
        ],
        index_offset + 4,
    )
}

pub struct VoxelMeshBuilder {
    current_cube_pos: cgmath::Vector3<u32>,
    indices: Vec<u32>,
    vertices: Vec<MeshVertex>,
    index_offset: u32,
}

const SIDE_VERTICES: [(Sides, [u32; 4]); 6] = [
    (Sides::TOP, [7, 6, 5, 4]),
    (Sides::BOTTOM, [0, 1, 2, 3]),
    (Sides::LEFT, [7, 4, 0, 3]),
    (Sides::RIGHT, [5, 6, 2, 1]),
    (Sides::FORWARD, [4, 5, 1, 0]),
    (Sides::BACKWARD, [6, 7, 3, 2]),
];

impl VoxelMeshBuilder {
    pub fn new() -> Self {
        Self {
            current_cube_pos: cgmath::Vector3::new(0, 0, 0),
            indices: Vec::new(),
            vertices: Vec::new(),
            index_offset: 0,
        }
    }

    pub fn set_position(&mut self, position: cgmath::Vector3<u32>) -> &mut VoxelMeshBuilder {
        self.current_cube_pos = position;
        self
    }

    pub fn move_position(&mut self, delta: cgmath::Vector3<u32>) -> &mut VoxelMeshBuilder {
        self.current_cube_pos += delta;
        self
    }

    pub fn generate_voxel(&mut self, sides: Sides) -> &mut VoxelMeshBuilder {
        for (side, indices) in SIDE_VERTICES.iter() {
            if sides.contains(*side) {
                self.build_quad(&indices);
            }
        }
        self
    }

    fn build_quad(&mut self, vertex_idx: &[u32; 4]) {
        for (vertex, uv) in vertex_idx.iter().zip(QUAD_UV_ORDER.iter()) {
            self.vertices.push(MeshVertex {
                position: CUBE_COORDINATES[vertex_idx[0] as usize].clone(),
                tex_coords: UVS[3].clone(),
                normal: [0.0, 1.0, 0.0],
            });
        }
        self.indices.extend(
            [
                3 + self.index_offset,
                1 + self.index_offset,
                0 + self.index_offset,
                3 + self.index_offset,
                2 + self.index_offset,
                1 + self.index_offset,
            ]
            .iter(),
        );
        self.index_offset += 4;
    }
}
