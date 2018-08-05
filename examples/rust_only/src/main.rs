use failure::Error;

use ggbasm::rom_builder::RomBuilder;
use ggbasm::header::{Header, ColorSupport, CartridgeType, RamType};

fn main() {
    run().unwrap(); // unwrap so that CI will fail on an error
    println!("Compiled project to rust_only.gb");
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

    RomBuilder::new()?
        .add_basic_interrupts_and_jumps()?
        .add_header(header)?
        .write_to_disk("rust_only.gb")?;
    Ok(())
}
