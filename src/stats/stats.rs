use crate::stats::StatAccessor;
use crate::count_idents;
use crate::stats::{BaseStat, Stats, Resource};
use paste::paste;

macro_rules! base_stat_check {
    ($m: ident, $($name:ident) *, $code:expr) => {
        match $m {
            $(ty @ BaseStat::$name => {
                $code(ty);
            }), *
        }
    };
    ($m: ident, $stats:ident, $($stat:ident, $($base_stat:ident) *); *, $($name:ident) *) => {
        base_stat_check!($m, $($name) *, |ty| { $($(if BaseStat::$base_stat == ty { $stats.update_stat(Stat::$stat) }) *) * })
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
        stat_check!($m, $($name1) *, $(|ty| { $(if ty == Stat::$name3 { $stats.update_stat(Stat::$name2) } ), *}); *)
    }
}

macro_rules! stats {
    ($([$($name:ident: $($base_stat:ident), *: $($stat:ident), *: $calculate:expr), *,]) *) => {
        stats!{$($($name: $($base_stat), *: $($stat), *: $calculate), *), *,}
    };
    ($($name:ident: $($base_stat:ident), *: $($stat:ident), *: $calculate:expr), *,) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub enum Stat {
            $($name), *
        }

        impl StatAccessor for Stat {
            
                fn get_value(&self, stats: &Stats) -> f32 { stats[*self] }
                fn get_base_value(&self, stats: &Stats) -> f32 { *stats.stats_uncalculated.get(self).unwrap_or(&0.0) }
        }
        const NUM_STATS: usize = count_idents!($($name), *);

        paste! {
            const NAMES: [&'static str; NUM_STATS] = [$(stringify!([<$name:lower>])), *];
        }

        pub fn base_stat_changed(stats: &mut Stats, changed: BaseStat) {
            base_stat_check!(changed, stats, $($name, $($base_stat) *); *,
            Hearing Smell Vision Strength Dexterity Intelligence Vitality Defence Sense Charisma Wisdom Luck Karma Weight Size Fire Ice Wind Electric Earth);
        }

        fn stat_changed(stats: &mut Stats, changed: Stat) {
            stat_check!(changed, stats, $($name) *, $($name [$($stat) *]) *)
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

pub fn distribution(x: f32, half: f32) -> f32 {
    1.0 - x / (x.abs() + half)
}

// Usage:
// NAME_OF_STAT: <Used base stats> : <Used stats>: |stats: &mut Stats| {
//      Code goes here...
// }
stats! {
    // Senses
    [
        Hearing: Hearing, Sense: : |stats: &mut Stats| {
            stats[BaseStat::Hearing] * stats[BaseStat::Sense]
        },
        Smell: Smell, Sense: : |stats: &mut Stats| {
            stats[BaseStat::Smell] * stats[BaseStat::Sense]
        },
        Vision: Vision, Sense: : |stats: &mut Stats| {
            stats[BaseStat::Vision] * stats[BaseStat::Sense]
        },

        ReactionTime: Sense: Vision: |stats: &mut Stats| {
            1.0 / (stats[BaseStat::Sense] * stats[Stat::Vision])
        },
    ]
    // Movement
    [
        Speed: Dexterity, Strength, Weight: : |stats: &mut Stats| {
            (stats[BaseStat::Dexterity] * 7.0 + stats[BaseStat::Strength] * 3.0) / stats[BaseStat::Weight].min(1.0)
        },
        JumpHeight: Dexterity, Strength, Weight: : |stats: &mut Stats| {
            (stats[BaseStat::Dexterity] * 5.0 + stats[BaseStat::Strength] * 5.0) / stats[BaseStat::Weight].min(1.0)
        },

        DodgeTime: : Speed: |stats: &mut Stats| {
            1.0 / stats[Stat::Speed]
        },
    ]
    // Damage
    [
        PhysicalCritChance: Dexterity, Luck: : |stats: &mut Stats| {
            1.0 - distribution(stats[BaseStat::Dexterity] * stats[BaseStat::Luck], 500.0).max(1.0)
        },
        MagicalCritChance: Wisdom, Luck: : |stats: &mut Stats| {
            1.0 - distribution(stats[BaseStat::Wisdom] * stats[BaseStat::Luck] * 0.1, 500.0).max(1.0)
        },
        ElementalCritChance: : : |_: &mut Stats| {
            0.0
        },


        PhysicalDamage: Strength: : |stats: &mut Stats| {
            1.0 + stats[BaseStat::Strength] * 0.01
        },
        CuttingDamage: Strength, Dexterity: : |stats: &mut Stats| {
            1.0 + stats[BaseStat::Strength] * 0.004 + stats[BaseStat::Dexterity] * 0.006
        },
        MagicalDamage: Intelligence: : |stats: &mut Stats| {
            1.0 + stats[BaseStat::Intelligence] * 0.012
        },
        MentalDamage: Charisma, Wisdom: : |stats: &mut Stats| {
            1.0 + stats[BaseStat::Charisma] * 0.007 + stats[BaseStat::Wisdom] * 0.005
        },
        CurseDamage: Intelligence, Karma: : |stats: &mut Stats| {
            1.0 + stats[BaseStat::Intelligence] * 0.0001 * 0.0f32.max(-stats[BaseStat::Karma])
        },
        HolyDamage: Intelligence, Karma: : |stats: &mut Stats| {
            1.0 + stats[BaseStat::Intelligence] * 0.0001 * 0.0f32.max(stats[BaseStat::Karma])
        },
        FireDamage: Fire: : |stats: &mut Stats| {
            1.0 + stats[BaseStat::Fire] * 0.01
        },
        IceDamage: Ice: : |stats: &mut Stats| {
            1.0 + stats[BaseStat::Ice] * 0.01
        },
        WindDamage: Wind: : |stats: &mut Stats| {
            1.0 + stats[BaseStat::Wind] * 0.01
        },
        ElectricDamage: Electric: : |stats: &mut Stats| {
            1.0 + stats[BaseStat::Electric] * 0.01
        },
        EarthDamage: Earth: : |stats: &mut Stats| {
            1.0 + stats[BaseStat::Earth] * 0.01
        },
    ]

    // Defence
    [
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
            -stats[BaseStat::Karma] * stats[BaseStat::Defence]
        },
        HolyArmor: Karma: : |stats: &mut Stats| {
            stats[BaseStat::Karma] * stats[BaseStat::Defence]
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
            distribution(stats[Stat::PhysicalArmor], 500.0)
        },
        PhysicalReductionFlat: : PhysicalArmor: |stats: &mut Stats| {
            stats[Stat::PhysicalArmor] * 0.1
        },
        CuttingReduction: : CuttingArmor: |stats: &mut Stats| {
            distribution(stats[Stat::CuttingArmor], 500.0)
        },
        CuttingReductionFlat: : CuttingArmor: |stats: &mut Stats| {
            stats[Stat::CuttingArmor] * 0.1
        },
        MagicalReduction: : MagicalArmor: |stats: &mut Stats| {
            distribution(stats[Stat::MagicalArmor], 500.0)
        },
        MagicalReductionFlat: : MagicalArmor: |stats: &mut Stats| {
            stats[Stat::MagicalArmor] * 0.1
        },
        MentalReduction: : MentalArmor: |stats: &mut Stats| {
            distribution(stats[Stat::MentalArmor], 500.0)
        },
        MentalReductionFlat: : MentalArmor: |stats: &mut Stats| {
            stats[Stat::MentalArmor] * 0.1
        },
        CurseReduction: : CurseArmor: |stats: &mut Stats| {
            distribution(stats[Stat::CurseArmor], 500.0)
        },
        CurseReductionFlat: : CurseArmor: |stats: &mut Stats| {
            stats[Stat::CurseArmor] * 0.1
        },
        HolyReduction: : HolyArmor: |stats: &mut Stats| {
            distribution(stats[Stat::HolyArmor], 500.0)
        },
        HolyReductionFlat: : HolyArmor: |stats: &mut Stats| {
            stats[Stat::HolyArmor] * 0.1
        },
        FireReduction: : FireArmor: |stats: &mut Stats| {
            distribution(stats[Stat::FireArmor], 500.0)
        },
        FireReductionFlat: : FireArmor: |stats: &mut Stats| {
            stats[Stat::FireArmor] * 0.1
        },
        IceReduction: : IceArmor: |stats: &mut Stats| {
            distribution(stats[Stat::IceArmor], 500.0)
        },
        IceReductionFlat: : IceArmor: |stats: &mut Stats| {
            stats[Stat::IceArmor] * 0.1
        },
        WindReduction: : WindArmor: |stats: &mut Stats| {
            distribution(stats[Stat::WindArmor], 500.0)
        },
        WindReductionFlat: : WindArmor: |stats: &mut Stats| {
            stats[Stat::WindArmor] * 0.1
        },
        ElectricReduction: : ElectricArmor: |stats: &mut Stats| {
            distribution(stats[Stat::ElectricArmor], 500.0)
        },
        ElectricReductionFlat: : ElectricArmor: |stats: &mut Stats| {
            stats[Stat::ElectricArmor] * 0.1
        },
        EarthReduction: : EarthArmor: |stats: &mut Stats| {
            distribution(stats[Stat::EarthArmor], 500.0)
        },
        EarthReductionFlat: : EarthArmor: |stats: &mut Stats| {
            stats[Stat::EarthArmor] * 0.1
        },
    ]
    // Resources
    [
        MaxHealth: Vitality, Strength: : |stats: &mut Stats| {
            let t = 0.0f32.max(stats[BaseStat::Vitality] * 7.0 + stats[BaseStat::Strength] * 3.0);
            if t > 0.0 { 
                stats.add_resource(Resource::HP, true);
            }
            t
        },
        HealthRegen: Vitality, Strength: : |stats: &mut Stats| {
            stats[BaseStat::Vitality] * 7.0 + stats[BaseStat::Strength] * 3.0
        },
        MaxMana: Wisdom, Intelligence: : |stats: &mut Stats| {
            let t = 0.0f32.max(stats[BaseStat::Wisdom] * 7.0 + (stats[BaseStat::Intelligence] - 20.0) * 3.0);
            if t > 0.0 { 
                stats.add_resource(Resource::Mana, true);
            }
            t
        },
        ManaRegen: Wisdom, Intelligence: : |stats: &mut Stats| {
            stats[BaseStat::Intelligence] * 7.0 + stats[BaseStat::Wisdom] * 3.0
        },
    ]
}
