use crate::ingredients::IngredientKind;
use crate::player::profile::PlayerProfile;
pub use crate::world::tiles::TileKind;
use mvengine::net::server::ClientId;
use mvutils::Savable;
use std::hash::Hash;

#[derive(Clone, Debug, Savable)]
pub struct ClientDataPacket {
    pub profile: PlayerProfile,
    pub render_distance: i32,
    pub client_id: ClientId,
}

#[derive(Clone, Savable)]
pub struct PlayerData {
    pub client_id: ClientId,
    pub data: ClientDataPacket
}

#[derive(Clone, Savable)]
pub struct ServerStatePacket {
    pub players: Vec<PlayerData>,
    pub tiles: Vec<TileKind>,
    pub ingredients: Vec<IngredientKind>,
    pub client_id: ClientId
}