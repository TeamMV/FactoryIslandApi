pub mod packets;

use std::io::{stdout, BufRead};
use std::{io, thread};
use log::{debug, error, LevelFilter};
pub use crate::server::packets::{ClientBoundPacket, ServerBoundPacket};
use mvengine::net::server::Server;
use mvutils::clock::Clock;
use crate::FactoryIsland;

pub type FactoryIslandServer = Server<ServerBoundPacket, ClientBoundPacket>;

pub const TPS: u16 = 20;
pub const INTERNAL_PORT: u16 = 4040;

pub fn startup_internal_server() {
    mvlogger::init(stdout(), LevelFilter::Debug);

    let mut server = FactoryIslandServer::new();
    let handler = server.listen::<FactoryIsland>(INTERNAL_PORT);
    let mut clock = Clock::new(TPS);

    //listening for commands in console
    let handler_cloned = handler.clone();
    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(command) => {
                    let mut lock = handler_cloned.lock();
                    lock.on_command(command, None);
                }
                Err(e) => error!("Error reading line: {}", e),
            }
        }
    });

    loop {
        if clock.ready_and_tick() {
            let mut lock = handler.lock();
            lock.tick();
        }
    }
}