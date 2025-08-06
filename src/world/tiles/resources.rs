use hashbrown::HashMap;
use mvengine::graphics::Drawable;
use mvengine::ui::res::MVR;
use mvutils::Savable;

#[derive(Clone, Savable)]
pub struct ClientTileRes {
    res: HashMap<usize, Drawable>
}

impl ClientTileRes {
    pub fn empty() -> Self {
        Self {
            res: HashMap::new(),
        }
    }

    pub fn of(state: usize, res: Drawable) -> Self {
        let mut map = HashMap::new();
        map.insert(state, res);
        Self {
            res: map
        }
    }

    pub fn and(mut self, state: usize, res: Drawable) -> Self {
        self.res.insert(state, res);
        self
    }

    pub fn map(&self, state: usize) -> Drawable {
        self.res.get(&state).cloned().unwrap_or(Drawable::Texture(MVR.texture.missing))
    }
}

#[macro_export]
macro_rules! tileset {
    ($set:ident.$tile:ident) => {
        Drawable::TileSet(R.tileset.$set, R.tile.$set.$tile)
    };
}