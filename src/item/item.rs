use crate::defs;
use super::*;

pub enum ItemType {
    CreatureDrop { creature: usize },

    Material { material: defs::Material },

    Tool { durability: f32, item_level: usize, tool: Tool },
}

pub struct ItemStack {
    ty: ItemType,
    count: usize,
}