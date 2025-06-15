use crate::command::{CommandExecutor, CommandSender};
use crate::FactoryIsland;

pub struct PlayersCommand;

impl CommandExecutor for PlayersCommand {
    fn on_command(&mut self, sender: CommandSender, _: String, _args: Vec<String>, fi: &mut FactoryIsland) {
        sender.send_message("Currently on the server:".to_string());
        if fi.players.is_empty() {
            sender.send_message("Noone".to_string());
        } else {
            for player in fi.players.values() {
                let lock = player.lock();
                let name = &lock.data.name;
                sender.send_message(format!("{name}"));
            }
        }
    }
}