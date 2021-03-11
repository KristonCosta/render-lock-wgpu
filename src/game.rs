use legion::*;

use crate::ecs::component::*;
use crate::{
    camera::{Camera, CameraController},
    event::Event,
};
use cgmath::{Rotation, Zero};

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
                position: cgmath::Vector3::new(0.5, 0.0, 0.0),
                rotation: cgmath::Quaternion::look_at(
                    cgmath::Vector3::new(0.5, 0.5, 0.5),
                    cgmath::Vector3::new(0.1, 1.0, 0.5),
                ),
            },
            ModelReference {
                asset_reference: crate::asset::ModelAsset::Cube,
            },
        ));

        world.push((
            Transform {
                position: cgmath::Vector3::new(1.0, 0.0, 1.0),
                rotation: cgmath::Quaternion::look_at(
                    cgmath::Vector3::new(0.5, 0.5, 0.5),
                    cgmath::Vector3::new(1.1, 1.0, 0.0),
                ),
            },
            ModelReference {
                asset_reference: crate::asset::ModelAsset::Room,
            },
        ));

        let schedule = Schedule::builder().build();
        let resources = Resources::default();
        Self {
            world,
            schedule,
            camera: Camera::new(),
            camera_controller: CameraController::new(0.02),
            resources,
        }
    }

    pub fn handle_input(&mut self, event: &Event) -> bool {
        self.camera_controller.process_event(event)
    }

    pub fn update(&mut self) {
        self.camera_controller.update(&mut self.camera);
        self.schedule.execute(&mut self.world, &mut self.resources);
    }
}
