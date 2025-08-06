use std::process::exit;
use log::info;
use crate::command::{CommandExecutor, CommandSender};
use crate::event::{Event, GameEndEvent};
use crate::FactoryIsland;

pub struct StopCommand;

impl CommandExecutor for StopCommand {
    fn on_command(&mut self, sender: CommandSender, cmd: String, args: Vec<String>, fi: &mut FactoryIsland) {
        let mut world = fi.world.lock();
        sender.send_message("Saving world...".to_string());
        world.save();
        drop(world);
        sender.send_message("World saved!".to_string());
        sender.send_message("Exiting...".to_string());
        fi.event_bus.dispatch(&mut Event::GameEndEvent(GameEndEvent));
        exit(0);
    }
}