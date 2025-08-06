mod res;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use bytebuffer::ByteBuffer;
use mvengine::ui::res::runtime::ResourceSavable;
use res::R;

fn main() {
    R::initialize();
    let mut buffer = ByteBuffer::new();
    R.save_res(&mut buffer);
    let output_dir = PathBuf::from("{{mod_tmp_dir}}");
    fs::create_dir_all(&output_dir).expect("Cannot create output directory");
    let out_file = Path::join(&output_dir, "compiled.r");
    let mut file = if out_file.exists() {
        File::options().write(true).open(&out_file).expect("Cannot open output file!")
    } else {
        File::create(&out_file).expect("Cannot create output file!")
    };
    file.write_all(buffer.as_bytes()).expect("Cannot write to output file!");
}