use std::hash::{Hash, Hasher};
use crate::ingredients::IngredientKind;
use crate::player::uuid::UUID;
use crate::registry::ObjectSource;
use mvengine::net::server::ClientId;
use mvutils::Savable;
use crate::player::profile::PlayerProfile;

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
    pub mods: Vec<String>,
    pub players: Vec<PlayerData>,
    pub tiles: Vec<TileKind>,
    pub ingredients: Vec<IngredientKind>,
    pub client_id: ClientId
}

#[derive(Clone, Savable)]
pub struct TileKind {
    pub id: usize,
    pub source: ObjectSource
}

impl Hash for TileKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u16(self.id as u16);
    }
}