use std::ops::Index;

use crate::count_idents;

// Takes a parameter pack of Full name as identifier, short version of name, bool if stat is increased with level-up
macro_rules! base_stats {
    ($($name:ident $short:literal $can_level:literal), *,) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub enum BaseStat {
            $($name), *
        }
        const NUM_BASE_STATS: usize = count_idents!($($name), *);
        const NAMES: [&'static str; NUM_BASE_STATS] = [$(stringify!($name)), *];
        const SHORTS: [&'static str; NUM_BASE_STATS] = [$($short), *];
        const CAN_LEVEL: [bool; NUM_BASE_STATS] = [$($can_level), *];
        impl BaseStat {
            pub fn get_name(&self) -> &str {
                NAMES[*self as usize]
            }
            pub fn get_short(&self) -> &str {
                SHORTS[*self as usize]
            }
            pub fn can_level(&self) -> bool {
                CAN_LEVEL[*self as usize]
            }
        }
    }
}

base_stats! {
    Hearing "hear" false,
    Smell "smell" false,
    Vision "vis" false,
    
    Strength "str" true,
    Dexterity "dex" true,
    Intelligence "int" true,
    Vitality "vit" true,
    Defence "def" true,
    Sense "sen" true,

    Wisdom "wis" false,
    Luck "luc" false,
    Karma "kar" false,
    Weight "wei" false,
    
    Fire "fire" false,
    Ice "ice" false,
    Wind "wind" false,
    Electric "ele" false,
    Earth "earth" false,
}

pub struct BaseStats {
    data: [f32; NUM_BASE_STATS],
}

impl Index<BaseStat> for BaseStats {
    type Output = f32;

    fn index(&self, index: BaseStat) -> &Self::Output {
        &self.data[index as usize]
    }
}