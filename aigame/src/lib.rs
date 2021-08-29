pub mod creature;
pub mod defs;
pub mod item;
pub mod macro_help;
pub mod stats;

pub mod network;

pub trait Printer {
    fn error(&self, string: String);
    fn warning(&self, string: String);
    fn info(&self, string: String);
}
