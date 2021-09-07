use super::{
    chunk::{ChunkData, ChunkPosition, Chunks},
    loader::{LoadSettings, TaskInfo},
    ordered_float::OrderedFloat,
    shader::Pipeline,
    voxel::{Face, Voxel, CHUNK_SIZE},
    ChunkGenerator,
};
use bevy::render::mesh::{Indices, VertexAttributeValues};
use bevy::render::pipeline::RenderPipeline;
use bevy::tasks::AsyncComputeTaskPool;
use futures_lite::future::{block_on, poll_once};

use bevy::{prelude::*, render::pipeline::PrimitiveTopology, tasks::Task};

// Translated to rust from https://github.com/roboleary/GreedyMesh
// TODO: sample mesh from neighbouring chunks.
fn greedy_meshing<F: FnMut(([f32; 3], [f32; 3], [f32; 3], [f32; 3]), Voxel, Face, bool)>(
    mut quad: F,
    data: &ChunkData,
) {
    let mut voxel_mask = [None; CHUNK_SIZE * CHUNK_SIZE];

    let mut face = Face::Top;
    let mut back_face = true;
    for _ in 0..2 {
        for d in 0..3 {
            let u = (d + 1) % 3;
            let v = (d + 2) % 3;

            let mut x = [0; 3];
            let mut q = [0; 3];
            q[d as usize] = 1;

            if d == 0 {
                face = if back_face { Face::West } else { Face::East }
            } else if d == 1 {
                face = if back_face { Face::Top } else { Face::Bottom }
            } else if d == 2 {
                face = if back_face { Face::South } else { Face::North }
            }

            for i in -1..CHUNK_SIZE as isize {
                x[d] = i;
                let mut n = 0;
                for i in 0..CHUNK_SIZE as isize {
                    x[v] = i;
                    for i in 0..CHUNK_SIZE as isize {
                        x[u] = i;
                        let v1 = data.get(x[0], x[1], x[2]);
                        let v2 = data.get(x[0] + q[0], x[1] + q[1], x[2] + q[2]);

                        voxel_mask[n] = if let (Some(v1), Some(v2)) = (v1, v2) {
                            if v1.is_empty() == v2.is_empty() {
                                None
                            } else if back_face {
                                Some(v1)
                            } else {
                                Some(v2)
                            }
                        } else if back_face {
                            v1
                        } else {
                            v2
                        };
                        n += 1;
                    }
                }

                x[d] += 1;

                n = 0;

                for j in 0..CHUNK_SIZE {
                    let mut i = 0;
                    while i < CHUNK_SIZE {
                        match voxel_mask[n] {
                            Some(v1) => {
                                let width = {
                                    let mut w = 1;
                                    while i + w < CHUNK_SIZE {
                                        if let Some(v2) = voxel_mask[n + w] {
                                            if v1.is_same_face(v2, face) {
                                                w += 1;
                                            } else {
                                                break;
                                            }
                                        } else {
                                            break;
                                        }
                                    }
                                    w
                                };

                                let height = {
                                    let mut h = 1;
                                    'calc: while j + h < CHUNK_SIZE {
                                        for k in 0..width {
                                            if let Some(Some(v2)) =
                                                voxel_mask.get(n + k + h * CHUNK_SIZE)
                                            {
                                                if !v1.is_same_face(v2, face) {
                                                    break 'calc;
                                                }
                                            } else {
                                                break 'calc;
                                            }
                                        }
                                        h += 1;
                                    }
                                    h
                                };

                                x[u] = i as isize;
                                x[v] = j as isize;
                                if !v1.is_empty() {
                                    // TODO: check if voxel is transparent
                                    let mut du = [0; 3];
                                    du[u] = width as isize;
                                    let mut dv = [0; 3];
                                    dv[v] = height as isize;

                                    quad(
                                        (
                                            [x[0] as f32, x[1] as f32, x[2] as f32],
                                            [
                                                (x[0] + du[0]) as f32,
                                                (x[1] + du[1]) as f32,
                                                (x[2] + du[2]) as f32,
                                            ],
                                            [
                                                (x[0] + du[0] + dv[0]) as f32,
                                                (x[1] + du[1] + dv[1]) as f32,
                                                (x[2] + du[2] + dv[2]) as f32,
                                            ],
                                            [
                                                (x[0] + dv[0]) as f32,
                                                (x[1] + dv[1]) as f32,
                                                (x[2] + dv[2]) as f32,
                                            ],
                                        ),
                                        *v1,
                                        face,
                                        back_face,
                                    );
                                }
                                for j in 0..height {
                                    for i in 0..width {
                                        voxel_mask[n + i + j * CHUNK_SIZE] = None;
                                    }
                                }

                                i += width;
                                n += width;
                            }

                            None => {
                                i += 1;
                                n += 1;
                            }
                        }
                    }
                }
            }
        }

        back_face = !back_face;
    }
}

fn construct_data(p: [f32; 3], face: Face, color: [f32; 3], light: f32) -> u32 {
    (p[0].round() as u32 & 31) << 27
        | (p[1].round() as u32 & 31) << 22
        | (p[2].round() as u32 & 31) << 17
        | (face as u32 & 7) << 14
        | ((color[0] * 7.) as u32 & 7) << 11
        | ((color[1] * 7.) as u32 & 7) << 8
        | ((color[0] * 7.) as u32 & 7) << 5
        | ((light * 31.) as u32 & 31)
}

pub struct ChunkMesh;

pub struct MeshData {
    vertices: Vec<u32>,
    indices: Vec<u16>,
}

fn chunk_mesh_generator(
    mut commands: Commands,
    chunk_container: Res<Chunks>,
    chunks: Query<(Entity, &ChunkPosition, &ChunkData), Without<ChunkMesh>>,
    thread_pool: Res<AsyncComputeTaskPool>,
    mut gen: Query<(Entity, &mut ChunkGenerator)>,
    load_settings: Res<LoadSettings>,
) {
    for (entity, mut gen) in gen.iter_mut() {
        if gen.mesh_job_count < load_settings.meshed_per_frame {
            let mut to_load: Vec<(Entity, &ChunkPosition, &ChunkData)> = chunks
                .iter()
                .filter(|(_, position, _)| {
                    chunk_container.is_generated(&(position.0 + IVec3::X))
                        && chunk_container.is_generated(&(position.0 - IVec3::X))
                        && chunk_container.is_generated(&(position.0 + IVec3::Y))
                        && chunk_container.is_generated(&(position.0 - IVec3::Y))
                        && chunk_container.is_generated(&(position.0 + IVec3::Z))
                        && chunk_container.is_generated(&(position.0 - IVec3::Z))
                })
                .collect();
            to_load.sort_by_cached_key(|(_, p, _)| {
                OrderedFloat(p.0.as_f32().distance_squared(gen.chunk_pos.as_f32()))
            });

            for i in 0..std::cmp::min(
                load_settings.meshed_per_frame - gen.mesh_job_count,
                to_load.len(),
            ) {
                if to_load[i].2.num_voxels == 0 {
                    commands
                        .entity(to_load[i].0)
                        .remove::<ChunkData>()
                        .remove_bundle::<MeshBundle>();
                } else if to_load[i].2.num_voxels == CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE {
                    commands
                        .entity(to_load[i].0)
                        .insert(ChunkMesh)
                        .remove_bundle::<MeshBundle>();
                } else {
                    let data = *to_load[i].2;
                    let task = thread_pool.spawn(async move {
                        let mut vertices: Vec<u32> = Vec::new();
                        let mut indices: Vec<u16> = Vec::new();

                        let quad = |q: ([f32; 3], [f32; 3], [f32; 3], [f32; 3]),
                                    voxel: Voxel,
                                    face: Face,
                                    backface: bool| {
                            let start = vertices.len() as u16;
                            let color = match voxel.id {
                                1 => [1., 0., 1.],
                                2 => [0.1, 0.7, 0.],
                                3 => [0.9, 0., 0.],
                                4 => [1.0, 0.15, 0.],
                                5 => [0., 0.25, 0.6],
                                6 => [0., 0.6, 0.25],
                                7 => [1., 1., 0.25],
                                8 => [0.25, 1., 0.25],
                                9 => [0.25, 0.1, 0.1],
                                _ => [0., 0., 0.],
                            };
                            let light = face.light();

                            vertices.push(construct_data(q.0, face, color, light)); //  1-------2
                            vertices.push(construct_data(q.1, face, color, light)); //  |       |
                            vertices.push(construct_data(q.2, face, color, light)); //  |       |
                            vertices.push(construct_data(q.3, face, color, light)); //  0-------3

                            if backface {
                                //-----------------------//  1---2
                                indices.push(start + 0); //  |  /
                                indices.push(start + 1); //  | /
                                indices.push(start + 2); //  0/

                                //-----------------------//    /2
                                indices.push(start + 0); //   / |
                                indices.push(start + 2); //  /  |
                                indices.push(start + 3); // 0---3
                            } else {
                                //-----------------------//  1---2
                                indices.push(start + 1); //  |  /
                                indices.push(start + 0); //  | /
                                indices.push(start + 2); //  0/

                                //-----------------------//    /2
                                indices.push(start + 2); //   / |
                                indices.push(start + 0); //  /  |
                                indices.push(start + 3); // 0---3
                            }
                        };
                        greedy_meshing(quad, &data);
                        MeshData { vertices, indices }
                    });

                    gen.mesh_job_count += 1;
                    commands
                        .entity(to_load[i].0)
                        .insert(task)
                        .insert(TaskInfo::new(entity))
                        .insert(ChunkMesh);
                }
            }
        }
    }
}

fn chunk_mesh_updater(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    pipeline: Res<Pipeline>,
    mut chunks: Query<(
        Entity,
        &ChunkPosition,
        &mut Task<MeshData>,
        &TaskInfo,
        Option<&mut MeshBundle>,
    )>,
    mut gen: Query<&mut ChunkGenerator>,
) {
    for (entity, pos, mut task, info, bundle) in chunks.iter_mut() {
        if let Some(mesh_data) = block_on(poll_once(&mut *task)) {
            if mesh_data.indices.is_empty() {
                commands.entity(entity).remove_bundle::<MeshBundle>();
            } else {
                let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                mesh.set_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    VertexAttributeValues::Uint(mesh_data.vertices),
                );
                mesh.set_indices(Some(Indices::U16(mesh_data.indices)));

                let mesh_handle: Handle<Mesh> = meshes.add(mesh);

                if let Some(mut bundle) = bundle {
                    bundle.mesh = mesh_handle;
                } else {
                    commands
                        .entity(entity)
                        .insert(ChunkMesh)
                        .insert_bundle(MeshBundle {
                            mesh: mesh_handle,
                            transform: Transform::from_translation(
                                pos.0.as_f32() * CHUNK_SIZE as f32,
                            ),
                            render_pipelines: RenderPipelines::from_pipelines(vec![
                                RenderPipeline::new(pipeline.handle.clone()),
                            ]),
                            ..Default::default()
                        });
                }
            }
            commands
                .entity(entity)
                .remove::<Task<MeshData>>()
                .remove::<TaskInfo>();
            if let Ok(mut gen) = gen.get_mut(info.sender) {
                gen.mesh_job_count -= 1;
            }
        }
    }
}

pub fn add_systems(app: &mut AppBuilder) {
    app.add_system(chunk_mesh_generator.system())
        .add_system(chunk_mesh_updater.system());
}
