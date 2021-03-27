use crate::mesh::MeshVertex;
use crate::worker::pool::Pool;
use crate::{ecs::component::*, worker::worker::Worker};
use legion::{Entity, World};
use noise::Fbm;
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    NoiseFn,
};
use std::{collections::HashSet, thread};
use std::{
    collections::{BinaryHeap, HashMap},
    sync::{mpsc, Arc, Mutex},
};

pub struct ChunkManager {
    current_idx: u32,
    pool: Pool<ChunkWork, ChunkWorkerInitializer, ChunkWorker>,
    pending: HashMap<MeshId, PendingWork>,
    active_position: cgmath::Vector2<i32>,
    live_chunks: HashMap<MeshId, Entity>,
}

struct PendingWork {
    position: cgmath::Vector3<f32>,
    killer: mpsc::Sender<bool>,
    complete: mpsc::Receiver<MeshReference>,
}

impl PendingWork {
    fn kill(&mut self) {
        self.killer.send(true).expect("Failed to kill pending task");
    }
}

impl ChunkManager {
    pub fn new(device: Arc<wgpu::Device>) -> Self {
        Self {
            current_idx: 1,
            pool: Pool::new(
                2,
                ChunkWorkerInitializer {
                    device: Arc::clone(&device),
                },
            ),
            pending: HashMap::new(),
            active_position: cgmath::Vector2::new(0, 0),
            live_chunks: HashMap::new(),
        }
    }

    pub fn dispatch(
        &mut self,
        position: cgmath::Vector3<f32>,
        chunk_location: cgmath::Vector2<i32>,
    ) {
        let (k_sender, k_receiver) = mpsc::channel();
        let (d_sender, d_receiver) = mpsc::channel();
        let pending_work = PendingWork {
            position,
            killer: k_sender,
            complete: d_receiver,
        };
        let work = ChunkWork {
            idx: self.current_idx,
            position: chunk_location,
            receiver: k_receiver,
            sender: d_sender,
        };
        self.pending.insert(
            MeshId(chunk_location.x as i32, chunk_location.y as i32),
            pending_work,
        );
        self.current_idx += 1;
        self.pool.dispatch(work);
    }

    pub fn load_region(&mut self, chunk_position: cgmath::Vector2<i32>) -> HashSet<Entity> {
        println!("Loading chunk around {:?}", chunk_position);
        let mut heap = BinaryHeap::new();
        let mut active_set = HashSet::new();
        for x in -CHUNK_RADIUS..CHUNK_RADIUS + 1 {
            for z in -CHUNK_RADIUS..CHUNK_RADIUS + 1 {
                let chunk_id = MeshId(chunk_position.x + x, chunk_position.y + z);
                if x * x + z * z > CHUNK_RADIUS * CHUNK_RADIUS {
                    continue;
                }
                active_set.insert(chunk_id.clone());
                if !self.live_chunks.contains_key(&chunk_id) {
                    heap.push(chunk_id);
                }
            }
        }
        let mut to_remove = HashSet::new();

        for (chunk, _) in &self.live_chunks {
            if !active_set.contains(chunk) {
                to_remove.insert(chunk.clone());
            }
        }

        let mut removed_entities = HashSet::new();
        for chunk in to_remove {
            removed_entities.insert(self.live_chunks.remove(&chunk).unwrap());
        }

        while let Some(chunk_id) = heap.pop() {
            let position = cgmath::Vector3::new(
                (chunk_id.0 * CHUNK_SIZE as i32) as f32,
                0.0,
                (chunk_id.1 * CHUNK_SIZE as i32) as f32,
            );
            let c_p = cgmath::Vector2::new(chunk_id.0, chunk_id.1);

            self.dispatch(position, c_p);
        }

        removed_entities
    }

    pub fn update(&mut self, world: &mut World, position: cgmath::Vector2<f32>) {
        let new_pos = Self::world_to_chunk_space(position);
        if self.active_position != new_pos {
            println!("Loading new chunk position");
            self.active_position = new_pos;
            self.pending.drain().for_each(|(_, pending)| {
                pending.killer.send(true);
            });
            let entities_to_delete = self.load_region(self.active_position);

            for entity in entities_to_delete {
                world.remove(entity);
            }
        }

        let mut complete_work = HashSet::new();
        for (idx, work) in &self.pending {
            if let Ok(chunk) = work.complete.try_recv() {
                complete_work.insert(idx.clone());

                let idx = chunk.idx.clone();
                let entity = world.push((
                    Transform {
                        position: work.position,
                        rotation: cgmath::Euler::new(
                            cgmath::Rad(0.0),
                            cgmath::Rad(0.0),
                            cgmath::Rad(0.0),
                        ),
                    },
                    chunk,
                ));
                self.live_chunks.insert(idx, entity);
                break;
            }
        }

        for work in complete_work {
            self.pending.remove(&work);
        }
    }

    pub fn world_to_chunk_space(position: cgmath::Vector2<f32>) -> cgmath::Vector2<i32> {
        cgmath::Vector2::new(
            (position.x as i32) / CHUNK_SIZE as i32,
            (position.y as i32) / CHUNK_SIZE as i32,
        )
    }
}

pub struct ChunkWorker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

pub struct ChunkWorkerInitializer {
    device: Arc<wgpu::Device>,
}

pub struct ChunkWork {
    idx: u32,
    position: cgmath::Vector2<i32>,
    receiver: mpsc::Receiver<bool>,
    sender: mpsc::Sender<MeshReference>,
}

impl Worker<ChunkWork, ChunkWorkerInitializer> for ChunkWorker {
    fn new(
        id: usize,
        bundle: &ChunkWorkerInitializer,
        receiver: Arc<Mutex<mpsc::Receiver<ChunkWork>>>,
    ) -> Self {
        let exector = ChunkExecutor {
            device: Arc::clone(&bundle.device),
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

pub struct ChunkExecutor {
    device: Arc<wgpu::Device>,
}

impl ChunkExecutor {
    fn execute(&self, data: ChunkWork) {
        let idx = data.idx;
        let mesh = make_mesh(idx as u32, data.position);
        data.sender.send(mesh).unwrap();
    }
}
const CHUNK_SIZE: usize = 32;
const CHUNK_RADIUS: i32 = 10;

bitflags! {
    pub struct Sides: u32 {
        const NONE      = 0b00000000;
        const TOP       = 0b00000001;
        const BOTTOM    = 0b00000010;
        const LEFT      = 0b00000100;
        const RIGHT     = 0b00001000;
        const FORWARD   = 0b00010000;
        const BACKWARD  = 0b00100000;
    }
}

const CUBE_COORDINATES: [[f32; 3]; 8] = [
    [-0.5, -0.5, 0.5],
    [0.5, -0.5, 0.5],
    [0.5, -0.5, -0.5],
    [-0.5, -0.5, -0.5],
    [-0.5, 0.5, 0.5],
    [0.5, 0.5, 0.5],
    [0.5, 0.5, -0.5],
    [-0.5, 0.5, -0.5],
];

const UVS: [[f32; 2]; 4] = [
    [0.125, 1.0 - 0.9375],
    [0.1875, 1.0 - 0.9375],
    [0.125, 1.0 - 1.0],
    [0.1875, 1.0 - 1.0],
];

const QUAD_UV_ORDER: [u32; 4] = [3, 2, 0, 1];

pub struct ChunkBuilder {
    idx: u32,
}

impl ChunkBuilder {
    pub fn new() -> Self {
        Self { idx: 1 }
    }

    pub fn make_mesh(&mut self, chunk_location: cgmath::Vector2<i32>) -> MeshReference {
        let res = make_mesh(self.idx, chunk_location);
        self.idx += 1;
        res
    }
}

pub fn make_mesh(idx: u32, chunk_location: cgmath::Vector2<i32>) -> MeshReference {
    let mut builder = VoxelMeshBuilder::new();
    let mut fbm = Fbm::new();
    fbm.octaves = 4;
    fbm.persistence = 0.5;

    PlaneMapBuilder::new(&fbm).set_size(1000, 100);
    let mut height_map = vec![vec![vec![false; CHUNK_SIZE + 2]; CHUNK_SIZE + 2]; CHUNK_SIZE + 2];
    for x in 0..CHUNK_SIZE + 2 {
        for z in 0..CHUNK_SIZE + 2 {
            let stone_height = fbm.get([
                (x as i32 + chunk_location.x * CHUNK_SIZE as i32) as f64 * 0.05,
                (z as i32 + chunk_location.y * CHUNK_SIZE as i32) as f64 * 0.05,
            ]) * 16.0
                + (CHUNK_SIZE as f64 / 2.0);

            for y in 0..CHUNK_SIZE {
                height_map[x][y][z] = y < stone_height as usize;
            }
        }
    }

    for x in 1..CHUNK_SIZE + 1 {
        for y in 1..CHUNK_SIZE + 1 {
            for z in 1..CHUNK_SIZE + 1 {
                let pos = cgmath::Vector3::new(x, y, z);
                if height_map[x][y][z] {
                    builder
                        .set_position(&pos)
                        .generate_voxel(get_sides(&height_map, &pos));
                }
            }
        }
    }

    builder.build(MeshId(chunk_location.x as i32, chunk_location.y as i32))
}

fn get_sides(height_map: &Vec<Vec<Vec<bool>>>, pos: &cgmath::Vector3<usize>) -> Sides {
    let mut sides = Sides::NONE;

    if !(height_map[(pos.x - 1)][pos.y][pos.z]) {
        sides |= Sides::LEFT
    }

    if !(height_map[(pos.x + 1)][pos.y][pos.z]) {
        sides |= Sides::RIGHT
    }

    if (pos.y as i32) > 0 && !(height_map[(pos.x)][pos.y - 1][pos.z]) {
        sides |= Sides::BOTTOM
    }

    if !(height_map[(pos.x)][pos.y + 1][pos.z]) {
        sides |= Sides::TOP
    }

    if !(height_map[(pos.x)][pos.y][pos.z - 1]) {
        sides |= Sides::BACKWARD
    }

    if !(height_map[(pos.x)][pos.y][pos.z + 1]) {
        sides |= Sides::FORWARD
    }

    sides
}

pub struct VoxelMeshBuilder {
    current_cube_pos: cgmath::Vector3<u32>,
    indices: Vec<u32>,
    vertices: Vec<MeshVertex>,
    index_offset: u32,
}

const SIDE_VERTICES: [(Sides, [u32; 4]); 6] = [
    (Sides::TOP, [7, 6, 5, 4]),
    (Sides::BOTTOM, [0, 1, 2, 3]),
    (Sides::LEFT, [7, 4, 0, 3]),
    (Sides::RIGHT, [5, 6, 2, 1]),
    (Sides::FORWARD, [4, 5, 1, 0]),
    (Sides::BACKWARD, [6, 7, 3, 2]),
];

impl VoxelMeshBuilder {
    pub fn new() -> Self {
        Self {
            current_cube_pos: cgmath::Vector3::new(0, 0, 0),
            indices: Vec::new(),
            vertices: Vec::new(),
            index_offset: 0,
        }
    }

    pub fn set_position(&mut self, position: &cgmath::Vector3<usize>) -> &mut VoxelMeshBuilder {
        self.current_cube_pos.x = position.x as u32;
        self.current_cube_pos.y = position.y as u32;
        self.current_cube_pos.z = position.z as u32;
        self
    }

    pub fn move_position(&mut self, delta: cgmath::Vector3<u32>) -> &mut VoxelMeshBuilder {
        self.current_cube_pos += delta;
        self
    }

    pub fn generate_voxel(&mut self, sides: Sides) -> &mut VoxelMeshBuilder {
        for (side, indices) in SIDE_VERTICES.iter() {
            if sides.contains(*side) {
                self.build_quad(&indices);
            }
        }
        self
    }

    pub fn build(self, idx: MeshId) -> MeshReference {
        MeshReference {
            idx,
            vertex_data: self.vertices.into_boxed_slice(),
            index_data: self.indices.into_boxed_slice(),
        }
    }

    fn build_quad(&mut self, vertex_idx: &[u32; 4]) {
        for (vertex, uv) in vertex_idx.iter().zip(QUAD_UV_ORDER.iter()) {
            let mut v = CUBE_COORDINATES[*vertex as usize].clone();
            v[0] += self.current_cube_pos.x as f32;
            v[1] += self.current_cube_pos.y as f32;
            v[2] += self.current_cube_pos.z as f32;
            self.vertices.push(MeshVertex {
                position: v,
                tex_coords: UVS[*uv as usize].clone(),
                normal: [0.0, 1.0, 0.0],
            });
        }
        self.indices.extend(
            [
                3 + self.index_offset,
                1 + self.index_offset,
                0 + self.index_offset,
                3 + self.index_offset,
                2 + self.index_offset,
                1 + self.index_offset,
            ]
            .iter(),
        );
        self.index_offset += 4;
    }
}
