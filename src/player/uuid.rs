use std::fmt::{Debug, Formatter};
use uuid::{Bytes, Uuid};
use mvutils::save::{Loader, Savable, Saver};
use std::ops::{Deref, DerefMut};

#[derive(Clone, PartialEq, Hash)]
pub struct UUID {
    inner: Uuid
}

impl UUID {
    pub fn new() -> Self {
        Self {
            inner: Uuid::new_v4(),
        }
    }
}

impl Savable for UUID {
    fn save(&self, saver: &mut impl Saver) {
        self.inner.as_bytes().save(saver);
    }

    fn load(loader: &mut impl Loader) -> Result<Self, String> {
        let bytes = Bytes::load(loader)?;
        Ok(Self {
            inner: Uuid::from_bytes(bytes),
        })
    }
}

impl Deref for UUID {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for UUID {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Debug for UUID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}