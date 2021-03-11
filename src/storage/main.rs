#[macro_use]
extern crate bytemuck;

use std::time::Instant;

use futures::executor::block_on;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use renderer::texture;

use crate::gui::Gui;
use crate::renderer::pipeline::Pipeline;

mod gui;
mod integrator;
mod light;
mod math;
mod model;
mod primitive;
mod ray;
mod renderer;
mod scene;

use crate::model::{DrawModel, Model, Vertex};
use cgmath::Vector3;
use renderer::camera::*;
use renderer::display::*;
use renderer::instance::*;
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
        let delta = current_time.duration_since(self.last_time).as_millis() as f64;
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

struct State {
    display: Display,

    camera: Camera,
    camera_controller: CameraController,

    pipeline: Pipeline,
    // --
    instances: Vec<Instance>,
    gui: Gui,
    clock: TimeStep,
    instance_buffer: wgpu::Buffer,
    instance_model: Rc<Model>,
}

impl State {
    async fn new(window: &Window) -> Self {
        let display = Display::new(window).await;
        let gui = Gui::new(window, &display);
        let camera = Camera {
            eye: (0.0, 0.0, 3.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: display.swap_chain_descriptor.width as f32
                / display.swap_chain_descriptor.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };
        let pipeline = Pipeline::new(&display);
        let camera_controller = CameraController::new(0.2);

        let resources = std::path::Path::new(env!("OUT_DIR")).join("resources");
        let obj_model = model::Model::load(
            &display.device,
            &display.queue,
            pipeline.texture_bind_group_layout(),
            resources.join("viking_room.obj"),
        )
        .unwrap();

        let obj_model_cube = model::Model::load(
            &display.device,
            &display.queue,
            pipeline.texture_bind_group_layout(),
            resources.join("untitled.obj"),
        )
        .unwrap();

        let mut instances = Vec::new();
        let obj_model_ref = Rc::new(obj_model);

        for x in 0..30 {
            for y in 0..1 {
                for z in 0..30 {
                    let mut obj_instance = obj_model_ref.new_instance(&display);

                    obj_instance.set_position(Vector3::new(
                        (x * 2) as f32,
                        (y * 2) as f32,
                        (z * 2) as f32,
                    ));
                    instances.push(obj_instance);
                }
            }
        }

        let instance_raw = instances.iter().map(|x| x.to_raw()).collect::<Vec<_>>();
        let buff = display
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(instance_raw.as_slice()),
                usage: wgpu::BufferUsage::VERTEX,
            });

        Self {
            display,
            camera,
            camera_controller,
            instances,
            pipeline,
            gui,
            clock: TimeStep::new(),
            instance_buffer: buff,
            instance_model: obj_model_ref,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.display.size = new_size;
        self.display.swap_chain_descriptor.width = new_size.width;
        self.display.swap_chain_descriptor.height = new_size.height;

        self.pipeline.resize(&self.display);
        self.display.swap_chain = self
            .display
            .device
            .create_swap_chain(&self.display.surface, &self.display.swap_chain_descriptor);
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.pipeline.project(&self.camera);
        self.pipeline.update(&self.display);
    }

    fn render<'a>(&mut self, window: &Window) -> Result<(), wgpu::SwapChainError> {
        let current_time = Instant::now();
        let dt = current_time.duration_since(self.clock.last_time);
        self.clock.delta();
        self.gui.context.io_mut().update_delta_time(dt);
        self.gui
            .platform
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
                    ui.text(imgui::im_str!(
                        "This is a demo of imgui-rs using imgui-wgpu!"
                    ));
                    ui.separator();
                    ui.text(imgui::im_str!("FPS: ({:.1})", fps,));
                });
        }
        let frame = self.display.swap_chain.get_current_frame().unwrap().output;
        let mut encoder =
            self.display
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = self.pipeline.render_pass(&frame, &mut encoder);
            // render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            for instance in &mut self.instances {
                instance.update(&self.display);
                render_pass.draw_instance(instance);
            }
        }
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
            depth_stencil_attachment: None,
        });
        self.gui
            .renderer
            .render(
                ui.render(),
                &self.display.queue,
                &self.display.device,
                &mut pass,
            )
            .unwrap();
        drop(pass);
        self.display.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}

fn main() {
    env_logger::init();

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
