#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Voxel {
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

    pub fn light(&self) -> f32 {
        match self {
            Self::Top => 1.0,
            Self::Bottom => 0.4,
            _ => 0.7,
        }
    }
}

impl Voxel {
    pub fn is_same_face(&self, other: &Voxel, face: Face) -> bool {
        self.id == other.id
    }

    pub fn is_empty(&self) -> bool {
        self.id == 0
    }
}

pub const CHUNK_SIZE: usize = 30;
pub type VoxelArray = [[[Voxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
