use hashbrown::HashMap;
use mvutils::Savable;
use crate::unit::Unit;

#[derive(Savable, PartialEq, Clone, Debug)]
pub struct IngredientMeta {
    fields: HashMap<String, MetaField>,
}

impl IngredientMeta {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: MetaField) {
        self.fields.insert(key.to_string(), value);
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
    Number(f32),
}