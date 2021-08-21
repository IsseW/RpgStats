use crate::count_idents;
use crate::stats::{Stats, BaseStat};
use paste::paste;


macro_rules! function_content {
    ($stats:ident, $name:ident, $base:ident, $($base_stat:ident), *) => {
        $(if BaseStat::$base_stat == BaseStat::$base { $stats.update_stat(Stat::$name) }) *
    }
}

macro_rules! functions {
    ($name:ident, $($base:ident [$($base_stat:ident), *]), *) => {
        paste! {
            $(
            fn [< $name:lower _on_base_ $base:lower _changed >](stats: &mut Stats) {
                function_content!(stats, $name, $base, $($base_stat), *);
            }
        ) *
    }
    }
}

macro_rules! stat_check {
    ($m:ident, $($name:ident) *, $($code:expr); *) => {
        match $m {
            $(ty @ Stat::$name => {
                $code(ty);
            }), *
        }
    };
    ($m:ident, $stats:ident, $($name1:ident) *, $($name2:ident [$($name3: ident), *]) *) => {
        stat_check!($m, $($name1) *, $(|ty| { $(if ty == Stat::$name3 { Stat::$name2.update($stats) } ), *}); *)
    }
}

macro_rules! stats {
    ($($name:ident: $($base_stat:ident), *: $($stat:ident), *: $calculate:expr), *,) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub enum Stat {
            $($name), *
        }

        const NUM_STATS: usize = count_idents!($($name), *);

        paste! {
            const NAMES: [&'static str; NUM_STATS] = [$(stringify!([<$name:lower>])), *];

            $(
                functions!($name, Hearing [$($base_stat), *], Smell [$($base_stat), *], Vision [$($base_stat), *],
                 Strength [$($base_stat), *], Dexterity [$($base_stat), *], Intelligence [$($base_stat), *], Vitality [$($base_stat), *], Defence [$($base_stat), *], 
                 Sense [$($base_stat), *], Wisdom [$($base_stat), *], Luck [$($base_stat), *], Karma [$($base_stat), *], Weight [$($base_stat), *], 
                 Fire [$($base_stat), *], Ice [$($base_stat), *], Wind [$($base_stat), *], Electric [$($base_stat), *], Earth [$($base_stat), *]);
            ) *

            pub fn base_stat_changed(stats: &mut Stats, changed: BaseStat) {
                match changed {
                    BaseStat::Hearing => {
                        $([< $name:lower _on_base_ hearing _changed >](stats)); *
                    }
                    BaseStat::Smell => {
                        $([< $name:lower _on_base_ smell _changed >](stats)); *
                    }
                    BaseStat::Vision => {
                        $([< $name:lower _on_base_ vision _changed >](stats)); *
                    }
                    BaseStat::Strength => {
                        $([< $name:lower _on_base_ strength _changed >](stats)); *
                    }
                    BaseStat::Dexterity => {
                        $([< $name:lower _on_base_ dexterity _changed >](stats)); *
                    }
                    BaseStat::Intelligence => {
                        $([< $name:lower _on_base_ intelligence _changed >](stats)); *
                    }
                    BaseStat::Vitality => {
                        $([< $name:lower _on_base_ vitality _changed >](stats)); *
                    }
                    BaseStat::Defence => {
                        $([< $name:lower _on_base_ defence _changed >](stats)); *
                    }
                    BaseStat::Sense => {
                        $([< $name:lower _on_base_ sense _changed >](stats)); *
                    }
                    BaseStat::Wisdom => {
                        $([< $name:lower _on_base_ wisdom _changed >](stats)); *
                    }
                    BaseStat::Luck => {
                        $([< $name:lower _on_base_ luck _changed >](stats)); *
                    }
                    BaseStat::Karma => {
                        $([< $name:lower _on_base_ karma _changed >](stats)); *
                    }
                    BaseStat::Weight => {
                        $([< $name:lower _on_base_ weight _changed >](stats)); *
                    }
                    BaseStat::Fire => {
                        $([< $name:lower _on_base_ fire _changed >](stats)); *
                    }
                    BaseStat::Ice => {
                        $([< $name:lower _on_base_ fire _changed >](stats)); *
                    }
                    BaseStat::Wind => {
                        $([< $name:lower _on_base_ fire _changed >](stats)); *
                    }
                    BaseStat::Electric => {
                        $([< $name:lower _on_base_ fire _changed >](stats)); *
                    }
                    BaseStat::Earth => {
                        $([< $name:lower _on_base_ fire _changed >](stats)); *
                    }
                }
            }
        
        
            fn stat_changed(stats: &mut Stats, changed: Stat) {
                stat_check!(changed, stats, $($name) *, $($name [$($stat) *]) *)
            }
        }

        impl Stat {
            pub fn update(&self, stats: &mut Stats) {
                let t = match self {
                    $(Stat::$name => $calculate(stats)), *
                };
                stats.set_stat(*self, t);
            }

            pub fn on_updated(&self, stats: &mut Stats) {
                stat_changed(stats, *self);
            }
        }

    }
}

stats! {

    // Senses
    Hearing: Hearing, Sense: : |stats: &mut Stats| {
        stats[BaseStat::Hearing] * stats[BaseStat::Sense]
    },
    Smell: Smell, Sense: : |stats: &mut Stats| {
        stats[BaseStat::Smell] * stats[BaseStat::Sense]
    },
    Vision: Vision, Sense: : |stats: &mut Stats| {
        stats[BaseStat::Vision] * stats[BaseStat::Sense]
    },

    // Movement
    Speed: Dexterity, Strength: : |stats: &mut Stats| {
        stats[BaseStat::Dexterity] * 7.0 + stats[BaseStat::Strength] * 3.0
    },
    JumpHeight: Dexterity, Strength: : |stats: &mut Stats| {
        stats[BaseStat::Dexterity] * 7.0 + stats[BaseStat::Strength] * 3.0
    },

    // Defence
    PhysicalArmor: Defence, Strength: : |stats: &mut Stats| {
        stats[BaseStat::Defence] * 7.0 + stats[BaseStat::Strength] * 3.0
    },
    CuttingArmor: Defence: : |stats: &mut Stats| {
        stats[BaseStat::Defence] * 5.0
    },
    MagicalArmor: Defence, Intelligence: : |stats: &mut Stats| {
        stats[BaseStat::Defence] * 7.0 + stats[BaseStat::Intelligence] * 3.0
    },
    MentalArmor: Wisdom: : |stats: &mut Stats| {
        stats[BaseStat::Wisdom]
    },
    CurseArmor: Karma: : |stats: &mut Stats| {
        -stats[BaseStat::Karma]
    },
    HolyArmor: Karma: : |stats: &mut Stats| {
        stats[BaseStat::Karma]
    },
    FireArmor: Fire: : |stats: &mut Stats| {
        stats[BaseStat::Fire] * 5.0
    },
    IceArmor: Ice: : |stats: &mut Stats| {
        stats[BaseStat::Ice] * 5.0
    },
    WindArmor: Wind: : |stats: &mut Stats| {
        stats[BaseStat::Wind] * 5.0
    },
    ElectricArmor: Electric: : |stats: &mut Stats| {
        stats[BaseStat::Electric] * 5.0
    },
    EarthArmor: Earth: : |stats: &mut Stats| {
        stats[BaseStat::Earth] * 5.0
    },

    
    PhysicalReduction: : PhysicalArmor: |stats: &mut Stats| {
        2.0
    },


    // Resources
    MaxHealth: Vitality, Strength: : |stats: &mut Stats| {
        0.0f32.max(stats[BaseStat::Vitality] * 7.0 + stats[BaseStat::Strength] * 3.0)
    },
    HealthRegen: Vitality, Strength: : |stats: &mut Stats| {
        stats[BaseStat::Vitality] * 7.0 + stats[BaseStat::Strength] * 3.0
    },
    MaxMana: Wisdom, Intelligence: : |stats: &mut Stats| {
        0.0f32.max(stats[BaseStat::Wisdom] * 7.0 + (stats[BaseStat::Intelligence] - 20.0) * 3.0)
    },
    ManaRegen: Wisdom, Intelligence: : |stats: &mut Stats| {
        stats[BaseStat::Intelligence] * 7.0 + stats[BaseStat::Wisdom] * 3.0
    },
}