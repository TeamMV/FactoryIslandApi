use bytebuffer::ByteBuffer;
use mvutils::bytebuffer::ByteBufferExtras;
use mvutils::Savable;
use mvutils::save::Savable;
use crate::world::tiles::update::{UpdateHandler, UpdateTile};

#[derive(Clone, Savable)]
pub struct Generator {
    #[unsaved]
    handler: UpdateHandler,

}

impl Generator {
    pub fn new() -> Self {
        Self {
            handler: UpdateHandler::new(),
        }
    }
}

impl UpdateTile for Generator {
    fn handler(&mut self) -> &mut UpdateHandler {
        &mut self.handler
    }

    fn on_update_receive(&mut self) -> bool {
        false
    }

    fn box_clone(&self) -> Box<dyn UpdateTile> {
        Box::new(self.clone())
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