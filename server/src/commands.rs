use aigame::net_commands;
use aigame::network::*;
use paste::paste;

net_commands! {
    fn message(data: (String, String), from: usize) {
        println!("{:?} from {}", data, from);
    }
}

pub use net_events::Command;