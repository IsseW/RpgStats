mod chunk_compressor;
mod chunk_generator;
mod clip_spheres;
mod frame_budget;
mod mesh_generator;
mod new_slot_detector;
mod sync_batch;
mod voxel_map;
mod voxel_mesh;

use bevy::prelude::*;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        todo!()
    }
}
