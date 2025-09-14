use bytebuffer::ByteBuffer;
use crate::world::tiles::{Orientation, TileInstance};

pub mod lamp;
pub mod conveyor;
pub mod static_tile;

pub struct Air;
impl TileInstance for Air {
    fn save(&self, _: &mut ByteBuffer) {}
    fn load_into(&mut self, _: &mut ByteBuffer) -> Result<(), String> { Ok(()) }
    fn box_clone(&self) -> Box<dyn TileInstance> { Box::new(Air) }

    fn has_client_state(&self) -> bool { false }
    fn save_client_state(&self, _: &mut ByteBuffer) {}

    fn orientation(&self) -> Orientation { Orientation::North }
    fn set_orientation(&mut self, _: Orientation) {}
}