use bytebuffer::ByteBuffer;
use mvutils::Savable;
use mvutils::save::Savable;
use crate::world::tiles::{Orientation, TileInstance};

#[derive(Clone, Savable)]
pub struct Lamp {
    orientation: Orientation,
    on: bool
}

impl Lamp {
    pub fn new() -> Self {
        Self {
            orientation: Orientation::North,
            on: false,
        }
    }
}

impl TileInstance for Lamp {
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
        self.on.save(saver);
    }

    fn orientation(&self) -> Orientation {
        self.orientation
    }

    fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
}