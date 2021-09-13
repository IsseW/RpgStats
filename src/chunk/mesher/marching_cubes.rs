use bevy::math::Vec3;

use crate::chunk::{
    chunk::CHUNK_SIZE,
    voxel::{VoxelArray, Voxels},
};

use super::{
    marching_cubes_tables::{EDGE_CONNECTION, EDGE_CROSSING_MASK, TRIANGLE_CONNECTION},
    Vertex,
};

fn march<F: FnMut([Vertex; 3])>(voxels: &VoxelArray, add_triangle: F) {
    for y in 0..(CHUNK_SIZE - 1) {
        for z in 0..(CHUNK_SIZE - 1) {
            for x in 0..(CHUNK_SIZE - 1) {
                let cube = [
                    voxels.at(x, y, z),
                    voxels.at(x + 1, y, z),
                    voxels.at(x, y + 1, z),
                    voxels.at(x + 1, y + 1, z),
                    voxels.at(x, y, z + 1),
                    voxels.at(x, y + 1, z + 1),
                    voxels.at(x + 1, y, z + 1),
                    voxels.at(x + 1, y + 1, z + 1),
                ];

                let index = (((cube[0].strength > 0) as usize) << 0)
                    | (((cube[1].strength > 0) as usize) << 1)
                    | (((cube[2].strength > 0) as usize) << 2)
                    | (((cube[3].strength > 0) as usize) << 3)
                    | (((cube[4].strength > 0) as usize) << 4)
                    | (((cube[5].strength > 0) as usize) << 5)
                    | (((cube[6].strength > 0) as usize) << 6)
                    | (((cube[7].strength > 0) as usize) << 7);

                
            }
        }
    }
}
