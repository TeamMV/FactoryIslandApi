use crate::command::{CommandExecutor, CommandSender};
use crate::mods::HMDS;
use crate::FactoryIsland;

pub trait ModCommandExecutor {
    fn on_mod_command(&mut self, context: HMDS, sender: CommandSender, cmd: String, args: Vec<String>, fi: &mut FactoryIsland);

    fn get_mod_id(&self) -> &str;
}

impl<T: ModCommandExecutor> CommandExecutor for T {
    fn on_command(&mut self, sender: CommandSender, cmd: String, args: Vec<String>, fi: &mut FactoryIsland) {
        let m = fi.mod_loader.loaded.get_mut(self.get_mod_id());
        unsafe {
            if let Some(m) = m {
                let handle = m.instance;
                self.on_mod_command(handle, sender, cmd, args, fi);
            }
        }
    }
}