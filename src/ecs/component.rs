use cgmath::{Euler, Rad};

use crate::mesh::MeshVertex;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub position: cgmath::Vector3<f32>,
    pub rotation: Euler<Rad<f32>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Momentum {
    pub rotation: Euler<Rad<f32>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModelReference {
    pub asset_reference: crate::asset::ModelAsset,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MeshReference {
    pub vertex_data: Box<[MeshVertex]>,
    pub index_data: Box<[u32]>,
    pub idx: MeshId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd)]
pub struct MeshId(pub i32, pub i32);

impl Ord for MeshId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.0 * self.0 + self.1 * self.1).cmp(&(other.0 * other.0 + other.1 * other.1))
    }
}
