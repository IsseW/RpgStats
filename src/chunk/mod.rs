use bevy::prelude::*;

mod chunk;
mod chunk_culling;
mod loader;
mod meshing;
mod ordered_float;
mod shader;
mod voxel;

mod chunk_edit;

pub use chunk_edit::SphereEdit;
pub use loader::ChunkGenerator;
pub use voxel::Voxel;
pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        loader::add_systems(app);
        meshing::add_systems(app);
        chunk_edit::add_systems(app);

        app.add_startup_system(shader::pipeline_setup.system())
            .add_system(chunk_culling::frustum_culling.system());
    }
}
