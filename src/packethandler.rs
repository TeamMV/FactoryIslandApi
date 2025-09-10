use std::sync::Arc;
use log::debug;
use mvengine::net::server::ClientEndpoint;
use crate::{FactoryIsland, PLAYERS};
use crate::server::packets::common::PlayerData;
use crate::server::{ClientBoundPacket, ServerBoundPacket};
use crate::server::packets::player::{OtherPlayerChatPacket, OtherPlayerJoinPacket, OtherPlayerMovePacket};

pub struct PacketHandler;

impl PacketHandler {
    pub fn check_packet(packet: ServerBoundPacket, client: &Arc<ClientEndpoint>, fi: &mut FactoryIsland) -> Option<ServerBoundPacket> {
        let mut players = PLAYERS.write();
        match packet {
            ServerBoundPacket::ClientData(packet) => {
                debug!("Client data packet arrived");
                if let Some(player) = players.get(&client.id()) {
                    let mut lock = player.lock();
                    lock.apply_data(packet.clone());

                    let id = client.id();
                    debug!("starting client join message");
                    for (_, other_player) in players.iter().filter(|(p, _)| **p != id) {
                        let lock = other_player.lock();
                        if let Some(endpoint) = lock.client_endpoint() {
                            endpoint.send(ClientBoundPacket::OtherPlayerJoin(OtherPlayerJoinPacket {
                                client_id: id,
                                client_data: packet.clone()
                            }));
                        }
                    }
                    debug!("finished client join message");
                }
            }
            ServerBoundPacket::PlayerMove(packet) => {
                if let Some(player) = players.get(&client.id()) {
                    let pos = packet.pos;

                    let pos = packet.pos;

                    let mut lock = player.lock();
                    lock.move_to(pos);
                    drop(lock);

                    for (_, other_player) in players.iter().filter(|(p, _)| **p != client.id()) {
                        let lock = other_player.lock();
                        if let Some(endpoint) = lock.client_endpoint() {
                            endpoint.send(ClientBoundPacket::OtherPlayerMove(OtherPlayerMovePacket {
                                client_id: client.id(),
                                pos,
                            }));
                        }
                    }
                }
            }
            ServerBoundPacket::PlayerChat(packet) => {
                if let Some(player) = players.get(&client.id()) {
                    let lock = player.lock();
                    let client_data = lock.data.clone();
                    drop(lock);
                    let data = PlayerData {
                        client_id: client.id(),
                        data: client_data,
                    };
                    if packet.message.chars().next() == Some('/') {
                        let command = packet.message[1..].trim().to_string();
                        fi.on_command(command, Some(data));
                    } else {
                        for (_, other_player) in players.iter() {
                            let lock = other_player.lock();
                            if let Some(endpoint) = lock.client_endpoint() {
                                endpoint.send(ClientBoundPacket::OtherPlayerChat(OtherPlayerChatPacket {
                                    player: data.clone(),
                                    message: packet.message.clone(),
                                }));
                            }
                        }
                    }
                }
            }
            ServerBoundPacket::RequestReload => {
                if let Some(player) = players.get(&client.id()) {
                    let mut lock = player.lock();
                    lock.loaded_chunks.clear();
                    let rdst = lock.data.render_distance;
                    lock.after_move(rdst);
                }
            }
            other => return Some(other),
        };
        None
    }
}