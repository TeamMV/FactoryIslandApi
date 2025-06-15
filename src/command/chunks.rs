use crate::command::{CommandExecutor, CommandSender};
use crate::FactoryIsland;

pub struct ChunksCommand;

impl CommandExecutor for ChunksCommand {
    fn on_command(&mut self, sender: CommandSender, _: String, args: Vec<String>, fi: &mut FactoryIsland) {
        if let CommandSender::Player(_) = &sender {
            sender.send_error_message("This command can only be ran through console".to_string());
            return;
        }
        
        if args.len() != 4 {
            sender.send_error_message("Usage: chunks <x1> <z1> <x2> <z2>".to_string());
            return;
        }

        let parse_arg = |s: &str| s.parse::<i32>();

        match (parse_arg(&args[0]), parse_arg(&args[1]), parse_arg(&args[2]), parse_arg(&args[3])) {
            (Ok(x1), Ok(z1), Ok(x2), Ok(z2)) => {
                let world = fi.world.lock();
                for z in (z1..=z2).rev() {
                    for x in x1..=x2 {
                        if world.is_loaded((x, z)) {
                            sender.send_message_raw("l ".to_string());
                        } else if world.exists_file((x, z)) {
                            sender.send_message_raw("* ".to_string());
                        } else {
                            sender.send_message_raw("- ".to_string());
                        }
                    }
                    sender.send_message(String::new());
                }
            }
            _ => {
                sender.send_error_message("Invalid arguments. All coordinates must be integers.".to_string());
            }
        }
    }
}