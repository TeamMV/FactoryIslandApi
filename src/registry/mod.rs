use std::sync::atomic::{AtomicBool, Ordering};
use abi_stable::std_types::RString;
use mvutils::Savable;
use mvutils::save::{Loader, Savable, Saver};
use mvutils::unsafe_utils::Unsafe;
use parking_lot::RwLock;
use crate::registry::ingredients::Ingredients;
use crate::registry::terrain::TerrainTiles;
use crate::registry::tiles::Tiles;

pub mod terrain;
pub mod tiles;
pub mod ingredients;

pub struct Registry<T: Registerable> {
    locked: AtomicBool,
    objects: RwLock<Vec<T>>
}

impl<T: Registerable> Registry<T> {
    pub fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
            objects: RwLock::new(vec![]),
        }
    }

    pub(crate) fn lock(&self) {
        self.locked.store(true, Ordering::Release);
    }

    pub fn register(&self, info: T::CreateInfo) -> usize {
        if !self.locked.load(Ordering::Acquire) {
            let mut vec = self.objects.write();
            let idx = vec.len();
            let object = T::with_id(idx, info);
            vec.push(object);
            idx
        } else {
            0
        }
    }

    pub fn create_object(&self, id: usize) -> Option<T> {
        let vec = self.objects.read();
        vec.get(id).cloned()
    }

    pub fn reference_object(&self, id: usize) -> Option<&T> {
        let vec = self.objects.read();
        let r = vec.get(id);
        match r {
            None => None,
            Some(r) => Some(unsafe { Unsafe::cast_lifetime(r) })
        }
    }

    pub fn len(&self) -> usize {
        self.objects.read().len()
    }
}

pub trait Registerable: Clone {
    type CreateInfo;

    fn with_id(id: usize, info: Self::CreateInfo) -> Self;
}

#[derive(Clone)]
pub struct GameObjects {
    pub terrain: TerrainTiles,
    pub tiles: Tiles,
    pub ingredients: Ingredients
}

#[derive(Clone, Debug, Savable)]
#[repr(C)]
pub enum ObjectSource {
    Vanilla,
    Mod(RString)
}