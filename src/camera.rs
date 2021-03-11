use crate::{display::Display, event::Event};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct CameraMetadata {
    aspect: f32,
}

impl CameraMetadata {
    pub fn new(display: &Display) -> Self {
        Self {
            aspect: display.swap_chain_descriptor.width as f32
                / display.swap_chain_descriptor.height as f32,
        }
    }
}

pub struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    field_of_view: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            eye: (0.0, 0.0, 3.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            field_of_view: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }

    pub fn projection(&self, metadata: &CameraMetadata) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let projection = cgmath::perspective(
            cgmath::Deg(self.field_of_view),
            metadata.aspect,
            self.znear,
            self.zfar,
        );

        OPENGL_TO_WGPU_MATRIX * projection * view
    }

    pub fn position(&self) -> cgmath::Vector4<f32> {
        self.eye.to_homogeneous()
    }
}

pub struct CameraController {
    speed: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_event(&mut self, event: &Event) -> bool {
        match event {
            Event::MoveCameraForward(pressed) => {
                self.is_forward_pressed = *pressed;
                true
            }
            Event::MoveCameraBackward(pressed) => {
                self.is_backward_pressed = *pressed;
                true
            }
            Event::MoveCameraLeft(pressed) => {
                self.is_left_pressed = *pressed;
                true
            }
            Event::MoveCameraRight(pressed) => {
                self.is_right_pressed = *pressed;
                true
            }
            Event::MoveCameraUp(pressed) => {
                self.is_up_pressed = *pressed;
                true
            }
            Event::MoveCameraDown(pressed) => {
                self.is_down_pressed = *pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();
        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        // Redo radius calc in case the up/ down is pressed.
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and eye so
            // that it doesn't change. The eye therefore still
            // lies on the circle made by the target and eye.
            camera.eye = camera.target - ((forward + right) * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - ((forward - right) * self.speed).normalize() * forward_mag;
        }

        if self.is_up_pressed {
            camera.eye =
                camera.target - ((forward + camera.up) * self.speed).normalize() * forward_mag;
        }

        if self.is_down_pressed {
            camera.eye =
                camera.target - ((forward - camera.up) * self.speed).normalize() * forward_mag;
        }
    }
}
