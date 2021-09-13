use bevy::{prelude::*, utils::HashMap};

use super::{generator::GeneratorTask, voxel::VoxelArray};

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_VOXELS: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
pub const MAX_DEPTH: u32 = 13;

const fn num_bits<T>() -> usize {
    std::mem::size_of::<T>() * 8
}
pub type ChunkVoxel =
    crate::get_type!({ num_bits::<usize>() - CHUNK_VOXELS.leading_zeros() as usize });

pub const fn chunk_size(depth: u32) -> usize {
    const DEPTH0_SIZE: usize = 1 << MAX_DEPTH;
    CHUNK_SIZE * (DEPTH0_SIZE >> depth)
}
fn convert_depth<const FROM: u32, const TO: u32>(pos: IVec3) -> IVec3 {
    let pos = pos.as_f32() * chunk_size(FROM) as f32;
    (pos / chunk_size(TO) as f32).as_i32()
}

pub fn parent_depth(pos: IVec3) -> IVec3 {
    pos / 2
}

pub fn get_child_index(pos: IVec3) -> usize {
    let pos = pos.abs();
    ((pos.y % 2) * 4 + (pos.z % 2) * 2 + (pos.x % 2)) as usize
}

pub fn get_child_position(index: usize) -> IVec3 {
    IVec3::new((index % 2) as i32, (index % 4) as i32 / 2, index as i32 / 4)
}

pub struct ChunkPos<const DEPTH: u32>(IVec3);

pub enum DataFlags {
    Empty,
    Full,
    None,
}

pub struct ChunkData<const DEPTH: u32> {
    pub voxels: Box<VoxelArray>,
    pub flags: DataFlags,
}

pub type ChunkDataTask<const DEPTH: u32> = GeneratorTask<ChunkData<DEPTH>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkState {
    Generating,
    Generated,
    Meshing,
    Ready,
}

#[derive(Default)]
pub struct Chunks<const DEPTH: u32> {
    pub chunks: HashMap<IVec3, Entity>,
}

#[derive(Debug)]
pub struct ChunkPosition(pub IVec3);

#[derive(Debug)]
pub struct ChildChunks(pub [Entity; 8]);

impl ChildChunks {
    pub fn get(&self, child: IVec3) -> Entity {
        self.0[get_child_index(child)]
    }
}
