use std::alloc::Layout;
use std::{alloc, ptr};
use abi_stable::std_types::RVec;
use mvutils::utils::TetrahedronOp;
use crate::{p, leak, rvec_load, rvec_save, ptr_invoke_clone};
use crate::mods::modsdk::MOpt;
use crate::world::tiles::tiles::{ObjControl, ObjControlTrait, TileState, TileStateTrait};
use crate::world::tiles::update::{This, UpdateHandler, UpdateTile, UpdateTileTrait};

pub struct LampTile {
    handler: *mut UpdateHandler,
    on: bool
}

impl LampTile {
    pub fn new() -> Self {
        Self {
            handler: leak!(UpdateHandler::new()),
            on: false,
        }
    }
}

impl UpdateTile for LampTile {
    fn create_update_trait() -> UpdateTileTrait {
        UpdateTileTrait {
            get_handler,
            on_update_receive,
        }
    }
}

impl TileState for LampTile {
    fn create_state_trait() -> TileStateTrait {
        TileStateTrait {
            save_to_vec,
            load_into_self,
            client_state,
            apply_client_state,
        }
    }
}

impl ObjControl for LampTile {
    fn create_oc_trait() -> ObjControlTrait {
        ObjControlTrait {
            create_copy,
            free,
        }
    }
}

pub fn get_handler(this: This) -> *mut UpdateHandler {
    let tile = p!(this as LampTile);
    tile.handler
}

pub fn on_update_receive(this: This) {

}
pub fn save_to_vec(this: This) -> RVec<u8> {
    let tile = p!(this as LampTile);
    rvec_save!(tile => on,)
}

pub fn load_into_self(vec: RVec<u8>, this: This) -> MOpt<()> {
    let tile = p!(this as LampTile);
    rvec_load!(vec for tile => {
        on: bool,
    });
    MOpt::Some(())
}

pub fn client_state(this: This) -> usize {
    let tile = p!(this as LampTile);
    tile.on.yn(1, 0)
}

pub fn apply_client_state(this: This, state: usize) {
    let tile = p!(this as LampTile);
    tile.on = state == 1;
}

pub fn create_copy(this: This) -> This {
    let tile = p!(this as LampTile);
    let handler = ptr_invoke_clone!(tile.handler);
    let new = LampTile {
        handler,
        on: tile.on,
    };
    let p = leak!(new);
    p!(p)
}

pub unsafe fn free(this: This) {
    let tile = p!(this as LampTile);
    let handler_lay = Layout::for_value(&*tile.handler);
    ptr::drop_in_place(tile.handler);
    alloc::dealloc(tile.handler as *mut u8, handler_lay);
    let tile_lay = Layout::for_value(tile);
    ptr::drop_in_place(this as *mut LampTile);
    alloc::dealloc(this as *mut u8, tile_lay);
}