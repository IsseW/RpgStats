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

pub struct ChunkPosition<const DEPTH: u32>(pub IVec3);

impl<const DEPTH: u32> ChunkPosition<DEPTH> {
    const SIZE: f32 = chunk_size(DEPTH) as f32;

    pub fn outside_plane(&self, point: Vec3, normal: Vec3) -> bool {
        let t = normal.dot(point);
        for p in self.get_corners() {
            if normal.dot(p) - t < 0. {
                return false;
            }
        }
        return true;
    }

    pub fn get_corners(&self) -> [Vec3; 8] {
        let p = self.0.as_f32() * Self::SIZE;
        let raw_corners = Self::get_raw_corners();
        crate::cmap!(raw_corners[0..8] | a: IVec3 | p + a.as_f32() * Self::SIZE)
    }

    pub fn get_raw_corners() -> [IVec3; 8] {
        [
            IVec3::new(0, 0, 0),
            IVec3::new(0, 0, 1),
            IVec3::new(0, 1, 0),
            IVec3::new(0, 1, 1),
            IVec3::new(1, 0, 0),
            IVec3::new(1, 0, 1),
            IVec3::new(1, 1, 0),
            IVec3::new(1, 1, 1),
        ]
    }
}

#[derive(PartialEq, Eq)]
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
pub struct ChildChunks(pub [Entity; 8]);

impl ChildChunks {
    pub fn get(&self, child: IVec3) -> Entity {
        self.0[get_child_index(child)]
    }
}
