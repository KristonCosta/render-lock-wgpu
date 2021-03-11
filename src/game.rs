use legion::*;

use crate::{
    camera::{Camera, CameraController},
    event::Event,
};

pub struct Game {
    world: World,

    camera: Camera,
    camera_controller: CameraController,
}

impl Game {
    pub fn new() -> Self {
        Self {
            world: World::default(),
            camera: Camera::new(),
            camera_controller: CameraController::new(0.2),
        }
    }

    pub fn handle_input(&mut self, event: &Event) -> bool {
        self.camera_controller.process_event(event)
    }

    pub fn update(&mut self) {
        self.camera_controller.update(&mut self.camera);
    }
}
