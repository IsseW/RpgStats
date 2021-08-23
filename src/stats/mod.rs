mod stats;
mod base_stat;
mod resource;
mod effect;
mod damage;

use crate::stats::damage::DmgResult;
use std::{collections::HashMap, ops::Index};

use rand::{Rng, thread_rng};
pub use stats::{Stat};
pub use base_stat::{BaseStat, BaseStats};
pub use resource::{Resource, ResourceConsumption, ConsumptionType};
pub use crate::stats::damage::{Dmg, DmgType};

use crate::dmg;

pub struct StatGain {
    base_stat_add: Vec<(BaseStat, f32)>,
    base_stat_mul: Vec<(BaseStat, f32)>,
    stat_add: Vec<(Stat, f32)>,
    stat_mul: Vec<(Stat, f32)>,
}

pub struct Stats {
    base_stats_uncalculated: BaseStats,
    base_stats_mul: BaseStats,
    base_stats: BaseStats,

    stats_uncalculated: HashMap<Stat, f32>,
    stats_add: HashMap<Stat, f32>,
    stats_mul: HashMap<Stat, f32>,
    stats: HashMap<Stat, f32>,

    resources: HashMap<Resource, f32>,
}

pub trait StatAccessor {
    fn get_value(&self, stats: &Stats) -> f32;
    fn get_base_value(&self, stats: &Stats) -> f32;
}

impl Stats {
    fn new(base: BaseStats) -> Self {
        let mut t = Self {
            base_stats_uncalculated: base,
            base_stats_mul: BaseStats::ones(),
            base_stats: Default::default(),
            stats_uncalculated: Default::default(),
            stats_add: Default::default(),
            stats_mul: Default::default(),
            stats: Default::default(),
            resources: Default::default(),
        };
        for stat in base_stat::BASE_STAT_ITER {
            t.recalculate_base_stat(stat);
        }
        t
    }

    pub fn set_stat(&mut self, stat: Stat, value: f32) {
        if self.stats_uncalculated.insert(stat, value) != Some(value) { 
            self.recalculate_stat(stat, value); 
        }
    }

    pub fn mul_stat(&mut self, stat: Stat, value: f32) {
        if value == 0.0 { panic!() }
        if value == 1.0 { return; }
        self.stats_add.insert(stat, self.stats_mul.get(&stat).unwrap_or(&1.0) * value);
        if self.stats_add[&stat] == 1.0 {
            self.stats.remove(&stat);
        }
        self.recalculate_stat(stat, *self.stats.get(&stat).unwrap_or(&0.0));
    }

    pub fn add_stat(&mut self, stat: Stat, value: f32) {
        if value == 0.0 { return; }
        self.stats_add.insert(stat, self.stats_add.get(&stat).unwrap_or(&0.0) + value);
        if self.stats_add[&stat] == 0.0 { 
            self.stats.remove(&stat);
        }
        self.recalculate_stat(stat, *self.stats.get(&stat).unwrap_or(&0.0));
    }

    fn recalculate_stat(&mut self, stat: Stat, current: f32) {
        self.stats.insert(stat, (current + self.stats_add.get(&stat).unwrap_or(&0.0)) 
                                    * self.stats_mul.get(&stat).unwrap_or(&1.0));
        stat.on_updated(self);
    }

    pub fn add_base(&mut self, stat: BaseStat, value: f32) {
        if value == 0.0 { return; }
        self.base_stats_uncalculated[stat] += value;
        self.recalculate_base_stat(stat);
    }

    pub fn mul_base(&mut self, stat: BaseStat, value: f32) {
        if value == 0.0 { panic!() }
        if value == 1.0 { return; }
        self.base_stats_mul[stat] *= value;
        self.recalculate_base_stat(stat);
    }

    pub fn recalculate_base_stat(&mut self, stat: BaseStat) {
        self.base_stats[stat] = self.base_stats_uncalculated[stat] * self.base_stats_mul[stat];
        stats::base_stat_changed(self, stat);
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

    pub fn consume_resource(&mut self, consumption: ResourceConsumption) -> bool {
        if let Some(val) = self.resources.get_mut(&consumption.resource) {
            match consumption.ty {
                ConsumptionType::Flat(f) => {
                    if *val >= f {
                        *val -= f;
                        true
                    }
                    else {
                        false
                    }
                }
                ConsumptionType::Percent(f) => {
                    if *val > 0.0 { 
                        *val *= 1.0 - f;
                        true
                    }
                    else {
                        false
                    }
                }
            }
        }
        else {
            false
        }
    }

    pub fn get_base<T : StatAccessor>(&self, accessor: T) -> f32 {
        accessor.get_base_value(self)
    }

    pub fn apply_damage(&mut self, dmg : &Dmg) -> DmgResult {
        let dodge = if dmg.can_dodge() {
            let z = dmg.get_speed() - self[Stat::DodgeTime] + self[Stat::ReactionTime];
            (if dmg.get_speed() < self[Stat::DodgeTime] - self[Stat::ReactionTime] + 0.5 {
                (z - 1.0) / (z - 1.5)
            }
            else {
                1.0 - z / (z + 0.5)
            }) < thread_rng().gen::<f32>()
        }
        else {
            false
        };
        if dodge {
            DmgResult::Dodge
        }
        else {
            *self.resources.get_mut(&Resource::HP).unwrap() -= dmg.calculate_taken(self).sum();
            DmgResult::Hit
        }
    }

    pub fn add_gain(&mut self, stat_gain: &StatGain) {
        for (stat, value) in &stat_gain.base_stat_add {
            self.add_base(*stat, *value);
        }
        for (stat, value) in &stat_gain.base_stat_mul {
            self.mul_base(*stat, *value);
        }
        for (stat, value) in &stat_gain.stat_add {
            self.add_stat(*stat, *value);
        }
        for (stat, value) in &stat_gain.stat_mul {
            self.mul_stat(*stat, *value);
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
