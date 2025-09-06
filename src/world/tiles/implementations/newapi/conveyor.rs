use crate::world::tiles::newapi::state::StateTile;
use bytebuffer::ByteBuffer;
use mvutils::save::Savable;

pub struct ConveyorTile {

}

#[derive(Clone)]
pub struct ConveyorState {
    ingredients: Vec<usize>
}

impl ConveyorState {
    pub fn new() -> Self {
        Self {
            ingredients: vec![],
        }
    }
}

impl StateTile for ConveyorState {
    fn save(&self, saver: &mut ByteBuffer) {
        self.ingredients.save(saver);
    }

    fn load_into(&mut self, loader: &mut ByteBuffer) -> Result<(), String> {
        self.ingredients = Vec::<usize>::load(loader)?;
        Ok(())
    }

    fn save_for_client(&self, saver: &mut ByteBuffer) {
        self.save(saver);
    }

    fn load_from_client(&mut self, loader: &mut ByteBuffer) -> Result<(), String> {
        self.load_into(loader)
    }

    fn box_clone(&self) -> Box<dyn StateTile> {
        Box::new(self.clone())
    }
}