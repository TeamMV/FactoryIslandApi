use crate::world::tiles::update::{UpdateHandler, UpdateTile};
use bytebuffer::ByteBuffer;
use mvutils::bytebuffer::ByteBufferExtras;
use mvutils::save::Savable;
use mvutils::Savable;

#[derive(Clone, Savable)]
pub struct TestUpdateTile {
    #[unsaved]
    handler: UpdateHandler,
    idk: u32
}

impl TestUpdateTile {
    pub fn new() -> Self {
        Self { handler: UpdateHandler::new(), idk: 0 }
    }
}

impl UpdateTile for TestUpdateTile {
    fn handler(&mut self) -> &mut UpdateHandler {
        &mut self.handler
    }

    fn on_update_receive(&mut self) -> bool {
        //idk
        false
    }

    fn box_clone(&self) -> Box<dyn UpdateTile> {
        let cloned = self.clone();
        Box::new(cloned)
    }

    fn save_to_vec(&self) -> Vec<u8> {
        let mut buffer = ByteBuffer::new_le();
        self.save(&mut buffer);
        buffer.into_vec()
    }

    fn load_into_self(&mut self, data: Vec<u8>) {
        let mut buffer = ByteBuffer::from_vec_le(data);
        if let Ok(s) = Self::load(&mut buffer) {
            *self = s;
        }
    }
}