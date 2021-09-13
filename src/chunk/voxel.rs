use std::convert::TryInto;

use super::chunk::{CHUNK_SIZE, CHUNK_VOXELS};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Voxel {
    //pub strength: i8,
    pub id: u8,
}
#[derive(Clone, Copy)]
pub enum Face {
    Top,
    Bottom,
    West,
    East,
    North,
    South,
}

impl Face {
    pub fn direction(&self) -> [f32; 3] {
        match self {
            Self::Top => [0., 1., 0.],
            Self::Bottom => [0., -1., 0.],
            Self::West => [1., 0., 0.],
            Self::East => [-1., 0., 0.],
            Self::North => [0., 0., 1.],
            Self::South => [0., 0., -1.],
        }
    }
    pub fn plane(&self) -> [f32; 3] {
        match self {
            Self::Top | Self::Bottom => [1., 0., 1.],
            Self::West | Self::East => [0., 1., 1.],
            Self::North | Self::South => [1., 1., 0.],
        }
    }
    pub fn opposite(&self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
            Self::West => Self::East,
            Self::East => Self::West,
            Self::North => Self::South,
            Self::South => Self::North,
        }
    }

    pub fn light(&self) -> f32 {
        match self {
            Self::Top => 1.0,
            Self::Bottom => 0.4,
            _ => 0.7,
        }
    }
}

impl<T: Into<usize>> From<T> for Face {
    fn from(i: T) -> Self {
        match i.into() {
            0 => Self::Top,
            1 => Self::Bottom,
            2 => Self::West,
            3 => Self::East,
            4 => Self::North,
            5 => Self::South,
            _ => panic!(),
        }
    }
}

impl Voxel {
    pub fn is_same_face(&self, other: &Voxel, _face: Face) -> bool {
        self.id == other.id
    }

    pub fn is_empty(&self) -> bool {
        self.id == 0
    }
}

pub type VoxelArray = [Voxel; CHUNK_VOXELS];

pub trait Voxels {
    fn all(voxel: Voxel) -> Self;
    fn empty() -> Self;

    fn at(&self, x: usize, y: usize, z: usize) -> Voxel;
    fn try_at<I: TryInto<usize>>(&self, x: I, y: I, z: I) -> Option<Voxel>;
    fn at_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Voxel;
}

impl Voxels for VoxelArray {
    fn all(voxel: Voxel) -> Self {
        [voxel; CHUNK_VOXELS]
    }

    fn empty() -> Self {
        [Voxel::default(); CHUNK_VOXELS]
    }

    fn at(&self, x: usize, y: usize, z: usize) -> Voxel {
        self[y * CHUNK_SIZE * CHUNK_SIZE + z * CHUNK_SIZE + x]
    }
    fn try_at<I: TryInto<usize>>(&self, x: I, y: I, z: I) -> Option<Voxel> {
        self.get(
            y.try_into().ok()? * CHUNK_SIZE * CHUNK_SIZE
                + z.try_into().ok()? * CHUNK_SIZE
                + x.try_into().ok()?,
        )
        .map(|t| *t)
    }
    fn at_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Voxel {
        &mut self[y * CHUNK_SIZE * CHUNK_SIZE + z * CHUNK_SIZE + x]
    }
}
