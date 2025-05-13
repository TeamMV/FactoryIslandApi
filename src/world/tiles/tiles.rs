use crate::registry::tiles::TILE_REGISTRY;
use crate::registry::Registerable;
use crate::world::tiles::update::UpdateTile;
use crate::world::tiles::ObjectSource;
use crate::world::tiles::Orientation;
use mvengine::utils::savers::SaveArc;
use mvutils::save::{Loader, Savable, Saver};
use mvutils::enum_val;
use parking_lot::RwLock;

pub type TileType = SaveArc<RwLock<Tile>>;

#[derive(Clone)]
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
            InnerTile::Update(b) => {
                1u8.save(saver);
                let vec = b.save_to_vec();
                vec.save(saver);
            }
        }
        if let Some(state) = &self.info.state {
            1u8.save(saver);
            let vec = state.save_to_vec();
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
                    let vec = Vec::<u8>::load(loader)?;
                    let inner = template.info.inner.clone();
                    let mut inner = enum_val!(InnerTile, inner, Update);
                    inner.load_into_self(vec);
                    Ok(InnerTile::Update(inner))
                },
                _ => Err("Illegal variant".to_string())
            }?;
            template.info.inner = inner;
            let has_state = u8::load(loader)?;
            if has_state == 1 { 
                let vec = Vec::<u8>::load(loader)?;
                if let Some(state) = &mut template.info.state {
                    state.load_into_self(vec);
                }
            }
            Ok(template)
        } else {
            Err(format!("Not registered: {id}"))
        }
    }
}

pub enum InnerTile {
    Static,
    Update(Box<dyn UpdateTile>)
}

impl Clone for InnerTile {
    fn clone(&self) -> Self {
        match self {
            InnerTile::Static => Self::Static,
            InnerTile::Update(b) => {
                InnerTile::Update(b.box_clone())
            }
        }
    }
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

pub struct TileInfo {
    pub orientation: Orientation,
    pub inner: InnerTile,
    pub source: ObjectSource,
    pub should_tick: bool,
    pub state: Option<Box<dyn TileState>>
}

impl Clone for TileInfo {
    fn clone(&self) -> Self {
        Self {
            orientation: self.orientation.clone(),
            inner: self.inner.clone(),
            source: self.source.clone(),
            should_tick: self.should_tick,
            state: self.state.as_ref().map(|b| b.box_clone()),
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
        }
    }
    
    pub fn vanilla_update(tile: impl UpdateTile + 'static) -> Self {
        Self {
            orientation: Orientation::North,
            inner: InnerTile::Update(Box::new(tile)),
            source: ObjectSource::Vanilla,
            should_tick: false,
            state: None,
        }
    }

    pub fn vanilla_update_ticking(tile: impl UpdateTile + 'static) -> Self {
        Self {
            orientation: Orientation::North,
            inner: InnerTile::Update(Box::new(tile)),
            source: ObjectSource::Vanilla,
            should_tick: true,
            state: None,
        }
    }

    pub fn vanilla_static_state(state: impl TileState + 'static) -> Self {
        Self {
            orientation: Orientation::North,
            inner: InnerTile::Static,
            source: ObjectSource::Vanilla,
            should_tick: false,
            state: Some(Box::new(state)),
        }
    }

    pub fn vanilla_update_state(tile: impl UpdateTile + 'static, state: impl TileState + 'static) -> Self {
        Self {
            orientation: Orientation::North,
            inner: InnerTile::Update(Box::new(tile)),
            source: ObjectSource::Vanilla,
            should_tick: false,
            state: Some(Box::new(state)),
        }
    }

    pub fn vanilla_update_ticking_state(tile: impl UpdateTile + 'static, state: impl TileState + 'static) -> Self {
        Self {
            orientation: Orientation::North,
            inner: InnerTile::Update(Box::new(tile)),
            source: ObjectSource::Vanilla,
            should_tick: true,
            state: Some(Box::new(state)),
        }
    }
}

pub trait TileState: Send + Sync {
    fn box_clone(&self) -> Box<dyn TileState>;

    fn save_to_vec(&self) -> Vec<u8>;

    fn load_into_self(&mut self, data: Vec<u8>);
    
    fn client_state(&self) -> usize;
}