pub mod players;
pub mod chunks;
pub mod stop;
pub mod save;

use hashbrown::HashMap;
use log::warn;
use mvutils::{enum_val, lazy};
use mvutils::unsafe_utils::DangerousCell;
use crate::command::chunks::ChunksCommand;
use crate::command::players::PlayersCommand;
use crate::command::save::SaveCommand;
use crate::command::stop::StopCommand;
use crate::event::common::ServerCommandEvent;
use crate::event::Event;
use crate::FactoryIsland;
use crate::server::packets::common::PlayerData;
use crate::world::tiles::ObjectSource;

lazy! {
    pub static COMMAND_PROCESSOR: CommandProcessor = CommandProcessor::new();
}

pub struct CommandProcessor {
    commands: DangerousCell<Vec<Command>>,
    key_map: DangerousCell<HashMap<String, usize>>
}

impl CommandProcessor {
    pub fn new() -> Self {
        Self {
            commands: DangerousCell::new(Vec::new()),
            key_map: DangerousCell::new(HashMap::new()),
        }
    }

    pub fn register(&self, cmd: Command, source: &ObjectSource) {
        let key_map = self.key_map.get_mut();
        let commands = self.commands.get_mut();
        let id = commands.len();
        if key_map.try_insert(cmd.key.clone(), id).is_err() {
            warn!("Command {} is already taken, skipping registering raw command", cmd.key);
        }

        for alias in &cmd.aliases {
            if key_map.try_insert(alias.clone(), id).is_err() {
                warn!("Command alias {alias} for command {} is already taken and will not be registered", cmd.key);
            }
        }

        if let ObjectSource::Mod(mod_id, _) = source {
            key_map.insert(format!("{mod_id}:{}", cmd.key), id);
            for alias in &cmd.aliases {
                key_map.insert(format!("{mod_id}:{alias}"), id);
            }
        }

        commands.push(cmd);
    }

    pub fn process(&self, sender: CommandSender, command: String, fi: &mut FactoryIsland) {
        let binding = command.clone();
        let mut parts = binding.split_whitespace();
        if let Some(cmd) = parts.next() {
            let cmd = cmd.to_string();
            if let Some(id) = self.key_map.get().get(&cmd) {

                let mut event = Event::ServerCommandEvent(ServerCommandEvent {
                    has_been_cancelled: false,
                    command,
                });
                fi.event_bus.dispatch(&mut event);
                let command_event = enum_val!(Event, event, ServerCommandEvent);
                if command_event.has_been_cancelled {
                    return;
                }
                let command = &mut self.commands.get_mut()[*id];
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
}