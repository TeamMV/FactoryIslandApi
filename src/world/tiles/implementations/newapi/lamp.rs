use bytebuffer::ByteBuffer;
use mvutils::save::Savable;
use crate::world::tiles::newapi::state::StateTile;
use crate::world::tiles::newapi::update::{HasChanged, UpdateTile};
use crate::world::tiles::update::UpdateHandler;

#[derive(Clone)]
pub struct LampTile {
    update_handler: UpdateHandler,
}

impl LampTile {
    pub fn new() -> Self {
        Self {
            update_handler: UpdateHandler::new(),
        }
    }
}

impl UpdateTile for LampTile {
    fn update_handler(&mut self) -> &mut UpdateHandler {
        &mut self.update_handler
    }

    fn receive_update(&mut self) -> HasChanged {
        false
    }

    fn box_clone(&self) -> Box<dyn UpdateTile> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct LampState {
    on: bool,
}

impl StateTile for LampState {
    fn save(&self, saver: &mut ByteBuffer) {
        self.on.save(saver);
    }

    fn load_into(&mut self, loader: &mut ByteBuffer) -> Result<(), String> {
        self.on = bool::load(loader)?;
        Ok(())
    }

    fn save_for_client(&self, saver: &mut ByteBuffer) {
        self.on.save(saver);
    }

    fn load_from_client(&mut self, loader: &mut ByteBuffer) -> Result<(), String> {
        self.on = bool::load(loader)?;
        Ok(())
    }

    fn box_clone(&self) -> Box<dyn StateTile> {
        Box::new(self.clone())
    }
}

impl LampState {
    pub fn new() -> Self {
        Self {
            on: false,
        }
    }
}