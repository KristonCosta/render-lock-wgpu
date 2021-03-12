#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Light {
    position: [f32; 3],
    _padding: u32,
    color: [f32; 3],
}

impl Light {}

impl Default for Light {
    fn default() -> Self {
        Self {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
        }
    }
}
