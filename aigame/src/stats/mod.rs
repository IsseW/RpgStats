mod stat;
mod base_stat;
mod resource;
mod effect;
mod damage;
mod stats;

pub use stat::{Stat};
pub use base_stat::{BaseStat, BaseStats};
pub use resource::{Resource, ResourceConsumption, ConsumptionType};
pub use damage::{Dmg, DmgType, DmgResult};
pub use stats::{StatGain, Stats, StatAccessor};