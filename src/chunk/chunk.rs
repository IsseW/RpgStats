use bevy::prelude::*;
use bevy::utils::HashMap;
use std::convert::TryInto;

use crate::cmap;

use super::voxel::{Voxel, VoxelArray, CHUNK_SIZE};

#[derive(Debug)]
pub struct ChunkPosition(pub IVec3);

impl ChunkPosition {
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
        const SIZE: f32 = CHUNK_SIZE as f32;
        let p = self.0.as_f32() * SIZE;
        let raw_corners = Self::get_raw_corners();
        cmap!(raw_corners[0..8] | a: IVec3 | p + a.as_f32() * SIZE)
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

#[derive(Clone, Copy)]
pub struct ChunkData {
    pub voxels: VoxelArray,
    pub num_voxels: usize,
}

impl ChunkData {
    pub fn get<T: TryInto<usize>>(&self, x: T, y: T, z: T) -> Option<&Voxel> {
        self.voxels
            .get(y.try_into().ok()?)?
            .get(z.try_into().ok()?)?
            .get(x.try_into().ok()?)
    }
    pub fn all(voxel: &Voxel) -> Self {
        Self {
            voxels: [[[*voxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
            num_voxels: if voxel.is_empty() {
                0
            } else {
                CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE
            },
        }
    }
}
pub struct Chunks {
    pub loaded: HashMap<IVec3, (Entity, bool)>,
}

impl Chunks {
    pub fn get(&self, pos: &IVec3) -> Option<&Entity> {
        Some(&self.loaded.get(pos)?.0)
    }
    pub fn is_generated(&self, pos: &IVec3) -> bool {
        if let Some(r) = self.loaded.get(pos) {
            r.1
        } else {
            false
        }
    }
}
