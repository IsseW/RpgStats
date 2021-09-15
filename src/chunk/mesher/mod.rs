// mod marching_cubes;
// mod marching_cubes_tables;

mod greedy_meshing;

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        pipeline::{PrimitiveTopology, RenderPipeline},
    },
    tasks::AsyncComputeTaskPool,
};
use futures_lite::future::{block_on, poll_once};

use crate::chunk::{chunk::CHUNK_SIZE, voxel::Face, Voxel};

use super::{
    chunk::{chunk_size, ChunkData, ChunkPosition, ChunkState, Chunks, DataFlags, MAX_DEPTH},
    generator::{Generator, GeneratorData, GeneratorTask},
    shader::Pipeline,
    GeneratorOptions,
};

pub struct Vertex {
    pos: Vec3,
    id: u8,
}

impl Vertex {
    pub fn new(pos: Vec3, id: u8) -> Self {
        Self { pos, id }
    }
}

pub struct ChunkMesh(pub bool);

struct MeshData {
    vertices: Vec<[u32; 2]>,
    indices: Vec<u16>,
}

fn chunk_meshing_system<const DEPTH: u32>(
    mut commands: Commands,
    mut generators: Query<(
        Entity,
        &Generator,
        &mut GeneratorData<DEPTH>,
        Option<&mut GeneratorData<{ DEPTH + 1 }>>,
    )>,
    chunks: Res<Chunks<DEPTH>>,
    options: Res<GeneratorOptions>,
    mut chunk_query: Query<(Entity, &mut ChunkState, &ChunkData<DEPTH>), Without<ChunkMesh>>,

    thread_pool: Res<AsyncComputeTaskPool>,
    test_mat: Res<Handle<StandardMaterial>>,
    test_mesh: Res<Handle<Mesh>>,
) {
    for (entity, gen, mut data, mut child_data) in generators.iter_mut() {
        while data.mesh_index < gen.load_order.len()
            && data.current_mesh_task_count < options.concurrent_meshing_tasks
        {
            let chunk_pos = data.position + gen.load_order[data.mesh_index];
            if let Some(&e) = chunks.chunks.get(&chunk_pos) {
                if let Ok((e, mut state, chunk_data)) = chunk_query.get_mut(e) {
                    if *state == ChunkState::Generated {
                        match chunk_data.flags {
                            _ => {
                                commands
                                    .entity(e)
                                    .insert(ChunkMesh(DEPTH == 0))
                                    .insert_bundle(PbrBundle {
                                        material: test_mat.clone(),
                                        mesh: test_mesh.clone(),
                                        transform: {
                                            let size = (chunk_size(DEPTH)) as f32;
                                            let mut transform = Transform::from_translation(
                                                chunk_pos.as_f32() * size + Vec3::splat(size / 2.0),
                                            );
                                            transform.scale = Vec3::splat(size);
                                            //
                                            transform
                                        },
                                        ..Default::default()
                                    });
                                *state = ChunkState::Ready;
                                if let Some(child_data) = child_data.as_mut() {
                                    child_data.gen_index = 0;
                                }
                            }
                            DataFlags::Empty | DataFlags::Full => {
                                commands.entity(entity).insert(ChunkMesh(false));
                                *state = ChunkState::Ready;
                            }
                            DataFlags::None => {
                                let data_copy = chunk_data.voxels.clone();
                                let mesh_task = thread_pool.spawn(async move {
                                    fn construct_data(
                                        p: [f32; 3],
                                        face: Face,
                                        color: [u8; 3],
                                        light: u8,
                                    ) -> [u32; 2] {
                                        [
                                            ((p[0].round() as u32 & 0xFF) << 24)
                                                | ((p[1].round() as u32 & 0xFF) << 16)
                                                | ((p[2].round() as u32 & 0xFF) << 8)
                                                | (face as u32 & 7) << 5,
                                            ((color[0] as u32) << 24)
                                                | ((color[1] as u32) << 16)
                                                | ((color[2] as u32) << 8)
                                                | light as u32,
                                        ]
                                    }
                                    let mut vertices: Vec<[u32; 2]> = vec![];
                                    let mut indices: Vec<u16> = vec![];
                                    let quad =
                                        |q: ([f32; 3], [f32; 3], [f32; 3], [f32; 3]),
                                         voxel: Voxel,
                                         face: Face,
                                         backface: bool| {
                                            let start = vertices.len() as u16;
                                            let color = match voxel.id {
                                                1 => [255, 0, 255],
                                                2 => [30, 200, 0],
                                                3 => [230, 0, 0],
                                                4 => [255, 40, 0],
                                                5 => [0, 69, 180],
                                                6 => [0, 180, 69],
                                                7 => [255, 255, 69],
                                                8 => [69, 255, 69],
                                                9 => [69, 30, 30],
                                                _ => [0, 0, 0],
                                            };

                                            vertices.push(construct_data(q.0, face, color, 0)); //  1-------2
                                            vertices.push(construct_data(q.1, face, color, 0)); //  |       |
                                            vertices.push(construct_data(q.2, face, color, 0)); //  |       |
                                            vertices.push(construct_data(q.3, face, color, 0)); //  0-------3

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
                                    greedy_meshing::greedy_meshing(quad, &data_copy);

                                    MeshData { vertices, indices }
                                });
                                data.current_mesh_task_count += 1;
                                *state = ChunkState::Meshing;
                                commands.entity(e).insert(GeneratorTask {
                                    task: mesh_task,
                                    sender: entity,
                                });
                            }
                        }
                    }
                }
            }
            data.mesh_index += 1;
        }
    }
}

fn poll_mesh_tasks<const DEPTH: u32>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    pipeline: Res<Pipeline>,
    mut chunks: Query<(
        Entity,
        &ChunkPosition<DEPTH>,
        &mut ChunkState,
        &mut GeneratorTask<MeshData>,
        Option<&mut MeshBundle>,
    )>,
    mut gen: Query<(
        &mut GeneratorData<DEPTH>,
        Option<&mut GeneratorData<{ DEPTH + 1 }>>,
    )>,
) {
    for (entity, pos, mut state, mut task, bundle) in chunks.iter_mut() {
        if let Some(mesh_data) = block_on(poll_once(&mut task.task)) {
            *state = ChunkState::Ready;
            if mesh_data.indices.len() == 0 {
                commands.entity(entity).insert(ChunkMesh);
            } else {
                let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                mesh.set_attribute("vdata", VertexAttributeValues::Uint2(mesh_data.vertices));
                mesh.set_indices(Some(Indices::U16(mesh_data.indices)));

                let mesh_handle: Handle<Mesh> = meshes.add(mesh);

                if let Some(mut bundle) = bundle {
                    bundle.mesh = mesh_handle;
                    commands.entity(entity).insert(ChunkMesh);
                } else {
                    commands
                        .entity(entity)
                        .insert(ChunkMesh(DEPTH == 0))
                        .insert_bundle(MeshBundle {
                            mesh: mesh_handle,
                            transform: {
                                let mut transform = Transform::from_translation(
                                    pos.0.as_f32() * chunk_size(DEPTH) as f32,
                                );
                                transform.scale = Vec3::ONE * chunk_size(DEPTH) as f32;

                                transform
                            },
                            render_pipelines: RenderPipelines::from_pipelines(vec![
                                RenderPipeline::new(pipeline.handle.clone()),
                            ]),
                            ..Default::default()
                        });
                }
            }

            commands.entity(entity).remove::<GeneratorTask<MeshData>>();
            if let Ok((mut gen, mut child_data)) = gen.get_mut(task.sender) {
                gen.current_mesh_task_count -= 1;
                if let Some(child_data) = child_data.as_mut() {
                    child_data.gen_index = 0;
                }
            }
        }
    }
}

fn startup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    server: Res<AssetServer>,
) {
    commands.insert_resource(meshes.add(Mesh::from(shape::Cube { size: 1.0 })));
    commands.insert_resource(materials.add(StandardMaterial {
        base_color_texture: Some(server.load("test_texture.png")),
        ..Default::default()
    }));
}

pub fn add_systems(app: &mut AppBuilder) {
    seq_macro::seq!(
        N in 0..13 {
    app.add_startup_system(startup_system.system())
    #(
        .add_system(chunk_meshing_system::<N>.system())
        .add_system(poll_mesh_tasks::<N>.system())
    )*;
        }
    );
}
