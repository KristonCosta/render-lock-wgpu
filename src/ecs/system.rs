use super::component::{Momentum, Transform};
use cgmath::InnerSpace;
use legion::*;

#[system(for_each)]
pub fn update_positions(transform: &mut Transform, momentum: &mut Momentum) {
    transform.rotation.x += momentum.rotation.x;
    transform.rotation.y += momentum.rotation.y;
    transform.rotation.z += momentum.rotation.z;
}
