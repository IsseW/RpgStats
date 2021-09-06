use bevy::prelude::Color;

use crate::defs::*;

struct Date {
    year: u32,
    month: u8,
    day: u8,
}

enum Shape {
    Narrow,
    Thick,
    Pointy,
    Large,
}

enum GeneType {
    Color(Color),
    Size(f32),
    Shape(Shape),
}

struct Gene {
    affected: BodyPart,
    ty: GeneType,
}

pub struct Person {
    birthday: Date,
    death_date: Option<Date>,

    first_name: String,
    last_name: String,

    species: Species,

    genes: Vec<Gene>,
}

impl Person {
    pub fn is_alive(&self) -> bool {
        self.death_date.is_none()
    }
}
