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
    pub idx: u32,
}
