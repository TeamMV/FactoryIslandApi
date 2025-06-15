use crate::command::{CommandExecutor, CommandSender};
use crate::FactoryIsland;

pub struct SaveCommand;

impl CommandExecutor for SaveCommand {
    fn on_command(&mut self, sender: CommandSender, cmd: String, args: Vec<String>, fi: &mut FactoryIsland) {
        let mut world = fi.world.lock();
        sender.send_message("Saving world...".to_string());
        world.save();
        sender.send_message("World saved!".to_string());
    }
}