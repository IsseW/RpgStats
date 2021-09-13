use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JoinData {
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameInfo {
    pub name: String,
    pub current_connections: u16,
    pub max_connections: u16,
    pub requires_password: bool,
}

#[macro_export]
macro_rules! net_commands {
    ($($channel:ident => [$(fn $name:ident($data:ident: ($($ty:ty), *), $from:ident : u32, $(, $($mutability:ident)? $arg:ident: $arg_ty:ty) * $(,)?)$block:block) *]) *) => {
        use paste::paste;
        paste!{
            pub mod net_events {
                use serde::{Deserialize, Serialize};

                #[derive(Serialize, Deserialize, Clone, Debug)]
                pub struct NetEvent<T> {
                    data: T,
                    from: u32,
                }

                impl<T> NetEvent<T> {
                    pub fn new(data: T, from: u32) -> Self {
                        Self {
                            data, from
                        }
                    }
                }
                $(
                    pub mod [< channel_ $channel:snake >] {
                        $(
                            #[derive(Serialize, Deserialize, Clone, Debug)]
                            pub struct [<$name:camel>](($($ty), *,));

                            impl From<($($ty), *,)> for [<$name:camel>] {
                                fn from(data: ($($ty), *,)) -> Self {
                                    Self(data)
                                }
                            }


                            pub fn $name(mut $data: EventReader<NetEvent<[<$name:camel>]>>, $($($mutability)? $arg: $arg_ty), *) {
                                fn [< $name _inner >]($data: &($($ty), *,), $from: usize, $($arg: & $($mutability)? $arg_ty), *) $block
                                for event in $data.iter() {
                                    [< $name _inner >](&event.data.0, event.from, $(&$($mutability)? $arg), *);
                                }
                            }
                        ) *
                    }
                ) *


                $(
                    #[derive(Serialize, Deserialize, Clone, Debug)]
                    pub enum [< $channel:camel >] {
                        $([<$name:camel>]([< channel $channel:snake >]::[<$name:camel>])), *
                    }
                ) *
            }

            pub struct NetCommands;

            impl bevy::prelude::Plugin for NetCommands {
                fn build(&self, app: &mut bevy::prelude::AppBuilder) {
                    use bevy::prelude::*;
                    app.add_plugin(bevy_networking_turbulence::NetworkingPlugin::default())
                        .add_startup_system(network_setup.system())
                    $(
                        .add_system([< recieve_ $channel:snake >].system().label(stringify!([< $channel:snake _read >])))
                        $(
                        .add_event::<net_events::NetEvent<net_events::[< channel $channel:snake >]::[<$name:camel>]>>()
                        .add_system(net_events::$name.system().after(stringify!([< $channel:snake _read >])))
                        ) *
                    ) *;
                }
            }



            pub fn network_setup(mut net: bevy::prelude::ResMut<bevy_networking_turbulence::NetworkResource>) {
                net.set_channels_builder(|builder: &mut bevy_networking_turbulence::ConnectionChannelsBuilder| {
                    $(
                        builder
                            .register::<net_events::[< $channel:camel >]>($channel)
                            .unwrap();
                    ) *
                });
            }


            $(
                fn [< recieve_ $channel:snake >](
                    mut net: bevy::prelude::ResMut<bevy_networking_turbulence::NetworkResource>,
                    $(mut [< $name _writer >]: bevy::prelude::EventWriter<net_events::NetEvent<net_events::[< channel $channel:snake >]::[<$name:camel>]>>), *
                ) {
                    for (_handle, connection) in net.connections.iter_mut() {
                        let channels = connection.channels().unwrap();
                        while let Some(client_message) = channels.recv::<net_events::[< $channel:camel >]>() {
                            match client_message {
                                $(net_events::[< $channel:camel >]::[< $name:camel >](data) => {
                                    [< $name _writer >].send(net_events::NetEvent::new(data, _handle));
                                }), *
                            }
                        }
                    }
                }
            ) *
            $(pub use net_events::[< $channel:camel >];) *
        }
    }
}
