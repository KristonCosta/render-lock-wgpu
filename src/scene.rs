use crate::ecs::*;
use crate::{
    asset::ModelAsset, display::Display, instance::Instance, mesh::Model, pipeline::Pipeline,
};
use cgmath::InnerSpace;
use legion::*;
use std::collections::HashMap;

pub struct Scene<'a> {
    pub model: &'a Model,
    pub instances: Vec<Instance>,
    pub next: Option<Box<Scene<'a>>>,
}

pub struct SceneManager {
    assets: HashMap<ModelAsset, Model>,
}
impl SceneManager {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    pub fn load_scene<P: Pipeline>(
        &mut self,
        world: &World,
        pipeline: &P,
        display: &Display,
    ) -> Option<Box<Scene>> {
        let mut query = <(&component::Transform, &component::ModelReference)>::query();
        let mut instance_bundle = HashMap::new();
        for (transform, model_ref) in query.iter(world) {
            let model_asset: ModelAsset = model_ref.asset_reference;
            if !instance_bundle.contains_key(&model_asset) {
                instance_bundle.insert(model_asset, Vec::new());
            }
            let q: cgmath::Quaternion<f32> = transform.rotation.into();
            let instance = Instance::new(transform.position.clone(), q.normalize());
            instance_bundle
                .get_mut(&model_asset)
                .unwrap()
                .push(instance)
        }
        let mut query = <(&component::Transform, &component::MeshReference)>::query();
        for (transform, mesh_ref) in query.iter(world) {
            let mesh_ref: &component::MeshReference = mesh_ref;
            let resources = std::path::Path::new(env!("OUT_DIR")).join("resources");
            let q: cgmath::Quaternion<f32> = transform.rotation.into();
            let instance = Instance::new(transform.position.clone(), q.normalize());
            let model_asset = ModelAsset::Dynamic(mesh_ref.idx);
            if !self.assets.contains_key(&model_asset) {
                let model = Model::load_from_vertex_data(
                    "dynamic".to_string(),
                    &display.device,
                    &display.queue,
                    pipeline,
                    &mesh_ref.vertex_data,
                    &mesh_ref.index_data,
                    resources.join("blockatlas.jpg"),
                )
                .unwrap();
                self.assets.insert(model_asset, model);
            }

            if !instance_bundle.contains_key(&model_asset) {
                instance_bundle.insert(model_asset, Vec::new());
            }
            instance_bundle
                .get_mut(&model_asset)
                .unwrap()
                .push(instance)
        }

        let mut current_scene: Option<Box<Scene>> = None;
        for (asset, _) in &instance_bundle {
            if !self.assets.contains_key(&asset) {
                self.assets
                    .insert(*asset, load_asset(asset, pipeline, display));
            }
        }
        for (asset, instances) in instance_bundle {
            current_scene = Some(Box::new(Scene {
                model: &self.assets.get(&asset).unwrap(),
                instances,
                next: current_scene,
            }))
        }

        current_scene
    }
}

fn load_asset<P: Pipeline>(model_asset: &ModelAsset, pipeline: &P, display: &Display) -> Model {
    let resources = std::path::Path::new(env!("OUT_DIR")).join("resources");
    match model_asset {
        ModelAsset::Cube => Model::load(
            "cube".to_string(),
            &display.device,
            &display.queue,
            pipeline,
            resources.join("cube.obj"),
        )
        .unwrap(),
        ModelAsset::Room => Model::load(
            "room".to_string(),
            &display.device,
            &display.queue,
            pipeline,
            resources.join("viking_room.obj"),
        )
        .unwrap(),
        _ => unimplemented!(),
    }
}
