use crate::{
    display::Display,
    instance::InstanceRaw,
    material::Material,
    pipeline::{Pipeline, PipelineBindGroupInfo},
};
use anyhow::*;
use std::{fmt::Debug, ops::Range};
use std::{path::Path, sync::Arc};
use wgpu::util::DeviceExt;

pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex for MeshVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,

    pub num_elements: u32,
    pub material: usize,
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub name: String,
    pub instance_buffer: wgpu::Buffer,
}

impl Model {
    pub fn load_instance_buffers(&self, display: &Display, instance_data: Vec<InstanceRaw>) {
        display.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(instance_data.as_slice()),
        );
    }

    pub fn load<F: AsRef<Path> + Debug>(
        name: String,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_info: Option<Arc<PipelineBindGroupInfo>>,
        file_path: F,
    ) -> Result<Self> {
        let (obj_models, obj_materials) = tobj::load_obj(file_path.as_ref(), true)?;
        let containing_folder = file_path
            .as_ref()
            .parent()
            .context("Directory has no parent")?;
        let mut materials = Vec::new();
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            size: (std::mem::size_of::<InstanceRaw>() * 100) as u64,
            mapped_at_creation: false,
        });
        for mat in obj_materials {
            let material = Material::load(
                device,
                queue,
                mat,
                bind_group_info.clone(),
                containing_folder,
            );

            materials.push(material?);
        }

        let mut meshes = Vec::new();
        for m in obj_models {
            let mut vertices = Vec::new();
            for i in 0..m.mesh.positions.len() / 3 {
                vertices.push(MeshVertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                })
            }

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_path.as_ref())),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsage::VERTEX,
            });

            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_path.as_ref())),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsage::INDEX,
            });

            meshes.push(Mesh {
                name: m.name,
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            })
        }

        Ok(Self {
            meshes,
            materials,
            name,
            instance_buffer,
        })
    }
    pub fn load_from_vertex_data<F: AsRef<Path> + Debug>(
        name: String,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_info: Option<Arc<PipelineBindGroupInfo>>,
        vertex_data: &[MeshVertex],
        index_data: &[u32],
        texture_path: F,
    ) -> Result<Self> {
        let mut materials = Vec::new();
        materials.push(Material::load_from_texture(
            device,
            queue,
            bind_group_info,
            texture_path,
        )?);
        let mut meshes = Vec::new();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Vertex Buffer", name)),
            contents: bytemuck::cast_slice(vertex_data),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", name)),
            contents: bytemuck::cast_slice(index_data),
            usage: wgpu::BufferUsage::INDEX,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            size: (std::mem::size_of::<InstanceRaw>()) as u64,
            mapped_at_creation: false,
        });

        meshes.push(Mesh {
            name: format!("Mesh {:?}", name),
            vertex_buffer,
            index_buffer,
            num_elements: index_data.len() as u32,
            material: 0,
        });

        println!("Loaded mesh with {:?} indexes", index_data.len());

        Ok(Self {
            meshes,
            materials,
            name,
            instance_buffer,
        })
    }
}

pub trait DrawModel<'a, 'b>
where
    'b: 'a,
{
    fn draw_model_instanced(
        &mut self,
        model: &'b Model,
        instances: Range<u32>,
        bind_material: bool,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        instances: Range<u32>,
        bind_material: bool,
    );
}

impl<'a, 'b> DrawModel<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        instances: Range<u32>,
        bind_material: bool,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        if bind_material {
            self.set_bind_group(0, &material.bind_group, &[]);
        }

        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_model_instanced(
        &mut self,
        model: &'b Model,
        instances: Range<u32>,
        bind_material: bool,
    ) {
        self.set_vertex_buffer(1, model.instance_buffer.slice(..));

        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(mesh, material, instances.clone(), bind_material);
        }
    }
}
