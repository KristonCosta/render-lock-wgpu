use crate::{
    math::Boundary,
    ray::{Ray, SurfaceInteraction},
};

pub struct Aggregate {
    primatives: Vec<Primitive>,
}

pub struct Primitive {
    shape: Shape,
    material: Material,
}

impl Aggregate {
    pub fn bound(&self) -> Boundary<f32> {
        unimplemented!()
    }

    pub fn intersect(&self, ray: &Ray) -> Option<SurfaceInteraction> {
        unimplemented!()
    }

    pub fn intersects(&self, ray: &Ray) -> bool {
        unimplemented!()
    }
}

pub struct Material {}
pub struct Shape {}
