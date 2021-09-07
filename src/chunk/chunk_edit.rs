use bevy::{
    prelude::*,
    tasks::{ComputeTaskPool, Task},
};

use super::{
    chunk::{ChunkData, Chunks},
    meshing::{ChunkMesh, MeshData},
    voxel::{Voxel, CHUNK_SIZE},
};

pub enum EditResult {
    All,
    Some(Vec<(u8, u8, u8)>),
}

pub trait ChunkEdit: Send + Sync + 'static {
    fn compute_edits(&mut self) {
        let chunks = self.get_affected_chunks();
        let mut result = Vec::with_capacity(chunks.len());
        for c in chunks {
            let chunk_pos = c.as_f32() * CHUNK_SIZE as f32;

            let mut changes = vec![];
            for y in 0..CHUNK_SIZE as u8 {
                for z in 0..CHUNK_SIZE as u8 {
                    for x in 0..CHUNK_SIZE as u8 {
                        let p = chunk_pos + Vec3::new(x as f32, y as f32, z as f32);
                        if self.conatins(p) {
                            changes.push((x, y, z));
                        }
                    }
                }
            }
            if changes.len() < CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE {
                result.push((c, EditResult::Some(changes)));
            } else {
                result.push((c, EditResult::All));
            }
        }
        self.set_result(result);
    }

    fn get_affected_chunks(&self) -> Vec<IVec3>;
    fn conatins(&self, pos: Vec3) -> bool;

    fn get_voxel(&self) -> &Voxel;

    fn set_result(&mut self, res: Vec<(IVec3, EditResult)>);

    fn get_result(&self) -> &Vec<(IVec3, EditResult)>;
}

pub struct SphereEdit {
    center: Vec3,
    radius: f32,
    voxel: Voxel,

    result: Vec<(IVec3, EditResult)>,
}

impl SphereEdit {
    pub fn new(center: Vec3, radius: f32, voxel: Voxel) -> Self {
        Self {
            center,
            radius,
            voxel,
            result: Default::default(),
        }
    }
}

impl ChunkEdit for SphereEdit {
    fn get_affected_chunks(&self) -> Vec<IVec3> {
        let e = Vec3::new(self.radius, self.radius, self.radius);
        let neg_extents = ((self.center - e) / CHUNK_SIZE as f32).as_i32();
        let pos_extents = ((self.center + e) / CHUNK_SIZE as f32).as_i32();
        let mut dist_squared = self.radius * self.radius;

        let mut chunks = vec![];

        for y in neg_extents.y..=pos_extents.y {
            for z in neg_extents.y..=pos_extents.z {
                for x in neg_extents.x..=pos_extents.x {
                    let c1 = (IVec3::new(x, y, z) * CHUNK_SIZE as i32).as_f32();
                    let c2 =
                        c1 + Vec3::new(CHUNK_SIZE as f32, CHUNK_SIZE as f32, CHUNK_SIZE as f32);
                    if self.center.x < c1.x {
                        dist_squared -= (self.center.x - c1.x).powi(2);
                    } else if self.center.x > c2.x {
                        dist_squared -= (self.center.x - c2.x).powi(2);
                    }
                    if self.center.y < c1.y {
                        dist_squared -= (self.center.y - c1.x).powi(2);
                    } else if self.center.y > c2.y {
                        dist_squared -= (self.center.y - c2.y).powi(2);
                    }
                    if self.center.z < c1.z {
                        dist_squared -= (self.center.z - c1.z).powi(2);
                    } else if self.center.z > c2.z {
                        dist_squared -= (self.center.z - c2.z).powi(2);
                    }
                    if dist_squared > 0. {
                        chunks.push(IVec3::new(x, y, z));
                    }
                }
            }
        }

        chunks
    }

    fn conatins(&self, pos: Vec3) -> bool {
        self.center.distance_squared(pos) < self.radius * self.radius
    }

    fn get_voxel(&self) -> &Voxel {
        &self.voxel
    }

    fn set_result(&mut self, res: Vec<(IVec3, EditResult)>) {
        self.result = res;
    }

    fn get_result(&self) -> &Vec<(IVec3, EditResult)> {
        &self.result
    }
}

fn chunk_edit_system<E: ChunkEdit>(
    mut commands: Commands,
    chunks: Res<Chunks>,
    mut edits: Query<(Entity, &mut E)>,
    mut data_query: Query<&mut ChunkData>,
    pool: Res<ComputeTaskPool>,
) {
    edits.par_for_each_mut(&pool, 16, |(_, mut e)| {
        e.compute_edits();
    });

    for (ee, e) in edits.iter_mut() {
        let res = e.get_result();
        for (chunk, res) in res {
            if let Some(&entity) = chunks.get(chunk) {
                if e.get_voxel().is_empty() {
                    if let Ok(mut data) = data_query.get_mut(entity) {
                        match res {
                            EditResult::All => {
                                commands
                                    .entity(entity)
                                    .remove::<ChunkData>()
                                    .remove::<ChunkMesh>()
                                    .remove_bundle::<MeshBundle>()
                                    .remove::<Task<ChunkData>>()
                                    .remove::<Task<MeshData>>();
                            }
                            EditResult::Some(edits) => {
                                for edit in edits {
                                    let old = data.voxels[edit.1 as usize][edit.2 as usize]
                                        [edit.0 as usize];

                                    if !old.is_empty() {
                                        data.num_voxels -= 1;
                                    }
                                    data.voxels[edit.1 as usize][edit.2 as usize]
                                        [edit.0 as usize] = *e.get_voxel();
                                }
                                commands
                                    .entity(entity)
                                    .remove::<ChunkMesh>()
                                    .remove::<Task<MeshData>>();
                            }
                        }
                    }
                } else {
                    if let Ok(mut data) = data_query.get_mut(entity) {
                        match res {
                            EditResult::All => {
                                data.num_voxels = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
                                data.voxels =
                                    [[[*e.get_voxel(); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
                            }
                            EditResult::Some(edits) => {
                                for edit in edits {
                                    let old = data.voxels[edit.1 as usize][edit.2 as usize]
                                        [edit.0 as usize];

                                    if old.is_empty() {
                                        data.num_voxels += 1;
                                    }
                                    data.voxels[edit.1 as usize][edit.2 as usize]
                                        [edit.0 as usize] = *e.get_voxel();
                                }
                            }
                        }
                        commands
                            .entity(entity)
                            .remove::<ChunkMesh>()
                            .remove::<Task<MeshData>>();
                    } else {
                        match res {
                            EditResult::All => {
                                commands
                                    .entity(entity)
                                    .insert(ChunkData {
                                        voxels: [[[*e.get_voxel(); CHUNK_SIZE]; CHUNK_SIZE];
                                            CHUNK_SIZE],
                                        num_voxels: CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE,
                                    })
                                    .remove::<Task<MeshData>>();
                            }
                            EditResult::Some(edits) => {
                                let mut data =
                                    [[[Voxel::default(); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
                                for edit in edits {
                                    data[edit.1 as usize][edit.2 as usize][edit.0 as usize] =
                                        *e.get_voxel();
                                }
                                commands.entity(entity).insert(ChunkData {
                                    voxels: data,
                                    num_voxels: edits.len(),
                                });
                            }
                        }
                    }
                }
            } else {
                todo!()
            }
        }
        commands.entity(ee).despawn();
    }
}

pub fn add_systems(app: &mut AppBuilder) {
    app.add_system(chunk_edit_system::<SphereEdit>.system());
}
