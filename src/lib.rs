//! The main API of GGBASM is the [RomBuilder] struct.
//!
//! Various methods are called on the RomBuilder to insert assembly, graphics and raw bytes.
//!
//!```
//! # fn foo() -> Result<(), anyhow::Error> {
//! # use ggbasm::header::*;
//! # let colors_map = std::collections::HashMap::new();
//! # let header = Header {
//! #     title:          String::from(""),
//! #     color_support:  ColorSupport::Unsupported,
//! #     licence:        String::new(),
//! #     sgb_support:    false,
//! #     cartridge_type: CartridgeType::Mbc5Ram,
//! #     ram_type:       RamType::Some32KB,
//! #     japanese:       false,
//! #     version_number: 0,
//! # };
//!
//! use ggbasm::RomBuilder;
//!
//! RomBuilder::new()?
//!    // Starts off in the first rom bank
//!    // A simple example doesnt need to deal with interrupts and jumps, so generate a dummy
//!    .add_basic_interrupts_and_jumps()?
//!
//!    // generate a header from data in the passed header struct
//!    .add_header(header)?
//!
//!    // Add game code via an asm file
//!    .add_asm_file("main.asm")?
//!
//!    // Add an image to the second rom bank
//!    .advance_address(1, 0)?
//!    .add_image("tiles.png", "Tileset", &colors_map)?
//!
//!    // Consume the RomBuilder and write the rom to disk
//!    .write_to_disk("my_cool_game.gb")?;
//! # Ok(())
//! # }
//!```
//!
//! The RomBuilder searches for images in the `graphics` directory and assembly files in the
//! `gbasm` directory.
//! These directories are in the root directory of the crate, the innermost directory containing a
//! `Cargo.toml` file.
//!
//! ## Parser
//!
//! If you are after a lower level api, the [parser] and [ast] modules can be used without the RomBuilder.
//! You can also construct the ast types yourself and give them to the RomBuilder.

#![recursion_limit = "1024"] // Used for large nom parsers

pub mod ast;
pub mod audio;
pub mod constants;
pub mod header;
pub mod parser;

mod rom_builder;
pub use self::rom_builder::Color;
pub use self::rom_builder::RomBuilder;
