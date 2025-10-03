use crate::command::{CommandExecutor, CommandSender};
use crate::FactoryIsland;

pub struct PrintCommand;

impl CommandExecutor for PrintCommand {
    fn on_command(&mut self, sender: &mut CommandSender, buffer: &[String], cmd: String, args: Vec<String>, fi: &mut FactoryIsland) {
        for m in args {
            sender.send_message_raw(m);
        }
    }
}