#[macro_use]
extern crate bitflags;

use std::{sync::Arc, time::Instant};

use pipeline::SimplePipeline;
use renderer::Renderer;
use scene::SceneManager;
use winit::{
    dpi::PhysicalPosition,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod asset;
mod bind_group;
mod camera;
mod chunk;
mod display;
mod ecs;
mod event;
mod game;
mod gui;
mod instance;
mod light;
mod material;
mod math;
mod mesh;
mod pipeline;
mod renderer;
mod scene;
mod texture;
mod timestep;
mod worker;

fn device_input_mapper(event: &DeviceEvent) -> Option<event::Event> {
    match event {
        DeviceEvent::MouseMotion { delta } => Some(event::Event::RotateCamera(delta.0, delta.1)),
        DeviceEvent::MouseWheel { delta } => {
            let scroll = match delta {
                MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
                MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
            };
            Some(event::Event::ZoomCamera(scroll as f64))
        }
        DeviceEvent::Key(KeyboardInput {
            state,
            virtual_keycode: Some(keycode),
            ..
        }) => {
            let is_pressed = *state == ElementState::Pressed;
            match keycode {
                VirtualKeyCode::Space => Some(event::Event::MoveCameraUp(is_pressed)),
                VirtualKeyCode::LShift => Some(event::Event::MoveCameraDown(is_pressed)),
                VirtualKeyCode::W | VirtualKeyCode::Up => {
                    Some(event::Event::MoveCameraForward(is_pressed))
                }
                VirtualKeyCode::A | VirtualKeyCode::Left => {
                    Some(event::Event::MoveCameraLeft(is_pressed))
                }
                VirtualKeyCode::S | VirtualKeyCode::Down => {
                    Some(event::Event::MoveCameraBackward(is_pressed))
                }
                VirtualKeyCode::D | VirtualKeyCode::Right => {
                    Some(event::Event::MoveCameraRight(is_pressed))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn input_mapper(event: &WindowEvent) -> Option<event::Event> {
    match event {
        WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
            ..
        } => {
            let is_pressed = *state == ElementState::Pressed;
            match keycode {
                VirtualKeyCode::Space => Some(event::Event::MoveCameraUp(is_pressed)),
                VirtualKeyCode::LShift => Some(event::Event::MoveCameraDown(is_pressed)),
                VirtualKeyCode::W | VirtualKeyCode::Up => {
                    Some(event::Event::MoveCameraForward(is_pressed))
                }
                VirtualKeyCode::A | VirtualKeyCode::Left => {
                    Some(event::Event::MoveCameraLeft(is_pressed))
                }
                VirtualKeyCode::S | VirtualKeyCode::Down => {
                    Some(event::Event::MoveCameraBackward(is_pressed))
                }
                VirtualKeyCode::D | VirtualKeyCode::Right => {
                    Some(event::Event::MoveCameraRight(is_pressed))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
            1200.0, 800.0,
        )))
        .build(&event_loop)
        .unwrap();
    let mut renderer: Renderer<SimplePipeline> = Renderer::new(&window);

    let mut clock = timestep::TimeStep::new();
    let mut game = game::Game::new(Arc::clone(&renderer.display.device));
    let mut scene_manager = SceneManager::new();

    let mut gui = gui::Gui::new(&window, &renderer.display);

    // let mut renderer = None;

    // let mut state = block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(_) => {}
        Event::DeviceEvent { ref event, .. } => {
            if let Some(game_event) = device_input_mapper(event) {
                game.handle_input(&game_event);
            }
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if let Some(game_event) = input_mapper(event) {
                game.handle_input(&game_event);
            } else {
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
                    WindowEvent::Resized(_) => {
                        // state.resize(*physical_size);
                        // TODO: ADD RENDERER HANDLING
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                        // state.resize(**new_inner_size)
                        // TODO: ADD RENDERER HANDLING
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(_) => {
            let current_time = Instant::now();
            let dt = current_time.duration_since(clock.last_time);
            clock.delta();
            let fps = clock.frame_rate;
            let frame = renderer
                .display
                .swap_chain
                .get_current_frame()
                .unwrap()
                .output;
            let mut encoder =
                renderer
                    .display
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });
            game.update(dt);
            let scene =
                scene_manager.load_scene(&game.world, &renderer.pipeline, &renderer.display);
            if let Some(scene) = scene {
                renderer.render(&frame, &mut encoder, scene, &game.camera());
            }
            gui.render(
                dt,
                fps as u32,
                &window,
                &frame,
                &mut encoder,
                &renderer.display,
            );
            renderer
                .display
                .queue
                .submit(std::iter::once(encoder.finish()));
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}
