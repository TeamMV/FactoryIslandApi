use crate::world::tiles::pos::TilePos;
use crate::world::World;
use mvutils::save::Savable;

pub type This = *mut ();

#[derive(Clone)]
#[repr(C)]
pub struct UpdateTileTrait {
    pub get_handler: fn(This) -> *mut UpdateHandler,
    pub on_update_receive: fn(This) -> bool,
}

impl UpdateTileTrait {
    pub(crate) fn send_update(&self, this: This, at: TilePos, world: &mut World) {
        unsafe {
            if (self.get_handler)(this).as_mut().is_some_and(|x| x.on_update(at.clone(), world)) {
                let has_changed = (self.on_update_receive)(this);
                if has_changed {
                    world.sync_tilestate(at.clone());
                    for pos in at.direct_neighbours() {
                        world.send_update(pos);
                    }
                }
            }
        }
    }

    pub(crate) fn end_tick(&self, this: This) {
        unsafe {
            let h = (self.get_handler)(this);
            if let Some(r) = h.as_mut() {
                r.end_tick();
            }
        }
    }
}

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

pub trait UpdateTile {
    fn create_update_trait() -> UpdateTileTrait;
}