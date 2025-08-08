use mvengine::net::server::ClientId;
use mvutils::Savable;
use crate::registry::ObjectSource;

#[derive(Clone, Debug, Savable)]
pub struct ClientDataPacket {
    pub name: String,
    pub render_distance: i32,
    pub client_id: ClientId
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
    pub tiles: Vec<TileKind>
}

#[derive(Clone, Savable)]
pub struct TileKind {
    pub id: usize,
    pub source: ObjectSource
}