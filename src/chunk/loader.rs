use std::mem::{self, MaybeUninit};

use crate::{chunk::chunk::get_child_position, cmap, world::WorldOptions};

use super::{
    chunk::{
        ChildChunks, ChunkData, ChunkDataTask, ChunkPosition, ChunkState, Chunks, DataFlags,
        CHUNK_SIZE,
    },
    generator::{Generator, GeneratorData},
    mesher::ChunkMesh,
    voxel::*,
    GeneratorOptions,
};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future::{block_on, poll_once};
use simdnoise::NoiseBuilder;

struct CPO<const V: u32>;

impl<const V: u32> CPO<V> {
    const RES: u32 = V + 1;
}
struct CMO<const V: u32>;

impl<const V: u32> CMO<V> {
    const RES: u32 = V - 1;
}

fn generate_voxel(
    x: usize,
    y: usize,
    z: usize,
    strength: f32,
    p: Vec3,
    voxels: &mut VoxelArray,
    empty: &mut bool,
    full: &mut bool,
) {
    fn get_id(x: f32, y: f32, z: f32) -> u8 {
        ((((x * z * y) as i32 ^ (x + z + y) as i32) % 9).abs() + 1) as u8
    }

    if strength <= 0.0 {
        *full = false;
        *voxels.at_mut(x, y, z) = Voxel { id: 0 }
    } else {
        *empty = false;
        *voxels.at_mut(x, y, z) = Voxel {
            id: get_id(p.x, p.y, p.z),
        }
    }
}

fn generate_chunk<const DEPTH: u32>(p: IVec3, seed: i32) -> ChunkData<DEPTH> {
    let p = (p.as_f32() * CHUNK_SIZE as f32 - Vec3::ONE) * 2f32.powi(DEPTH as i32);

    let noise = NoiseBuilder::fbm_2d_offset(p.x, CHUNK_SIZE, p.z, CHUNK_SIZE)
        .with_seed(seed)
        .with_freq(0.005 / (2 << DEPTH) as f32)
        .with_octaves(8)
        .generate()
        .0;
    let mut voxels = VoxelArray::empty();
    let mut empty = true;
    let mut full = true;

    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let p = p + Vec3::new(x as f32, y as f32, z as f32) * 2f32.powi(DEPTH as i32);
                let strength = noise[z * CHUNK_SIZE + x] * 300. - p.y;

                generate_voxel(x, y, z, strength, p, &mut voxels, &mut empty, &mut full);
            }
        }
    }
    ChunkData::<DEPTH> {
        voxels: Box::new(voxels),
        flags: if empty {
            DataFlags::Empty
        } else if full {
            DataFlags::Full
        } else {
            DataFlags::None
        },
    }
}

fn load_chunks(
    mut commands: Commands,
    mut chunks: ResMut<Chunks<0>>,
    options: Res<GeneratorOptions>,
    mut generators: Query<(Entity, &Generator, &mut GeneratorData<0>)>,
    thread_pool: Res<AsyncComputeTaskPool>,
    world_options: Res<WorldOptions>,
) {
    let seed = world_options.seed;
    for (entity, gen, mut data) in generators.iter_mut() {
        while data.gen_index < gen.load_order.len()
            && data.current_gen_task_count < options.concurrent_generate_tasks
        {
            let chunk_pos = data.position + gen.load_order[data.gen_index];
            if !chunks.chunks.contains_key(&chunk_pos) {
                let generate_task =
                    thread_pool.spawn(async move { generate_chunk::<0>(chunk_pos, seed) });
                let e = commands
                    .spawn()
                    .insert(ChunkPosition(chunk_pos))
                    .insert(ChunkState::Generating)
                    .insert(ChunkDataTask {
                        sender: entity,
                        task: generate_task,
                    })
                    .id();
                data.current_gen_task_count += 1;
                chunks.chunks.insert(chunk_pos, e);
            }
            data.gen_index += 1;
        }
    }
}

fn promote_chunks<const DEPTH: u32>(
    mut commands: Commands,
    mut high_chunks: ResMut<Chunks<DEPTH>>,
    low_chunks: ResMut<Chunks<{ DEPTH - 1 }>>,
    mut generators: Query<(Entity, &Generator, &mut GeneratorData<DEPTH>)>,
    options: Res<GeneratorOptions>,
    thread_pool: Res<AsyncComputeTaskPool>,
    world_options: Res<WorldOptions>,
) {
    let seed = world_options.seed;
    for (entity, gen, mut data) in generators.iter_mut() {
        while data.gen_index < gen.load_order_half.len()
            && data.current_gen_task_count < options.concurrent_generate_tasks
        {
            let parent_chunk_pos = data.position / 2 + gen.load_order_half[data.gen_index];
            if let Some(&parent) = low_chunks.chunks.get(&parent_chunk_pos) {
                if !high_chunks.chunks.contains_key(&(parent_chunk_pos * 2)) {
                    let mut children: [MaybeUninit<Entity>; 8] =
                        unsafe { MaybeUninit::uninit().assume_init() };
                    for i in 0..8 {
                        let p = parent_chunk_pos * 2 + get_child_position(i);
                        let generate_task =
                            thread_pool.spawn(async move { generate_chunk::<DEPTH>(p, seed) });

                        let e = commands
                            .spawn()
                            .insert(ChunkPosition(p))
                            .insert(ChunkState::Generating)
                            .insert(ChunkDataTask {
                                sender: entity,
                                task: generate_task,
                            })
                            .id();
                        children[i] = MaybeUninit::new(e);

                        high_chunks.chunks.insert(p, e);
                    }

                    data.current_gen_task_count += 8;

                    commands.entity(parent).insert(ChildChunks(unsafe {
                        mem::transmute::<_, [Entity; 8]>(children)
                    }));
                }
            }
            data.gen_index += 1;
        }
    }
}

fn demote_chunks<const DEPTH: u32>(
    high_chunks: Res<Chunks<DEPTH>>,
    mut low_chunks: ResMut<Chunks<{ DEPTH - 1 }>>,
) {
}

fn chunk_generation_task_handler<const DEPTH: u32>(
    mut commands: Commands,
    mut tasks: Query<(
        Entity,
        &mut ChunkState,
        &ChunkPosition,
        &mut ChunkDataTask<DEPTH>,
    )>,
    mut generator_data: Query<&mut GeneratorData<DEPTH>>,
) {
    for (entity, mut state, position, mut task) in tasks.iter_mut() {
        if let Some(data) = block_on(poll_once(&mut task.task)) {
            match data.flags {
                DataFlags::Empty => {
                    commands.entity(entity).insert(ChunkMesh); // CHUNKMESH
                    *state = ChunkState::Ready;
                }
                DataFlags::Full => {
                    commands.entity(entity).insert(data).insert(ChunkMesh); // CHUNKMESH
                    *state = ChunkState::Ready;
                }
                DataFlags::None => {
                    commands.entity(entity).insert(data);
                    *state = ChunkState::Generated;
                }
            }

            commands.entity(entity).remove::<ChunkDataTask<DEPTH>>();

            if let Ok(mut sender) = generator_data.get_mut(task.sender) {
                sender.current_gen_task_count -= 1;
            }
        }
    }
}

fn hide_parent(
    mut commands: Commands,
    parents: Query<(Entity, &ChildChunks), With<Handle<Mesh>>>,
    child_query: Query<&ChunkMesh>,
) {
    parents
        .iter()
        .filter(|(_, children)| children.0.iter().all(|&e| child_query.get(e).is_ok()))
        .for_each(|(entity, _)| {
            commands.entity(entity).remove_bundle::<PbrBundle>();
        });
}

pub fn add_systems(app: &mut AppBuilder) {
    app.add_system(load_chunks.system())
        .add_system(hide_parent.system())
        .insert_resource(GeneratorOptions {
            concurrent_generate_tasks: 10,
            concurrent_meshing_tasks: 5,
        });
    seq_macro::seq!(N in 1..=13 {
        app
        #(
            .add_system(promote_chunks::<N>.system())
            .add_system(demote_chunks::<N>.system())
        )*;
    });
    seq_macro::seq!(N in 0..=13 {
        app
        #(
            .insert_resource(Chunks::<N>::default())
            .add_system(chunk_generation_task_handler::<N>.system())
        )*;
    });
}
