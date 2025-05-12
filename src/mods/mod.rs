use std::{env, fs};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::ops::Deref;
use std::path::PathBuf;
use hashbrown::HashMap;
use libloading::{Library, Symbol};
use log::{error, info};
use mvengine::event::EventBus;
use mvutils::lazy;
use mvutils::utils::TetrahedronOp;
use crate::event::Event;
use crate::registry;
use crate::registry::{GameObjects, Registry};
use crate::registry::terrain::TerrainTiles;
use crate::world::tiles::terrain::TerrainTile;

lazy! {
    pub static DEFAULT_MOD_DIR: PathBuf = {
        let appdata = env::var("APPDATA").expect("Couldnt get appdata");
        let mut buf = PathBuf::from(appdata);
        buf.push(".factoryisland/mods");
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
                objects
            };

            for path in paths {
                unsafe {
                    if let Ok(mod_path) = path {
                        let path = mod_path.path();
                        info!("Loading mod: {:?}", path.file_name().unwrap_or(OsStr::new("unknown filename")));
                        let fi_mod = LoadedMod::load(path, &mut context);
                        if let Ok(fi_mod) = fi_mod {
                            info!("Mod '{}' has been loaded", fi_mod.inner.id);
                            self.loaded.insert(fi_mod.inner.id.clone(), fi_mod);
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
            .filter_map(|(k, v)| v.inner.uses_resources.yn(Some(k), None))
            .cloned()
            .collect()
    }
}

pub struct LoadedMod {
    pub(crate) inner: Mod,
    library: Library,
    init_fn: fn(&mut ModContext) -> Mod,
}

impl LoadedMod {
    pub unsafe fn load(path: PathBuf, ctx: &mut ModContext) -> Result<Self, String> {
        let lib = Library::new(path).map_err(|e| e.to_string())?;
        let init_fn: Symbol<fn(&mut ModContext) -> Mod> = lib.get(b"init").map_err(|e| e.to_string())?;
        let init_fn = *init_fn.deref();
        let m = init_fn(ctx);

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
    pub objects: GameObjects
}

#[repr(C)]
pub struct Mod {
    pub id: String,
    uses_resources: bool
}

impl Mod {
    pub fn no_resources(id: &str) -> Self {
        Self {
            id: id.to_string(),
            uses_resources: false,
        }
    }
    
    pub fn with_resources(id: &str) -> Self {
        Self {
            id: id.to_string(),
            uses_resources: true,
        }
    }
}