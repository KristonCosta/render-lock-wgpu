pub trait BindGroup {
    fn desc<'a>() -> wgpu::BindGroupLayoutDescriptor<'a>;
    fn bind_group_type() -> BindGroupType;
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
pub enum BindGroupType {
    Material,
}
