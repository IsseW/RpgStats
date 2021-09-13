use super::person::Person;

pub struct WorldOptions {
    pub size: usize,
    pub num_settlements: usize,
    pub seed: i32,
}

impl Default for WorldOptions {
    fn default() -> Self {
        Self {
            size: 250,
            num_settlements: 20,
            seed: 6969,
        }
    }
}

#[derive(Default, Copy, Clone)]
struct Region {
    height: f32,
    temperature: f32,
    humidity: f32,
    magic: f32,
}

struct World {
    population: Vec<Person>,

    size: usize,
    regions: Vec<Region>,
}

impl World {
    fn generate(options: WorldOptions) -> World {
        let mut world = World {
            size: options.size,
            population: Vec::with_capacity(options.num_settlements * 15),
            regions: vec![Default::default(); options.size * options.size],
        };

        world
    }
}
