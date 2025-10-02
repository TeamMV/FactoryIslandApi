use std::fmt::Display;
use crate::inventory::InventoryData;
use hashbrown::HashMap;
use mvutils::Savable;
use mvutils::utils::TetrahedronOp;

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

    pub fn iter(&self) -> impl Iterator<Item=(&String, &MetaField)> {
        self.fields.iter()
    }
}

#[derive(Savable, PartialEq, Clone, Debug)]
pub struct MetaField {
    pub key: String,
    pub value: MetaValue,
}

impl Display for MetaField {
    fn fmt(&self, f1: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match &self.value {
            MetaValue::Str(s) => s.clone(),
            MetaValue::Bool(b) => b.yn("True", "False").to_string(),
            MetaValue::Integer(i) => {
                i.to_string()
            }
            MetaValue::Float(f) => {
                f.to_string()
            }
            MetaValue::Inventory(_) => "Inner data".to_string(),
        };
        write!(f1, "{}", str)
    }
}

#[derive(Savable, PartialEq, Clone, Debug)]
pub enum MetaValue {
    Str(String),
    Bool(bool),
    Integer(i32),
    Float(f32),
    Inventory(InventoryData)
}