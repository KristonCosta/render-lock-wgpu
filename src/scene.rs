use crate::{
    light::Light,
    math::Boundary,
    primitive::Aggregate,
    ray::{Ray, SurfaceInteraction},
};

pub struct Scene {
    aggregate: Aggregate,
    lights: Vec<Light>,
    boundary: Boundary<f32>,
}

impl Scene {
    pub fn intersect(&self, ray: &Ray) -> Option<SurfaceInteraction> {
        self.aggregate.intersect(ray)
    }

    pub fn intersects(&self, ray: &Ray) -> bool {
        self.aggregate.intersects(ray)
    }
}
