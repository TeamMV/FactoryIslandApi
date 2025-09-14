use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use log::{error, warn};
use mvutils::Savable;
use parsing::xml::{parse_rsx, Entity, XmlValue};
use crate::meta::Meta;
use crate::meta::{MetaField, MetaValue};
use crate::registry::ingredients::INGREDIENT_REGISTRY;
use crate::registry::Registerable;
use crate::unit::parsing::parse_number_and_unit;
use crate::unit::Unit;
use crate::utils::AssertOnFalse;

#[derive(Clone)]
pub struct Ingredient {
    kind: IngredientKind,
    static_meta: Meta,
    default_dynamic_meta: Meta,
}

pub struct IngredientCreator {
    static_m: Meta,
    dynamic_m: Meta
}

impl IngredientCreator {
    pub fn new(static_m: Meta, dynamic_m: Meta) -> Self {
        Self { static_m, dynamic_m }
    }

    pub fn build() -> Self {
        Self {
            static_m: Meta::new(),
            dynamic_m: Meta::new(),
        }
    }

    pub fn with_static_num(mut self, key: &str, num: f32, unit: Unit) -> Self {
        self.static_m.set(key, MetaField {
            key: key.to_string(),
            value: MetaValue::Float(num),
            unit,
        });
        self
    }

    pub fn with_static_str(mut self, key: &str, value: &str) -> Self {
        self.static_m.set(key, MetaField {
            key: key.to_string(),
            value: MetaValue::Str(value.to_string()),
            unit: Unit::None,
        });
        self
    }

    pub fn with_dynamic_num(mut self, key: &str, num: f32, unit: Unit) -> Self {
        self.dynamic_m.set(key, MetaField {
            key: key.to_string(),
            value: MetaValue::Float(num),
            unit,
        });
        self
    }

    pub fn with_dynamic_str(mut self, key: &str, value: &str) -> Self {
        self.dynamic_m.set(key, MetaField {
            key: key.to_string(),
            value: MetaValue::Str(value.to_string()),
            unit: Unit::None,
        });
        self
    }

    fn get_attrib(en: &Entity, name: &str) -> String {
        let value = en.get_attrib(name).expect(&format!("{name} not found for ingredient!"));
        if let XmlValue::Str(s) = value {
            s.clone()
        } else {
            panic!("{name} is not a valid attribute for ingredient!");
        }
    }

    fn get_tree<'a>(en: &'a Entity, name: &str) -> &'a Entity {
        if let Some(XmlValue::Entities(e)) = en.inner() {
            let found = e.iter().find(|e| {
                e.name() == name
            }).expect(&format!("Didnt find {name} tag inside ingredient!"));
            found
        } else {
            panic!("{name} is required to be inside <ingredient>!");
        }
    }

    fn tree_to_meta(en: &Entity) -> Meta {
        if let Some(XmlValue::Entities(e)) = en.inner() {
            let mut meta = Meta::new();
            for inner in e {
                (inner.name() == "meta").assert("ingredient meta is composed of <meta> tags!");
                let name = Self::get_attrib(inner, "name");
                let val = Self::get_attrib(inner, "val");

                let result = parse_number_and_unit(&val);

                let (value, unit) = match result {
                    Ok((value, unit)) => (MetaValue::Float(value), unit),
                    Err(s) => (MetaValue::Str(s), Unit::None),
                };

                let parsed: MetaField = MetaField {
                    key: name.clone(),
                    value,
                    unit,
                };
                meta.set(&name, parsed);
            }
            meta
        } else {
            Meta::new()
        }
    }

    pub fn read(xml: &str) -> Self {
        let en = parse_rsx(xml.to_string()).unwrap();
        (en.name() == "ingredient").assert("Invalid XML for ingredient");
        let statics = Self::get_tree(&en, "static");
        let static_m = Self::tree_to_meta(statics);

        let dynamics = Self::get_tree(&en, "dynamic");
        let dynamic_m = Self::tree_to_meta(dynamics);

        Self::new(static_m, dynamic_m)
    }
}

impl Registerable for Ingredient {
    type CreateInfo = IngredientCreator;

    fn with_id(id: usize, info: Self::CreateInfo) -> Self {
        Self {
            kind: id,
            static_meta: info.static_m,
            default_dynamic_meta: info.dynamic_m,
        }
    }
}

pub type IngredientKind = usize;

#[derive(Clone, Savable, PartialEq)]
pub struct IngredientStack {
    pub ingredient: IngredientKind,
    pub amount: u64,
    pub meta: Meta
}

impl IngredientStack {
    pub fn new(ingredient: IngredientKind, amount: u64) -> Self {
        if let Some(obj) = INGREDIENT_REGISTRY.reference_object(ingredient) {
            Self {
                ingredient,
                amount,
                meta: obj.default_dynamic_meta.clone(),
            }
        } else {
            warn!("Tried to create a new IngredientStack with illegal ingredient: {ingredient}!");
            Self {
                ingredient,
                amount,
                meta: Meta::new(),
            }
        }
    }

    pub fn is_mergeable(&self, other: &IngredientStack) -> bool {
        self.ingredient == other.ingredient &&
            self.meta == other.meta &&
            self.amount.checked_add(other.amount).is_some()
    }

    pub fn get_static_meta(&self) -> &Meta {
        if let Some(ing) = INGREDIENT_REGISTRY.reference_object(self.ingredient) {
            &ing.static_meta
        } else {
            warn!("Tried to obtain meta for an IngredientStack with illegal ingredient: {}!", self.ingredient);
            //No other way. give up and die
            &self.meta
        }
    }
}

impl Debug for IngredientStack {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s_meta = self.get_static_meta();
        f.debug_struct("IngredientStack")
            .field("ingredient", &self.ingredient)
            .field("amount", &self.amount)
            .field("meta", &self.meta)
            .field("static_meta", s_meta)
            .finish()
    }
}