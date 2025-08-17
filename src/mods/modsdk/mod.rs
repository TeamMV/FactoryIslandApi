use std::any::Any;
use std::{alloc, ptr};
use std::alloc::Layout;
use std::ops::{ControlFlow, FromResidual, Try};
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

impl<T> Try for MOpt<T> {
    type Output = T;
    type Residual = MOpt<core::convert::Infallible>;

    fn from_output(output: Self::Output) -> Self {
        MOpt::Some(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            MOpt::Some(v) => ControlFlow::Continue(v),
            MOpt::None => ControlFlow::Break(MOpt::None),
        }
    }
}

impl<T> FromResidual<MOpt<core::convert::Infallible>> for MOpt<T> {
    fn from_residual(residual: MOpt<core::convert::Infallible>) -> Self {
        match residual {
            MOpt::None => MOpt::None,
            MOpt::Some(_) => unreachable!(),
        }
    }
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
    pub(crate) static LOCKS: Mutex<HashMap<u64, Box<dyn Any + Send>, U64IdentityHasher>> = Mutex::new(HashMap::with_hasher(U64IdentityHasher::default()));
    pub(crate) static LOCK_ID: AtomicU64 = AtomicU64::new(0);
}

struct FuckURustThisWholeCodebaseIsNotSafeSoDontComplainAbtThisOneThingFor500000Years<T> {
    t: T
}

unsafe impl<T> Send for FuckURustThisWholeCodebaseIsNotSafeSoDontComplainAbtThisOneThingFor500000Years<T> {}

pub(crate) fn store_lock<T: Any>(lock: T) -> LockHandle {
    let mut locks_lock = LOCKS.lock();
    let num = LOCK_ID.fetch_add(1, Ordering::Acquire);
    let f = FuckURustThisWholeCodebaseIsNotSafeSoDontComplainAbtThisOneThingFor500000Years { t: lock };
    locks_lock.insert(num, Box::new(f));
    num
}

#[no_mangle]
pub extern "C" fn fim_free_access(handle: LockHandle) {
    let mut locks_lock = LOCKS.lock();
    locks_lock.remove(&handle);
}

#[no_mangle]
pub extern "C" fn fim_allocate(size: usize, alignment: usize) -> *mut () {
    unsafe {
        let num_size = size_of::<u64>();
        let layout = Layout::from_size_align_unchecked(size + num_size + num_size, alignment);
        let ptr = alloc::alloc(layout) as *mut u64;
        ptr.write(size as u64);
        let ptr = ptr.add(num_size);
        ptr.write(alignment as u64);
        let ptr = ptr.add(num_size);
        ptr as *mut ()
    }
}

#[no_mangle]
pub extern "C" fn fim_free(pointer: *mut ()) {
    unsafe {
        let num_size = size_of::<u64>();
        let actual_ptr = pointer.sub(num_size + num_size) as *mut u64;
        let size = actual_ptr.read();
        let actual_ptr = actual_ptr.add(num_size);
        let align = actual_ptr.read();
        let layout = Layout::from_size_align_unchecked(size as usize + num_size + num_size, align as usize);
        alloc::dealloc(pointer as *mut u8, layout);
    }
}