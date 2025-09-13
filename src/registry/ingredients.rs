use mvutils::lazy;
use crate::ingredients::{Ingredient, IngredientCreator};
use crate::unit::{Unit, UnitPrefix, KELVIN_CELSIUS_OFFSET};
use crate::registry::Registry;

lazy! {
    pub static INGREDIENT_REGISTRY: Registry<Ingredient> = Registry::new();
}

macro_rules! define_ingredients {
    ($struct_name:ident, $func_name:ident, [$($ingredient_name:ident = $($creator:expr)*),* $(,)?]) => {
        #[derive(Clone)]
        pub struct $struct_name {
            $(pub $ingredient_name: usize),*
        }

        pub fn $func_name() -> $struct_name {
            $struct_name {
                $(
                    $ingredient_name: INGREDIENT_REGISTRY.register($($creator)*),
                )*
            }
        }
    };
}

define_ingredients!(Ingredients, register_all, [
    stone = IngredientCreator::read(include_str!("files/ingredients/stone.xml")),
]);
