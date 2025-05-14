use std::fmt::{Debug, Formatter};
use mvutils::Savable;
use mvutils::save::{Loader, Savable, Saver};
use crate::world::{ChunkPos, CHUNK_SIZE};

#[derive(Clone, Default)]
pub struct TilePos {
    pub raw: (i32, i32),
    pub in_chunk_x: usize,
    pub in_chunk_z: usize,
    pub world_chunk_x: i32,
    pub world_chunk_z: i32,
    pub chunk_pos: ChunkPos
}

impl TilePos {
    pub fn new(x: i32, z: i32) -> Self {
        let cx = (x as f64 / CHUNK_SIZE as f64).floor() as i32;
        let cz = (z as f64 / CHUNK_SIZE as f64).floor() as i32;
        Self {
            raw: (x, z),
            in_chunk_x: x as usize % CHUNK_SIZE as usize,
            in_chunk_z: z as usize % CHUNK_SIZE as usize,
            world_chunk_x: cx,
            world_chunk_z: cz,
            chunk_pos: (cx, cz),
        }
    }

    pub fn from_in_chunk(chunk_pos: ChunkPos, x: i32, z: i32) -> Self {
        Self::new(
            chunk_pos.0 * CHUNK_SIZE + x,
            chunk_pos.1 * CHUNK_SIZE + z
        )
    }

    pub fn up(&self, n: i32) -> Self {
        Self::new(self.raw.0, self.raw.1 - n)
    }

    pub fn down(&self, n: i32) -> Self {
        Self::new(self.raw.0, self.raw.1 + n)
    }

    pub fn left(&self, n: i32) -> Self {
        Self::new(self.raw.0 - n, self.raw.1)
    }

    pub fn right(&self, n: i32) -> Self {
        Self::new(self.raw.0 + n, self.raw.1)
    }

    pub fn neighbours(&self) -> [Self; 8] {
        [
            self.up(1),
            self.up(1).right(1),
            self.right(1),
            self.down(1).right(1),
            self.down(1),
            self.down(1).left(1),
            self.left(1),
            self.up(1).left(1)
        ]
    }
    
    pub fn direct_neighbours(&self) -> [Self; 4] {
        [
            self.up(1),
            self.right(1),
            self.down(1),
            self.left(1)
        ]
    }
}

impl From<(i32, i32)> for TilePos {
    fn from(value: (i32, i32)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl Debug for TilePos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("raw:{:?}, cx:{}, cz:{}, wx:{}, wz:{}", self.raw, self.in_chunk_x, self.in_chunk_z, self.world_chunk_x, self.world_chunk_z))
    }
}

impl Savable for TilePos {
    fn save(&self, saver: &mut impl Saver) {
        self.raw.save(saver);
    }

    fn load(loader: &mut impl Loader) -> Result<Self, String> {
        <(i32, i32)>::load(loader).map(Self::from)
    }
}