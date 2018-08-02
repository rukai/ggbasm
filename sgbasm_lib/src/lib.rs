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
                    "bin" => SourceFile::Binary (load_binary(&path)?),
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

static LOGO: [u8; 0x30] = [0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00,
                           0x83, 0x00, 0x0C, 0x00, 0x0D, 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89,
                           0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99, 0xBB,
                           0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F,
                           0xBB, 0xB9, 0x33, 0x3E];

pub fn source_to_rom(source_files: &HashMap<String, SourceFile>) -> Result<Vec<u8>, Error> {
    if let Some(SourceFile::Asm(_)) = source_files.get("main") {
        let name = String::from("Heartache");

        let mut rom: Vec<u8> = vec!();

        for _ in 0..256 {
            rom.push(0);
        }
        rom.push(0x0);

        // jump to 0x0 because why not
        rom.push(0xc3);
        rom.push(0x00);
        rom.push(0x00);

        rom.extend(LOGO.iter());

        rom.extend(name.as_bytes());
        rom.push(0x00);

        // fill up 32 KB remaining
        for _ in 0..0x8000-rom.len() {
            rom.push(0x00);
        }

        Ok(rom)
    } else {
        bail!("No main.asm file");
    }
}
