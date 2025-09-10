use crate::registry::Registerable;
use crate::world::tiles::pos::{TileDistance, TilePos};
use crate::world::{TileExtent, World};
use hashbrown::HashMap;
use mvutils::Savable;
use std::hash::{BuildHasher, Hasher};
use std::process::exit;
use hashbrown::hash_map::Entry;
use log::error;
use crate::player::uuid::UUID;

pub type Amount = usize;

#[derive(Clone)]
struct TileKindHasher {
    id: u16
}

impl Default for TileKindHasher {
    fn default() -> Self {
        Self { id: 0 }
    }
}

impl Hasher for TileKindHasher {
    fn finish(&self) -> u64 {
        self.id as u64
    }

    fn write(&mut self, _: &[u8]) {
        panic!("Nah this hasher can only be used for Tile kinds imma be real with u right here")
    }

    fn write_u16(&mut self, i: u16) {
        self.id = i;
    }
}

impl BuildHasher for TileKindHasher {
    type Hasher = Self;

    fn build_hasher(&self) -> Self::Hasher {
        Self::default()
    }
}

#[derive(Clone)]
pub struct MultiTile {
    pub id: usize,
    pub required: HashMap<u16, Amount, TileKindHasher>,
    pub size: TileExtent,
}

impl MultiTile {
    pub fn new(id: usize, extent: TileExtent, required: &[(usize, Amount)]) -> Self {
        let mut req = HashMap::with_capacity_and_hasher(required.len(), TileKindHasher::default());
        for (id, amt) in required {
            req.insert(*id as u16, *amt);
        }
        if req.values().sum::<usize>() as i32 != extent.0 * extent.1 {
            panic!("IllegalMultiblockException at package.dev.mv.api.server.multitiles.mod:57\nTf kinda multiblock did you just define huh?\nStack trace:\nCalled by package.dev.mv.api.server.multiline.mod:53 MultiTile::new\nCalled by class.loader.loadClass<?>(?):unknown");
        }
        Self {
            id,
            required: req,
            size: extent,
        }
    }

    pub fn check_completion(&self, world: &mut World, pos: TilePos, placed: u16) -> Option<MultiTilePlacement> {
        if !self.required.contains_key(&placed) {
            return None;
        }

        if world.is_multitile_at(&pos) {
            return None;
        }

        let extents: &[TileExtent] = if self.size.0 != self.size.1 {
            &[(self.size.0, self.size.1), (self.size.1, self.size.0)]
        } else {
            &[(self.size.0, self.size.1)]
        };

        for extent in extents {
            for x in 0..extent.0 {
                'check:
                for y in 0..extent.1 {
                    let bottom_left = pos.left(x).down(y);

                    let mut required = self.required.clone();

                    for i in 0..extent.0 {
                        for j in 0..extent.1 {
                            let tile = bottom_left.right(i).up(j);
                            if world.is_multitile_at(&tile) {
                                continue 'check;
                            }
                            if let Some(tile) = world.get_tile_id_at(tile) {
                                if let Entry::Occupied(mut entry) = required.entry(tile) {
                                    let amount = entry.get_mut();
                                    *amount -= 1;
                                    if *amount == 0usize {
                                        entry.remove();
                                    }
                                } else {
                                    continue 'check;
                                }
                            } else {
                                continue 'check;
                            }
                        }
                    }
                    return Some(MultiTilePlacement::new(0, bottom_left, extent.clone()));
                }
            }
        }
        
        None
    }
}

pub struct MultiTileCreateInfo {
    pub extent: TileExtent,
    pub required: Vec<(usize, Amount)>
}

impl MultiTileCreateInfo {
    pub fn new(extent: TileExtent, required: &[(usize, Amount)]) -> Self {
        Self {
            extent,
            required: required.to_vec(),
        }
    }
}

impl Registerable for MultiTile {
    type CreateInfo = MultiTileCreateInfo;

    fn with_id(id: usize, info: Self::CreateInfo) -> Self {
        Self::new(id, info.extent, &info.required)
    }
}

#[derive(Clone, Savable)]
pub struct MultiTilePlacement {
    pub uuid: UUID,
    pub mt_id: usize,
    pub pos: TilePos,
    pub extent: TileExtent,
}

impl MultiTilePlacement {
    pub fn new(multiblock: u16, pos: TilePos, extent: TileExtent) -> Self {
        Self {
            uuid: UUID::new(),
            mt_id: multiblock as usize,
            pos,
            extent,
        }
    }

    // helloAABBCollider ahh code
    pub fn includes(&self, pos: &TilePos) -> bool {
        pos.raw.0 >= self.pos.raw.0 &&
            pos.raw.0 < self.pos.raw.0 + self.extent.0 &&
            pos.raw.1 >= self.pos.raw.1 &&
            pos.raw.1 < self.pos.raw.1 + self.extent.1
    }
}

/*


Multiblock HelloMultiBlock:

Requires:
1x A tile
2x Hello tile
1x World tile
2x HelloTile tile

(2x3)/(3x2)

Valid structures:

A H  W
H HT HT

A HT
W HT
H H

A H
W HT
HT H

*/
