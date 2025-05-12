use std::sync::atomic::{AtomicBool, Ordering};
use mvutils::save::{Loader, Savable, Saver};
use parking_lot::RwLock;
use crate::registry::terrain::TerrainTiles;
use crate::registry::tiles::Tiles;

pub mod terrain;
pub mod tiles;

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

    pub fn len(&self) -> usize {
        self.objects.read().len()
    }
}

pub trait Registerable: Clone + Savable {
    type CreateInfo;

    fn with_id(id: usize, info: Self::CreateInfo) -> Self;

    fn load_with_registry(loader: &mut impl Loader, registry: &Registry<Self>) -> Result<Self, String> {
        if let Some(id) = loader.pop_u16() {
            if let Some(t) = registry.create_object(id as usize) {
                Ok(t)
            } else {
                Err(format!("Object with id {id} is not registered!"))
            }
        } else {
            Err("Couldnt read object id".to_string())
        }
    }
}

impl<T: Registerable> Savable for Registry<T> {
    fn save(&self, saver: &mut impl Saver) {
        let lock = self.objects.read();
        lock.save(saver);
    }

    fn load(loader: &mut impl Loader) -> Result<Self, String> {
        let objects = Vec::<T>::load(loader)?;
        let objects = RwLock::new(objects);
        Ok(Self {
            locked: AtomicBool::new(true),
            objects,
        })
    }
}

#[derive(Clone)]
pub struct GameObjects {
    pub terrain: TerrainTiles,
    pub tiles: Tiles
}