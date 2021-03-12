use pipeline::SimplePipeline;
use renderer::Renderer;
use scene::SceneManager;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod asset;
mod bind_group;
mod camera;
mod display;
mod ecs;
mod event;
mod game;
mod instance;
mod light;
mod material;
mod math;
mod mesh;
mod pipeline;
mod renderer;
mod scene;
mod texture;

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
        WindowEvent::ModifiersChanged(_) => None,
        WindowEvent::CursorMoved {
            device_id,
            position,
            modifiers,
        } => None,
        WindowEvent::MouseWheel {
            device_id,
            delta,
            phase,
            modifiers,
        } => None,
        WindowEvent::MouseInput {
            device_id,
            state,
            button,
            modifiers,
        } => None,

        _ => None,
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut game = game::Game::new();
    let mut scene_manager = SceneManager::new();
    let mut renderer: Renderer<SimplePipeline> = Renderer::new(&window);

    // let mut renderer = None;

    // let mut state = block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(_) => {}
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
                    WindowEvent::Resized(physical_size) => {
                        // state.resize(*physical_size);
                        // TODO: ADD RENDERER HANDLING
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // state.resize(**new_inner_size)
                        // TODO: ADD RENDERER HANDLING
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(_) => {
            game.update();
            let scene =
                scene_manager.load_scene(&game.world, &renderer.pipeline, &renderer.display);
            if let Some(scene) = scene {
                renderer.render(scene, &game.camera);
            }
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}
