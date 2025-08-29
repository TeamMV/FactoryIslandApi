use crate::world::tiles::pos::TilePos;
use crate::world::tiles::update::UpdateHandler;
use crate::world::World;

pub type HasChanged = bool;

pub trait UpdateTile {
    fn update_handler(&mut self) -> &mut UpdateHandler;
    
    fn receive_update(&mut self) -> HasChanged;
    
    fn box_clone(&self) -> Box<dyn UpdateTile>;

    fn send_update(&mut self, at: TilePos, world: &mut World) {
        unsafe {
            if (self.update_handler().on_update(at.clone(), world)) {
                let has_changed = self.receive_update();
                if has_changed {
                    world.sync_tilestate(at.clone());
                    for pos in at.direct_neighbours() {
                        world.send_update(pos);
                    }
                }
            }
        }
    }

    fn end_tick(&mut self) {
        self.update_handler().end_tick();
    }
}