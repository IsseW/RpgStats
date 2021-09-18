use crate::chunk::{
    chunk::CHUNK_SIZE,
    voxel::{Face, VoxelArray, Voxels},
    Voxel,
};

pub fn greedy_meshing<F: FnMut(([f32; 3], [f32; 3], [f32; 3], [f32; 3]), Voxel, Face, bool)>(
    mut quad: F,
    data: &VoxelArray,
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
                        let v1 = data.try_at(x[0], x[1], x[2]);
                        let v2 = data.try_at(x[0] + q[0], x[1] + q[1], x[2] + q[2]);

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
                                            if v1.is_same_face(&v2, face) {
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
                                        v1,
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
