use bevy::prelude::*;

mod chunk;
mod chunk_culling;
mod generator;
mod loader;
mod mesher;
mod ordered_float;
mod shader;
mod voxel;

pub use generator::{GeneratorBundle, GeneratorOptions};
pub use voxel::Voxel;
pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        loader::add_systems(app);
        mesher::add_systems(app);
        chunk_culling::add_systems(app);
        app.add_startup_system(shader::pipeline_setup.system())
            .add_system_to_stage(CoreStage::PreUpdate, generator::update_generators.system());
    }
}
