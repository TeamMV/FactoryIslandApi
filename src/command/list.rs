use crate::command::{CommandExecutor, CommandSender};
use crate::FactoryIsland;

pub struct ListCommand;

impl CommandExecutor for ListCommand {
    fn on_command(&mut self, sender: &mut CommandSender, buffer: &[String], cmd: String, args: Vec<String>, fi: &mut FactoryIsland) {
        for arg in args {
            sender.send_message(arg);
        }
    }
}