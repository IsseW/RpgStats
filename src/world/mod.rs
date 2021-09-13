mod person;
mod world;

use bevy::prelude::*;
pub use world::WorldOptions;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(WorldOptions {
            size: 1000,
            num_settlements: 100,
            seed: 6969,
        });
    }
}
