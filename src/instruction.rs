use std::collections::HashMap;

use failure::Error;
use failure::bail;
use byteorder::{LittleEndian, ByteOrder};

use crate::constants::*;

#[derive(PartialEq, Debug)]
pub enum ExprU8 {
    Ident (String),
    U8 (u8),
}

#[derive(PartialEq, Debug)]
pub enum ExprU16 {
    Ident (String),
    U16 ([u8; 2]),
}

impl ExprU16 {
    pub fn get_bytes(&self, ident_to_address: &HashMap<String, u32>) -> Result<[u8; 2], Error> {
        match self {
            ExprU16::Ident (ident) => {
                match ident_to_address.get(ident) {
                    Some(address) => {
                        let mut result = [0, 0];
                        LittleEndian::write_u16(&mut result, *address as u16);
                        Ok(result)
                    }
                    None => bail!("Identifier {} can not be found", ident)
                }
            }
            ExprU16::U16 (bytes) => Ok(bytes.clone())
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Instruction {
    /// Keeping track of empty lines makes it easier to refer errors back to a line number
    EmptyLine,
    /// the address within the current ROM bank
    AdvanceAddress (u16),
    Label (String),
    Db (Vec<u8>),
    Nop,
    Stop,
    Halt,
    Di,
    Ei,
    Ret,
    Reti,
    Jp (ExprU16),
}

impl Instruction {
    pub fn write_to_rom(&self, rom: &mut Vec<u8>, ident_to_address: &HashMap<String, u32>) -> Result<(), Error> {
        match self {
            Instruction::AdvanceAddress (advance_address) => {
                let address_bank = (rom.len() as u32 % ROM_BANK_SIZE) as u16;
                assert!(*advance_address >= address_bank, "This should be handled by RomBuilder::add_instructions_inner");
                for _ in 0..(advance_address - address_bank) {
                    rom.push(0x00);
                }
            }
            Instruction::EmptyLine  => { }
            Instruction::Label (_)  => { }
            Instruction::Db (bytes) => rom.extend(bytes.iter()),
            Instruction::Nop        => rom.push(0x00),
            Instruction::Stop       => rom.push(0x10),
            Instruction::Halt       => rom.push(0x76),
            Instruction::Di         => rom.push(0xF3),
            Instruction::Ei         => rom.push(0xFB),
            Instruction::Ret        => rom.push(0xC9),
            Instruction::Reti       => rom.push(0xD9),
            Instruction::Jp (expr) => {
                rom.push(0xC3);
                rom.extend(expr.get_bytes(ident_to_address)?.iter());
            }
        }
        Ok(())
    }

    pub fn len(&self, start_address: u16) -> u16 {
        match self {
            Instruction::AdvanceAddress (advance_address) => advance_address - start_address,
            Instruction::EmptyLine  => 0,
            Instruction::Label (_)  => 0,
            Instruction::Db (bytes) => bytes.len() as u16,
            Instruction::Nop        => 1,
            Instruction::Stop       => 1,
            Instruction::Halt       => 1,
            Instruction::Di         => 1,
            Instruction::Ei         => 1,
            Instruction::Ret        => 1,
            Instruction::Reti       => 1,
            Instruction::Jp (_)     => 3,
        }
    }
}
