use ggbasm::rom_builder::RomBuilder;

fn main() {
    // unwrap so that CI will fail on an error
    RomBuilder::new().unwrap()
        .add_asm_file("header.asm").unwrap()
        .write_to_disk("asm_header.gb").unwrap();
    println!("Compiled project to asm_header.gb");
}
