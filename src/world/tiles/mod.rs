use mvengine::rendering::RenderContext;
use mvengine::ui::context::UiResources;
use mvutils::Savable;
use crate::world::tiles::resources::ClientTileRes;

pub mod pos;
pub mod terrain;
pub mod tiles;
pub mod update;
pub mod implementations;
pub mod resources;

#[derive(Savable, Clone, Copy, Debug)]
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

#[derive(Clone, Savable)]
pub enum ObjectSource {
    Vanilla,
    Mod(String, ClientTileRes)
}

impl ObjectSource {
    pub fn mod_simple(id: &str) -> Self {
        Self::Mod(id.to_string(), ClientTileRes::empty())
    }

    pub fn mod_res(id: &str, res: ClientTileRes) -> Self {
        Self::Mod(id.to_string(), res)
    }
}