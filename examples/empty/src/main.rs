#![feature(rust_2018_preview)]

use std::env;
use std::fs::File;
use std::io::Write;

use failure::Error;

use ggbasm::RomBuilder;
use ggbasm::header::{Header, ColorSupport, CartridgeType, RamType};

fn main() {
    if let Err(error) = run() {
        println!("{}", error);
    }
}

fn run() -> Result<(), Error> {
    let header = Header {
        title:          String::from("Empty"),
        color_support:  ColorSupport::Unsupported,
        licence:        String::new(),
        sgb_support:    false,
        cartridge_type: CartridgeType::RomOnly,
        ram_type:       RamType::None,
        japanese:       false,
        version_number: 0,
    };

    let rom = RomBuilder::new()
        .add_dummy_interrupts_and_jumps()?
        .add_header(header)?
        .compile()?;

    let output = env::current_dir()?.join("empty.gb");
    let mut output_file = File::create(&output)?;
    output_file.write(&rom)?;
    if let Some(name) = output.file_name() {
        if let Some(name) = name.to_str() {
            println!("Compiled project to: {}", name);
        }
    }
    Ok(())
}
