use rand::Rng;
use rand::thread_rng;
use std::collections::HashMap;

use crate::stats::Stats;
use crate::stats::Stat;


macro_rules! damages {
    ($($name:ident, dmg => $dmg_calc:expr, red => $red_calc:expr), *,) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub enum DmgType {
            $($name), *
        }

        pub struct Dmg {
            dmgs: HashMap<DmgType, f32>,
            dodgeable: bool, 
            speed: f32,
        }

        impl Dmg {
            pub fn create(dmgs: HashMap<DmgType, f32>) -> Self {
                Self {
                    dmgs,
                    dodgeable: false,
                    speed: 0.0,
                }
            }
            pub fn new(dmgs: HashMap<DmgType, f32>, dodgeable: bool, speed: f32) -> Self {
                Self {
                    dmgs,
                    dodgeable,
                    speed,
                }
            }

            pub fn calculate_dealt(&self, stats: &Stats) -> Self {
                Self::new(self.dmgs.iter().map(|(ty, v)| {
                    (*ty, match ty {
                        $(DmgType::$name => $dmg_calc(stats, *v)), *
                    })
                }).collect(), self.dodgeable, self.speed)
            }

            pub fn calculate_taken(&self, stats: &Stats) -> Self {
                Self::new(self.dmgs.iter().map(|(ty, v)| {
                    (*ty, match ty {
                        $(DmgType::$name => $red_calc(stats, *v)), *
                    })
                }).collect(), self.dodgeable, self.speed)
            }

            pub fn can_dodge(&self) -> bool {
                self.dodgeable
            }
            pub fn get_speed(&self) -> f32 {
                self.speed
            }

            pub fn sum(&self) -> f32 { 
                let mut sum = 0.0;
                for (_, &val) in &self.dmgs {
                    sum += val;
                }
                sum
            }
        }
    }
}


damages! {
    Physical,
    dmg => |stats: &Stats, val: f32| {
        val * stats[Stat::PhysicalDamage] * if thread_rng().gen::<f32>() < stats[Stat::PhysicalCritChance] { 2.0 } else { 1.0 } // Maybe have a crit damage stat?
    },
    red => |stats: &Stats, val: f32| {
        (val - stats[Stat::PhysicalReductionFlat]) * stats[Stat::PhysicalReduction]
    },
    Cutting,
    dmg => |stats: &Stats, val: f32| {
        val * stats[Stat::CuttingDamage] * if thread_rng().gen::<f32>() < stats[Stat::PhysicalCritChance] { 2.0 } else { 1.0 } // Maybe have a crit damage stat?
    },
    red => |stats: &Stats, val: f32| {
        (val - stats[Stat::CuttingReductionFlat]) * stats[Stat::CuttingReduction]
    },
    Magic,
    dmg => |stats: &Stats, val: f32| {
        val * stats[Stat::MagicalDamage] * if thread_rng().gen::<f32>() < stats[Stat::MagicalCritChance] { 2.0 } else { 1.0 } // Maybe have a crit damage stat?
    },
    red => |stats: &Stats, val: f32| {
        (val - stats[Stat::MagicalReductionFlat]) * stats[Stat::MagicalReduction]
    },
    Mental,
    dmg => |stats: &Stats, val: f32| {
        val * stats[Stat::MentalDamage]
    },
    red => |stats: &Stats, val: f32| {
        (val - stats[Stat::MentalReductionFlat]) * stats[Stat::MentalReduction]
    },
    Curse,
    dmg => |stats: &Stats, val: f32| {
        val * stats[Stat::CurseDamage] * if thread_rng().gen::<f32>() < stats[Stat::MagicalCritChance] { 2.0 } else { 1.0 } // Maybe have a crit damage stat?
    },
    red => |stats: &Stats, val: f32| {
        (val - stats[Stat::CurseReductionFlat]) * stats[Stat::CurseReduction]
    },
    Holy,
    dmg => |stats: &Stats, val: f32| {
        val * stats[Stat::HolyDamage] * if thread_rng().gen::<f32>() < stats[Stat::MagicalCritChance] { 2.0 } else { 1.0 } // Maybe have a crit damage stat?
    },
    red => |stats: &Stats, val: f32| {
        (val - stats[Stat::HolyReductionFlat]) * stats[Stat::HolyReduction]
    },
    Fire,
    dmg => |stats: &Stats, val: f32| {
        val * stats[Stat::FireDamage] * if thread_rng().gen::<f32>() < stats[Stat::ElementalCritChance] { 2.0 } else { 1.0 } // Maybe have a crit damage stat?
    },
    red => |stats: &Stats, val: f32| {
        (val - stats[Stat::FireReductionFlat]) * stats[Stat::FireReduction]
    },
    Ice,
    dmg => |stats: &Stats, val: f32| {
        val * stats[Stat::IceDamage] * if thread_rng().gen::<f32>() < stats[Stat::ElementalCritChance] { 2.0 } else { 1.0 } // Maybe have a crit damage stat?
    },
    red => |stats: &Stats, val: f32| {
        (val - stats[Stat::IceReductionFlat]) * stats[Stat::IceReduction]
    },
    Wind,
    dmg => |stats: &Stats, val: f32| {
        val * stats[Stat::WindDamage] * if thread_rng().gen::<f32>() < stats[Stat::ElementalCritChance] { 2.0 } else { 1.0 } // Maybe have a crit damage stat?
    },
    red => |stats: &Stats, val: f32| {
        (val - stats[Stat::WindReductionFlat]) * stats[Stat::WindReduction]
    },
    Electric,
    dmg => |stats: &Stats, val: f32| {
        val * stats[Stat::ElectricDamage] * if thread_rng().gen::<f32>() < stats[Stat::ElementalCritChance] { 2.0 } else { 1.0 } // Maybe have a crit damage stat?
    },
    red => |stats: &Stats, val: f32| {
        (val - stats[Stat::ElectricReductionFlat]) * stats[Stat::ElectricReduction]
    },
    Earth,
    dmg => |stats: &Stats, val: f32| {
        val * stats[Stat::EarthDamage] * if thread_rng().gen::<f32>() < stats[Stat::ElementalCritChance] { 2.0 } else { 1.0 } // Maybe have a crit damage stat?
    },
    red => |stats: &Stats, val: f32| {
        (val - stats[Stat::EarthReductionFlat]) * stats[Stat::EarthReduction]
    },
    Pure,
    dmg => |_: &Stats, val: f32| {
        val
    },
    red => |_: &Stats, val: f32| {
        val
    },
}

pub enum DmgResult {
    Hit,
    Dodge,
    Miss,
}

#[macro_export]
macro_rules! dmg {
    ($($type:ident: $value:expr), *) => {
        crate::stats::damage::Dmg::create([$((crate::stats::damage::DmgType::$type, $value)), *].iter().cloned().collect())
    };
    ($($type:ident: $value:expr), *; $speed:expr) => {
        crate::stats::damage::Dmg::new([$((crate::stats::damage::DmgType::$type, $value)), *].iter().cloned().collect(), true, $speed)
    };
    ([$map:expr]) => {
        crate::stats::damage::Dmg::create($map)
    };
    ([$map:expr], $speed:expr) => {
        crate::stats::damage::Dmg::new($map, true, $speed)
    };
}