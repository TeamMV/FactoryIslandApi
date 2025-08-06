use crate::command::{CommandExecutor, CommandSender, COMMAND_PROCESSOR};
use crate::FactoryIsland;

pub struct CommandsCommand;

impl CommandExecutor for CommandsCommand {
    fn on_command(&mut self, sender: CommandSender, cmd: String, args: Vec<String>, fi: &mut FactoryIsland) {
        sender.send_message("------".to_string());
        for (cmd, _) in COMMAND_PROCESSOR.key_map.get() {
            sender.send_message(format!("'{}'", cmd));
        }
        sender.send_message("------".to_string());
    }
}