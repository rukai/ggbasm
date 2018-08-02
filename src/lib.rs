#![feature(rust_2018_preview)]

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::fs;
use std::io::{Read, Write};
use std::path::{PathBuf, Path};

use failure::Error;
use failure::bail;

pub mod header;

use crate::header::{Header, CartridgeType};

pub enum Instruction {
    Label (String),
    Nop
}

impl Instruction {
    pub fn bytes(&self) -> Vec<u8> {
        let mut bytes = vec!();
        bytes
    }
}

pub enum Data {
    Instructions (Vec<Instruction>),
    Binary       (Vec<u8>),
    Header (Header),
    DummyInterruptsAndJumps,
}

impl Data {
    pub fn len(&self) -> usize {
        match self {
            Data::Instructions (instructions) => instructions.iter().map(|x| x.bytes().len()).sum(),
            Data::Binary (bytes)              => bytes.len(),
            Data::Header (_)                  => 0x45,
            Data::DummyInterruptsAndJumps     => 0x104,
        }
    }
}

pub enum DataSource {
    File (String),
    Code /* TODO: Include stacktrace */
}

pub struct DataHolder {
    data:    Data,
    source:  DataSource,
    address: u32,
}

/// Keeps track of the state of a rom as it is being constructed.
/// Keeps track of the current address and inserts binary data and instructions at that address.
/// The address is advanced when binary data or instructions are added and can also be manually advanced.
/// The address can only be advanced, it can never go backwards.
///
/// The offsets specified by a section instruction will cause the space between to be skipped.
/// Because the builder can not go backwards, this means the space in between is now unusable.
pub struct RomBuilder {
    data:    Vec<DataHolder>,
    address: u32,
}

impl RomBuilder {
    /// Creates a RomBuilder.
    pub fn new() -> RomBuilder {
        RomBuilder {
            data:    vec!(),
            address: 0,
        }
    }

    /// Adds dummy intterupt and jump data from 0x0000 to 0x0103.
    /// The entry point jumps to 0x0150.
    /// The interrupts return immediately.
    /// The RST commands jump to the entry point.
    /// Returns an error if the RomBuilder address is not at 0x0000.
    pub fn add_dummy_interrupts_and_jumps(mut self) -> Result<Self, Error> {
        if self.address != 0x0000 {
            bail!("Attempted to add header data when address != 0x0000");
        }

        self.data.push(DataHolder {
            data:    Data::DummyInterruptsAndJumps,
            address: 0,
            source:  DataSource::Code,
        });
        self.address = 0x104;

        Ok(self)
    }

    /// Adds provided header data at 0x0104 to 0x149.
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
            data:    Data::Header (header),
            address: self.address,
            source:  DataSource::Code,
        });
        self.address = 0x149;

        Ok(self)
    }

    /// Includes binary data in the rom.
    /// The name is used to reference the address in assembly code.
    /// Returns an error if crosses rom bank boundaries
    pub fn add_binary_data(mut self, data: Vec<u8>, name: &str) -> Result<Self, Error> {
        Ok(self)
    }

    /// This function is used to include a *.asm file from the gbasm folder.
    /// Returns an error if crosses rom bank boundaries.
    /// Returns an error if encounters file system issues.
    pub fn add_asm_file(mut self, file_name: &str) -> Result<Self, Error> {
        Ok(self)
    }

    /// This function is used to include instructions in the rom.
    /// Returns an error if crosses rom bank boundaries.
    pub fn add_instructions(mut self, instructions: Vec<Instruction>) -> Result<Self, Error> {
        let len: usize = instructions.iter().map(|x| x.bytes().len()).sum();
        self.data.push(DataHolder {
            data:    Data::Instructions(instructions),
            address: self.address,
            source:  DataSource::Code,
        });

        self.address += len as u32;
        Ok(self)
    }

    /// Sets the current address and bank as specified.
    /// Returns an error if attempts to go backwards.
    pub fn advance_address(mut self, address: u32, rom_bank: u32) -> Result<Self, Error> {
        let new_address = address + rom_bank * ROM_BANK_SIZE;
        if new_address >= self.address {
            bail!("Attempted to advance to a previous address")
        } else {
            self.address = new_address;
            Ok(self)
        }
    }

    /// Gets the current address within the entire rom
    pub fn get_address_global(&self) -> u32 {
        self.address
    }

    /// Gets the current address within the current bank
    pub fn get_address_bank(&self) -> u32 {
        self.address % ROM_BANK_SIZE
    }

    /// Gets the current address within the current bank
    pub fn get_bank(&self) -> u32 {
        self.address / ROM_BANK_SIZE
    }

    /// Compiles assembly and binary data into binary rom data
    pub fn compile(self) -> Result<Vec<u8>, Error> {
        let mut rom = vec!();

        let rom_size_factor = if let Some(data) = self.data.last() {
            let size = data.address + data.data.len() as u32;
            if size <= ROM_BANK_SIZE * 2 {
                0
            } else if size <= ROM_BANK_SIZE * 4 {
                1
            } else if size <= ROM_BANK_SIZE * 8 {
                2
            } else if size <= ROM_BANK_SIZE * 16 {
                3
            } else if size <= ROM_BANK_SIZE * 32 {
                4
            } else if size <= ROM_BANK_SIZE * 64 {
                5
            } else if size <= ROM_BANK_SIZE * 128 {
                6
            } else if size <= ROM_BANK_SIZE * 256 {
                7
            } else if size <= ROM_BANK_SIZE * 512 {
                8
            } else {
                bail!("ROM is too big, there is no MBC that supports a ROM size larger than 8MB, raw ROM size was {}", size);
            }
        } else {
            bail!("No instructions or binary data was added to the RomBuilder");
        };

        for data in self.data {
            match data.data {
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
                    for _ in 0..0x98 {
                        rom.push(0x00);
                    }

                    // jump to 0x0150 because why not
                    rom.push(0x00);
                    rom.push(0xc3);
                    rom.push(0x01);
                    rom.push(0x50);
                }
                Data::Header (header) => {
                    header.write(&mut rom, rom_size_factor as u8);
                }
                Data::Binary (_) => {
                    // TODO
                }
                Data::Instructions (_) => {
                    // TODO
                }
            }

            // pad to address
            for _ in 0..data.address as i32 - rom.len() as i32 {
                rom.push(0x00);
            }
        }

        if rom.len() < 0x14F {
            bail!("ROM is too small, header is not finished.");
        }

        // verify cartridge_type and rom_size_factor are compatible
        let cartridge_type = CartridgeType::variant(rom[0x0147]);
        let final_size_factor = rom[0x0148];
        let final_size = (ROM_BANK_SIZE * 2) << final_size_factor;
        match cartridge_type {
            CartridgeType::RomOnly | CartridgeType::RomRam | CartridgeType::RomRamBattery => {
                if final_size_factor != 0 {
                    bail!("ROM is too big, there is no MBC so ROM size must be <= 32KB, was actually {}", final_size);
                }
            }
            CartridgeType::Mbc1 | CartridgeType::Mbc1Ram |CartridgeType::Mbc1RamBattery => {
                if final_size_factor > 6 {
                    bail!("ROM is too big, using MBC1 so ROM size must be <= 2MB, was actually {}", final_size);
                }
            }
            CartridgeType::Mbc2 | CartridgeType::Mbc2Battery => {
                if final_size_factor > 3 {
                    bail!("ROM is too big, using MBC2 so ROM size must be <= 256KB, was actually {}", final_size);
                }
            }
            CartridgeType::Mmm01 | CartridgeType::Mmm01Ram | CartridgeType::Mmm01RamBattery => {
                // TODO
            }
            CartridgeType::Mbc3TimerBattery | CartridgeType::Mbc3TimerRamBattery | CartridgeType::Mbc3 |
            CartridgeType::Mbc3Ram | CartridgeType::Mbc3RamBattery => {
                if final_size_factor > 6 {
                    bail!("ROM is too big, using MBC3 so ROM size must be <= 2MB, was actually {}", final_size);
                }
            }
            CartridgeType::Mbc5 | CartridgeType::Mbc5Ram | CartridgeType::Mbc5RamBattery |
            CartridgeType::Mbc5Rumble | CartridgeType::Mbc5RumbleRam | CartridgeType::Mbc5RumbleRamBattery => {
                if final_size_factor > 8 {
                    bail!("ROM is too big, using MBC5 so ROM size must be <= 8MB, was actually {}", final_size);
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
                    bail!("ROM is too big, using HuC1 so ROM size must be <= 2MB, was actually {}", final_size);
                }
            }
            CartridgeType::Unknown (_) => {
                // Hopefully you know what your doing ...
            }
        }

        // pad remainder of rom with 0's to fill size
        for _ in 0..final_size-rom.len() as u32 {
            rom.push(0x00);
        }

        Ok(rom)
    }
}

pub const CPU_SPEED_HZ:  u32 = 4_194_304;
pub const ROM_BANK_SIZE: u32 = 0x4000;
pub const RAM_BANK_SIZE: u32 = 0x2000;
pub const SCREEN_WIDTH:  u32 = 160;
pub const SCREEN_HEIGHT: u32 = 144;
pub const SCREEN_PIXELS: u32 = SCREEN_WIDTH * SCREEN_HEIGHT;
