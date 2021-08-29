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
    ($(fn $name:ident($data:ident: ($($ty:ty), * $(,)?), $from:ident : usize $(, $($mutability:ident)? $arg:ident: $arg_ty:ty) * $(,)?) $block:block) *) => {
        paste! {
            pub mod net_events {
                use bevy::prelude::*;
                use serde::{Deserialize, Serialize};

                #[derive(Serialize, Deserialize, Clone, Debug)]
                pub struct NetEvent<T> {
                    data: T,
                    from: usize,
                }


                impl<T> NetEvent<T> {
                    pub fn new(data: T, from: usize) -> Self {
                        Self {
                            data, from
                        }
                    }
                }

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

                #[derive(Serialize, Deserialize, Clone, Debug)]
                pub enum Command {
                    $([<$name:camel>]([<$name:camel>])), *
                }
                #[derive(Serialize, Deserialize, Clone, Debug)]
                pub struct NetMessage(pub Vec<Command>);
            }

            pub trait AddNetCommands {
                fn net_events(&mut self) -> &mut Self;
            }

            impl AddNetCommands for bevy::prelude::AppBuilder {
                fn net_events(&mut self) -> &mut Self {
                    use bevy::prelude::*;
                    self
                        .add_event::<NetData>()
                        .add_system(recieve_message_system.system().label("net_events"))
                    $(
                        .add_event::<net_events::NetEvent<net_events::[<$name:camel>]>>()
                        .add_system(net_events::$name.system().after("net_events"))
                    ) *
                }
            }

            pub struct NetData {
                data: Vec<u8>,
                from: usize,
            }



            fn recieve_message_system(
                mut event_reader: bevy::prelude::EventReader<NetData>,
                $(mut [< $name _writer >]: bevy::prelude::EventWriter<net_events::NetEvent<net_events::[<$name:camel>]>>), *
            ) {
                for message in event_reader.iter() {
                    if let Ok(deserialized) = serde_cbor::from_slice::<net_events::NetMessage>(&message.data) {
                        for command in deserialized.0 {
                            match command {
                                $(net_events::Command::[< $name:camel >](data) => {
                                    [< $name _writer >].send(net_events::NetEvent::new(data, message.from));
                                }), *
                            }
                        }
                    }
                }
            }
        }

    }
}
