use crate::stats::{Stat, BaseStat, Stats};
use crate::count_idents;

macro_rules! resources {
    ($($name:ident, max => $max:expr, regen => $regen:expr) *) => {

        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub enum Resource {
            $($name), *
        }
        const NUM_RESOURCES: usize = count_idents!($($name), *);
        const NAMES: [&'static str; NUM_RESOURCES] = [$(stringify!($name)), *];
        impl Resource {
            pub fn name(&self) -> &'static str {
                NAMES[*self as usize]
            }

            pub fn regen(&self, stats: &Stats) -> f32 {
                match self {
                    $(Resource::$name => $regen(stats)), *
                }
            }

            pub fn max(&self, stats: &Stats) -> f32 {
                match self {
                    $(Resource::$name => $max(stats)), *
                }
            }
        }
    }
}

resources! {
    HP,
    max => |stats: &Stats| {
        stats[Stat::MaxHealth]
    },
    regen => |stats: &Stats| {
        stats[Stat::HealthRegen]
    }
    Mana,
    max => |stats: &Stats| {
        stats[Stat::MaxMana]
    },
    regen => |stats: &Stats| {
        stats[Stat::ManaRegen]
    }
}

pub enum ConsumptionType {
    Percent(f32),
    Flat(f32),
}

pub struct ResourceConsumption {
    pub resource: Resource,
    pub ty: ConsumptionType,
}