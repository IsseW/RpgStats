use bevy::utils::HashMap;
use bevy::{math::DVec3, prelude::*};
use bevy::{
    math::{Vec3Swizzles, Vec4Swizzles},
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future::{block_on, poll_once};
use rand::{thread_rng, Rng};
use simdnoise::NoiseBuilder;

use super::{
    chunk::{ChunkData, ChunkPosition, Chunks},
    ordered_float::OrderedFloat,
    voxel::{VoxelArray, CHUNK_SIZE},
};

#[derive(Default)]
pub struct ChunkGenerator {
    pub chunk_pos: IVec3,
    pub gen_list_index: usize,
    pub gen_job_count: usize,
    pub mesh_job_count: usize,

    deleting: bool,
}

fn calculate_chunk_pos(
    mut query: Query<(&mut ChunkGenerator, &GlobalTransform), Changed<GlobalTransform>>,
) {
    for (mut generator, transform) in query.iter_mut() {
        let new = (transform.translation / CHUNK_SIZE as f32).as_i32();
        if generator.chunk_pos != new {
            generator.chunk_pos = new;
            generator.gen_list_index = 0;
        }
    }
}

fn chunk_generator_handler(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut Task<ChunkData>, &TaskInfo)>,
    mut gen: Query<&mut ChunkGenerator>,
) {
    for (entity, mut task, info) in tasks.iter_mut() {
        if let Some(data) = block_on(poll_once(&mut *task)) {
            if data.num_voxels > 0 {
                commands.entity(entity).insert(data);
            }
            commands
                .entity(entity)
                .remove::<Task<ChunkData>>()
                .remove::<TaskInfo>();
            if let Ok(mut gen) = gen.get_mut(info.sender) {
                gen.gen_job_count -= 1;
            }
        }
    }
}

fn generate_chunk(pos: IVec3, seed: i32) -> ChunkData {
    let p = pos.as_f32() * CHUNK_SIZE as f32;
    let noise1 = NoiseBuilder::fbm_2d_offset(p.x, CHUNK_SIZE, p.z, CHUNK_SIZE)
        .with_seed(seed)
        .with_freq(0.005)
        .with_octaves(8)
        .generate()
        .0;
    let noise2 = NoiseBuilder::ridge_3d_offset(p.x, CHUNK_SIZE, p.y, CHUNK_SIZE, p.z, CHUNK_SIZE)
        .with_seed(seed)
        .generate()
        .0;

    let mut voxels = VoxelArray::default();
    let mut num_voxels = 0;
    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let p = pos.as_f32() * CHUNK_SIZE as f32 + Vec3::new(x as f32, y as f32, z as f32)
                    - Vec3::ONE * CHUNK_SIZE as f32 / 2.;

                if noise1[z * CHUNK_SIZE + x] * 300. > p.y
                    && noise2[y * CHUNK_SIZE * CHUNK_SIZE + z * CHUNK_SIZE + x] > 0.
                {
                    num_voxels += 1;
                    voxels[y][z][x].id = ((p.x * p.z * p.y % 9.).abs() + 1.) as u8;
                }
            }
        }
    }
    ChunkData { voxels, num_voxels }
}

fn chunk_loader(
    mut commands: Commands,
    mut chunks: ResMut<Chunks>,
    load_settings: Res<LoadSettings>,
    mut gen: Query<(Entity, &mut ChunkGenerator)>,
    thread_pool: Res<AsyncComputeTaskPool>,
) {
    for (entity, mut gen) in gen.iter_mut() {
        if gen.gen_list_index == 0 && !gen.deleting {
            let max = load_settings.unload_distance * load_settings.unload_distance;
            let chunks: Vec<(IVec3, Entity)> =
                chunks.loaded.iter().map(|(k, v)| (*k, *v)).collect();
            let pos = gen.chunk_pos.as_f32() * CHUNK_SIZE as f32;
            let task = thread_pool.spawn(async move {
                let mut delete = vec![];
                for (p, e) in chunks {
                    let d = (p * CHUNK_SIZE as i32).as_f32().distance_squared(pos);
                    if d > max {
                        delete.push((p, e));
                    }
                }
                DeleteChunks { chunks: delete }
            });
            gen.deleting = true;
            commands.spawn().insert(task).insert(TaskInfo::new(entity));
        }

        let mut count = 0;
        while count < load_settings.generated_per_frame
            && gen.gen_list_index < load_settings.load_chunks.len()
        {
            let c = gen.chunk_pos + load_settings.load_chunks[gen.gen_list_index];
            if !chunks.loaded.contains_key(&c) {
                count += 1;

                let gen_task = thread_pool.spawn(async move { generate_chunk(c, 6969) });
                gen.gen_job_count += 1;
                let e = commands
                    .spawn()
                    .insert(ChunkPosition(c))
                    .insert(gen_task)
                    .insert(TaskInfo::new(entity))
                    .id();
                chunks.loaded.insert(c, e);
            }

            gen.gen_list_index += 1;
        }
    }
}

pub struct TaskInfo {
    pub sender: Entity,
}

impl TaskInfo {
    pub fn new(sender: Entity) -> Self {
        Self { sender }
    }
}

struct DeleteChunks {
    chunks: Vec<(IVec3, Entity)>,
}

struct Seed(u64);

fn chunk_setup(mut commands: Commands) {
    commands.insert_resource(Seed(1337));
    commands.insert_resource(Chunks {
        loaded: HashMap::default(),
    })
}

fn chunk_deleter(
    mut commands: Commands,
    mut chunks: ResMut<Chunks>,
    mut query: Query<(Entity, &mut Task<DeleteChunks>, &TaskInfo)>,
    mut generators: Query<&mut ChunkGenerator>,
) {
    for (entity, mut task, info) in query.iter_mut() {
        if let Some(delete) = block_on(poll_once(&mut *task)) {
            for (pos, e) in delete.chunks {
                chunks.loaded.remove(&pos);
                commands.entity(e).despawn();
            }
            commands.entity(entity).despawn();
            if let Ok(mut gen) = generators.get_mut(info.sender) {
                gen.deleting = false;
            }
        }
    }
}

#[derive(Default)]
pub struct LoadSettings {
    pub load_distance: f32,
    pub unload_distance: f32,
    pub generated_per_frame: usize,
    pub meshed_per_frame: usize,

    load_chunks: Vec<IVec3>,
}

fn init_load_settings(mut commands: Commands) {
    let mut load_settings = LoadSettings {
        load_distance: 600.,
        unload_distance: 700.,
        generated_per_frame: 30,
        meshed_per_frame: 10,
        ..Default::default()
    };
    let extent = (load_settings.load_distance / CHUNK_SIZE as f32) as i32;
    let max = load_settings.load_distance * load_settings.load_distance;
    for y in -extent..extent {
        for z in -extent..extent {
            for x in -extent..extent {
                let pos = IVec3::new(x, y, z);
                let p = pos.as_f32() * CHUNK_SIZE as f32;
                let l = p.length_squared();
                if l <= max {
                    load_settings.load_chunks.push(pos);
                }
            }
        }
    }
    load_settings
        .load_chunks
        .sort_by_cached_key(|a| OrderedFloat(a.as_f32().length_squared()));
    commands.insert_resource(load_settings);
}

pub fn add_systems(app: &mut AppBuilder) {
    app.add_startup_system(chunk_setup.system())
        .add_startup_system(init_load_settings.system())
        .add_system_to_stage(CoreStage::PreUpdate, chunk_deleter.system())
        .add_system(chunk_generator_handler.system())
        .add_system(calculate_chunk_pos.system())
        .add_system(chunk_loader.system());
}
