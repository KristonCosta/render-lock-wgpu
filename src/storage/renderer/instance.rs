use crate::model::{DrawModel, Model};
use crate::renderer::display::Display;
use cgmath::{Vector3, Zero};
use std::rc::Rc;
use wgpu::util::DeviceExt;

pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub buff: wgpu::Buffer,
    pub model: Rc<Model>,
    dirty: bool,
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation))
            .into(),
        }
    }

    pub fn update(&mut self, display: &Display) {
        if self.dirty {
            display
                .queue
                .write_buffer(&self.buff, 0, bytemuck::cast_slice(&[self.to_raw()]));
            self.dirty = false;
        }
    }

    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
        self.dirty = true;
    }
}

pub trait DrawInstance<'a, 'b>
where
    'b: 'a,
{
    fn draw_instance(&mut self, instance: &'b Instance);
}

impl<'a, 'b> DrawInstance<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_instance(&mut self, instance: &'b Instance) {
        self.set_vertex_buffer(1, instance.buff.slice(..));
        self.draw_model(&instance.model);
    }
}

pub trait NewInstance {
    fn new_instance(&self, display: &Display) -> Instance;
}

impl NewInstance for Rc<Model> {
    fn new_instance(&self, display: &Display) -> Instance {
        let raw = InstanceRaw::default();
        let buff = display
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&[raw]),
                usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            });

        Instance {
            position: cgmath::Vector3::zero(),
            rotation: cgmath::Quaternion::zero(),
            buff,
            model: self.clone(),
            dirty: true,
        }
    }
}
