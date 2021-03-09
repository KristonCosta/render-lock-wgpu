#[macro_use]
extern crate bytemuck;

use std::ops::Range;
use std::time::{Duration, Instant};

use bytemuck::bytes_of;
use cgmath::prelude::*;
use futures::executor::block_on;
use image::GenericImageView;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use renderer::texture;

use crate::gui::Gui;

mod gui;
mod renderer;
mod model;

use renderer::display::*;
use renderer::camera::*;
use renderer::texture::*;
use renderer::instance::*;
use crate::model::{Vertex, Model};
use cgmath::{Vector3, Quaternion};
use std::rc::Rc;

#[derive(Debug)]
pub struct TimeStep {
    last_time: Instant,
    delta_time: f64,
    frame_count: u32,
    frame_time: f64,
    frame_rate: u64,
}

impl TimeStep {
    // https://gitlab.com/flukejones/diir-doom/blob/master/game/src/main.rs
    // Grabbed this from here
    pub fn new() -> TimeStep {
        TimeStep {
            last_time: Instant::now(),
            delta_time: 0.0,
            frame_count: 0,
            frame_time: 0.0,
            frame_rate: 0,
        }
    }

    pub fn delta(&mut self) -> f64 {
        let current_time = Instant::now();
        let delta =  current_time.duration_since(self.last_time).as_millis() as f64;
        self.last_time = current_time;
        self.delta_time = delta;
        self.frame_count += 1;
        self.frame_time += self.delta_time;

        // per second
        if self.frame_time >= 1000.0 {
            self.frame_rate = self.frame_count as u64;
            self.frame_count = 0;
            self.frame_time = 0.0;
        }
        delta
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Light {
    position: [f32; 3],
    _padding: u32,
    color: [f32; 3],
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

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_position = camera.eye.to_homogeneous().into();
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

struct State {
    display: Display,

    camera: Camera,
    camera_controller: CameraController,

    // --
    render_pipeline: wgpu::RenderPipeline,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,

    depth_texture: texture::Texture,
    obj_instance: Instance,
    obj_model_instance : Instance,
    light: Light,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    gui: Gui,
    clock: TimeStep,
    last_cursor: Option<imgui::MouseCursor>
}

impl State {
    async fn new(window: &Window) -> Self {
        let display = Display::new(window).await;
        let gui = Gui::new(window, &display);
        let texture_bind_group_layout =
            display.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            });


        let camera = Camera {
            eye: (0.0, 0.0, 3.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: display.swap_chain_descriptor.width as f32 / display.swap_chain_descriptor.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(&camera);

        let uniform_buffer = display.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bind_group_layout =
            display.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let uniform_bind_group = display.device.create_bind_group(&wgpu::BindGroupDescriptor {
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

        let light = Light {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
        };

        let light_buffer = display.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let light_bind_group_layout =
            display.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let light_bind_group = display.device.create_bind_group(&wgpu::BindGroupDescriptor {
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
            display.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &uniform_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });


        let render_pipeline = {
            Self::create_render_pipeline(
                "Render Pipeline",
                &display.device,
                &render_pipeline_layout,
                display.swap_chain_descriptor.format,
                texture::Texture::DEPTH_FORMAT,
                &[model::ModelVertex::desc(), InstanceRaw::desc()],
                &wgpu::include_spirv!("../resources/shaders/shader.vert.spv"),
                &wgpu::include_spirv!("../resources/shaders/shader.frag.spv"),
            )
        };

        let depth_texture =
            texture::Texture::create_depth_texture(&display.device, &display.swap_chain_descriptor, "depth_texture");


        let camera_controller = CameraController::new(0.2);


        let instance_buffer = display.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&[raw]),
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });

        let resources = std::path::Path::new(env!("OUT_DIR")).join("resources");
        let obj_model = model::Model::load(
            &display.device,
            &display.queue,
            &texture_bind_group_layout,
            resources.join("viking_room.obj"),
        )
        .unwrap();

        let obj_instance = Instance {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::zero(),
            buff: instance_buffer,
            model: Rc::new(obj_model),
        };

        let obj_model_cube = model::Model::load(
            &display.device,
            &display.queue,
            &texture_bind_group_layout,
            resources.join("cube.obj"),
        ).unwrap();



        let obj_model_instance = Instance {
            position: Vector3::new(10.0, 0.0, 0.0),
            rotation: Quaternion::zero(),
            buff: instance_buffer,
            model: Rc::new(obj_model_cube)
        };

        Self {
            display,
            render_pipeline,
            camera,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            camera_controller,
            depth_texture,
            obj_instance,
            obj_model_instance,
            light,
            light_buffer,
            light_bind_group,
            gui,
            clock: TimeStep::new(),
            last_cursor: None
        }
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

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.display.size = new_size;
        self.display.swap_chain_descriptor.width = new_size.width;
        self.display.swap_chain_descriptor.height = new_size.height;
        self.depth_texture =
            texture::Texture::create_depth_texture(&self.display.device, &self.display.swap_chain_descriptor, "depth_texture");
        self.display.swap_chain = self.display.device.create_swap_chain(&self.display.surface, &self.display.swap_chain_descriptor);
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    fn update(&mut self) {

        self.camera_controller.update_camera(&mut self.camera);
        self.uniforms.update_view_proj(&self.camera);
        self.display.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );

        use cgmath::Rotation3;
        let old_position: cgmath::Vector3<_> = self.light.position.into();
        self.light.position =
            (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0))
                * old_position)
                .into();
        self.display.queue
            .write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light]));
    }

    fn render(&mut self, window: &Window) -> Result<(), wgpu::SwapChainError> {
        let current_time = Instant::now();
        let dt = current_time.duration_since(self.clock.last_time);
        self.clock.delta();
        self.gui.context.io_mut().update_delta_time(dt);
        self.gui.platform
            .prepare_frame(self.gui.context.io_mut(), window)
            .unwrap();
        let fps = self.clock.frame_rate;
        let ui = self.gui.context.frame();
        {
            let window = imgui::Window::new(imgui::im_str!("Hello Imgui from WGPU!"));
            window
                .size([300.0, 100.0], imgui::Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text(imgui::im_str!("Hello world!"));
                    ui.text(imgui::im_str!("This is a demo of imgui-rs using imgui-wgpu!"));
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(imgui::im_str!(
                "FPS: ({:.1})",
                fps,
            ));
                });
        }
        let frame = self.display.swap_chain.get_current_frame()?.output;
        let mut encoder = self
            .display.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
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
            self.display.queue.write_buffer(
                &self.obj_instance.buff,
                0,
                bytemuck::cast_slice(&[self.obj_instance.to_raw()])
            );

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(1, self.obj_instance.buff.slice(..));
            use model::DrawModel;
            render_pass.draw_model(
                &self.obj_instance.model,
                &self.uniform_bind_group,
                &self.light_bind_group
            );



            self.display.queue.write_buffer(
                &self.obj_model_instance.buff,
                0,
                bytemuck::cast_slice(&[self.obj_model_instance.to_raw()])
            );
            render_pass.set_vertex_buffer(1, self.obj_model_instance.buff.slice(..));

            render_pass.draw_model(
                &self.obj_model_instance.model,
                &self.uniform_bind_group,
                &self.light_bind_group
            );
        }
        // Render the UI

        self.gui.platform.prepare_render(&ui, &window);

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None
        });
        self.gui.renderer.render(ui.render(), &self.display.queue, &self.display.device, &mut pass);
        drop(pass);
        self.display.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}

fn main() {
    env_logger::init();

    let img = image::open("resources/viking_room.png").unwrap();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(_) => {}
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    },
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size)
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(_) => {
            state.update();
            match state.render(&window) {
                Ok(_) => {}
                Err(wgpu::SwapChainError::Lost) => state.resize(state.display.size),
                Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}
