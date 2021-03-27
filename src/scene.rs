use crate::{
    asset::ModelAsset,
    display::Display,
    ecs::component::MeshReference,
    instance::Instance,
    mesh::Model,
    pipeline::{Pipeline, PipelineBindGroupInfo},
};
use crate::{
    ecs::*,
    worker::{pool::Pool, worker::Worker},
};
use cgmath::InnerSpace;
use legion::*;
use std::{
    collections::{HashMap, HashSet},
    sync::{
        mpsc::{self, TryRecvError},
        Arc, Mutex,
    },
    thread,
};

pub struct Scene<'a> {
    pub model: &'a Model,
    pub instances: Vec<Instance>,
    pub next: Option<Box<Scene<'a>>>,
}

struct PendingWork {
    killer: mpsc::Sender<bool>,
    complete: mpsc::Receiver<Model>,
}
pub struct SceneManager {
    assets: HashMap<ModelAsset, Model>,
    asset_workers: Pool<AssetWork, AssetWorkerInitializer, AssetWorker>,
    pending_assets: HashMap<ModelAsset, PendingWork>,
}
impl SceneManager {
    pub fn new(display: &Display) -> Self {
        let initializer = AssetWorkerInitializer {
            device: Arc::clone(&display.device),
            queue: Arc::clone(&display.queue),
        };
        Self {
            assets: HashMap::new(),
            asset_workers: Pool::new(2, initializer),
            pending_assets: HashMap::new(),
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

        let mut finished_work = HashSet::new();
        for (model_asset, pending_work) in &self.pending_assets {
            match pending_work.complete.try_recv() {
                Ok(mesh) => {
                    self.assets.insert(model_asset.clone(), mesh);
                    finished_work.insert(model_asset.clone());
                }
                Err(TryRecvError::Empty) => {}
                _ => {
                    finished_work.insert(model_asset.clone());
                }
            }
        }

        for work in finished_work {
            self.pending_assets.remove(&work);
        }

        let mut query = <(&component::Transform, &component::MeshReference)>::query();
        for (transform, mesh_ref) in query.iter(world) {
            let mesh_ref: &component::MeshReference = mesh_ref;

            let q: cgmath::Quaternion<f32> = transform.rotation.into();
            let instance = Instance::new(transform.position.clone(), q.normalize());
            let model_asset = ModelAsset::DynamicMesh(mesh_ref.idx);

            if self.pending_assets.contains_key(&model_asset) {
                continue;
            }
            if !self.assets.contains_key(&model_asset) {
                let (k_sender, k_receiver) = mpsc::channel();
                let (d_sender, d_receiver) = mpsc::channel();

                let asset_work = AssetWork {
                    mesh: mesh_ref.clone(),
                    bind_group_info: pipeline
                        .bind_group_layout(crate::bind_group::BindGroupType::Material),
                    receiver: k_receiver,
                    sender: d_sender,
                };

                self.asset_workers.dispatch(asset_work);

                let pending_work = PendingWork {
                    killer: k_sender,
                    complete: d_receiver,
                };
                self.pending_assets.insert(model_asset, pending_work);
                continue;
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

pub struct AssetWorker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

pub struct AssetWorkerInitializer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

pub struct AssetWork {
    mesh: MeshReference,
    bind_group_info: Option<Arc<PipelineBindGroupInfo>>,
    receiver: mpsc::Receiver<bool>,
    sender: mpsc::Sender<Model>,
}

impl Worker<AssetWork, AssetWorkerInitializer> for AssetWorker {
    fn new(
        id: usize,
        bundle: &AssetWorkerInitializer,
        receiver: Arc<Mutex<mpsc::Receiver<AssetWork>>>,
    ) -> Self {
        let exector = AssetExecutor {
            device: Arc::clone(&bundle.device),

            queue: Arc::clone(&bundle.queue),
        };
        let thread = thread::spawn(move || loop {
            let work = receiver.lock().unwrap().recv().unwrap();
            let skip = match work.receiver.try_recv() {
                Ok(true) => true,
                Err(mpsc::TryRecvError::Disconnected) => true,
                _ => false,
            };
            if skip {
                println!("Skipping");
                continue;
            }

            exector.execute(work);
        });

        Self { id, thread }
    }
}

pub struct AssetExecutor {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

impl AssetExecutor {
    fn execute(&self, data: AssetWork) {
        let resources = std::path::Path::new(env!("OUT_DIR")).join("resources");
        data.sender.send(
            Model::load_from_vertex_data(
                "dynamic".to_string(),
                &self.device,
                &self.queue,
                data.bind_group_info,
                &data.mesh.vertex_data,
                &data.mesh.index_data,
                resources.join("blockatlas.jpg"),
            )
            .unwrap(),
        );
    }
}

fn load_asset<P: Pipeline>(model_asset: &ModelAsset, pipeline: &P, display: &Display) -> Model {
    let resources = std::path::Path::new(env!("OUT_DIR")).join("resources");
    match model_asset {
        ModelAsset::Cube => Model::load(
            "cube".to_string(),
            display.device.as_ref(),
            &display.queue,
            pipeline.bind_group_layout(crate::bind_group::BindGroupType::Material),
            resources.join("cube.obj"),
        )
        .unwrap(),
        ModelAsset::Room => Model::load(
            "room".to_string(),
            &display.device,
            &display.queue,
            pipeline.bind_group_layout(crate::bind_group::BindGroupType::Material),
            resources.join("viking_room.obj"),
        )
        .unwrap(),
        _ => unimplemented!(),
    }
}
