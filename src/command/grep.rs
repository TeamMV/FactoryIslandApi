use std::ops::RangeBounds;
use mvengine::ui::parse::parse_range;
use crate::command::{CommandExecutor, CommandSender};
use crate::FactoryIsland;

pub struct GrepCommand;

impl CommandExecutor for GrepCommand {
    fn on_command(&mut self, sender: &mut CommandSender, buffer: &[String], _: String, args: Vec<String>, _fi: &mut FactoryIsland) {
        let mut string_filter: Option<String> = None;
        let mut index_filter: Option<usize> = None;
        let mut range_filter: Option<(usize, usize)> = None;

        for arg in args {
            let arg = arg.trim();

            if arg.starts_with('"') && arg.ends_with('"') {
                string_filter = Some(arg[1..arg.len()-1].to_string());
            } else if let Ok(num) = arg.parse::<usize>() {
                index_filter = Some(num);
            } else if let Ok((start, end)) = parse_range(arg) {
                range_filter = Some((start, end));
            }
        }

        let mut filtered: Vec<String> = Vec::new();

        for (i, line) in buffer.iter().enumerate() {
            let mut keep = true;

            if let Some(idx) = index_filter {
                keep &= i == idx;
            }

            if let Some(ref substr) = string_filter {
                keep &= line.contains(substr);
            }

            if let Some((start, end)) = range_filter {
                keep &= (start <= i && end >= i);
            }

            if keep {
                filtered.push(line.clone());
            }
        }

        for msg in filtered {
            sender.send_message_raw(msg);
        }
    }
}