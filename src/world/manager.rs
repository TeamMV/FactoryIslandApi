use crate::world::chunk::Chunk;
use crate::world::{ChunkPos, ChunkType, World};
use bytebuffer::ByteBuffer;
use log::{debug, error};
use mvengine::event::EventBus;
use mvutils::enum_val;
use mvutils::save::Savable;
use std::fs::File;
use std::io::{Read, Write};

use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use mvengine::utils::savers::SaveArc;
use parking_lot::Mutex;

pub const PATH: &str = ".factoryisland/worlds";

pub struct ChunkManager;

impl ChunkManager {
    pub fn try_load_chunk(&self, world: &World, chunk_pos: ChunkPos) -> Option<ChunkType> {
        let dir = world.chunk_directory();
        let filename = format!("c{}_{}.chunk", chunk_pos.0, chunk_pos.1);
        if let Some(mut file) = dir.read_file(&filename) {
            let mut compressed = Vec::new();
            if let Err(_) = file.read_to_end(&mut compressed) {
                return None;
            }

            if let Ok(decompressed) = decompress_size_prepended(compressed.as_slice()) {
                let mut buffer = ByteBuffer::from(decompressed);

                debug!("loaded chunk {chunk_pos:?}");

                let mut chunk = Chunk::load(&mut buffer).ok()?;
                chunk.generate_terrain(world.generator(), world.objects());
                chunk.terrain.apply_modifications();
                Some(SaveArc::new(Mutex::new(chunk)))
            } else {
                error!("Error decompressing chunk file");
                None
            }
        } else {
            None
        }
    }

    pub fn try_save_chunk(&self, world: &World, chunk: &Chunk) {
        let dir = world.chunk_directory();
        let filename = format!("c{}_{}.chunk", chunk.position.0, chunk.position.1);
        if let Some(mut file) = dir.write_file(&filename) {
            let mut buffer = ByteBuffer::new();
            chunk.save(&mut buffer);

            let compressed = compress_prepend_size(buffer.as_bytes());

            file.write_all(compressed.as_slice()).expect("Failed to write to file");
            debug!("Saved chunk {:?}", chunk.position);
        } else {
            error!("failed to create or open file: {:?} in {:?}", filename, dir);
        }
    }
}