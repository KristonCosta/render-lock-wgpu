use crate::texture::Texture;
use crate::{bind_group, pipeline::Pipeline};
use anyhow::*;

use std::path::Path;

pub struct Material {
    pub name: String,
    pub diffuse_texture: Texture,
    pub bind_group: wgpu::BindGroup,
}

impl bind_group::BindGroup for Material {
    fn desc<'a>() -> wgpu::BindGroupLayoutDescriptor<'a> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: false,
                    },
                    count: None,
                },
            ],
        }
    }

    fn bind_group_type() -> bind_group::BindGroupType {
        bind_group::BindGroupType::Material
    }
}

impl Material {
    pub fn load<F: AsRef<Path>, P: Pipeline>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        material: tobj::Material,
        pipeline: &P,
        containing_folder: F,
    ) -> Result<Self> {
        let diffuse_path = material.diffuse_texture;

        let path = containing_folder.as_ref().join(diffuse_path);
        let diffuse_texture = Texture::load(device, queue, path)?;
        let bind_group_info = pipeline.bind_group_layout(bind_group::BindGroupType::Material);
        let bind_group = bind_group_info.map(|info| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &info.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    },
                ],
                label: None,
            })
        });
        Ok(Self {
            name: material.name,
            diffuse_texture,
            bind_group: bind_group.unwrap(),
        })
    }
}
