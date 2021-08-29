use aigame::network::*;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_console::*;
use serde_cbor;
use server::*;
use std::net::TcpStream;
use bevy::core::FixedTimestep;

pub fn run(ip_address: (), port: ()) {}

#[derive(Default)]
struct Server {
    stream: Option<TcpStream>,
}

impl Server {
    fn write(&mut self, command: ServerCommand) -> serde_cbor::Result<()> {
        if let Some(stream) = &mut self.stream {
            serde_cbor::to_writer(stream, &command)
        }
        else {
            Err(serde_cbor::Error::message("Not connected to a server"))
        }
    }
}

fn listen_to_console_events(
    mut events: EventReader<ConsoleCommandEntered>,
    mut console_line: EventWriter<PrintConsoleLine>,
    mut app_exit_events: EventWriter<AppExit>,
    mut server: ResMut<Server>,
) {
    for event in events.iter() {
        let event: &ConsoleCommandEntered = event;
        info!("Commands: {:?}", event);
        match event.command.as_str() {
            "info" => {
                println!("Info");
                let mut stream = TcpStream::connect("127.0.0.1:3141").unwrap();

                let info: GameInfo = serde_cbor::from_reader(&mut stream).unwrap();

                console_line.send(PrintConsoleLine::new(format!("Info: \n{:?}", info)));
            }
            "join" => {
                let mut stream = TcpStream::connect("127.0.0.1:5926").unwrap();
                println!("Setting up connection");
                serde_cbor::to_writer(
                    &mut stream,
                    &JoinData {
                        password: event.args.clone(),
                    },
                ).expect("Error when trying to join");
                println!("Sent join data");

                println!("Waiting for server response...");
                if serde_cbor::from_reader::<bool, _>(&mut stream).unwrap_or(false) {
                    console_line.send(PrintConsoleLine::new("Joined server!".to_string()));
                    server.stream = Some(stream);
                } else {
                    console_line.send(PrintConsoleLine::new("Incorrect password!".to_string()));
                }
            }
            "quit" => {
                app_exit_events.send(AppExit);
            }
            _ => continue, // unknown command
        }
        console_line.send(PrintConsoleLine::new("Ok".to_string()));
    }
}

fn say_hello(mut server: ResMut<Server>, mut console_line: EventWriter<PrintConsoleLine>) {
    if server.write(ServerCommand::Message(("Hi".into(), "Hello".into()).into())).is_err() {
        // console_line.send(PrintConsoleLine::new("Could not say hi to server :(".into()));
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(ConsolePlugin)
        .insert_resource(ConsoleConfiguration {
            keys: vec![
                ToggleConsoleKey::ScanCode(41), // this is the console key on a swedish keyboard
                ToggleConsoleKey::KeyCode(KeyCode::Grave), // US console key
            ],
            ..Default::default()
        })
        .add_system(listen_to_console_events.system())
        .add_stage_after(CoreStage::Update, "fixed_update", SystemStage::parallel()
        .with_system(say_hello.system().with_run_criteria(FixedTimestep::step(1.0))))
        .insert_resource(Server::default())
        .run();
}
