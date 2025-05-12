use crate::event::Event;
use crate::world::tiles::pos::TilePos;
use crate::world::World;
use mvengine::event::EventBus;
use mvutils::save::Savable;

pub trait UpdateTile: Send + Sync {
    fn send_update(&mut self, at: TilePos, world: &mut World, event_bus: &mut EventBus<Event>) {
        if self.handler().on_update(at.clone(), world, event_bus) {
            let has_changed = self.on_update_receive();
            if has_changed { 
                for pos in at.direct_neighbours() {
                    world.send_update(pos, event_bus);
                }
            }
        }
    }
    fn end_tick(&mut self) {
        self.handler().end_tick(); 
    }

    fn handler(&mut self) -> &mut UpdateHandler;
    
    ///Should return true when the tile has changed and thus should update its neighbours
    fn on_update_receive(&mut self) -> bool;
    
    fn box_clone(&self) -> Box<dyn UpdateTile>;
    
    fn save_to_vec(&self) -> Vec<u8>;
    
    fn load_into_self(&mut self, data: Vec<u8>);
}

#[derive(Clone)]
pub struct UpdateHandler {
    received: bool
}

impl UpdateHandler {
    pub fn new() -> Self {
        Self {
            received: false,
        }
    }
    
    pub fn on_update(&mut self, at: TilePos, world: &mut World, event_bus: &mut EventBus<Event>) -> bool {
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