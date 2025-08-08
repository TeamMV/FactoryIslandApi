use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use abi_stable::std_types::RString;
use hashbrown::HashMap;
use mvutils::hashers::U64IdentityHasher;
use mvutils::lazy;
use parking_lot::Mutex;

pub mod events;
pub mod player;
pub mod world;

pub use events::*;
pub use player::*;
pub use world::*;

#[repr(C)]
pub(crate) struct InnerModCtx {
    pub(crate) id: RString
}

pub type ModCtx = *const InnerModCtx;

pub type ModData = *mut ();

#[repr(C)]
pub struct FiVTable {

}

#[repr(C)]
pub enum MOpt<T> {
    None,
    Some(T)
}

impl<T> MOpt<T> {
    pub fn to_rust(self) -> Option<T> {
        match self {
            MOpt::None => None,
            MOpt::Some(t) => Some(t)
        }
    }
}

pub trait ToMOpt<T> {
    fn to_m(self) -> MOpt<T>;
}

impl<T> ToMOpt<T> for Option<T> {
    fn to_m(self) -> MOpt<T> {
        match self {
            None => MOpt::None,
            Some(t) => MOpt::Some(t)
        }
    }
}

#[repr(C)]
pub struct MUniqueAccess<T> {
    pub lock_handle: LockHandle,
    pub data: T
}

pub type LockHandle = u64;

lazy! {
    pub(crate) static LOCKS: Mutex<HashMap<u64, Box<dyn Any>, U64IdentityHasher>> = Mutex::new(HashMap::with_hasher(U64IdentityHasher::default()));
    pub(crate) static LOCK_ID: AtomicU64 = AtomicU64::new(0);
}

pub(crate) fn store_lock(lock: dyn Any) -> LockHandle {
    let mut locks_lock = LOCKS.lock();
    let num = LOCK_ID.fetch_add(1, Ordering::Acquire);
    locks_lock.insert(num, Box::new(lock));
    num
}

#[no_mangle]
pub extern "C" fn fim_free_access(handle: LockHandle) {
    let mut locks_lock = LOCKS.lock();
    locks_lock.remove(&handle);
}