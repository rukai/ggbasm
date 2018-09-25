# GGBASM (Generating Gameboy Assembler)
[![Build Status](https://travis-ci.org/rukai/ggbasm.svg?branch=master)](https://travis-ci.org/rukai/ggbasm) [![dependency status](https://deps.rs/repo/github/rukai/ggbasm/status.svg)](https://deps.rs/repo/github/rukai/ggbasm) [![Crates.io](https://img.shields.io/crates/v/ggbasm.svg)](https://crates.io/crates/ggbasm)

A gameboy assembler as a rust crate library.
Being a library instead of command line application, allows for an elegant combination of:
*   raw bytes and instructions generated from rust code
*   instructions read from *.asm files.

## Rust version

Requires nightly rust, because I thought it would be fun to use rust 2018 and I didn't realize I would make progress this quickly >.>

## Docs

[![docs.rs](https://docs.rs/mio/ggbasm.svg)](https://docs.rs/crate/ggbasm)
Docs.rs doesnt work because ggbasm requires nightly.
Instead you can use `cargo doc` on your local machine.

## RomBuilder

The RomBuilder is the core rust api of GGBASM.

```rust
RomBuilder::new()?
    // Starts off in the first rom bank
    // A simple example doesnt need to deal with interrupts and jumps, so generate a dummy
    .add_basic_interrupts_and_jumps()?

    // generate a header from data in the passed header struct
    .add_header(header)?

    // Add game code via an asm file
    .add_asm_file("main.asm")?

    // Add an image to the second rom bank
    .advance_address(1, 0)?
    .add_image("tiles.png", "GraphicsBinary", &colors_map)?

    // Consume the RomBuilder and write the rom to disk
    .write_to_disk("my_cool_game.gb")?;
```

## Examples

Check out the [examples folder](https://github.com/rukai/ggbasm/tree/master/examples) and [heartacheGB](https://github.com/rukai/HeartacheGB).

## Comparison with [RGBDS](https://github.com/rednex/rgbds)

*   RGBDS requires only *.asm files, while GGBASM requires *.asm, and an entire rust crate.
*   RGBDS needs to run `RGBDS -o main.obj src/main.asm; rgblink -m game.map -n game.sym -o out.gb main.obj; rgbfix -p 0 -v out.gb` to build the rom, while GGBASM uses `cargo run` to build the rom
*   RGBDS uses includes inside the *.asm files, while GGBASM uses rust to insert instructions and raw bytes at the correct location in the rom.
*   GGBASM has helper functions for generating bytes such as: png_to_gb_sprite
*   RGBDS has its own intel-like syntax, GGBASM syntax uses RGBDS syntax with a few additions. Changes from RGBDS are:
    +   hexadecimal can be represented as 0x2a as well as $2a
    +   uses `advance_address 0xYYYY` instead of `section "FOO",$HOME[$YY]`
