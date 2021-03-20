use std::{sync::Arc, time::Duration};

use legion::*;

use crate::{
    camera::{Camera, CameraController},
    chunk::ChunkManager,
    ecs::system::*,
    event::Event,
};

pub struct Game {
    pub world: World,
    schedule: Schedule,
    resources: Resources,
    camera_controller: CameraController,
    chunk_manager: ChunkManager,
    player: legion::Entity,
}

impl Game {
    pub fn new(device: Arc<wgpu::Device>) -> Self {
        let mut world = World::default();
        let mut chunk_manager = ChunkManager::new(device);
        chunk_manager.load_region(cgmath::Vector2::new(0, 0));
        let player = world.push((Camera::new(
            (-1.0, 5.0, -1.0),
            cgmath::Deg(-180.0),
            cgmath::Deg(-20.0),
        ),));

        let schedule = Schedule::builder()
            .add_system(update_positions_system())
            .build();
        let resources = Resources::default();
        Self {
            world,
            schedule,
            player,
            camera_controller: CameraController::new(20.0, 0.4),
            resources,
            chunk_manager,
        }
    }

    pub fn handle_input(&mut self, event: &Event) -> bool {
        self.camera_controller.process_event(event)
    }

    pub fn camera(&self) -> Camera {
        let entry = self.world.entry_ref(self.player).unwrap();
        entry.get_component::<Camera>().unwrap().clone()
    }

    pub fn update(&mut self, dt: Duration) {
        let mut entry = self.world.entry(self.player).unwrap();
        let mut camera = entry.get_component_mut::<Camera>().unwrap();
        self.camera_controller.update(&mut camera, dt);
        let position = cgmath::Vector2::new(camera.position.x, camera.position.z);
        self.chunk_manager.update(&mut self.world, position);
        self.schedule.execute(&mut self.world, &mut self.resources);
    }
}
