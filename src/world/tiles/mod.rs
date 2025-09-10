use bytebuffer::ByteBuffer;
use mvengine::rendering::RenderContext;
use mvengine::ui::context::UiResources;
use mvutils::bytebuffer::ByteBufferExtras;
use mvutils::Savable;
use crate::world::chunk::ToClientObject;
use crate::world::tiles::tiles::TileType;

pub mod pos;
pub mod terrain;
pub mod tiles;
pub mod update;
pub mod implementations;
pub mod special;

#[derive(Savable, Clone, Copy, Debug)]
#[repr(C)]
pub enum Orientation {
    North,
    South,
    East,
    West
}

impl Orientation {
    pub fn apply(&self, uv: [(f32, f32); 4]) -> [(f32, f32); 4] {
        match self {
            Orientation::North => uv,
            Orientation::East => [uv[3], uv[0], uv[1], uv[2]],
            Orientation::South => [uv[2], uv[3], uv[0], uv[1]],
            Orientation::West => [uv[1], uv[2], uv[3], uv[0]],
        }
    }
}

pub fn tile_to_client(tile_type: &TileType) -> ToClientObject {
    let lock = tile_type.read();

    let state = if let Some(s) = &lock.info.state {
        let mut buf = ByteBuffer::new_le();
        s.save_for_client(&mut buf);
        buf.into_vec()
    } else {
        vec![]
    };
    
    ToClientObject {
        id: lock.id as u16,
        source: lock.info.source.clone(),
        orientation: lock.info.orientation,
        state,
    }
}