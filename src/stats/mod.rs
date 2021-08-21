mod stats;
mod base_stat;
mod resource;
mod effect;
mod damage;

use std::{collections::HashMap, ops::Index};

pub use stats::{Stat};
pub use base_stat::{BaseStat, BaseStats};
pub use resource::{Resource};

pub struct Stats {
    base_stats_uncalculated: BaseStats,
    base_stats: BaseStats,
    stats_uncalculated: HashMap<Stat, f32>,
    stats: HashMap<Stat, f32>,

    resources: HashMap<Resource, f32>,
}

impl Stats {
    pub fn set_stat(&mut self, stat: Stat, value: f32) {
        self.stats_uncalculated.insert(stat, value);
        
        self.recalculate_stat(stat, value);
    }

    fn recalculate_stat(&mut self, stat: Stat, current: f32) {
        self.stats.insert(stat, current);
        stat.on_updated(self);
    }

    pub fn update_stat(&mut self, stat: Stat) {
        stat.update(self);
    }

    pub fn update_resources(&mut self, delta: f32) {
        let t: Vec<_> = self.resources.iter().map(|(&res, &val)| {
            (res, (val + res.regen(self) * delta).clamp(0.0, res.max(self)))
        }).collect();
        self.resources.extend(t);
    }

    pub fn add_resource(&mut self, resource: Resource, max: bool) {
        if !self.resources.contains_key(&resource) {
            self.resources.insert(resource, if max { resource.max(self) } else { 0.0 });
        }
    }
}

impl Index<BaseStat> for Stats {
    type Output = f32;

    fn index(&self, index: BaseStat) -> &Self::Output {
        &self.base_stats[index]
    }
}

impl Index<Stat> for Stats {
    type Output = f32;

    fn index(&self, index: Stat) -> &Self::Output {
        self.stats.get(&index).unwrap_or(&0.0)
    }
}
impl Index<Resource> for Stats {
    type Output = f32;

    fn index(&self, index: Resource) -> &Self::Output {
        self.resources.get(&index).unwrap_or(&0.0)
    }
}
