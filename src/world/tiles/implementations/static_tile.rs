use bytebuffer::ByteBuffer;
use mvutils::Savable;
use mvutils::save::Savable;
use crate::world::tiles::{Orientation, TileInstance};

#[derive(Clone, Savable)]
pub struct StaticTile {
    orientation: Orientation,
    health: f32,
    max_health: f32
}

impl StaticTile {
    pub fn new(max_health: f32) -> Self {
        Self {
            orientation: Orientation::North,
            health: max_health,
            max_health,
        }
    }
}

impl TileInstance for StaticTile {
    fn save(&self, saver: &mut ByteBuffer) {
        Savable::save(self, saver);
    }

    fn load_into(&mut self, loader: &mut ByteBuffer) -> Result<(), String> {
        *self = Self::load(loader)?;
        Ok(())
    }

    fn box_clone(&self) -> Box<dyn TileInstance> {
        Box::new(self.clone())
    }

    fn has_client_state(&self) -> bool {
        true
    }

    fn save_client_state(&self, saver: &mut ByteBuffer) {
        self.health.save(saver);
        self.max_health.save(saver);
    }

    fn orientation(&self) -> Orientation {
        self.orientation
    }

    fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
}