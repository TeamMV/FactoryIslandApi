use mvutils::Savable;
use crate::registry::Registerable;

#[derive(Clone)]
pub struct Ingredient {
    id: usize,
}

impl Registerable for Ingredient {
    type CreateInfo = ();

    fn with_id(id: usize, info: Self::CreateInfo) -> Self {
        Self {
            id,
        }
    }
}

#[derive(Clone, Savable)]
pub struct IngredientKind {
    pub id: usize,
}