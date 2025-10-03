pub mod players;
pub mod chunks;
pub mod stop;
pub mod save;
pub mod tp;
pub mod print;
pub mod grep;
mod list;

use std::collections::HashMap;
use std::num::ParseIntError;
use std::str::FromStr;
use log::{debug, info, warn};
use mvutils::{enum_val, enum_val_ref, lazy};
use mvutils::unsafe_utils::DangerousCell;
use parking_lot::RwLock;
use crate::command::chunks::ChunksCommand;
use crate::command::grep::GrepCommand;
use crate::command::list::ListCommand;
use crate::command::players::PlayersCommand;
use crate::command::print::PrintCommand;
use crate::command::save::SaveCommand;
use crate::command::stop::StopCommand;
use crate::command::tp::TpCommand;
use crate::FactoryIsland;
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

    pub fn register(&self, cmd: Command) {
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

        this.commands.push(cmd);
    }

    pub fn process(&self, sender: &mut CommandSender, command: String, fi: &mut FactoryIsland) {
        let mut this = self.inner.write();

        let mut commands = vec![];
        let mut current = String::new();
        let mut last_op = '&';
        for c in command.chars() {
            match c {
                '|' | '&' => {
                    if !current.trim().is_empty() {
                        commands.push((last_op, current.trim().to_string()));
                    }
                    current = String::new();
                    last_op = c;
                }
                _ => current.push(c),
            }
        }
        //append last one lol that would break badly
        if !current.trim().is_empty() {
            commands.push((last_op, current.trim().to_string()));
        }

        let mut buffered_sender = CommandSender::Buffer(Vec::new());
        let mut messages = vec![];
        for (op, cmd) in commands {
            let buffer = enum_val_ref!(CommandSender, buffered_sender, Buffer);
            for message in buffer {
                match message {
                    Ok(m) => {
                        messages.push(m.clone());

                        if op == '&' {
                            sender.send_message_raw(m.clone());
                        }
                    }
                    Err(e) => {
                        sender.send_error_message(format!("Error in command '{cmd}':\n{e}"));
                    }
                }
            }

            buffered_sender = CommandSender::Buffer(Vec::new());

            let mut parts = cmd.split_whitespace();
            if let Some(cmd) = parts.next() {
                let cmd_name = cmd.to_string();
                if let Some(id) = this.key_map.get(&cmd_name).cloned() {
                    let command = &mut this.commands[id];
                    let args = parts.map(|x| {
                        let mut iter = x.chars();
                        if iter.next() == Some('%') {
                            let remaining: String = iter.collect();
                            match usize::from_str(&remaining) {
                                Ok(index) => {
                                    let msg_len = messages.len();
                                    if index < msg_len {
                                        messages[index].clone()
                                    } else {
                                        sender.send_error_message(format!("Index {index} out of bounds for buffer length {msg_len}!"));
                                        String::new()
                                    }
                                }
                                Err(_) => {
                                    sender.send_error_message(format!("Illegal position argument '{x}'!"));
                                    String::new()
                                }
                            }
                        } else {
                            x.to_string()
                        }
                    }).collect();
                    command.executor.on_command(&mut buffered_sender, &messages, cmd_name, args, fi);
                } else {
                    sender.send_error_message("Unknown command".to_string());
                }
            }

            if op == '|' {
                messages.clear();
            }
        }

        let buffer = enum_val_ref!(CommandSender, buffered_sender, Buffer);
        for message in buffer {
            match message {
                Ok(m) => {
                    sender.send_message_raw(m.clone());
                }
                Err(e) => {
                    sender.send_error_message(format!("Error in command: {e}"));
                }
            }
        }
    }
}

unsafe impl Send for CommandProcessor {}
unsafe impl Sync for CommandProcessor {}

pub enum CommandSender {
    Console,
    Player(PlayerData),
    Buffer(Vec<Result<String, String>>)
}

impl CommandSender {
    pub fn send_message(&mut self, message: String) {
        // TODO
        match self {
            CommandSender::Console => {
                println!("{}", message);
            }
            CommandSender::Player(_) => {},
            CommandSender::Buffer(b) => { b.push(Ok(format!("{message}\n"))); }
        }
    }

    pub fn send_message_raw(&mut self, message: String) {
        // TODO
        match self {
            CommandSender::Console => {
                print!("{}", message);
            }
            CommandSender::Player(_) => {},
            CommandSender::Buffer(b) => { b.push(Ok(message)); }
        }
    }

    pub fn send_error_message(&mut self, message: String) {
        // also TODO
        match self {
            CommandSender::Console => {
                println!("{}", message);
            }
            CommandSender::Player(_) => {},
            CommandSender::Buffer(b) => { b.push(Err(format!("{message}\n"))); }
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
    fn on_command(&mut self, sender: &mut CommandSender, buffer: &[String], cmd: String, args: Vec<String>, fi: &mut FactoryIsland);
}

pub(crate) fn register_commands() {
    COMMAND_PROCESSOR.register(Command::new("players", vec![], None, PlayersCommand).unwrap());
    COMMAND_PROCESSOR.register(Command::new("chunks", vec![], None, ChunksCommand).unwrap());
    COMMAND_PROCESSOR.register(Command::new("save", vec![], None, SaveCommand).unwrap());
    COMMAND_PROCESSOR.register(Command::new("stop", vec![], None, StopCommand).unwrap());
    COMMAND_PROCESSOR.register(Command::new("tp", vec!["teleport"], None, TpCommand).unwrap());
    COMMAND_PROCESSOR.register(Command::new("print", vec![], None, PrintCommand).unwrap());
    COMMAND_PROCESSOR.register(Command::new("grep", vec![], None, GrepCommand).unwrap());
    COMMAND_PROCESSOR.register(Command::new("list", vec![], None, ListCommand).unwrap());
}