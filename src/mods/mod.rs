pub mod modsdk;

use std::{env, fs};
use std::alloc::Layout;
use std::any::Any;
use std::ffi::{CStr, CString, OsStr, OsString};
use std::fs::File;
use std::ops::Deref;
use std::path::PathBuf;
use std::ptr::null;
use abi_stable::std_types::RString;
use abi_stable::traits::IntoReprC;
use bytebuffer::ByteBuffer;
use hashbrown::HashMap;
use libloading::{Library, Symbol};
use log::{error, info, warn};
use mvengine::event::{EventBus, EventQueue, EventReceiver};
use mvutils::{lazy, Savable};
use mvutils::bytebuffer::ByteBufferExtras;
use mvutils::once::CreateOnce;
use mvutils::save::Savable;
use mvutils::unsafe_utils::Unsafe;
use mvutils::utils::TetrahedronOp;
use parking_lot::RwLock;
use crate::command::CommandProcessor;
use crate::{command, registry};
use crate::mods::modsdk::events::{Event, EventHandler, EventResponse};
use crate::mods::modsdk::ModData;
use crate::registry::{GameObjects, Registry};
use crate::registry::terrain::TerrainTiles;
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

    pub static LOADED_MODS: RwLock<HashMap<RString, LoadedMod>> = RwLock::new(HashMap::new());
}
pub struct ModLoader;

impl ModLoader {
    pub fn load(directory: &PathBuf, objects: GameObjects) {
        info!("Loading mods from {directory:?}");

        unsafe {
            if let Ok(paths) = fs::read_dir(directory) {
                let mut loaded = LOADED_MODS.write();
                for path in paths {
                    unsafe {
                        if let Ok(mod_path) = path {
                            let path = mod_path.path();
                            info!("Loading mod: {:?}", path.file_name().unwrap_or(OsStr::new("unknown filename")));
                            let fi_mod = LoadedMod::load(path);
                            if let Ok(fi_mod) = fi_mod {
                                info!("Mod '{}' has been loaded", fi_mod.inner.modid);
                                let modid = fi_mod.inner.modid.clone();
                                let modid = modid.into_c();
                                if loaded.contains_key(&modid) {
                                    warn!("Two mods have the same modid! That is considered illegal and will be handled by the authorities.");
                                    continue;
                                }
                                loaded.insert(fi_mod.inner.modid.clone().into_c(), fi_mod);
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
    }

    pub fn res_mod_ids() -> Vec<String> {
        let loaded = LOADED_MODS.read();
        loaded
            .iter()
            .filter_map(|(k, v)| v.inner.specs.res.yn(Some(k.to_string()), None))
            .collect()
    }

    pub fn dispatch_event(mut event: Event) -> Event {
        let loaded = LOADED_MODS.read();
        for m in loaded.values() {
            let data = m.mod_data;
            for listener in &m.event_listeners {
                let resp = listener(event.clone(), data);
                event = match resp {
                    EventResponse::None => event,
                    EventResponse::Changed(e) => e
                };
            }
        }
        event
    }

    pub fn unload() {
        let mut mods = LOADED_MODS.write();
        for m in mods.values_mut() {
            m.unload();
        }
    }
}

pub struct LoadedMod {
    pub(crate) inner: ModInfo,
    library: Library,
    event_listeners: Vec<EventHandler>,
    mod_data: ModData,
    free_fn: fn(ModData)
}

impl LoadedMod {
    pub unsafe fn load(path: PathBuf) -> Result<Self, String> {
        let bytes = fs::read(&path).map_err(|x| x.to_string())?;
        let mut buffer = ByteBuffer::from_vec_le(bytes);
        let m = ModInfo::load(&mut buffer)?;
        let dll_bytes = Vec::<u8>::load(&mut buffer)?;
        let dll_path = TMP_DIR.join(&format!("{}.dll", m.modid));
        fs::write(&dll_path, dll_bytes).map_err(|x| x.to_string())?;

        let lib = Library::new(dll_path).map_err(|e| e.to_string())?;
        let init_fn: Symbol<fn() -> ModData> = lib.get(b"server_init").map_err(|e| e.to_string())?;
        let init_fn = *init_fn.deref();

        let free_fn: Symbol<fn(ModData)> = lib.get(b"server_stop").map_err(|e| e.to_string())?;
        let free_fn = *free_fn.deref();

        let data = init_fn();

        Ok(Self {
            inner: m,
            library: lib,
            event_listeners: vec![],
            mod_data: data,
            free_fn,
        })
    }

    pub fn unload(&mut self) {
        (self.free_fn)(self.mod_data)
    }
}

unsafe impl Send for LoadedMod {}
unsafe impl Sync for LoadedMod {}

#[derive(Savable)]
pub struct ModInfo {
    pub name: String,
    pub modid: String,
    pub makers: Vec<String>,
    pub versions: ModInfoVersions,
    pub specs: ModInfoSpecs
}

#[derive(Savable)]
pub struct ModInfoVersions {
    pub game: String,
    pub r#mod: String,
}

#[derive(Savable)]
pub struct ModInfoSpecs {
    pub res: bool,
    pub targets: Vec<String>
}