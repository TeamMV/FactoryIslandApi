use crate::command::{CommandExecutor, CommandSender};
use crate::{FactoryIsland, PLAYERS};

pub struct TpCommand;

impl CommandExecutor for TpCommand {
    fn on_command(&mut self, sender: &mut CommandSender, buffer: &[String], cmd: String, args: Vec<String>, fi: &mut FactoryIsland) {
        let mut players = PLAYERS.write();
        /*

        /tp v22 0 0

         */
    }
}