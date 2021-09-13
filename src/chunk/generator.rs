use bevy::{prelude::*, tasks::Task};

use super::{
    chunk::{chunk_size, CHUNK_SIZE},
    ordered_float::OrderedFloat,
};

pub struct Generator {
    pub load_distance: i32,
    pub unload_distance: i32,
    pub load_order: Vec<IVec3>,
    pub load_order_half: Vec<IVec3>,
    last_forward: IVec3,
    last_translation: IVec3,
}

impl Generator {
    pub fn new(load_distance: i32, unload_distance: i32) -> Self {
        let mut load_order = Vec::with_capacity(
            (4. * std::f32::consts::FRAC_PI_3 * load_distance.pow(3) as f32) as usize,
        );
        let mut load_order_half = Vec::with_capacity(
            (4. * std::f32::consts::FRAC_PI_3 * (load_distance / 2).pow(3) as f32) as usize,
        );
        let squared = load_distance * load_distance;
        for y in 0..=load_distance {
            for z in 0..=load_distance {
                for x in 0..=load_distance {
                    let distance = x * x + y * y + z * z;
                    if distance < squared {
                        load_order.push(IVec3::new(x, y, z));
                        load_order.push(IVec3::new(x, y, -z));
                        load_order.push(IVec3::new(x, -y, z));
                        load_order.push(IVec3::new(x, -y, -z));
                        load_order.push(IVec3::new(-x, y, z));
                        load_order.push(IVec3::new(-x, y, -z));
                        load_order.push(IVec3::new(-x, -y, z));
                        load_order.push(IVec3::new(-x, -y, -z));
                        if distance < squared / 4 {
                            load_order_half.push(IVec3::new(x, y, z));
                            load_order_half.push(IVec3::new(x, y, -z));
                            load_order_half.push(IVec3::new(x, -y, z));
                            load_order_half.push(IVec3::new(x, -y, -z));
                            load_order_half.push(IVec3::new(-x, y, z));
                            load_order_half.push(IVec3::new(-x, y, -z));
                            load_order_half.push(IVec3::new(-x, -y, z));
                            load_order_half.push(IVec3::new(-x, -y, -z));
                        }
                    }
                }
            }
        }

        Self {
            load_distance,
            unload_distance,
            load_order,
            load_order_half,
            last_forward: IVec3::ZERO,
            last_translation: IVec3::ZERO,
        }
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::new(5, 7)
    }
}

pub struct GeneratorOptions {
    pub concurrent_generate_tasks: usize,
    pub concurrent_meshing_tasks: usize,
}

#[derive(Default)]
pub struct GeneratorData<const DEPTH: u32> {
    pub position: IVec3,

    pub gen_index: usize,
    pub current_gen_task_count: usize,

    pub mesh_index: usize,
    pub current_mesh_task_count: usize,
}

seq_macro::seq!(N in 0..=13 {
    #[derive(Bundle, Default)]
    pub struct GeneratorBundle {
        generator: Generator,
        #(generator_data#N: GeneratorData<N>,)*
    }
});

impl GeneratorBundle {
    pub fn new(load_distance: i32, unload_distance: i32) -> Self {
        Self {
            generator: Generator::new(load_distance, unload_distance),
            ..Default::default()
        }
    }
}

seq_macro::seq!(N in 0..=13 {
    pub fn update_generators(
        mut generators: Query<(&GlobalTransform, &mut Generator, (#(&mut GeneratorData<N>, )*)), Changed<Transform>>,
    ) {
        for (transform, mut generator, mut sub) in generators.iter_mut() {
            let translation_changed = (transform.translation / CHUNK_SIZE as f32).as_i32() != generator.last_translation;
            if translation_changed {
                generator.last_translation = (transform.translation / CHUNK_SIZE as f32).as_i32();
                #(
                    let new = (transform.translation / chunk_size(N) as f32).as_i32();
                    if new != sub.N.position {
                        sub.N.position = new;
                        sub.N.gen_index = 0;
                        sub.N.mesh_index = 0;
                    }
                )*
            }
            let forward = transform.rotation * Vec3::Z;
            let rotation_changed = (forward * 100.).as_i32() != generator.last_forward;
            if rotation_changed {
                generator.load_order.sort_by_cached_key(|v| {
                    OrderedFloat(forward.dot(v.as_f32()) / (1. + v.as_f32().length_squared()))
                });
                #(
                    sub.N.gen_index = 0;
                    sub.N.mesh_index = 0;
                )*
                generator.last_forward = (forward * 100.).as_i32();
            }
        }
    }
});

pub struct GeneratorTask<T> {
    pub sender: Entity,
    pub task: Task<T>,
}
