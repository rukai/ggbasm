use anyhow::Error;

use ggbasm::RomBuilder;
use ggbasm::header::{Header, ColorSupport, CartridgeType, RamType};

fn main() {
    run().unwrap(); // unwrap so that CI will fail on an error
    println!("Compiled project to ferris.gb");
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
        .add_audio_player()?
        .add_audio_file("ferris_theme.txt")?
        .add_asm_file("ram.asm")?
        .write_to_disk("ferris.gb")?;
    Ok(())
}
