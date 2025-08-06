use std::{env, fs};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::ops::Deref;
use std::path::PathBuf;
use bytebuffer::ByteBuffer;
use hashbrown::HashMap;
use libloading::{Library, Symbol};
use log::{error, info, warn};
use mvengine::event::EventBus;
use mvutils::{lazy, Savable};
use mvutils::bytebuffer::ByteBufferExtras;
use mvutils::save::Savable;
use mvutils::utils::TetrahedronOp;
use crate::command::CommandProcessor;
use crate::event::Event;
use crate::{command, registry};
use crate::registry::{GameObjects, Registry};
use crate::registry::terrain::TerrainTiles;
use crate::world::tiles::ObjectSource;
use crate::world::tiles::resources::ClientTileRes;
use crate::world::tiles::terrain::TerrainTile;
use crate::world::tiles::tiles::Tile;

lazy! {
    pub static DEFAULT_MOD_DIR: PathBuf = {
        let appdata = env::var("APPDATA").expect("Couldnt get appdata");
        let mut buf = PathBuf::from(appdata);
        buf.push(".factoryisland/mods");
        buf
    };

    pub static TMP_DIR: PathBuf = {
        let appdata = env::var("APPDATA").expect("Couldnt get appdata");
        let mut buf = PathBuf::from(appdata);
        buf.push(".factoryisland/tmp/server");
        buf
    };
}

pub struct ModLoader {
    loaded: HashMap<String, LoadedMod>
}

impl ModLoader {
    pub fn new() -> Self {
        Self {
            loaded: HashMap::new(),
        }
    }

    pub fn load(&mut self, directory: &PathBuf, events: &mut EventBus<Event>, objects: GameObjects) {
        info!("Loading mods from {directory:?}");

        if let Ok(paths) = fs::read_dir(directory) {
            let mut context = ModContext {
                events,
                terrain_registry: &registry::terrain::TERRAIN_REGISTRY,
                tile_registry: &registry::tiles::TILE_REGISTRY,
                command_processor: &command::COMMAND_PROCESSOR,
                objects,
            };

            for path in paths {
                unsafe {
                    if let Ok(mod_path) = path {
                        let path = mod_path.path();
                        info!("Loading mod: {:?}", path.file_name().unwrap_or(OsStr::new("unknown filename")));
                        let fi_mod = LoadedMod::load(path, &mut context);
                        if let Ok(fi_mod) = fi_mod {
                            info!("Mod '{}' has been loaded", fi_mod.inner.modid);
                            if self.loaded.contains_key(&fi_mod.inner.modid) {
                                warn!("Two mods have the same modid! That is considered illegal and will be handled by the authorities.");
                                continue;
                            }
                            self.loaded.insert(fi_mod.inner.modid.clone(), fi_mod);
                        } else {
                            if let Err(e) = fi_mod {
                                error!("Couldnt load mod: {e}");
                            }
                        }
                    }
                }
            }
            info!("Mod loading complete");
        }
    }

    pub fn res_mod_ids(&self) -> Vec<String> {
        self.loaded
            .iter()
            .filter_map(|(k, v)| v.inner.specs.res.yn(Some(k), None))
            .cloned()
            .collect()
    }
}

pub struct LoadedMod {
    pub(crate) inner: Mod,
    library: Library,
    init_fn: fn(&mut ModContext),
}

impl LoadedMod {
    pub unsafe fn load(path: PathBuf, ctx: &mut ModContext) -> Result<Self, String> {
        let bytes = fs::read(&path).map_err(|x| x.to_string())?;
        let mut buffer = ByteBuffer::from_vec_le(bytes);
        let m = Mod::load(&mut buffer)?;
        let dll_bytes = Vec::<u8>::load(&mut buffer)?;
        let dll_path = TMP_DIR.join(&format!("{}.dll", m.modid));
        fs::write(&dll_path, dll_bytes).map_err(|x| x.to_string())?;

        let lib = Library::new(dll_path).map_err(|e| e.to_string())?;
        let init_fn: Symbol<fn(&mut ModContext)> = lib.get(b"server_init").map_err(|e| e.to_string())?;
        let init_fn = *init_fn.deref();

        init_fn(ctx);

        Ok(Self {
            inner: m,
            library: lib,
            init_fn,
        })
    }
}

pub struct ModContext<'a> {
    pub events: &'a mut EventBus<Event>,
    pub terrain_registry: &'a Registry<TerrainTile>,
    pub tile_registry: &'a Registry<Tile>,
    pub command_processor: &'a CommandProcessor,
    pub objects: GameObjects,
}

#[derive(Savable)]
pub struct Mod {
    pub name: String,
    pub modid: String,
    pub makers: Vec<String>,
    pub versions: ModJsonVersions,
    pub specs: ModJsonSpecs
}

#[derive(Savable)]
pub struct ModJsonVersions {
    pub game: String,
    pub r#mod: String,
}

#[derive(Savable)]
pub struct ModJsonSpecs {
    pub res: bool,
    pub targets: Vec<String>
}