pub mod packets;

pub use crate::server::packets::{ClientBoundPacket, ServerBoundPacket};
use crate::FactoryIsland;
use abi_stable::pmr::IsAccessible::No;
use log::{debug, error, info, LevelFilter};
use mvengine::net::server::Server;
use mvutils::clock::Clock;
use parking_lot::{Condvar, Mutex};
use std::io::{stdout, BufRead};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{io, thread};

pub type FactoryIslandServer = Server<ServerBoundPacket, ClientBoundPacket>;

pub const TPS: u16 = 20;
pub const INTERNAL_PORT: u16 = 4040;

#[derive(Clone)]
pub struct ServerSync {
    pub stop_signal: Option<Arc<AtomicBool>>,
    pub semaphore: Option<Arc<(Mutex<bool>, Condvar)>>
}

impl ServerSync {
    pub fn no_sync() -> Self {
        Self {
            stop_signal: None,
            semaphore: None,
        }
    }

    pub fn new() -> Self {
        Self {
            stop_signal: Some(Arc::new(AtomicBool::new(false))),
            semaphore: Some(Arc::new((Mutex::new(false), Condvar::new()))),
        }
    }

    pub fn is_stop(&self) -> bool {
        self.stop_signal.as_ref().is_some_and(|a| a.load(Ordering::Acquire))
    }

    pub fn stop(&mut self) {
        if let Some(x) = &self.stop_signal {
            x.store(true, Ordering::Release);
        }
    }

    pub fn lock(&self) {
        if let Some(semaphore) = &self.semaphore {
            let (mutex, _) = &**semaphore;
            let mut done = mutex.lock();
            *done = false;
        }
    }

    pub fn unlock(&self) {
        if let Some(semaphore) = &self.semaphore {
            let (mutex, cvar) = &**semaphore;
            let mut done = mutex.lock();
            *done = true;
            cvar.notify_all();
        }
    }

    pub fn wait(&self) {
        if let Some(semaphore) = &self.semaphore {
            let (mutex, cvar) = &**semaphore;
            let mut done = mutex.lock();
            while !*done {
                cvar.wait(&mut done);
            }
        }
    }
}

pub fn startup_internal_server(logger: bool, mut sync: ServerSync) {
    if logger {
        mvlogger::init(stdout(), LevelFilter::Debug);
    }

    let mut server = FactoryIslandServer::new();
    let handler = server.listen::<FactoryIsland>(INTERNAL_PORT);
    let mut clock = Clock::new(TPS);

    //listening for commands in console
    let handler_cloned = handler.clone();
    let handle = thread::spawn(move || {
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
        if sync.is_stop() {
            let mut lock = handler.lock();
            info!("Stopping and saving server...");
            lock.save();
            sync.unlock();
            break;
        }
    }
}