use crate::stats::stat::distribution;
use crate::stats::{Stat, Stats};
use crate::dmg;

struct EffectData {
    strength: f32,
    data: f32,
}

macro_rules! effects {
    ($($name:ident $(, start => $start:expr)? $(, update => $update:expr)? $(, end => $end:expr)?) *) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        enum Effect {
            $($name), *
        }

        impl Effect {
            fn name(&self) -> &'static str {
                match self {
                    $(Self::$name => stringify!($name)), *
                }
            }

            fn start(&self, stats: &mut Stats, data: &mut EffectData) {
                match self {
                    $($(Self::$name => { $start(stats, data);},)?) *
                    _ => {}
                }
            }
            fn update(&self, stats: &mut Stats, data: &mut EffectData, delta: f32) {
                match self {
                    $($(Self::$name => { $update(stats, data, delta);},)?) *
                    _ => {}
                }
            }
            fn end(&self, stats: &mut Stats, data: &mut EffectData) {
                match self {
                    $($(Self::$name => { $end(stats, data);},)?) *
                    _ => {}
                }
            }

        }
    }
}


effects! { 
    Fire,
    update => |stats: &mut Stats, data: &mut EffectData, delta: f32| {
        stats.apply_damage(&dmg!(Fire: data.strength * delta));
    }
    Poison,
    update => |stats: &mut Stats, data: &mut EffectData, delta: f32| {
        stats.apply_damage(&dmg!(Curse: data.strength * delta));
    }
    Slow,
    start => |stats: &mut Stats, data: &mut EffectData| {
        stats.mul_stat(Stat::Speed, distribution(data.strength, 4.0));
    },
    end => |stats: &mut Stats, data: &mut EffectData| {
        stats.mul_stat(Stat::Speed, 1.0 / distribution(data.strength, 4.0));
    }
}