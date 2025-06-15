use bytebuffer::ByteBuffer;
use mvutils::bytebuffer::ByteBufferExtras;
use mvutils::Savable;
use mvutils::save::Savable;
use mvutils::utils::TetrahedronOp;
use crate::world::tiles::tiles::TileState;
use crate::world::tiles::update::{UpdateHandler, UpdateTile};

#[derive(Clone, Savable)]
pub struct LampState {
    on: bool
}

impl LampState {
    pub fn new() -> Self {
        LampState {
            on: false,
        }
    }
    
    pub fn on() -> Box<Self> {
        Box::new(Self {
            on: true
        })
    }
    
    pub fn off() -> Box<Self> {
        Box::new(Self {
            on: false
        })
    }
}

impl TileState for LampState {
    fn box_clone(&self) -> Box<dyn TileState> {
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

    fn client_state(&self) -> usize {
        self.on.yn(1, 0)
    }

    fn apply_client_state(&mut self, client_state: usize) {
        self.on = client_state == 1;
    }
}

#[derive(Clone, Savable)]
pub struct Lamp {
    #[unsaved]
    handler: UpdateHandler,
    state: LampState,
}

impl Lamp {
    pub fn new() -> Self {
        Lamp {
            handler: Default::default(),
            state: LampState::new(),
        }
    }
}

impl UpdateTile for Lamp {
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