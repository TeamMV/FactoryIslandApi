use crate::registry::Registry;
use mvutils::lazy;
use crate::world::tiles::implementations::Air;
use crate::world::tiles::implementations::conveyor::Conveyor;
use crate::world::tiles::implementations::lamp::Lamp;
use crate::world::tiles::implementations::static_tile::StaticTile;
use crate::world::tiles::Tile;

lazy! {
    pub static TILE_REGISTRY: Registry<Tile> = Registry::new();
}

macro_rules! define_tiles {
    ($struct_name:ident, $func_name:ident, [$($tile_name:ident = $tile_init:expr),* $(,)?]) => {
        #[derive(Clone)]
        pub struct $struct_name {
            $(pub $tile_name: usize),*
        }

        pub fn $func_name() -> $struct_name {
            $struct_name {
                $(
                    $tile_name: TILE_REGISTRY.register(Box::new($tile_init)),
                )*
            }
        }
    };
}

define_tiles!(Tiles, register_all, [
    air = Air,
    wood = StaticTile::new(100.0),
    lamp = Lamp::new(),
    conveyor = Conveyor::new()
]);