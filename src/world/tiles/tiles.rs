use crate::registry::tiles::TILE_REGISTRY;
use crate::registry::{ObjectSource, Registerable};
use crate::world::tiles::special::state::StateTile;
use crate::world::tiles::special::update::UpdateTile;
use crate::world::tiles::Orientation;
use mvengine::utils::savers::SaveArc;
use mvutils::save::{Loader, Savable, Saver};
use parking_lot::RwLock;

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
            InnerTile::Update(_) => {
                1u8.save(saver);
            }
        }
        if let Some(b) = &self.info.state {
            1u8.save(saver);
            if let Some(buf) = saver.as_any_mut().downcast_mut() {
                b.save(buf);
            }
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
                if let Some(buf) = loader.as_any_mut().downcast_mut() {
                    if let Some(state) = &mut template.info.state {
                        state.load_into(buf)?;
                    }
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
    Update(Box<dyn UpdateTile>),
}

impl Clone for InnerTile {
    fn clone(&self) -> Self {
        match self {
            InnerTile::Static => InnerTile::Static,
            InnerTile::Update(b) => InnerTile::Update(b.box_clone())
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

#[repr(C)]
pub struct TileInfo {
    pub orientation: Orientation,
    pub inner: InnerTile,
    pub source: ObjectSource,
    pub should_tick: bool,
    pub state: Option<Box<dyn StateTile>>,
}

impl Clone for TileInfo {
    fn clone(&self) -> Self {
        Self {
            orientation: self.orientation,
            inner: self.inner.clone(),
            source: self.source.clone(),
            should_tick: self.should_tick,
            state: self.state.as_ref().map(|x| x.box_clone()),
        }
    }
}

unsafe impl Send for TileInfo {}
unsafe impl Sync for TileInfo {}

macro_rules! to_opt_box {
    ($n:ident) => {{
        match $n {
            None => None,
            Some(a) => Some(Box::new(a))
        }
    }};
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

    pub fn vanilla_update<U: UpdateTile + 'static>(update: U) -> Self {
        Self {
            orientation: Orientation::North,
            inner: InnerTile::Update(Box::new(update)),
            source: ObjectSource::Vanilla,
            should_tick: false,
            state: None,
        }
    }

    pub fn vanilla_static_with<S: StateTile + 'static>(state: S) -> Self {
        Self {
            orientation: Orientation::North,
            inner: InnerTile::Static,
            source: ObjectSource::Vanilla,
            should_tick: false,
            state: Some(Box::new(state)),
        }
    }

    pub fn vanilla_update_with<U: UpdateTile + 'static, S: StateTile + 'static>(update: U, state: S) -> Self {
        Self {
            orientation: Orientation::North,
            inner: InnerTile::Update(Box::new(update)),
            source: ObjectSource::Vanilla,
            should_tick: false,
            state: Some(Box::new(state)),
        }
    }
}