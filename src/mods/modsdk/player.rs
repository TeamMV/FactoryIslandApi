use std::any::Any;
use std::mem;
use abi_stable::pointer_trait::TransmuteElement;
use crate::mods::modsdk::{MUniqueAccess, MOpt, store_lock};
use crate::world::{TileUnit, World};
use crate::PLAYERS;
use abi_stable::std_types::RString;
use abi_stable::traits::IntoReprC;
use mvengine::net::server::ClientId;
use mvutils::unsafe_utils::Unsafe;
use parking_lot::{MutexGuard, RawMutex};
use crate::mods::modsdk::world::{MTileUnit, MWorld};

pub type MPlayer = ClientId;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct MPlayerData {
    pub name: RString,
    pub pos: MTileUnit
}

#[no_mangle]
pub extern "C" fn fim_player_data(player: MPlayer) -> MOpt<MPlayerData> {
    let players = PLAYERS.read();
    if let Some(p) = players.get(&player) {
        let p_lock = p.lock();
        let name = p_lock.data.profile.name.clone().into_c();
        let pos = p_lock.position;
        let pos = MTileUnit::from_normal(pos);
        MOpt::Some(MPlayerData {
            name,
            pos,
        })
    } else {
        MOpt::None
    }
}

#[no_mangle]
pub extern "C" fn fim_player_move_to(player: MPlayer, pos: MTileUnit) {
    let players = PLAYERS.read();
    if let Some(p) = players.get(&player) {
        let mut p_lock = p.lock();
        let pos = pos.to_normal();
        p_lock.move_to(pos);
    }
}

#[no_mangle]
pub extern "C" fn fim_player_move_by(player: MPlayer, pos: MTileUnit) {
    let players = PLAYERS.read();
    if let Some(p) = players.get(&player) {
        let mut p_lock = p.lock();
        let pos = pos.to_normal();
        p_lock.move_by(pos);
    }
}

#[no_mangle]
pub extern "C" fn fim_player_world(player: MPlayer) -> MOpt<MUniqueAccess<MWorld>> {
    let players = PLAYERS.read();
    if let Some(p) = players.get(&player) {
        let mut p_lock = p.lock();
        if let Some(world) = p_lock.world() {
            let mut world_lock = world.lock();
            let ptr: MWorld = &mut *world_lock;
            let tr = unsafe {
                mem::transmute::<_, MutexGuard<'static, World>>(world_lock)
            };
            let handle = store_lock(tr);
            return MOpt::Some(MUniqueAccess {
                lock_handle: handle,
                data: ptr,
            });
        }
    }
    MOpt::None
}