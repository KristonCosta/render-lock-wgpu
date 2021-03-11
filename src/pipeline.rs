use std::collections::HashMap;

use wgpu::util::DeviceExt;

use super::display::Display;
use crate::{
    bind_group::{BindGroup, BindGroupType},
    instance::InstanceRaw,
    light::Light,
    material::Material,
    mesh::{MeshVertex, Vertex},
    texture::Texture,
};

pub struct PipelineBindGroupInfo {
    pub layout: wgpu::BindGroupLayout,
    pub index: u64,
}

pub trait Pipeline {
    fn new(window: &Display) -> Self;
    fn update_view_projection(&mut self, projection: cgmath::Matrix4<f32>);
    fn update_view_position(&mut self, position: cgmath::Vector4<f32>);
    fn bind_group_layout(&self, bind_group_type: BindGroupType) -> Option<&PipelineBindGroupInfo>;
    fn depth_stencil_attachment(&self) -> Option<wgpu::RenderPassDepthStencilAttachmentDescriptor>;
    fn prepare(&self, display: &Display);
    fn bind<'a, 'b: 'a>(&'b self, render_pass: &mut wgpu::RenderPass<'a>);
}

pub struct SimplePipeline {
    render_pipeline: wgpu::RenderPipeline,

    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,

    light: Light,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,

    bind_group_layouts: HashMap<BindGroupType, PipelineBindGroupInfo>,
    depth_texture: Texture,
}

impl Pipeline for SimplePipeline {
    fn prepare(&self, display: &Display) {
        display.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        )
    }
    fn bind<'a, 'b: 'a>(&'b self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(2, &self.light_bind_group, &[]);
    }

    fn new(display: &Display) -> Self {
        let mut bind_group_layouts = HashMap::new();

        let texture_bind_group_layout = display.device.create_bind_group_layout(&Material::desc());

        let uniforms = Uniforms::new();

        let uniform_buffer = display
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            });

        let uniform_bind_group_layout =
            display
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("uniform_bind_group_layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: Default::default(),
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let uniform_bind_group = display
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("uniform_bind_group"),
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: None,
                    },
                }],
            });

        let light = Light::default();

        let light_buffer = display
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Light VB"),
                contents: bytemuck::cast_slice(&[light]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            });

        let light_bind_group_layout =
            display
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let light_bind_group = display
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &light_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &light_buffer,
                        offset: 0,
                        size: None,
                    },
                }],
            });

        let render_pipeline_layout =
            display
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &texture_bind_group_layout,
                        &uniform_bind_group_layout,
                        &light_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let depth_texture = Texture::create_depth_texture(
            &display.device,
            &display.swap_chain_descriptor,
            "depth_texture",
        );

        let render_pipeline = {
            Self::create_render_pipeline(
                "Render Pipeline",
                &display.device,
                &render_pipeline_layout,
                display.swap_chain_descriptor.format,
                Texture::DEPTH_FORMAT,
                &[MeshVertex::desc(), InstanceRaw::desc()],
                &wgpu::include_spirv!("../resources/shaders/shader.vert.spv"),
                &wgpu::include_spirv!("../resources/shaders/shader.frag.spv"),
            )
        };

        bind_group_layouts.insert(
            Material::bind_group_type(),
            PipelineBindGroupInfo {
                layout: texture_bind_group_layout,
                index: 0,
            },
        );

        Self {
            render_pipeline,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            light,
            light_buffer,
            light_bind_group,
            depth_texture,
            bind_group_layouts,
        }
    }
    fn update_view_projection(&mut self, projection: cgmath::Matrix4<f32>) {
        self.uniforms.view_proj = projection.into();
    }

    fn update_view_position(&mut self, position: cgmath::Vector4<f32>) {
        self.uniforms.view_position = position.into();
    }

    fn bind_group_layout(&self, bind_group_type: BindGroupType) -> Option<&PipelineBindGroupInfo> {
        self.bind_group_layouts.get(&bind_group_type)
    }

    fn depth_stencil_attachment(&self) -> Option<wgpu::RenderPassDepthStencilAttachmentDescriptor> {
        Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
            attachment: &self.depth_texture.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: true,
            }),
            stencil_ops: None,
        })
    }
}

impl SimplePipeline {
    pub fn render_pass<'a>(
        &'a self,
        frame: &'a wgpu::SwapChainTexture,
        encoder: &'a mut wgpu::CommandEncoder,
    ) -> wgpu::RenderPass<'a> {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(2, &self.light_bind_group, &[]);
        render_pass
    }

    pub fn update(&self, display: &Display) {
        display.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }

    pub fn resize(&mut self, display: &Display) {
        self.depth_texture = Texture::create_depth_texture(
            &display.device,
            &display.swap_chain_descriptor,
            "depth_texture",
        );
    }

    fn create_render_pipeline(
        name: &str,
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        vertex_descs: &[wgpu::VertexBufferLayout],
        vs_src: &wgpu::ShaderModuleDescriptor,
        fs_src: &wgpu::ShaderModuleDescriptor,
    ) -> wgpu::RenderPipeline {
        let vs_module = device.create_shader_module(vs_src);
        let fs_module = device.create_shader_module(fs_src);

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(name),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: vertex_descs,
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: color_format,
                    color_blend: wgpu::BlendState::REPLACE,
                    alpha_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: wgpu::CullMode::Back,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
                clamp_depth: false,
            }),
            multisample: Default::default(),
        })
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl Uniforms {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
}
