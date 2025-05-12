use bytebuffer::ByteBuffer;
use mvengine::net::server::ClientId;
use mvutils::Savable;

#[derive(Clone, Debug, Savable)]
pub struct ClientDataPacket {
    pub name: String,
    pub render_distance: i32,
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
}