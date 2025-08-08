use crate::world::chunk::Chunk;
use crate::world::{ChunkPos, World};
use bytebuffer::ByteBuffer;
use log::{debug, error};
use mvengine::event::EventBus;
use mvutils::enum_val;
use mvutils::save::Savable;
use std::fs::File;
use std::io::{Read, Write};

use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use crate::mods::ModLoader;

pub const PATH: &str = ".factoryisland/worlds";

pub struct ChunkManager;

impl ChunkManager {
    pub fn try_load_chunk(&self, world: &World, chunk_pos: ChunkPos) -> Option<Chunk> {
        let mut dir = world.chunk_directory().clone();
        let filename = format!("c{}_{}.chunk", chunk_pos.0, chunk_pos.1);
        dir.push(&filename);

        let mut file = File::options().read(true).open(&dir);
        match file {
            Ok(mut file) => {
                let mut compressed = Vec::new();
                if let Err(_) = file.read_to_end(&mut compressed) {
                    return None;
                }
                
                if let Ok(decompressed) = decompress_size_prepended(compressed.as_slice()) {
                    let mut buffer = ByteBuffer::from(decompressed);

                    debug!("loaded chunk {chunk_pos:?}");

                    let chunk = Chunk::load(&mut buffer).ok()?;
                    let mut chunk = chunk.generate_terrain(world.generator(), world.objects());
                    chunk.terrain.apply_modifications();
                    Some(chunk)
                } else {
                    error!("Error decompressing chunk file");
                    None
                }
            }
            Err(_) => None
        }
    }

    pub fn try_save_chunk(&self, world: &World, chunk: &Chunk) {
        let mut dir = world.chunk_directory().clone();
        let filename = format!("c{}_{}.chunk", chunk.position.0, chunk.position.1);
        dir.push(&filename);

        let mut file = File::options().write(true).open(&dir);
        if let Err(_) = file {
            file = File::create(&dir);
        }
        if let Ok(mut file) = file {
            let mut buffer = ByteBuffer::new();
            chunk.save(&mut buffer);
            
            let compressed = compress_prepend_size(buffer.as_bytes());
            
            file.write_all(compressed.as_slice()).expect("Failed to write to file");
            debug!("Saved chunk {:?}", chunk.position);
            return;
        }
        error!("failed to create or open file: {:?} in {:?}", file, dir);
    }
}