use crate::ingredients::IngredientStack;
use crate::world::tiles::InventoryTarget;
use mvutils::utils::TetrahedronOp;
use mvutils::{utils, Savable};
use std::ops::{BitOr, BitOrAssign};

#[derive(Clone, Savable)]
pub enum InventoryOwner {
    Tile(InventoryTarget),
    Player
}

#[derive(Clone, Copy, Savable)]
#[repr(transparent)]
pub struct ItemAction(u8);

impl ItemAction {
    pub const TRANSFER_TO_PLAYER: ItemAction = ItemAction(1 << 0);
    pub const TRANSFER_FROM_PLAYER: ItemAction = ItemAction(1 << 1);
    pub const DROP: ItemAction = ItemAction(1 << 2);

    pub fn can_transfer_from_player(&self) -> bool {
        self.0 & Self::TRANSFER_FROM_PLAYER.0 > 0
    }

    pub fn can_transfer_to_player(&self) -> bool {
        self.0 & Self::TRANSFER_TO_PLAYER.0 > 0
    }

    pub fn can_drop(&self) -> bool {
        self.0 & Self::DROP.0 > 0
    }
}

impl BitOr for ItemAction {
    type Output = ItemAction;

    fn bitor(self, rhs: Self) -> Self::Output {
        ItemAction(self.0 | rhs.0)
    }
}

impl BitOrAssign for ItemAction {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Clone, Savable, PartialEq, Debug)]
pub struct InventoryData {
    pub stacks: Vec<IngredientStack>,
    pub current_amt: u64,
    pub item_limit: u64,
    pub width: u64,
    pub id: u64,
}

impl InventoryData {
    pub fn new(limit: u64, width: u64, is_player: bool) -> Self {
        Self {
            stacks: Vec::new(),
            current_amt: 0,
            item_limit: limit,
            width,
            id: is_player.yn(0, utils::next_id("InventoryId") + 1),
        }
    }

    /// WARNING 😱🚨: This function will consume the stack, whether it is added or not. Do not let it eat stacks it cannot handle!
    pub fn add_stack(&mut self, other_stack: IngredientStack) {
        if self.can_handle(&other_stack) {
            self.current_amt += other_stack.amount;
            for stack in &mut self.stacks {
                if stack.is_mergeable(&other_stack) {
                    stack.amount += other_stack.amount;
                    return;
                }
            }
            self.stacks.push(other_stack);
        }
    }

    pub fn can_handle(&self, other_stack: &IngredientStack) -> bool {
        if let Some(amount) = self.current_amt.checked_add(other_stack.amount) { amount <= self.item_limit } else { false }
    }
}