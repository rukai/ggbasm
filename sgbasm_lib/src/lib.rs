#![feature(rust_2018_preview)]

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::fs;
use std::io::{Read, Write};
use std::path::{PathBuf, Path};

use failure::Error;
use failure::bail;

pub enum SourceFile {
    Asm (String),
    Binary (Vec<u8>),
}

/// parse the assembly source file at the given path
pub fn parse_asm(path: &Path) -> Result<String, Error> {
    let mut output = String::new();
    File::open(path)?.read_to_string(&mut output)?;
    Ok(output)
}

/// load the binary file at the given path
pub fn load_binary(path: &Path) -> Result<Vec<u8>, Error> {
    let mut output = Vec::<u8>::new();
    File::open(path)?.read(&mut output)?;
    Ok(output)
}

/// Parse and return all *.asm and *.inc files in the current directory
pub fn source_files() -> Result<HashMap<String, SourceFile>, Error> {
    let mut source_files = HashMap::new();
    for path in fs::read_dir(env::current_dir()?)? {
        let path = path?.path();
        if let Some(extension) = path.extension() {
            if let Some(extension) = extension.to_str() {
                let source_file = match extension.to_lowercase().as_ref() {
                    "asm" => SourceFile::Asm (parse_asm(&path)?),
                    "inc" => SourceFile::Binary (load_binary(&path)?),
                    _ => { continue; }
                };
                if let Some(file_stem) = path.file_stem() {
                    if let Some(file_stem) = file_stem.to_str() {
                        source_files.insert(file_stem.to_string(), source_file);
                    }
                }
            }
        }
    }
    Ok(source_files)
}
