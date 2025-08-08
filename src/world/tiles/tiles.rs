use abi_stable::std_types::RVec;
use crate::registry::tiles::TILE_REGISTRY;
use crate::registry::{ObjectSource, Registerable};
use crate::world::tiles::Orientation;
use mvengine::utils::savers::SaveArc;
use mvutils::save::{Loader, Savable, Saver};
use mvutils::enum_val;
use parking_lot::RwLock;
use crate::mods::modsdk::MOpt;
use crate::world::tiles::update::{This, UpdateTileTrait};

pub type TileType = SaveArc<RwLock<Tile>>;

#[derive(Clone)]
#[repr(C)]
pub struct Tile {
    pub id: usize,
    pub info: TileInfo,
}

impl Tile {
    pub fn to_type(self) -> TileType {
        SaveArc::new(RwLock::new(self))
    }
}

impl Savable for Tile {
    fn save(&self, saver: &mut impl Saver) {
        self.id.save(saver);
        self.info.orientation.save(saver);
        match &self.info.inner {
            InnerTile::Static => 0u8.save(saver),
            InnerTile::Update(b, this) => {
                1u8.save(saver);
            }
        }
        if let Some((tile, state)) = &self.info.state {
            1u8.save(saver);
            let vec = (state.save_to_vec)(*tile);
            vec.save(saver);
        } else {
            0u8.save(saver);
        }
    }

    fn load(loader: &mut impl Loader) -> Result<Self, String> {
        let id = usize::load(loader)?;
        let orientation = Orientation::load(loader)?;
        let variant = u8::load(loader)?;
        if let Some(mut template) = TILE_REGISTRY.create_object(id) {
            template.info.orientation = orientation;
            let inner = match variant {
                0 => Ok(InnerTile::Static),
                1 => {
                    let inner = template.info.inner.clone();
                    Ok(inner)
                },
                _ => Err("Illegal variant".to_string())
            }?;
            template.info.inner = inner;
            let has_state = u8::load(loader)?;
            if has_state == 1 {
                let vec = RVec::<u8>::load(loader)?;
                if let Some((tile, state)) = &mut template.info.state {
                    let o = (state.load_into_self)(vec, *tile);
                    o.to_rust().ok_or(format!("Loading of state failed for id: {id}"))?;
                }
            }
            Ok(template)
        } else {
            Err(format!("Not registered: {id}"))
        }
    }
}

#[derive(Clone)]
pub enum InnerTile {
    Static,
    Update(UpdateTileTrait, This),
}

impl Registerable for Tile {
    type CreateInfo = TileInfo;

    fn with_id(id: usize, info: Self::CreateInfo) -> Self {
        Self {
            id,
            info,
        }
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct TileInfo {
    pub orientation: Orientation,
    pub inner: InnerTile,
    pub source: ObjectSource,
    pub should_tick: bool,
    pub state: Option<(This, TileStateTrait)>,
    pub oct: Option<(This, ObjControlTrait)>
}

impl Clone for TileInfo {
    fn clone(&self) -> Self {
        let (inner, state, oct) = if let Some((tile, oct)) = &self.oct {
            let new = (oct.create_copy)(*tile);
            let inner = if let InnerTile::Update(t, _) = &self.inner {
                InnerTile::Update(t.clone(), new)
            } else {
                self.inner.clone()
            };
            let state = self.state.clone().map(|x| (new, x.1));
            let oct = self.oct.clone().map(|x| (new, x.1));
            (inner, state, oct)
        } else {
            (self.inner.clone(), self.state.clone(), self.oct.clone())
        };
        Self {
            orientation: self.orientation.clone(),
            inner,
            source: self.source.clone(),
            should_tick: self.should_tick,
            state,
            oct: oct,
        }
    }
}

impl Drop for TileInfo {
    fn drop(&mut self) {
        if let Some((tile, oct)) = &self.oct {
            unsafe {
                (oct.free)(*tile)
            }
        }
    }
}

impl TileInfo {
    pub fn vanilla_static() -> Self {
        Self {
            orientation: Orientation::North,
            inner: InnerTile::Static,
            source: ObjectSource::Vanilla,
            should_tick: false,
            state: None,
            oct: None,
        }
    }
    
    pub fn vanilla_update(tile: This, handler: UpdateTileTrait, oct: ObjControlTrait) -> Self {
        Self {
            orientation: Orientation::North,
            inner: InnerTile::Update(handler, tile),
            source: ObjectSource::Vanilla,
            should_tick: false,
            state: None,
            oct: Some((tile, oct)),
        }
    }

    pub fn vanilla_update_ticking(tile: This, handler: UpdateTileTrait, oct: ObjControlTrait) -> Self {
        Self {
            orientation: Orientation::North,
            inner: InnerTile::Update(handler, tile),
            source: ObjectSource::Vanilla,
            should_tick: true,
            state: None,
            oct: Some((tile, oct)),
        }
    }

    pub fn vanilla_static_state(tile: This, state: TileStateTrait, oct: ObjControlTrait) -> Self {
        Self {
            orientation: Orientation::North,
            inner: InnerTile::Static,
            source: ObjectSource::Vanilla,
            should_tick: false,
            state: Some((tile, state)),
            oct: Some((tile, oct)),
        }
    }

    pub fn vanilla_update_state(tile: This, handler: UpdateTileTrait, state: TileStateTrait, oct: ObjControlTrait) -> Self {
        Self {
            orientation: Orientation::North,
            inner: InnerTile::Update(handler, tile),
            source: ObjectSource::Vanilla,
            should_tick: false,
            state: Some((tile, state)),
            oct: Some((tile, oct)),
        }
    }

    pub fn vanilla_update_ticking_state(tile: This, handler: UpdateTileTrait, state: TileStateTrait, oct: ObjControlTrait) -> Self {
        Self {
            orientation: Orientation::North,
            inner: InnerTile::Update(handler, tile),
            source: ObjectSource::Vanilla,
            should_tick: true,
            state: Some((tile, state)),
            oct: Some((tile, oct)),
        }
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct TileStateTrait {
    pub save_to_vec: fn(This) -> RVec<u8>,
    pub load_into_self: fn(RVec<u8>, This) -> MOpt<()>,
    pub client_state: fn(This) -> usize,
    pub apply_client_state: fn(This, usize),
}

pub trait TileState {
    fn create_trait() -> TileStateTrait;
}

#[derive(Clone)]
#[repr(C)]
pub struct ObjControlTrait {
    pub create_copy: fn(This) -> This,
    pub free: unsafe fn(This)
}

pub trait ObjControl {
    fn create_trait() -> ObjControlTrait;
}