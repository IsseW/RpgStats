use std::mem::{self, MaybeUninit};

use crate::{chunk::chunk::get_child_position, cmap, world::WorldOptions};

use super::{GeneratorOptions, chunk::{CHUNK_SIZE, ChildChunks, ChunkData, ChunkDataTask, ChunkPosition, ChunkState, Chunks, DataFlags, MAX_DEPTH, chunk_size}, generator::{Generator, GeneratorData}, mesher::ChunkMesh, voxel::*};
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
    let p = p.as_f32() * CHUNK_SIZE as f32;

    let noise = NoiseBuilder::fbm_2d_offset(p.x, CHUNK_SIZE, p.z, CHUNK_SIZE)
        .with_seed(seed)
        .with_freq(0.005 / (2 << (MAX_DEPTH - DEPTH)) as f32)
        .with_octaves(8)
        .generate()
        .0;
    let mut voxels = VoxelArray::empty();
    let mut empty = true;
    let mut full = true;

    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let p = (p + Vec3::new(x as f32, y as f32, z as f32)) * chunk_size(DEPTH) as f32
                    / CHUNK_SIZE as f32;
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
                    .insert(ChunkPosition::<0>(chunk_pos))
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
    mut child_chunks: ResMut<Chunks<DEPTH>>,
    parent_chunks: ResMut<Chunks<{ DEPTH - 1 }>>,
    parent_query: Query<(&ChunkData<{ DEPTH - 1 }>, &ChunkState), Without<ChildChunks>>,
    mut generators: Query<(Entity, &Generator, &mut GeneratorData<DEPTH>)>,
    options: Res<GeneratorOptions>,
    world_options: Res<WorldOptions>,
    thread_pool: Res<AsyncComputeTaskPool>,
) {
    let seed = world_options.seed;
    for (entity, gen, mut data) in generators.iter_mut() {
        while data.gen_index < gen.load_order_half.len()
            && data.current_gen_task_count < options.concurrent_generate_tasks
        {
            let parent_chunk_pos = data.position / 2 + gen.load_order_half[data.gen_index];
            if let Some(&parent) = parent_chunks.chunks.get(&parent_chunk_pos) {
                if let Ok((d, &state)) = parent_query.get(parent) {
                    if state == ChunkState::Ready {
                        if !child_chunks.chunks.contains_key(&(parent_chunk_pos * 2)) {
                            let mut children: [MaybeUninit<Entity>; 8] =
                                unsafe { MaybeUninit::uninit().assume_init() };
                            for i in 0..8 {
                                let p = parent_chunk_pos * 2 + get_child_position(i);
                                let generate_task = thread_pool
                                    .spawn(async move { generate_chunk::<DEPTH>(p, seed) });

                                let e = commands
                                    .spawn()
                                    .insert(ChunkPosition::<DEPTH>(p))
                                    .insert(ChunkState::Generating)
                                    .insert(ChunkDataTask {
                                        sender: entity,
                                        task: generate_task,
                                    })
                                    .id();
                                children[i] = MaybeUninit::new(e);

                                child_chunks.chunks.insert(p, e);
                            }

                            data.current_gen_task_count += 8;

                            commands.entity(parent).insert(ChildChunks(unsafe {
                                mem::transmute::<_, [Entity; 8]>(children)
                            }));
                        }
                    }
                }
            }
            data.gen_index += 1;
        }
    }
}

fn chunk_generation_task_handler<const DEPTH: u32>(
    mut commands: Commands,
    mut tasks: Query<(
        Entity,
        &mut ChunkState,
        &ChunkPosition<DEPTH>,
        &mut ChunkDataTask<DEPTH>,
    )>,
    mut generator_data: Query<&mut GeneratorData<DEPTH>>,
) {
    for (entity, mut state, position, mut task) in tasks.iter_mut() {
        if let Some(data) = block_on(poll_once(&mut task.task)) {
            match data.flags {
                DataFlags::Empty => {
                    commands.entity(entity).insert(ChunkMesh(DEPTH == 0));
                    *state = ChunkState::Ready;
                }
                DataFlags::Full => {
                    commands
                        .entity(entity)
                        .insert(data)
                        .insert(ChunkMesh(DEPTH == 0));
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

                sender.mesh_index = 0;
            }
        }
    }
}

fn hide_parent<const DEPTH: u32>(
    mut commands: Commands,
    mut parents: Query<(
        Entity,
        &ChunkPosition<{ DEPTH - 1 }>,
        &ChildChunks,
        &mut ChunkMesh,
    )>,
    mut child_query: Query<
        (&mut ChunkMesh, &ChunkState),
        (
            With<ChunkPosition<DEPTH>>,
            Without<ChunkPosition<{ DEPTH - 1 }>>,
        ),
    >,
    mut child_chunks: ResMut<Chunks<DEPTH>>,
    gen: Query<(&Generator, &GeneratorData<DEPTH>)>,
) {
    'parent_loop: for (entity, pos, children, mut mesh) in parents.iter_mut() {
        if mesh.0 {
            for e in children.0 {
                if let Ok((_, &state)) = child_query.get_mut(e) {
                    if state != ChunkState::Ready {
                        continue 'parent_loop;
                    }
                } else {
                    continue 'parent_loop;
                }
            }
            mesh.0 = false;
            for e in children.0 {
                if let Ok((mut mesh, _)) = child_query.get_mut(e) {
                    mesh.0 = true;
                }
            }
        } 
        //else if gen.iter().all(|(gen, data)| {
        //    // TODO : make check async
        //    let dif = data.position - pos.0 * 2;
        //    dif.x * dif.x + dif.y * dif.y + dif.z * dif.z
        //        > (gen.unload_distance + 1).pow(2)
        //}) {
        //    mesh.0 = true;
        //    commands.entity(entity).remove::<ChildChunks>();
        //    for i in 0..8 {
        //        commands.entity(children.0[i]).despawn();
        //        child_chunks
        //            .chunks
        //            .remove(&(pos.0 * 2 + get_child_position(i)));
        //    }
        //}
    }
}

pub fn add_systems(app: &mut AppBuilder) {
    app.add_system(load_chunks.system())
        .insert_resource(GeneratorOptions {
            concurrent_generate_tasks: 10,
            concurrent_meshing_tasks: 5,
        });
    seq_macro::seq!(N in 1..=13 {
        app
        #(
            .add_system(promote_chunks::<N>.system())
            .add_system_to_stage(CoreStage::PostUpdate, hide_parent::<N>.system())
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
