#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModelReference {
    pub asset_reference: crate::asset::ModelAsset,
}
