pub mod players;
pub mod chunks;
pub mod stop;
pub mod save;
pub mod commands;

use std::collections::HashMap;
use log::{debug, info, warn};
use mvutils::{enum_val, lazy};
use mvutils::unsafe_utils::DangerousCell;
use parking_lot::RwLock;
use crate::command::chunks::ChunksCommand;
use crate::command::commands::CommandsCommand;
use crate::command::players::PlayersCommand;
use crate::command::save::SaveCommand;
use crate::command::stop::StopCommand;
use crate::FactoryIsland;
use crate::registry::ObjectSource;
use crate::server::packets::common::PlayerData;

lazy! {
    pub static COMMAND_PROCESSOR: CommandProcessor = CommandProcessor::new();
}

struct InnerProcessor {
    commands: Vec<Command>,
    key_map: HashMap<String, usize>
}

impl InnerProcessor {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            key_map: HashMap::new(),
        }
    }
}

pub struct CommandProcessor {
    inner: RwLock<InnerProcessor>
}

impl CommandProcessor {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(InnerProcessor::new()),
        }
    }

    pub fn register(&self, cmd: Command, source: &ObjectSource) {
        let mut this = self.inner.write();
        let id = this.commands.len();
        if this.key_map.try_insert(cmd.key.clone(), id).is_err() {
            warn!("Command {} is already taken, skipping registering raw command", cmd.key);
        }

        for alias in &cmd.aliases {
            if this.key_map.try_insert(alias.clone(), id).is_err() {
                warn!("Command alias {alias} for command {} is already taken and will not be registered", cmd.key);
            }
        }

        if let ObjectSource::Mod(mod_id) = source {
            this.key_map.insert(format!("{mod_id}:{}", cmd.key), id);
            for alias in &cmd.aliases {
                this.key_map.insert(format!("{mod_id}:{alias}"), id);
            }
        }

        this.commands.push(cmd);
    }

    pub fn process(&self, sender: CommandSender, command: String, fi: &mut FactoryIsland) {
        let mut this = self.inner.write();
        let binding = command.clone();
        let mut parts = binding.split_whitespace();
        if let Some(cmd) = parts.next() {
            let cmd = cmd.to_string();
            let cmd_b = cmd.as_bytes();
            debug!("Processing command: '{cmd}'");
            debug!("Processing command dbg: '{cmd_b:?}'");
            debug!("Command list: {:?}", this.key_map);
            debug!("Contains: {}", this.key_map.contains_key(&cmd));
            if let Some(id) = this.key_map.get(&cmd).cloned() {
                debug!("found!");
                //todo
                //let mut event = Event::ServerCommandEvent(ServerCommandEvent {
                //    has_been_cancelled: false,
                //    command,
                //});
                //fi.mod_loader.dispatch_event(&mut event);
                //let command_event = enum_val!(Event, event, ServerCommandEvent);
                //if command_event.has_been_cancelled {
                //    return;
                //}
                let command = &mut this.commands[id];
                command.executor.on_command(sender, cmd, parts.map(ToString::to_string).collect(), fi);
            } else {
                sender.send_error_message("Unknown command".to_string());
            }
        }
    }
}

unsafe impl Send for CommandProcessor {}
unsafe impl Sync for CommandProcessor {}

pub enum CommandSender {
    Console,
    Player(PlayerData),
}

impl CommandSender {
    pub fn send_message(&self, message: String) {
        // TODO
        match self {
            CommandSender::Console => {
                println!("{}", message);
            }
            CommandSender::Player(_) => {},
        }
    }

    pub fn send_message_raw(&self, message: String) {
        // TODO
        match self {
            CommandSender::Console => {
                print!("{}", message);
            }
            CommandSender::Player(_) => {},
        }
    }

    pub fn send_error_message(&self, message: String) {
        // also TODO
        match self {
            CommandSender::Console => {
                println!("{}", message);
            }
            CommandSender::Player(_) => {},
        }
    }
}


pub struct Command {
    key: String,
    aliases: Vec<String>,
    usage: String,
    executor: Box<dyn CommandExecutor>,
}

impl Command {
    pub fn new(key: &str, aliases: Vec<&str>, usage: Option<&str>, executor: impl CommandExecutor + 'static) -> Result<Self, String> {
        Self::verify_name(key)?;
        let mut aliases_string = Vec::new();
        for alias in &aliases {
            if Self::verify_name(alias).is_ok() {
                aliases_string.push(alias.to_string());
            } else {
                warn!("Command alias {alias} is illegal!");
            }
        }
        Ok(Self {
            key: key.to_string(),
            aliases: aliases_string,
            usage: usage.map_or(String::new(), ToString::to_string),
            executor: Box::new(executor),
        })
    }

    fn verify_name(s: &str) -> Result<(), String> {
        for c in s.chars() {
            if c.is_whitespace() {
                return Err(format!("Illegal command name: {s}"));
            }
        }
        Ok(())
    }
}

pub trait CommandExecutor {
    fn on_command(&mut self, sender: CommandSender, cmd: String, args: Vec<String>, fi: &mut FactoryIsland);
}

pub(crate) fn register_commands() {
    COMMAND_PROCESSOR.register(Command::new("players", vec![], None, PlayersCommand).unwrap(), &ObjectSource::Vanilla);
    COMMAND_PROCESSOR.register(Command::new("chunks", vec![], None, ChunksCommand).unwrap(), &ObjectSource::Vanilla);
    COMMAND_PROCESSOR.register(Command::new("save", vec![], None, SaveCommand).unwrap(), &ObjectSource::Vanilla);
    COMMAND_PROCESSOR.register(Command::new("stop", vec![], None, StopCommand).unwrap(), &ObjectSource::Vanilla);
    COMMAND_PROCESSOR.register(Command::new("commands", vec![], None, CommandsCommand).unwrap(), &ObjectSource::Vanilla);
}