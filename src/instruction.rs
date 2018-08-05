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
    U16 (u16),
}

impl ExprU16 {
    pub fn get_bytes(&self, ident_to_address: &HashMap<String, u32>) -> Result<[u8; 2], Error> {
        let value = match self {
            ExprU16::Ident (ident) => {
                match ident_to_address.get(ident) {
                    Some(address) => (*address as u16),
                    None => bail!("Identifier {} can not be found", ident)
                }
            }
            ExprU16::U16 (value) => *value,
        };
        let mut result = [0, 0];
        LittleEndian::write_u16(&mut result, value);
        Ok(result)
    }
}

#[derive(PartialEq, Debug)]
pub enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(PartialEq, Debug)]
pub enum Reg16 {
    BC,
    DE,
    HL,
    SP,
}

#[derive(PartialEq, Debug)]
pub enum Reg16Push {
    BC,
    DE,
    HL,
    AF,
}

#[derive(PartialEq, Debug)]
pub enum Instruction {
    /// Keeping track of empty lines makes it easier to refer errors back to a line number
    EmptyLine, // TODO: Combine this and the Option returned by the parser into a new enum
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
    LdReg16Immediate (Reg16, ExprU16),
    Jp (ExprU16),
    Push (Reg16Push),
    Pop (Reg16Push),
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
            Instruction::LdReg16Immediate (reg, expr) => {
                match reg {
                    Reg16::BC => rom.push(0x01),
                    Reg16::DE => rom.push(0x11),
                    Reg16::HL => rom.push(0x21),
                    Reg16::SP => rom.push(0x31),
                }
                rom.extend(expr.get_bytes(ident_to_address)?.iter());
            }
            Instruction::Push (reg) => {
                match reg {
                    Reg16Push::BC => rom.push(0xC5),
                    Reg16Push::DE => rom.push(0xD5),
                    Reg16Push::HL => rom.push(0xE5),
                    Reg16Push::AF => rom.push(0xF5),
                }
            }
            Instruction::Pop (reg) => {
                match reg {
                    Reg16Push::BC => rom.push(0xC1),
                    Reg16Push::DE => rom.push(0xD1),
                    Reg16Push::HL => rom.push(0xE1),
                    Reg16Push::AF => rom.push(0xF1),
                }
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
            Instruction::LdReg16Immediate (_, _) => 3,
            Instruction::Push (_) => 1,
            Instruction::Pop  (_) => 1,
        }
    }
}
