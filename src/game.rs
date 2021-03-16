use std::time::Duration;

use legion::*;

use crate::{
    camera::{Camera, CameraController},
    chunk::ChunkBuilder,
    ecs::{component::*, system::*},
    event::Event,
};

pub struct Game {
    pub world: World,
    schedule: Schedule,
    pub camera: Camera,
    resources: Resources,
    camera_controller: CameraController,
}

impl Game {
    pub fn new() -> Self {
        use cgmath::InnerSpace;
        let mut world = World::default();
        let mut mesh_builder = ChunkBuilder::new();
        for x in -10..11 {
            for z in -10..11 {
                let chunk_position = cgmath::Vector2::new((x as f32) * 32.0, (z as f32) * 32.0);
                if chunk_position.magnitude2() > 65536 as f32 {
                    continue;
                }
                world.push((
                    Transform {
                        position: cgmath::Vector3::new(chunk_position.x, 0.0, chunk_position.y),
                        rotation: cgmath::Euler::new(
                            cgmath::Rad(0.0),
                            cgmath::Rad(0.0),
                            cgmath::Rad(0.0),
                        ),
                    },
                    mesh_builder.make_mesh(chunk_position),
                ));
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
            camera_controller: CameraController::new(4.0, 0.4),
            resources,
        }
    }

    pub fn handle_input(&mut self, event: &Event) -> bool {
        self.camera_controller.process_event(event)
    }

    pub fn update(&mut self, dt: Duration) {
        self.camera_controller.update(&mut self.camera, dt);
        self.schedule.execute(&mut self.world, &mut self.resources);
    }
}
