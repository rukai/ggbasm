# GGBASM (Generating Gameboy Assembler)

A gameboy assembler as a rust crate library.
Being a library instead of command line application, allows for an elegant combination of:
*   binary data and instructions be generated from rust code
*   instructions read from *.asm files.

## RomBuilder

The RomBuilder is the core rust api of GGBASM.

```rust
let rom = RomBuilder::new()
    // Starts off in the first rom bank
    // A simple example doesnt need to deal with interrupts and jumps, so generate a dummy
    .add_dummy_interrupts_and_jumps()?

    // generate a header from data in the passed header struct
    .add_header(header)?

    // Add game code via an asm file
    .add_asm_file("main.asm")?

    // Add an image to the second rom bank
    .advance_address(0, 1)?
    .add_binary(image);

    // Consume the RomBuilder into a Vec<u8>
    .compile()?;
```

## Comparison with rgbasm

rgbasm is the only other gameboy assembler i've used.
I assume other assemblers are similar.

*   rgbasm requires only *.asm files, while ggbasm requires *.asm, and an entire rust crate.
*   rgbasm needs to run `rgbasm -o main.obj src/main.asm; rgblink -m game.map -n game.sym -o out.gb main.obj; rgbfix -p 0 -v out.gb` to build the rom, while ggbasm uses `cargo run` to build the rom
*   rgbasm uses includes inside the *.asm files, while ggbasm uses rust to insert instructions and binary data at the correct location in the rom.
*   ggbasm has helper functions for generating binary data such as: png_to_gb_sprite
*   rgbasm has its own variant of intel syntax, while ggbasm uses actual intel syntax
