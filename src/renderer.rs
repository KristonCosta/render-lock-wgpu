use futures::executor::block_on;
use winit::window::Window;

use crate::{
    camera::{Camera, CameraMetadata},
    display::Display,
    scene::Scene,
};

pub struct Renderer {
    camera_metadata: CameraMetadata,
    display: Display,
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        let display = block_on(Display::new(window));
        let camera_metadata = CameraMetadata::new(&display);
        Self {
            display,
            camera_metadata,
        }
    }

    pub fn render(&self, scene: &Scene, camera: &Camera) {}
}
