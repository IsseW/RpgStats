use aigame::network::*;
use bevy::{app::ScheduleRunnerSettings, prelude::*, utils::Duration};

use serde_cbor;
use std::net::{TcpListener, TcpStream};
use bevy_networking_turbulence::{NetworkEvent, NetworkResource, NetworkingPlugin, Packet};
use std::{net::SocketAddr};

mod commands;

use commands::*;

struct InfoListener {
    listener: TcpListener,
}
struct ConnectionListener {
    listener: TcpListener,
}

struct GameSettings {
    name: String,
    current_connections: u16,
    max_connections: u16,

    password: String,
}

impl GameSettings {
    fn get_info(&self) -> GameInfo {
        GameInfo {
            name: self.name.clone(),
            current_connections: self.current_connections,
            max_connections: self.max_connections,
            requires_password: self.password.len() > 0,
        }
    }
}

fn info_listener(info: Res<GameSettings>, listener: Res<InfoListener>) {
    for connection in listener.listener.incoming() {
        if let Ok(mut connection) = connection {
            serde_cbor::to_writer(&mut connection, &info.get_info());
            println!("Sent info");
        }
    }
}

struct Client {
    stream: TcpStream,
    id: usize,
}

fn connection_listener(
    mut commands: Commands,
    settings: ResMut<GameSettings>,
    listener: Res<ConnectionListener>,
    mut id: Local<usize>,
) {
    for connection in listener.listener.incoming() {
        if let Ok(mut connection) = connection {
            println!("Connection OK!");
            if let Ok(data) = serde_cbor::from_reader::<JoinData, _>(&mut connection) {
                println!("Data OK!");
                if data.password == settings.password {
                    println!("Password OK!");
                    serde_cbor::to_writer(&mut connection, &true);
                    commands.spawn().insert(Client {
                        stream: connection,
                        id: *id,
                    });
                    *id += 1;
                } else {
                    println!("Password ERROR!");
                    serde_cbor::to_writer(&mut connection, &false);
                }
            }
        }
    }
}

fn client_listener(mut event_writer: EventWriter<NetData>, query: Query<(Entity, &Client)>) {
    for (entity, client) in query.iter() {}
}

fn main() {
    println!("Running server!");

    let info_port: u16 = 3141;
    let game_port: u16 = 5926;
    let name = "A game!!!".to_string();
    let max_connections = 16;
    let password = "".to_string();

    //let info_lis = TcpListener::bind(format!("127.0.0.1:{}", info_port)).unwrap();
    //info_lis
    //    .set_nonblocking(true)
    //    .expect("Cannot set non-blocking");

    let connection_lis = TcpListener::bind(format!("127.0.0.1:{}", game_port)).unwrap();
    connection_lis
        .set_nonblocking(true)
        .expect("Cannot set non-blocking");

    App::build()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugins(MinimalPlugins)
    //    .insert_resource(InfoListener { listener: info_lis })
    //    .add_system(info_listener.system())
        .insert_resource(ConnectionListener {
            listener: connection_lis,
        })
        .add_system(connection_listener.system())
        .insert_resource(GameSettings {
            name,
            current_connections: 0,
            max_connections,
            password,
        })
        .add_stage_before(CoreStage::First, "network", SystemStage::parallel()
        .with_system(client_listener.system()))
        .net_events()
        .run();
}
