#![feature(rust_2018_preview)]

use failure::Error;

use ggbasm::rom_builder::RomBuilder;
use ggbasm::header::{Header, ColorSupport, CartridgeType, RamType};

fn main() {
    if let Err(error) = run() {
        println!("{}", error);
        println!("{}", error.backtrace());
    }
    else {
        println!("Compiled project to ferris.gb");
    }
}

fn run() -> Result<(), Error> {
    let header = Header {
        title:          String::from("Ferris"),
        color_support:  ColorSupport::Unsupported,
        licence:        String::new(),
        sgb_support:    false,
        cartridge_type: CartridgeType::Mbc5Ram,
        ram_type:       RamType::Some32KB,
        japanese:       false,
        version_number: 0,
    };

    RomBuilder::new()?
        .add_basic_interrupts_and_jumps()?
        .add_header(header)?
        .add_asm_file("main.asm")?
        .write_to_disk("ferris.gb")?;
    Ok(())
}
