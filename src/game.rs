use std::{sync::Arc, time::Duration};

use legion::*;

use crate::{
    camera::{Camera, CameraController},
    chunk::ChunkManager,
    ecs::{component::*, system::*},
    event::Event,
};

pub struct Game {
    pub world: World,
    schedule: Schedule,
    pub camera: Camera,
    resources: Resources,
    camera_controller: CameraController,
    chunk_manager: ChunkManager,
}

impl Game {
    pub fn new(device: Arc<wgpu::Device>) -> Self {
        use cgmath::InnerSpace;
        let mut world = World::default();
        let mut chunk_manager = ChunkManager::new(device);
        for x in -10..11 {
            for z in -10..11 {
                let chunk_position = cgmath::Vector2::new((x as f32) * 32.0, (z as f32) * 32.0);
                if chunk_position.magnitude2() > 65536 as f32 {
                    continue;
                }
                let position = cgmath::Vector3::new(chunk_position.x, 0.0, chunk_position.y);
                chunk_manager.dispatch(position, chunk_position);
            }
        }

        let schedule = Schedule::builder()
            .add_system(update_positions_system())
            .build();
        let resources = Resources::default();
        Self {
            world,
            schedule,
            camera: Camera::new((-1.0, 5.0, -1.0), cgmath::Deg(-180.0), cgmath::Deg(-20.0)),
            camera_controller: CameraController::new(10.0, 0.4),
            resources,
            chunk_manager,
        }
    }

    pub fn handle_input(&mut self, event: &Event) -> bool {
        self.camera_controller.process_event(event)
    }

    pub fn update(&mut self, dt: Duration) {
        self.camera_controller.update(&mut self.camera, dt);
        self.chunk_manager.update(&mut self.world);
        self.schedule.execute(&mut self.world, &mut self.resources);
    }
}
