extern crate clacks_tl_codegen;
extern crate failure;

use std::io::{Read, Write};
use std::{fs, path};

const TL_DIR: &str = "tl";

fn main() -> Result<(), failure::Error> {
    let out_dir = std::env::var("OUT_DIR")?;
    let generated_path = std::path::Path::new(&out_dir).join("mtproto.rs");

    println!("cargo:rerun-if-changed={}", TL_DIR);
    let mut files = fs::read_dir(TL_DIR)?
        .map(|r| r.map(|d| d.path()))
        .collect::<Result<Vec<path::PathBuf>, _>>()?;
    files.sort();
    let mut input = String::new();
    for file in files {
        fs::File::open(&file)?.read_to_string(&mut input)?;
        println!("cargo:rerun-if-changed={}", file.to_string_lossy());
    }
    let code = clacks_tl_codegen::generate_code_for(&input)?;
    fs::File::create(generated_path)?.write_all(code.as_bytes())?;
    Ok(())
}
