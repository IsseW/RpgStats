#![feature(float_interpolation)]
#![feature(int_roundings)]
mod animation;
mod chunk;
mod creature;
#[cfg(feature = "debug")]
mod debug;
mod defs;
mod flycam;
mod item;
mod macro_help;
mod network;
mod region;
mod stats;
mod world;

use bevy::prelude::*;

fn printer(mut messages: EventReader<defs::Message>) {
    for message in messages.iter() {
        println!("{}", message);
    }
}

struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut AppBuilder) {
        #[cfg(feature = "debug")]
        app.add_plugin(debug::DebugPlugin);
    }
}

fn main() {
    #[cfg(feature = "debug")]
    {
        puffin::set_scopes_on(true);
    }
    App::build()
        .insert_resource(Msaa { samples: 8 })
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugPlugin)
        .add_plugin(world::WorldPlugin)
        .add_plugin(chunk::ChunkPlugin)
        .add_plugin(flycam::PlayerPlugin)
        .add_event::<defs::Message>()
        .add_plugin(defs::Definitions)
        .add_system(printer.system())
        .run();
}
