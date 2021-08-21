

use crate::stats::Stats;
use std::collections::HashMap;

macro_rules! damages {
    ($($name:ident, dmg => $dmg_calc:expr, red => $red_calc:expr), *,) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub enum DmgType {
            $($name), *
        }

        pub struct Dmg {
            dmgs: HashMap<DmgType, f32>,
        }

        impl Dmg {
            pub fn new(dmgs: HashMap<DmgType, f32>) -> Self {
                Self {
                    dmgs: dmgs,
                }
            }

            pub fn calculate_dealt(&self, stats: &Stats) -> Self {
                Self::new(self.dmgs.iter().map(|(ty, v)| {
                    (*ty, match ty {
                        $(DmgType::$name => $dmg_calc(*v)), *
                    })
                }).collect())
            }

            pub fn calculate_taken(&self, stats: &Stats) -> Self {
                Self::new(self.dmgs.iter().map(|(ty, v)| {
                    (*ty, match ty {
                        $(DmgType::$name => $red_calc(*v)), *
                    })
                }).collect())
            }
        }
    }
}

damages! {
    Physical,
    dmg => |val: f32| {
        val / 2.0
    },
    red => |val: f32| {
        val / 2.0
    },

    // Cutting,
    // Mental,
    // Magic,
    // Curse,
    // Holy,
    // Fire,
    // Ice,
    // Wind,
    // Electric,
    // Earth,
    // Pure,
}
