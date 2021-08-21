use crate::stats::{Stat, BaseStat, Stats};

struct EffectData {
    strength: f32,
    time: f32,
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
                    $($name => stringify!($name)), *
                }
            }

            fn start(&self, stats: &mut Stats, data: &mut EffectData) {
                match self {
                    $($($name => $start(stats, data),)?) *
                    _ => {}
                }
            }
            fn update(&self, stats: &mut Stats, data: &mut EffectData) {
                match self {
                    $($($name => $update(stats, data),)?) *
                    _ => {}
                }
            }
            fn end(&self, stats: &mut Stats, data: &mut EffectData) {
                match self {
                    $($($name => $end(stats, data),)?) *
                    _ => {}
                }
            }

        }
    }
}


effects! { 
    Fire,
    update => |stats, data| {
        
    }
    Poison

}