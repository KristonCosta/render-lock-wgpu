#[derive(Clone, Copy, Debug)]
pub enum Event {
    MoveCameraForward(bool),
    MoveCameraBackward(bool),
    MoveCameraLeft(bool),
    MoveCameraRight(bool),
    MoveCameraUp(bool),
    MoveCameraDown(bool),
}
