use super::command_defs::NetCommands;
use crate::defs;

use {
    bevy::{
        app::{ScheduleRunnerPlugin, ScheduleRunnerSettings},
        prelude::*,
        utils::Duration,
    },
    bevy_networking_turbulence::{NetworkEvent, NetworkResource},
    std::net::SocketAddr,
};

const SERVER_PORT: u16 = 3141;

fn printer(mut prints: EventReader<defs::Message>) {
    for print in prints.iter() {
        println!("{}", print);
    }
}

fn server_setup(mut net: ResMut<NetworkResource>) {
    let ip_address =
        bevy_networking_turbulence::find_my_ip_address().expect("can't find ip address");
    let socket_address = SocketAddr::new(ip_address, SERVER_PORT);
    println!("Starting server");
    net.listen(socket_address, None, None);
}

struct Player {
    handle: u32,
}

fn handle_packets(
    mut commands: Commands,
    mut net: ResMut<NetworkResource>,
    mut network_events: EventReader<NetworkEvent>,
) {
    for event in network_events.iter() {
        match event {
            NetworkEvent::Connected(handle) => match net.connections.get_mut(handle) {
                Some(connection) => match connection.remote_address() {
                    Some(remote_address) => {
                        println!(
                            "Incoming connection on [{}] from [{}]",
                            handle, remote_address
                        );

                        commands.spawn_bundle((Player { handle: *handle },));
                    }
                    None => {
                        println!("Connected on [{}]", handle);
                    }
                },
                None => panic!("Got packet for non-existing connection [{}]", handle),
            },
            _ => {}
        }
    }
}

struct Server;

impl Plugin for Server {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugins(MinimalPlugins)
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(defs::Definitions)
        .add_system(printer.system())
        .add_startup_system(server_setup.system())
        .add_plugin(NetCommands);
    }
}
