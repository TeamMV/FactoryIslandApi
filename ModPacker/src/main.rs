mod serialize;

use crate::serialize::ModJson;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fs, io};
use std::process::{Command, Stdio};
use bytebuffer::ByteBuffer;
use mvutils::bytebuffer::ByteBufferExtras;
use mvutils::save::Savable;

pub trait MapErrorToString<T> {
    fn mets(self) -> Result<T, String>;
}

impl<T, E: ToString> MapErrorToString<T> for Result<T, E> {
    fn mets(self) -> Result<T, String> {
        self.map_err(|x| x.to_string())
    }
}

trait UnixString {
    fn to_unix_string(&self) -> String;
}

impl UnixString for PathBuf {
    fn to_unix_string(&self) -> String {
        self.to_string_lossy().replace('\\', "/")
    }
}

impl UnixString for std::path::Path {
    fn to_unix_string(&self) -> String {
        self.to_string_lossy().replace('\\', "/")
    }
}

fn main() {
    if env::args().any(|x| x == "-doesexist") {
        //i might use this for the ModCreatorTool which will also make the intelliJ run configs
        println!("FiModPacker is installed on this machine.");
        return;
    }
    if let Err(e) = wrapper() {
        eprintln!("An error occured during packing: {e}");
    } else {
        println!("Your mod was successfully packed!");
    }
}

fn wrapper() -> Result<(), String> {
    let dir = env::current_dir().mets()?;
    let conf_file_path = dir.join("mod.json");
    if !fs::exists(&conf_file_path).mets()? {
        return Err("Invalid Mod project! (missing mod.json)").mets()
    }
    if let Ok(conf_file) = File::options().read(true).open(&conf_file_path) {
        let mod_json: ModJson = serde_json::from_reader(conf_file).mets()?;
        println!("Preparing to pack mod with id: {}", mod_json.modid);
        //If the mod has resources, compile them to bytes.
        if mod_json.specs.res {
            if env::args().any(|x| x == "-old_r") {
                println!("Skipping resource compilation. WARNING: If you changed something in res.rs or /resources, please run without '-old_r'");
            } else {
                compile_resources(&mod_json.modid, &dir)?;
            }
            // the generated resource file will be at "TMP/<modid>/compiled.r"
        }

        let mut r_file: Vec<u8> = Vec::new();
        //search for the compiled resources
        if mod_json.specs.res {
            //append the compiled resources
            let appdata = env::var("APPDATA").mets()?;
            let appdata = PathBuf::from_str(&appdata).mets()?;
            let appdata = appdata.join("FiModPacker");
            let appdata = appdata.join("TMP");
            let appdata = appdata.join(&mod_json.modid);
            let r_file_path = appdata.join("compiled.r");
            if !fs::exists(&r_file_path).mets()? {
                return Err("Cannot find the compiled resources! Please run without '-old_r' to have them regenerated!").mets();
            }
            r_file = fs::read(&r_file_path).mets()?;
        }

        //compile the mod crate
        println!("Compiling the mod... this might take a bit!");
        let output = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to start cargo process: {}", e))?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(err);
        }
        println!("The mod compilation was successful!");
        //retrieve the dll file
        let dll_path = dir.join(&format!("target/release/{}.dll", mod_json.modid));

        //create the whole .fim file
        let mut buffer = ByteBuffer::new_le();
        mod_json.save(&mut buffer);
        let dll = fs::read(&dll_path).mets()?;
        dll.save(&mut buffer);
        r_file.save(&mut buffer);

        let target = dir.join("target");
        let output = target.join(&format!("{}.fim", mod_json.modid));
        fs::write(&output, buffer.as_bytes()).mets()?;
    }
    Ok(())
}

fn compile_resources(modid: &str, dir: &PathBuf) -> Result<(), String> {
    println!("Preparing resources compilation...");
    let r_file_path = dir.join("src/res.rs");
    let resources_dir_file_path = dir.join("resources");
    let appdata = env::var("APPDATA").mets()?;
    let appdata = PathBuf::from_str(&appdata).mets()?;
    let appdata = appdata.join("FiModPacker");
    let appdata = appdata.join("TMP");
    let appdata = appdata.join(modid);
    let mod_tmp_dir = appdata.clone();
    let appdata = appdata.join("Resources");
    let cargo_dir = appdata.clone();
    let main_rs_contents = include_str!("main.template.rs");
    let main_rs_contents = main_rs_contents.replace("{{mod_tmp_dir}}", &mod_tmp_dir.to_unix_string());
    println!("Generating cargo project...");
    create_rust_project(&cargo_dir, &main_rs_contents)?;
    copy_dir_all(&resources_dir_file_path, &appdata.join("resources")).mets()?;
    let appdata = appdata.join("src/res.rs");
    fs::copy(&r_file_path, &appdata).mets()?;
    println!("Running compilation... This might take a bit!");
    run_rust_project(&cargo_dir)?;
    //delete the cargo project
    println!("Cleaning stuff up...");
    fs::remove_dir_all(&cargo_dir).mets()?;
    println!("Resource compilation finished!");
    Ok(())
}

fn create_rust_project(location: &PathBuf, contents_main_rs: &str) -> Result<(), String> {
    fs::create_dir_all(location.join("src")).mets()?;

    let lib_rs = location.join("src/main.rs");
    fs::write(&lib_rs, contents_main_rs).mets()?;

    const TEMPLATE: &str = include_str!("Cargo.template.toml");

    let mvengine_home = env::var("MVENGINE_HOME").mets()?;
    let mvengine_path = PathBuf::from(&mvengine_home);
    let mvengine_proc_path = mvengine_path.join("Proc");

    let cargo_toml = TEMPLATE
        .replace("{{crate_name}}", "dummy_crate")
        .replace("{{mvengine_path}}", &mvengine_path.to_unix_string())
        .replace("{{mvengine_proc_path}}", &mvengine_proc_path.to_unix_string());

    fs::write(location.join("Cargo.toml"), cargo_toml).mets()?;

    Ok(())
}

fn run_rust_project(location: &PathBuf) -> Result<(), String> {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .current_dir(location)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to start cargo process: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(err);
    }

    Ok(())
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}