//! Contains the main API of GGBASM.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Error};

use crate::ast::{Expr, ExprRunError, Instruction};
use crate::audio;
use crate::constants::*;
use crate::header::{CartridgeType, Header};
use crate::parser;

/// Represents a color in modern images.
/// Used when mapping colors from modern images to gameboy graphics.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Color {
        Color { red, green, blue }
    }
}

enum Data {
    Instructions(Vec<Instruction>),
    Binary(Vec<u8>),
    Header(Header),
    DummyInterruptsAndJumps,
}

/// Keeps track of where data came from, used to generate error messages.
enum DataSource {
    AsmFile(String),
    AudioFile(String),
    AudioPlayer,
    Code, /* TODO: Include stacktrace */
}

impl DataSource {
    pub fn description(&self) -> String {
        match self {
            DataSource::Code => "data generated by rust code".to_string(),
            DataSource::AudioPlayer => {
                "instructions generated by the built-in ggbasm audio player".to_string()
            }
            DataSource::AudioFile(name) => {
                format!("instructions generated by audio file: {}", name)
            }
            DataSource::AsmFile(name) => format!("instructions generated by asm file {}", name),
        }
    }
}

struct DataHolder {
    data: Data,
    #[allow(dead_code)]
    source: DataSource,
    /// address within the entire rom
    address: u32,
}

/// Keeps track of the state of a rom as it is being constructed.
///
/// Keeps track of the current address and inserts binary data and instructions at that address.
/// The address is advanced when binary data or instructions are added and can also be manually advanced.
/// When manually advanced the area in between is filled with zeroes.
/// The address can only be advanced, it can never go backwards.
///
/// In *.asm files, the advance_address instruction will cause the space between the last instruction .
/// and the new address to be filled with zeroes.
pub struct RomBuilder {
    data: Vec<DataHolder>,
    address: u32,
    root_dir: PathBuf,
    constants: HashMap<String, i64>,
}

impl RomBuilder {
    /// Creates a RomBuilder.
    pub fn new() -> Result<RomBuilder, Error> {
        Ok(RomBuilder {
            data: vec![],
            address: 0,
            root_dir: RomBuilder::root_dir()?,
            constants: HashMap::new(),
        })
    }

    /// Adds basic interrupt and jump data from 0x0000 to 0x0103.
    ///
    /// The entry point jumps to 0x0150.
    /// The interrupts return immediately.
    /// The RST commands jump to the entry point.
    /// Returns an error if the RomBuilder address is not at 0x0000.
    pub fn add_basic_interrupts_and_jumps(mut self) -> Result<Self, Error> {
        if self.address != 0x0000 {
            bail!("Attempted to add header data when address != 0x0000");
        }

        self.data.push(DataHolder {
            data: Data::DummyInterruptsAndJumps,
            address: 0,
            source: DataSource::Code,
        });
        self.address = 0x104;

        Ok(self)
    }

    /// Adds provided header data at 0x0104 to 0x149.
    ///
    /// Returns an error if the RomBuilder address is not at 0x104
    pub fn add_header(mut self, header: Header) -> Result<Self, Error> {
        if self.address != 0x0104 {
            bail!("Attempted to add header data when address != 0x0104");
        }

        if header.title.as_bytes().len() > 0x10 {
            bail!("Header title was larger than 16 bytes.");
        }

        if header.title.as_bytes().len() == 0x10 && header.color_support.is_supported() {
            bail!("Header title was 16 bytes while supporting color.");
        }

        if header.licence.as_bytes().len() > 2 {
            bail!("Header licence was larger than 2 bytes.");
        }

        self.data.push(DataHolder {
            data: Data::Header(header),
            address: self.address,
            source: DataSource::Code,
        });
        self.address = 0x150;

        Ok(self)
    }

    /// Includes raw bytes in the rom.
    /// The name is used to reference the address in assembly code.
    /// Returns an error if crosses rom bank boundaries.
    pub fn add_bytes(mut self, bytes: Vec<u8>, identifier: &str) -> Result<Self, Error> {
        let len = bytes.len() as u32;
        if self
            .constants
            .insert(identifier.to_string(), self.address as i64)
            .is_some()
        {
            // TODO: Display first usage
            bail!("Identifier {} is already used", identifier)
        }

        self.data.push(DataHolder {
            data: Data::Binary(bytes),
            address: self.address,
            source: DataSource::Code,
        });

        let prev_bank = self.get_bank();
        self.address += len as u32;
        if prev_bank == self.get_bank() {
            Ok(self)
        } else {
            bail!("The added instructions cross bank boundaries.");
        }
    }

    /// Includes graphics data generated from the provided image file in the graphics folder.
    ///
    /// The name is used to reference the address in assembly code.
    /// Returns an error if crosses rom bank boundaries.
    /// The color_map argument specifes how to convert 24 bit rgb color values into the 2 bit color values used by the gameboy.
    ///
    /// TODO: Describe the format of generated images.
    pub fn add_image(
        mut self,
        file_name: &str,
        identifier: &str,
        color_map: &HashMap<Color, u8>,
    ) -> Result<Self, Error> {
        if self
            .constants
            .insert(identifier.to_string(), self.address as i64)
            .is_some()
        {
            // TODO: Display first usage
            bail!("Identifier {} is already used", identifier)
        }

        let path = self.root_dir.as_path().join("graphics").join(file_name);
        let image = match image::open(path) {
            Ok(image) => image,
            Err(err) => bail!("Cannot read file {} because: {}", file_name, err),
        };
        let mut bytes = vec![];
        let image = image.to_rgb8();
        for vert_tile in 0..(image.height() / 8) {
            for hor_tile in 0..(image.width() / 8) {
                for vert_line in 0..8 {
                    let mut byte0 = 0x00;
                    let mut byte1 = 0x00;
                    for hor_line in 0..8 {
                        let x = hor_tile * 8 + hor_line;
                        let y = vert_tile * 8 + vert_line;
                        let rgb = image.get_pixel(x, y);
                        let color = Color::new(rgb[0], rgb[1], rgb[2]);

                        if let Some(gb_color) = color_map.get(&color) {
                            byte0 |= (gb_color & 0b01) << (7 - hor_line);
                            byte1 |= ((gb_color & 0b10) >> 1) << (7 - hor_line);
                        } else {
                            bail!("Color::new(0x{:x}, 0x{:x}, 0x{:x}) is not mapped to a gameboy color", color.red, color.green, color.blue);
                        }
                    }
                    bytes.push(byte0);
                    bytes.push(byte1);
                }
            }
        }
        let size = bytes.len();

        self.data.push(DataHolder {
            data: Data::Binary(bytes),
            address: self.address,
            source: DataSource::Code,
        });

        let prev_bank = self.get_bank();
        self.address += size as u32;
        if prev_bank == self.get_bank() {
            Ok(self)
        } else {
            bail!("The added bytes cross bank boundaries.");
        }
    }

    /// Includes audio data generated from the provided ggbasm audio text file in the audio folder.
    ///
    /// Returns an error if crosses rom bank boundaries.
    ///
    /// Currently only supports playing one track at a time, playing sound effects on top of
    /// background music might not be possible with the current architecture. Oops...
    ///
    /// # Format
    ///
    /// There are long lines containing the data for each sound register and how long to wait
    /// before running the next command.
    /// There are also various commands to e.g. set a label or continue playing from a specific label
    ///
    /// ```gbaudio
    /// label my_cool_song
    /// 07                       D6:2:10:7:4Y:NY
    /// 07                       D6:2:10:7:4Y:NY
    /// 07  D6:2:10:7:4Y:NY:Y00                 
    /// 07                       D6:2:10:7:4Y:NY
    /// 07  D6:2:10:7:4Y:NY:Y00  D6:2:10:7:4Y:NY
    /// playfrom my_cool_song
    /// ```
    ///
    /// # Channel formats
    ///
    /// Data for each channel is written on the same line like this:
    ///
    /// ```gbaudio
    /// RST CHANNEL1             CHANNEL2         CHANNEL3      CHANNEL4
    /// 0F  D6:2:10:7:4Y:NY:Y00  D6:2:10:7:4Y:NY  TODO          TODO
    /// ```
    ///
    /// Only changes between lines are included in the audio data.
    ///
    /// ## Channel 1 format:
    ///
    /// ```gbaudioformat
    /// AB:C:DD:E:FG:HI:JKL
    /// ```
    ///
    /// Key:
    ///
    /// *   A:  Note                    A-G (natural), a-g (sharp)
    /// *   B:  Octave                  1-8
    /// *   C:  Duty                    0-3
    /// *   DD: length                  0-3F
    /// *   E:  envelope initial volume 0-F
    /// *   F:  envelope argument       0-7
    /// *   G:  envelope increase       Y/N
    /// *   H:  enable length           Y/N
    /// *   I:  initial                 Y/N
    /// *   J:  sweep increase          Y/N
    /// *   K:  sweep time              0-7
    /// *   L:  number of sweeps        0-7
    ///
    /// For example: `D6:2:10:7:4Y:NY:Y00`
    ///
    /// ## Channel 2 format:
    ///
    /// ```gbaudioformat
    /// AB:C:DD:E:FG:HI
    /// ```
    ///
    /// Key:
    ///
    /// *   A:  Note                    A-G (natural), a-g (sharp)
    /// *   B:  Octave                  1-8
    /// *   C:  Duty                    0-3
    /// *   DD: length                  0-3F
    /// *   E:  envelope initial volume 0-F
    /// *   F:  envelope argument       0-7
    /// *   G:  envelope increase       Y/N
    /// *   H:  enable length           Y/N
    /// *   I:  initial                 Y/N
    ///
    /// For example: `D6:2:10:7:4Y:NY`
    ///
    /// ## Channel 3 format:
    ///
    /// TODO
    ///
    /// ## Channel 4 format:
    ///
    /// TODO
    ///
    /// # Control lines
    ///
    /// *   rest AA - rest AA frames before continuing
    /// *   jp foo  - set the GGBASMAudio
    /// *   disable - disables audio by setting the value at GGBASMAudioEnable to 0
    ///
    /// TODO: Maybe syntax highlighting could help make the audio format more readable
    pub fn add_audio_file(self, file_name: &str) -> Result<Self, Error> {
        let path = self.root_dir.as_path().join("audio").join(file_name);
        let text = match fs::read_to_string(path) {
            Ok(file) => file,
            Err(err) => bail!("Cannot read audio file {} because: {}", file_name, err),
        };

        let lines = match audio::parse_audio_text(&text) {
            Ok(lines) => lines,
            Err(err) => bail!("Cannot parse audio file {} because: {}", file_name, err),
        };

        let data = match audio::generate_audio_data(lines) {
            Ok(lines) => lines,
            Err(err) => bail!(
                "Cannot generate audio from file {} because: {}",
                file_name,
                err
            ),
        };

        self.add_instructions_inner(data, DataSource::AudioFile(file_name.to_string()))
    }

    /// Includes bytecodes generated from the audio player
    ///
    /// Returns an error if crosses rom bank boundaries.
    ///
    /// # Functions
    ///
    /// This should be called once during initialization:
    /// ```asm
    /// call GGBASMInitAudio
    /// ```
    ///
    /// This should be called once per frame:
    /// ```asm
    /// call GGBASMStepAudio
    /// ```
    ///
    /// # RAM Locations
    ///
    /// These identifiers need to be set to some unused ram values.
    /// ```asm
    /// GGBASMAudioEnable    EQU 0xC020 ; dont process music when 0 otherwise process it
    /// GGBASMAudioBank      EQU 0xC021 ; the bank the currently playing song is stored on
    /// GGBASMAudioPointerLo EQU 0xC022 ; pointer to the currently playing song
    /// GGBASMAudioPointerHi EQU 0xC023
    /// GGBASMAudioRest      EQU 0xC024 ; rest for this many steps
    /// ```
    ///
    /// Change the currently playing song by setting GGBASMAudioBank, GGBASMAudioPointerHi and GGBASMAudioPointerLo to the
    /// address of the song you want to play
    ///
    /// Make sure the memory is accessible (correct bank enabled) whenever an audio function is called.
    pub fn add_audio_player(self) -> Result<Self, Error> {
        let text = include_str!("audio_player.asm");
        let instructions = parser::parse_asm(text)
            .unwrap()
            .into_iter()
            .enumerate()
            .map(|(i, x)| {
                x.unwrap_or_else(|| {
                    panic!("Invalid instruction on line {} of audio_player.asm", i + 1)
                })
            })
            .collect();
        self.add_instructions_inner(instructions, DataSource::AudioPlayer)
    }

    /// Includes bytecodes generated from the provided assembly file in the gbasm folder.
    ///
    /// TODO: Document the syntax.
    /// Its very similar to the [RGBDS syntax](https://rednex.github.io/rgbds/gbz80.7.html) with the addition of the advance_address command.
    /// However we should have our syntax documentation listing every instruction and every operator in rom compile time expressions.
    ///
    /// Returns an error if crosses rom bank boundaries.
    /// Returns an error if encounters file system issues.
    pub fn add_asm_file(self, file_name: &str) -> Result<Self, Error> {
        let path = self.root_dir.as_path().join("gbasm").join(file_name);
        let text = match fs::read_to_string(path) {
            Ok(file) => file,
            Err(err) => bail!("Cannot read asm file {} because: {}", file_name, err),
        };

        let option_instructions = match parser::parse_asm(&text) {
            Ok(instructions) => instructions,
            Err(err) => bail!("Cannot parse asm file {} because: {}", file_name, err),
        };

        let mut instructions = vec![];
        for (i, instruction) in option_instructions.into_iter().enumerate() {
            match instruction {
                Some(instruction) => instructions.push(instruction),
                None => {
                    // TODO: Return a proper BuildError enum instead of relying on failure::Error
                    // TODO: Then I can have a pretty_print() method on it that displays something like:
                    // ```
                    // 103: halt   // color green
                    // 104: nop    // color green
                    // 105: foobar // color red
                    // 106: nop    // color white
                    // An error occured on line 105 of foo_file.asm // color red
                    // ```
                    //
                    // TODO: Even better I could handle multiple errors in one message, I have the
                    // information given from the parser already, I just need to handle it from RomBuilder.
                    bail!("Invalid instruction on line {} of {}", i + 1, file_name)
                }
            }
        }

        self.add_instructions_inner(instructions, DataSource::AsmFile(file_name.to_string()))
    }

    /// This function is used to include instructions in the rom.
    /// Returns an error if crosses rom bank boundaries.
    pub fn add_instructions(self, instructions: Vec<Instruction>) -> Result<Self, Error> {
        self.add_instructions_inner(instructions, DataSource::Code)
    }

    fn add_instructions_inner(
        mut self,
        instructions: Vec<Instruction>,
        source: DataSource,
    ) -> Result<Self, Error> {
        let mut cur_address = self.address;
        for (i, instruction) in instructions.iter().enumerate() {
            if let Instruction::Label(label) = instruction {
                if self
                    .constants
                    .insert(label.to_string(), cur_address as i64)
                    .is_some()
                {
                    // TODO: Display first usage
                    bail!(
                        "Identifier {} is used twice: One usage occured in {} on line {}",
                        label,
                        source.description(),
                        i + 1
                    );
                }
            } else {
                cur_address += instruction.bytes_len((cur_address % ROM_BANK_SIZE) as u16) as u32;
            }
        }

        self.data.push(DataHolder {
            data: Data::Instructions(instructions),
            address: self.address,
            source,
        });

        let prev_bank = self.get_bank();
        self.address = cur_address as u32;
        if prev_bank == self.get_bank() {
            Ok(self)
        } else {
            bail!("The added instructions cross bank boundaries.");
        }
    }

    /// Sets the current address and bank as specified.
    /// Returns an error if attempts to go backwards.
    /// To cross bank boundaries you need to use this function.
    pub fn advance_address(mut self, rom_bank: u32, address: u32) -> Result<Self, Error> {
        let new_address = address + rom_bank * ROM_BANK_SIZE;
        if new_address >= self.address {
            self.address = new_address;
            Ok(self)
        } else {
            bail!("Attempted to advance to a previous address.")
        }
    }

    /// Gets the current address within the entire rom.
    pub fn get_address_global(&self) -> u32 {
        self.address
    }

    /// Gets the current address within the current bank.
    pub fn get_address_bank(&self) -> u16 {
        (self.address % ROM_BANK_SIZE) as u16
    }

    /// Gets the current bank.
    pub fn get_bank(&self) -> u32 {
        self.address / ROM_BANK_SIZE
    }

    // TODO: Doesnt include EQU constants. consume self, move EQU processing into another function
    // then call it here as well.
    pub fn print_variables_by_value(self) -> Result<Self, Error> {
        let mut sorted: Vec<_> = self.constants.iter().collect();
        sorted.sort_by_key(|x| x.1);
        for (ident, value) in sorted {
            println!("0x{:x} - {}", value, ident);
        }
        Ok(self)
    }

    pub fn print_variables_by_identifier(self) -> Result<Self, Error> {
        let mut sorted: Vec<_> = self.constants.iter().collect();
        sorted.sort_by_key(|x| x.0.to_lowercase());
        for (ident, value) in sorted {
            println!("{} - 0x{:x}", ident, value);
        }
        Ok(self)
    }

    /// Compiles assembly and binary data into binary rom data.
    pub fn compile(mut self) -> Result<Vec<u8>, Error> {
        if self.data.last().is_none() {
            bail!("No instructions or binary data was added to the RomBuilder");
        }

        let rom_size_factor = if self.address <= ROM_BANK_SIZE * 2 {
            0
        } else if self.address <= ROM_BANK_SIZE * 4 {
            1
        } else if self.address <= ROM_BANK_SIZE * 8 {
            2
        } else if self.address <= ROM_BANK_SIZE * 16 {
            3
        } else if self.address <= ROM_BANK_SIZE * 32 {
            4
        } else if self.address <= ROM_BANK_SIZE * 64 {
            5
        } else if self.address <= ROM_BANK_SIZE * 128 {
            6
        } else if self.address <= ROM_BANK_SIZE * 256 {
            7
        } else if self.address <= ROM_BANK_SIZE * 512 {
            8
        } else {
            bail!("ROM is too big, there is no MBC that supports a ROM size larger than 8MB, raw ROM size was {}", self.address);
        };

        let mut rom = vec![];

        #[derive(Clone)]
        struct EquHolder<'a> {
            pub ident: &'a String,
            pub expr: &'a Expr,
            pub source: &'a DataSource,
            pub line: u64,
        }
        let mut equs = vec![];

        for data in &self.data {
            match &data.data {
                Data::DummyInterruptsAndJumps => {}
                Data::Header(_) => {}
                Data::Binary { .. } => {}
                Data::Instructions(instructions) => {
                    for (i, instruction) in instructions.iter().enumerate() {
                        if let Instruction::Equ(ident, expr) = instruction {
                            equs.push(EquHolder {
                                expr,
                                ident,
                                source: &data.source,
                                line: i as u64 + 1,
                            });
                        }
                    }
                }
            }
        }

        let constants = &mut self.constants;
        while !equs.is_empty() {
            let prev_size = equs.len();
            let mut outer_error = None;
            let mut missing_idents = vec![];
            equs.retain(|equ| {
                match equ.expr.run(constants) {
                    Ok(value) => {
                        if constants.insert(equ.ident.clone(), value).is_some() {
                            // TODO: Display first usage
                            outer_error = Some(format!("Identifier {} is declared twice: One usage occured in {} on line {}", &equ.ident, equ.source.description(), equ.line));
                        }
                        false
                    }
                    Err(ExprRunError::MissingIdentifier (ident)) => {
                        // MissingIdentifier can mean:
                        // *    There is a reference to an identifier that hasnt been processed yet. And it is succesfully processed later.
                        // *    There is a reference to an identifier that hasnt been processed yet. But it turns out to be an infinite loop.
                        // *    There is a reference to an identifier that is not declared anywhere.
                        // We store the values so we can handle these cases after the `retain`, due to the mutable borrow.
                        missing_idents.push((equ.clone(), ident));
                        true
                    }
                    Err(ExprRunError::ResultDoesntFit (error)) |
                    Err(ExprRunError::ArithmeticError (error)) => {
                        outer_error = Some(format!("Error occured in {} on line {}: {}", equ.source.description(), equ.line, error));
                        true
                    }
                }
            });
            if let Some(error) = outer_error {
                bail!(error);
            }

            // Check if the reason the ident was missing is because it is never declared.
            for (missing_ident_equ, missing_ident) in missing_idents {
                let mut found_ident = false;
                for search_equ in &equs {
                    if &missing_ident == search_equ.ident {
                        found_ident = true;
                        break;
                    }
                }
                if !found_ident {
                    bail!(format!(
                        "Identifier {} is used in {} on line {} but is never declared.",
                        missing_ident,
                        missing_ident_equ.source.description(),
                        missing_ident_equ.line
                    ));
                }
            }

            // Generic check for an infinite loop.
            if prev_size == equs.len() {
                let mut fail_string = String::from("Cannot resolve constants, there is an infinite loop involving the following identifiers:\n");
                for equ in equs {
                    fail_string.push_str(&format!("*   {}\n", equ.ident));
                }
                bail!(fail_string);
            }
        }

        // generate rom
        for data in &self.data {
            // pad to address
            rom.resize(data.address as usize, 0x00);

            match &data.data {
                Data::DummyInterruptsAndJumps => {
                    // jumps
                    for _ in 0..8 {
                        rom.push(0xc3);
                        rom.push(0x00);
                        rom.push(0x01);
                        rom.push(0x00);

                        rom.push(0x00);
                        rom.push(0x00);
                        rom.push(0x00);
                        rom.push(0x00);
                    }

                    // interrupts
                    for _ in 0..5 {
                        rom.push(0xd9);
                        rom.push(0x00);
                        rom.push(0x00);
                        rom.push(0x00);

                        rom.push(0x00);
                        rom.push(0x00);
                        rom.push(0x00);
                        rom.push(0x00);
                    }

                    // padding
                    rom.resize(rom.len() + 0x98, 0x00);

                    // jump to 0x0150
                    rom.push(0x00);
                    rom.push(0xc3);
                    rom.push(0x50);
                    rom.push(0x01);
                }
                Data::Header(header) => {
                    header.write(&mut rom, rom_size_factor as u8);
                }
                Data::Binary(bytes) => {
                    rom.extend(bytes);
                }
                Data::Instructions(instructions) => {
                    for (i, instruction) in instructions.iter().enumerate() {
                        if let Err(err) = instruction.write_to_rom(&mut rom, &self.constants) {
                            bail!(
                                "Error occured in {} on line {}: {}",
                                data.source.description(),
                                i + 1,
                                err
                            );
                        }
                    }
                }
            }
        }

        if rom.len() < 0x14F {
            bail!(
                "ROM is too small, header is not finished. ROM was only {} bytes",
                rom.len()
            );
        }

        // verify cartridge_type and rom_size_factor are compatible
        let cartridge_type = CartridgeType::variant(rom[0x0147]);
        let final_size_factor = rom[0x0148];
        if final_size_factor >= 0x20 {
            bail!(
                "ROM size factor (0x0148) is too big, needs to be less than 32 was {}",
                final_size_factor
            );
        }
        let final_size = (ROM_BANK_SIZE * 2) << final_size_factor;
        match cartridge_type {
            CartridgeType::RomOnly | CartridgeType::RomRam | CartridgeType::RomRamBattery => {
                if final_size_factor != 0 {
                    bail!("ROM is too big, there is no MBC so ROM size must be <= 32KB, was actually {}", final_size);
                }
            }
            CartridgeType::Mbc1 | CartridgeType::Mbc1Ram | CartridgeType::Mbc1RamBattery => {
                if final_size_factor > 6 {
                    bail!(
                        "ROM is too big, using MBC1 so ROM size must be <= 2MB, was actually {}",
                        final_size
                    );
                }
            }
            CartridgeType::Mbc2 | CartridgeType::Mbc2Battery => {
                if final_size_factor > 3 {
                    bail!(
                        "ROM is too big, using MBC2 so ROM size must be <= 256KB, was actually {}",
                        final_size
                    );
                }
            }
            CartridgeType::Mmm01 | CartridgeType::Mmm01Ram | CartridgeType::Mmm01RamBattery => {
                // TODO
            }
            CartridgeType::Mbc3TimerBattery
            | CartridgeType::Mbc3TimerRamBattery
            | CartridgeType::Mbc3
            | CartridgeType::Mbc3Ram
            | CartridgeType::Mbc3RamBattery => {
                if final_size_factor > 6 {
                    bail!(
                        "ROM is too big, using MBC3 so ROM size must be <= 2MB, was actually {}",
                        final_size
                    );
                }
            }
            CartridgeType::Mbc5
            | CartridgeType::Mbc5Ram
            | CartridgeType::Mbc5RamBattery
            | CartridgeType::Mbc5Rumble
            | CartridgeType::Mbc5RumbleRam
            | CartridgeType::Mbc5RumbleRamBattery => {
                if final_size_factor > 8 {
                    bail!(
                        "ROM is too big, using MBC5 so ROM size must be <= 8MB, was actually {}",
                        final_size
                    );
                }
            }
            CartridgeType::PocketCamera => {
                if final_size_factor > 8 {
                    bail!("ROM is too big, using PocketCamera so ROM size must be <= 1MB, was actually {}", final_size);
                }
            }
            CartridgeType::HuC3 => {
                // TODO
            }
            CartridgeType::HuC1RamBattery => {
                if final_size_factor > 6 {
                    bail!(
                        "ROM is too big, using HuC1 so ROM size must be <= 2MB, was actually {}",
                        final_size
                    );
                }
            }
            CartridgeType::Unknown(_) => {
                // Hopefully you know what your doing ...
            }
        }

        // pad remainder of rom with 0's to fill size
        rom.resize(final_size as usize, 0x00);

        Ok(rom)
    }

    /// Compile the ROM then write it to disk at the root of the project.
    /// The root of the project is the outermost directory containing a Cargo.toml file.
    pub fn write_to_disk(self, name: &str) -> Result<(), Error> {
        let output = self.root_dir.as_path().join(name);
        let rom = self.compile()?;
        fs::write(output, rom)?;
        Ok(())
    }

    /// Provide some sort of mechanism to generate an html file with embedded gb emulator and rom data.
    /// Use Cargo.toml metadata to generate a link to repository, include developers name etc. (use panic-handler as a reference here)
    /// This is completely unimplemented, its just a reminder to do this some day.
    pub fn write_to_disk_html(self, _name: &str) -> Result<(), Error> {
        unimplemented!();
    }

    /// Iteratively search for the innermost Cargo.toml starting at the current.
    /// working directory and working up through its parents.
    /// Returns the path to the directory the Cargo.toml is in.
    /// Or an error if the file couldn't be found.
    ///
    /// TODO: This function returns the wrong path if called like `cargo run -p crate_name` in a workspace.
    /// The only way to fix this would be get rustc provide an equivalent of env!("CARGO_MANIFEST_DIR") provided at runtime.
    fn root_dir() -> Result<PathBuf, Error> {
        let current_dir = env::current_dir()?;
        let mut current = current_dir.as_path();

        loop {
            let toml = current.join("Cargo.toml");
            if fs::metadata(&toml).is_ok() {
                return Ok(toml.parent().unwrap().to_path_buf());
            }

            match current.parent() {
                Some(p) => current = p,
                None => bail!("Cant find a Cargo.toml in any of the parent directories"),
            }
        }
    }
}
