use crate::ecs::component::MeshId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModelAsset {
    Cube,
    Room,
    Dynamic(u32),
    DynamicMesh(MeshId),
}
