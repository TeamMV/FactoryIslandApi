use bytebuffer::ByteBuffer;
use log::error;
use mvengine::rendering::RenderContext;
use mvengine::ui::context::UiResources;
use mvengine::utils::savers::SaveArc;
use mvutils::bytebuffer::ByteBufferExtras;
use mvutils::Savable;
use mvutils::save::{Loader, Savable, Saver};
use parking_lot::RwLock;
use crate::inventory::InventoryData;
use crate::meta::Meta;
use crate::registry::Registerable;
use crate::registry::tiles::TILE_REGISTRY;
use crate::world::chunk::ToClientObject;
use crate::world::tiles::implementations::Air;
use crate::world::tiles::pos::TilePos;
use crate::world::tiles::update::UpdateHandler;

pub mod pos;
pub mod terrain;
pub mod update;
pub mod implementations;

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

pub type TileType = SaveArc<RwLock<Tile>>;

pub struct Tile {
    pub id: TileKind,
    pub(crate) instance: Box<dyn TileInstance>
}

impl Tile {
    pub fn to_type(self) -> TileType {
        SaveArc::new(RwLock::new(self))
    }
}

impl Registerable for Tile {
    type CreateInfo = Box<dyn TileInstance>;

    fn with_id(id: usize, info: Self::CreateInfo) -> Self {
        Self {
            id: id as TileKind,
            instance: info,
        }
    }
}

unsafe impl Send for Tile {}
unsafe impl Sync for Tile {}

pub type TileKind = u16;

impl Savable for Tile {
    fn save(&self, saver: &mut impl Saver) {
        self.id.save(saver);
        if let Some(saver) = saver.as_any_mut().downcast_mut::<ByteBuffer>() {
            self.instance.save(saver);
        }
    }

    fn load(loader: &mut impl Loader) -> Result<Self, String> {
        let id = TileKind::load(loader)?;
        if let Some(mut template) = TILE_REGISTRY.create_object(id as usize) {
            let mut template: Tile = template;
            template.instance.set_kind(id);
            if let Some(loader) = loader.as_any_mut().downcast_mut::<ByteBuffer>() {
                if let Err(e) = template.instance.load_into(loader) {
                    error!("Error when loading Tile: {e}");
                }
            }

            Ok(template)
        } else {
            Ok(Tile {
                id: 0,
                instance: Box::new(Air),
            })
        }
    }
}

impl Clone for Tile {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            instance: self.instance.box_clone(),
        }
    }
}

pub trait TileInstance {
    fn set_kind(&mut self, kind: TileKind) {}

    fn save(&self, saver: &mut ByteBuffer);
    fn load_into(&mut self, loader: &mut ByteBuffer) -> Result<(), String>;
    fn box_clone(&self) -> Box<dyn TileInstance>;

    fn has_client_state(&self) -> bool;
    fn save_client_state(&self, saver: &mut ByteBuffer);

    //getters and setters
    fn orientation(&self) -> Orientation;
    fn set_orientation(&mut self, orientation: Orientation);

    fn has_inventory(&self, index: u8) -> bool { false }
    fn inventory(&self, index: u8) -> Option<&InventoryData> { None }
    fn inventory_mut(&mut self, index: u8) -> Option<&mut InventoryData> { None }

    //update
    fn update_handler(&mut self) -> Option<&mut UpdateHandler> { None }
    fn end_tick(&mut self) {}
}

pub fn tile_to_client(tile_type: &TileType) -> ToClientObject {
    let lock = tile_type.read();

    let state = if lock.instance.has_client_state() {
        let mut buf = ByteBuffer::new_le();
        lock.instance.save_client_state(&mut buf);
        buf.into_vec()
    } else {
        vec![]
    };
    
    ToClientObject {
        id: lock.id,
        orientation: lock.instance.orientation(),
        state,
    }
}

pub fn create_tile(id: usize) -> TileType {
    if let Some(x) = TILE_REGISTRY.create_object(id) {
        x.to_type()
    } else {
        let air = TILE_REGISTRY.create_object(0).unwrap();
        air.to_type()
    }
}

#[derive(Clone, Savable)]
pub struct InventoryTarget {
    pub pos: TilePos,
    pub index: u8,
}