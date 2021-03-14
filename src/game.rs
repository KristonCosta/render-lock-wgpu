use std::time::Duration;

use legion::*;

use crate::{
    camera::{Camera, CameraController},
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
        let mut world = World::default();

        world.push((
            Transform {
                position: cgmath::Vector3::new(0.0, 0.0, 0.0),
                rotation: cgmath::Euler::new(cgmath::Rad(0.0), cgmath::Rad(0.0), cgmath::Rad(0.0)),
            },
            Momentum {
                rotation: cgmath::Euler::new(cgmath::Rad(0.0), cgmath::Rad(0.0), cgmath::Rad(0.0)),
            },
            crate::chunk::make_mesh(),
        ));

        // world.push((
        //     Transform {
        //         position: cgmath::Vector3::new(0.0, 0.0, 0.0),
        //         rotation: cgmath::Euler::new(cgmath::Rad(0.0), cgmath::Rad(0.0), cgmath::Rad(0.0)),
        //     },
        //     Momentum {
        //         rotation: cgmath::Euler::new(cgmath::Rad(0.01), cgmath::Rad(0.0), cgmath::Rad(0.0)),
        //     },
        //     ModelReference {
        //         asset_reference: crate::asset::ModelAsset::Room,
        //     },
        // ));

        // world.push((
        //     Transform {
        //         position: cgmath::Vector3::new(2.0, 0.0, 1.0),
        //         rotation: cgmath::Euler::new(cgmath::Rad(0.0), cgmath::Rad(0.0), cgmath::Rad(0.0)),
        //     },
        //     Momentum {
        //         rotation: cgmath::Euler::new(
        //             cgmath::Rad(0.01),
        //             cgmath::Rad(0.01),
        //             cgmath::Rad(0.01),
        //         ),
        //     },
        //     ModelReference {
        //         asset_reference: crate::asset::ModelAsset::Cube,
        //     },
        // ));

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
