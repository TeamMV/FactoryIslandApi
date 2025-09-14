use crate::inventory::{InventoryData, InventoryOwner, ItemAction};
use mvutils::Savable;
use crate::player::uuid::UUID;

//C2S
#[derive(Clone, Savable)]
pub struct InventoryOpenPacket {
    pub owner: InventoryOwner,
}

#[derive(Clone, Savable)]
pub struct InventoryItemActionPacket {
    pub inventory: u64,
    pub action: ItemAction,
    pub stack: u64,
    pub amount: u64,
    pub request_id: UUID,
}

//S2C
#[derive(Clone, Savable)]
pub struct InventoryDataPacket {
    pub data: InventoryData,
    pub player_inventory: Option<InventoryData>,
    pub owner: InventoryOwner,
    pub item_actions: ItemAction,
}

#[derive(Clone, Savable)]
pub struct InventoryItemActionResponsePacket {
    pub success: bool,
    pub request_id: UUID
}