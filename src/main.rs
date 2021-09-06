mod chunk;
mod creature;
#[cfg(feature = "debug")]
mod debug;
mod defs;
mod item;
mod macro_help;
mod network;
mod stats;
mod world;

use crate::chunk::ChunkGenerator;
use bevy::{prelude::*, render::camera::Camera};
use bevy_flycam::{MovementSettings, PlayerPlugin};

fn printer(mut messages: EventReader<defs::Message>) {
    for message in messages.iter() {
        println!("{}", message);
    }
}

fn add_chunk_generator_to_camera(
    mut commands: Commands,
    query: Query<Entity, (With<Camera>, Without<ChunkGenerator>)>,
) {
    for e in query.iter() {
        commands.entity(e).insert(ChunkGenerator::default());
    }
}

fn movement_input(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mut settings: ResMut<MovementSettings>,
    camera: Query<&GlobalTransform, With<Camera>>,
) {
    const BOOST: f32 = 5.;
    if keys.just_pressed(KeyCode::LControl) {
        settings.speed *= BOOST;
    } else if keys.just_released(KeyCode::LControl) {
        settings.speed /= BOOST;
    }
    if keys.just_pressed(KeyCode::C) {
        settings.speed *= BOOST * 10.;
    } else if keys.just_released(KeyCode::C) {
        settings.speed /= BOOST * 10.;
    }

    if keys.just_pressed(KeyCode::R) {
        for c in camera.iter() {
            commands.spawn().insert(chunk::SphereEdit::new(
                c.translation,
                200.,
                chunk::Voxel::default(),
            ));
        }
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
    App::build()
        .insert_resource(Msaa { samples: 8 })
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugPlugin)
        .add_plugin(chunk::ChunkPlugin)
        .add_plugin(PlayerPlugin)
        .add_event::<defs::Message>()
        .add_plugin(defs::Definitions)
        .add_system(add_chunk_generator_to_camera.system())
        .add_system(printer.system())
        .add_system(movement_input.system())
        .run();
}
