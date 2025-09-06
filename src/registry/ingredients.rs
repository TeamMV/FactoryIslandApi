use mvutils::lazy;
use crate::ingredients::Ingredient;
use crate::registry::Registry;

lazy! {
    pub static INGREDIENT_REGISTRY: Registry<Ingredient> = Registry::new();
}

macro_rules! define_ingredients {
    ($struct_name:ident, $func_name:ident, [$($ingredient_name:ident),* $(,)?]) => {
        #[derive(Clone)]
        pub struct $struct_name {
            $(pub $ingredient_name: usize),*
        }

        pub fn $func_name() -> $struct_name {
            $struct_name {
                $(
                    $ingredient_name: INGREDIENT_REGISTRY.register(()),
                )*
            }
        }
    };
}

define_ingredients!(Ingredients, register_all, [
    stone
]);