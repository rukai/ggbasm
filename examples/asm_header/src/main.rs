#![feature(rust_2018_preview)]

use failure::Error;

use ggbasm::rom_builder::RomBuilder;

fn main() {
    if let Err(error) = run() {
        println!("{}", error);
        println!("{}", error.backtrace());
    }
    else {
        println!("Compiled project to asm_header.gb");
    }
}

fn run() -> Result<(), Error> {
    RomBuilder::new()?
        .add_asm_file("header.asm")?
        .write_to_disk("asm_header.gb")?;
    Ok(())
}
