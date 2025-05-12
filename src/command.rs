use std::process::exit;
use log::{error, info};
use mvengine::event::EventBus;
use crate::event::Event;
use crate::FactoryIsland;

pub struct CommandProcessor;

impl CommandProcessor {
    pub fn new() -> Self {
        Self
    }

    pub fn process(command: &str, fi: &mut FactoryIsland) {
        if let Some(start) = command.split_whitespace().next() {
            match start {
                "stop" => {
                    let mut world = fi.world.lock();
                    info!("Saving world...");
                    world.save();
                    drop(world);
                    info!("World saved!");
                    info!("Exiting...");
                    exit(0);
                },
                "save" => {
                    let mut world = fi.world.lock();
                    info!("Saving world...");
                    world.save();
                    info!("World saved!");
                },
                "chunks" => {
                    let parts: Vec<&str> = command.split_whitespace().collect();
                    if parts.len() != 5 {
                        error!("Usage: chunks <x1> <z1> <x2> <z2>");
                        return;
                    }

                    let parse_arg = |s: &str| s.parse::<i32>();

                    match (parse_arg(parts[1]), parse_arg(parts[2]), parse_arg(parts[3]), parse_arg(parts[4])) {
                        (Ok(x1), Ok(z1), Ok(x2), Ok(z2)) => {
                            let world = fi.world.lock();
                            for z in (z1..=z2).rev() {
                                for x in x1..=x2 {
                                    if world.is_loaded((x, z)) {
                                        print!("l ")
                                    } else if world.exists_file((x, z)) {
                                        print!("* ")
                                    } else {
                                        print!("- ")
                                    }
                                }
                                println!();
                            }
                        }
                        _ => {
                            error!("Invalid arguments. All coordinates must be integers.");
                        }
                    }
                }
                "players" => {
                    println!("Currently on the server:");
                    if fi.players.is_empty() { 
                        println!("Noone");
                    } else {
                        for player in fi.players.values() {
                            let lock = player.lock();
                            let name = &lock.data.name;
                            println!("{name}");
                        }
                    }
                }
                _ => error!("Unknown command: {}", start)
            }
        }
    }
}