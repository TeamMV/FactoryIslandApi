use crate::inventory::InventoryData;
use crate::unit::Unit;
use hashbrown::HashMap;
use mvutils::Savable;

#[derive(Savable, PartialEq, Clone, Debug)]
pub struct Meta {
    fields: HashMap<String, MetaField>,
}

impl Meta {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: MetaField) {
        self.fields.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<&MetaField> {
        self.fields.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut MetaField> {
        self.fields.get_mut(key)
    }
}

#[derive(Savable, PartialEq, Clone, Debug)]
pub struct MetaField {
    pub key: String,
    pub value: MetaValue,
    pub unit: Unit,
}

#[derive(Savable, PartialEq, Clone, Debug)]
pub enum MetaValue {
    Str(String),
    Bool(bool),
    Integer(i32),
    Float(f32),
    Inventory(InventoryData)
}