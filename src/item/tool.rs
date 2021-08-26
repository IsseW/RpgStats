
use crate::bitmap;
use crate::defs;

use crate::serializable;
use serde::{Serialize, Deserialize};
use crate::count_idents;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolPart {
    Handle,
    Head,
    Hilt,
    Bow,
    String,
}

serializable! {
    pub struct ToolProficiencies {
        mining: f32,
        wood_cutting: f32,
        
        armor_piercing: f32,
        cutting: f32,
        blunt_force: f32,
    
        blocking: f32,
    
        bow_proficiency: f32,
    }

    pub struct ToolProfeciency {
        damage_add: f32,
        damage_mul: f32,
        durability_add: f32,
        durability_mul: f32,
        speed_add: f32,
        speed_mul: f32,
    }
}

pub struct Tool {
    id : defs::Tool,
    materials: Vec<defs::Material>,
    damage: f32,
    max_durability: f32,
}
