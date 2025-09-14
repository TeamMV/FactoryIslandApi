use crate::world::tiles::pos::TilePos;
use crate::world::World;
use mvutils::save::Savable;

#[derive(Clone)]
#[repr(C)]
pub struct UpdateHandler {
    received: bool
}

impl UpdateHandler {
    pub fn new() -> Self {
        Self {
            received: false,
        }
    }
    
    pub fn on_update(&mut self, at: TilePos, world: &mut World) -> bool {
        if !self.received {
            self.received = true;
            return true;
        }
        false
    }
    
    pub(crate) fn end_tick(&mut self) {
        self.received = false;
    }
}

impl Default for UpdateHandler {
    fn default() -> Self {
        Self::new()
    }
}