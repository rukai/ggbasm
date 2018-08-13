use std::collections::HashMap;

use failure::Error;
use failure::bail;
use byteorder::{LittleEndian, ByteOrder};

use crate::constants::*;

#[derive(PartialEq, Debug)]
pub enum Expr {
    Ident (String),
    Const (i64),
    Binary (Box<BinaryExpr>),
    Unary (Box<UnaryExpr>),
}

#[derive(PartialEq, Debug)]
pub struct BinaryExpr {
    left: Expr,
    operator: BinaryOperator,
    right: Expr,
}

#[derive(PartialEq, Debug)]
pub struct UnaryExpr {
    operator: UnaryOperator,
    expr: Expr,
}

#[derive(PartialEq, Debug)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

#[derive(PartialEq, Debug)]
pub enum UnaryOperator {
    Minus,
}

impl Expr {
    pub fn get_2bytes(&self, ident_to_address: &HashMap<String, u32>) -> Result<[u8; 2], Error> {
        let mut result = [0, 0];
        LittleEndian::write_u16(&mut result, self.run(ident_to_address)? as u16);
        Ok(result)
    }

    pub fn get_byte(&self, ident_to_address: &HashMap<String, u32>) -> Result<u8, Error> {
        Ok(self.run(ident_to_address)? as u8)
    }

    fn run(&self, ident_to_address: &HashMap<String, u32>) -> Result<i64, Error> {
        Ok(match self {
            Expr::Ident (ident) => {
                match ident_to_address.get(ident) {
                    Some(address) => {
                        *address as i64
                    }
                    None => bail!("Identifier {} can not be found", ident)
                }
            }
            Expr::Const (value) => *value,
            Expr::Binary (binary) => {
                let left = binary.left.run(ident_to_address)?;
                let right = binary.left.run(ident_to_address)?;
                match binary.operator {
                    BinaryOperator::Add => {
                        match left.checked_add(right) {
                            Some(value) => value,
                            None        => bail!("Addition overflowed: {:?} + {:?}", binary.left, binary.right)
                        }
                    }
                    BinaryOperator::Sub => {
                        match left.checked_sub(right) {
                            Some(value) => value,
                            None        => bail!("Subtraction underflowed: {:?} - {:?}", binary.left, binary.right)
                        }
                    }
                    BinaryOperator::Mul => {
                        match left.checked_mul(right) {
                            Some(value) => value,
                            None        => bail!("Multiplication overflowed: {:?} * {:?}", binary.left, binary.right)
                        }
                    }
                    BinaryOperator::Div => {
                        if right == 0 {
                            bail!("Attempted to divide by zero: {:?} / {:?}", binary.left, binary.right)
                        }
                        match left.checked_div(right) {
                            Some(value) => value,
                            None        => bail!("Division overflowed: {:?} / {:?}", binary.left, binary.right)
                        }
                    }
                    BinaryOperator::Rem => {
                        if right == 0 {
                            bail!("Attempted to divide by zero (remainder): {:?} % {:?}", binary.left, binary.right)
                        }
                        match left.checked_div(right) {
                            Some(value) => value,
                            None        => bail!("Remainder overflowed: {:?} % {:?}", binary.left, binary.right)
                        }
                    }
                }
            }
            Expr::Unary (unary) => {
                match unary.operator {
                    UnaryOperator::Minus => {
                        let value = unary.expr.run(ident_to_address)?;
                        match value.checked_neg() {
                            Some(value) => value,
                            None        => bail!("Failed to get negative value of: {}", value)
                        }
                    }
                }
            }
        })
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
pub enum Flag {
    Always,
    Z,
    NZ,
    C,
    NC
}

/// Key:
/// R16 - 16 bit register
/// R8  - 8 bit register
/// Rhl - the hl register
/// Ra  - the a register
/// MR16 - 8 bit value in memory pointed at by a 16 bit register
/// MRhl - 8 bit value in memory pointed at by the HL register
/// MRc - 8 bit value in memory pointed at by 0xFF + the register C
/// I8  - immediate 8 bit value
/// I16 - immediate 16 bit value
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
    Ret (Flag),
    Reti,
    Call  (Flag, Expr),
    JpI16 (Flag, Expr),
    JpRhl,
    Jr (Flag, Expr),
    IncR16 (Reg16),
    IncR8  (Reg8),
    IncMRhl,
    DecR16 (Reg16),
    DecR8  (Reg8),
    DecMRhl,
    LdR16I16 (Reg16, Expr),
    LdMI16Rsp (Expr),
    LdMRbcRa,
    LdMRdeRa,
    LdRaMRbc,
    LdRaMRde,
    LdR8R8  (Reg8, Reg8),
    LdR8I8  (Reg8, Expr),
    LdR8MRhl (Reg8),
    LdMRhlR8 (Reg8),
    LdMRhlI8 (Expr),
    LdMI16Ra (Expr),
    LdRaMI16 (Expr),
    LdhRaMI8 (Expr),
    LdhMI8Ra (Expr),
    LdhRaMRc,
    LdhMRcRa,
    LdiMRhlRa,
    LddMRhlRa,
    LdiRaMRhl,
    LddRaMRhl,
    LdRhlRspI8 (Expr),
    LdRspRhl,
    Push (Reg16Push),
    Pop  (Reg16Push),
}

impl Instruction {
    /// Writes the instructions bytes to the passed rom
    /// If an expr in the instruction uses an identifier than it looks up the value for it in ident_to_address
    /// Will return Err if ident_to_address doesnt contain the required label
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
            Instruction::Reti       => rom.push(0xD9),
            Instruction::Ret (flag)  => {
                match flag {
                    Flag::Always => rom.push(0xC9),
                    Flag::Z      => rom.push(0xC8),
                    Flag::C      => rom.push(0xC9),
                    Flag::NZ     => rom.push(0xC0),
                    Flag::NC     => rom.push(0xD0),
                }
            }
            Instruction::Call (flag, expr)  => {
                match flag {
                    Flag::Always => rom.push(0xCD),
                    Flag::Z      => rom.push(0xCC),
                    Flag::C      => rom.push(0xDC),
                    Flag::NZ     => rom.push(0xC4),
                    Flag::NC     => rom.push(0xD4),
                }
                rom.extend(expr.get_2bytes(ident_to_address)?.iter());
            }
            Instruction::JpI16 (flag, expr) => {
                match flag {
                    Flag::Always => rom.push(0xC3),
                    Flag::Z      => rom.push(0xCA),
                    Flag::C      => rom.push(0xDA),
                    Flag::NZ     => rom.push(0xC2),
                    Flag::NC     => rom.push(0xD2),
                }
                rom.extend(expr.get_2bytes(ident_to_address)?.iter());
            }
            Instruction::JpRhl => rom.push(0xE9),
            Instruction::Jr (flag, expr) => {
                match flag {
                    Flag::Always => rom.push(0x18),
                    Flag::Z      => rom.push(0x28),
                    Flag::C      => rom.push(0x38),
                    Flag::NZ     => rom.push(0x20),
                    Flag::NC     => rom.push(0x30),
                }
                rom.push(expr.get_byte(ident_to_address)?);
            }
            Instruction::IncR16 (reg) => {
                match reg {
                    Reg16::BC => rom.push(0x03),
                    Reg16::DE => rom.push(0x13),
                    Reg16::HL => rom.push(0x23),
                    Reg16::SP => rom.push(0x33),
                }
            }
            Instruction::IncR8 (reg) => {
                match reg {
                    Reg8::A => rom.push(0x3C),
                    Reg8::B => rom.push(0x04),
                    Reg8::C => rom.push(0x0C),
                    Reg8::D => rom.push(0x14),
                    Reg8::E => rom.push(0x1C),
                    Reg8::H => rom.push(0x24),
                    Reg8::L => rom.push(0x2C),
                }
            }
            Instruction::IncMRhl => rom.push(0x034),
            Instruction::DecR16 (reg) => {
                match reg {
                    Reg16::BC => rom.push(0x0B),
                    Reg16::DE => rom.push(0x1B),
                    Reg16::HL => rom.push(0x2B),
                    Reg16::SP => rom.push(0x3B),
                }
            }
            Instruction::DecR8 (reg) => {
                match reg {
                    Reg8::A => rom.push(0x3D),
                    Reg8::B => rom.push(0x05),
                    Reg8::C => rom.push(0x0D),
                    Reg8::D => rom.push(0x15),
                    Reg8::E => rom.push(0x1D),
                    Reg8::H => rom.push(0x25),
                    Reg8::L => rom.push(0x2D),
                }
            }
            Instruction::DecMRhl => rom.push(0x035),
            Instruction::LdR16I16 (reg, expr) => {
                match reg {
                    Reg16::BC => rom.push(0x01),
                    Reg16::DE => rom.push(0x11),
                    Reg16::HL => rom.push(0x21),
                    Reg16::SP => rom.push(0x31),
                }
                rom.extend(expr.get_2bytes(ident_to_address)?.iter());
            }
            Instruction::LdMI16Rsp (expr) => {
                rom.push(0x08);
                rom.extend(expr.get_2bytes(ident_to_address)?.iter());
            }
            Instruction::LdR8I8 (reg, expr) => {
                match reg {
                    Reg8::A => rom.push(0x3E),
                    Reg8::B => rom.push(0x06),
                    Reg8::C => rom.push(0x0E),
                    Reg8::D => rom.push(0x16),
                    Reg8::E => rom.push(0x1E),
                    Reg8::H => rom.push(0x26),
                    Reg8::L => rom.push(0x2E),
                }
                rom.push(expr.get_byte(ident_to_address)?);
            }
            Instruction::LdR8R8 (reg_in, reg_out) => {
                let mut byte = 0;
                // first 5 bits
                byte |= match reg_in {
                    Reg8::A => 0x78,
                    Reg8::B => 0x40,
                    Reg8::C => 0x48,
                    Reg8::D => 0x50,
                    Reg8::E => 0x58,
                    Reg8::H => 0x60,
                    Reg8::L => 0x68,
                };

                // last 3 bits
                byte |= match reg_out {
                    Reg8::A => 0x07,
                    Reg8::B => 0x00,
                    Reg8::C => 0x01,
                    Reg8::D => 0x02,
                    Reg8::E => 0x03,
                    Reg8::H => 0x04,
                    Reg8::L => 0x05,
                };

                rom.push(byte);
            }
            Instruction::LdMRbcRa => rom.push(0x02),
            Instruction::LdMRdeRa => rom.push(0x12),
            Instruction::LdRaMRbc => rom.push(0x0A),
            Instruction::LdRaMRde => rom.push(0x1A),
            Instruction::LdR8MRhl (reg) => {
                match reg {
                    Reg8::A => rom.push(0x7E),
                    Reg8::B => rom.push(0x46),
                    Reg8::C => rom.push(0x4E),
                    Reg8::D => rom.push(0x56),
                    Reg8::E => rom.push(0x5E),
                    Reg8::H => rom.push(0x66),
                    Reg8::L => rom.push(0x6E),
                }
            }
            Instruction::LdMRhlR8 (reg) => {
                match reg {
                    Reg8::A => rom.push(0x77),
                    Reg8::B => rom.push(0x70),
                    Reg8::C => rom.push(0x71),
                    Reg8::D => rom.push(0x72),
                    Reg8::E => rom.push(0x73),
                    Reg8::H => rom.push(0x74),
                    Reg8::L => rom.push(0x75),
                }
            }
            Instruction::LdMRhlI8 (expr) => {
                rom.push(0x36);
                rom.push(expr.get_byte(ident_to_address)?);
            }
            Instruction::LdMI16Ra (expr) => {
                rom.push(0xEA);
                rom.extend(expr.get_2bytes(ident_to_address)?.iter());
            }
            Instruction::LdRaMI16 (expr) => {
                rom.push(0xFA);
                rom.extend(expr.get_2bytes(ident_to_address)?.iter());
            }
            Instruction::LdhRaMI8 (expr) => {
                rom.push(0xF0);
                rom.push(expr.get_byte(ident_to_address)?);
            }
            Instruction::LdhMI8Ra (expr) => {
                rom.push(0xE0);
                rom.push(expr.get_byte(ident_to_address)?);
            }
            Instruction::LdhRaMRc  => rom.push(0xF2),
            Instruction::LdhMRcRa  => rom.push(0xE2),
            Instruction::LdiMRhlRa => rom.push(0x22),
            Instruction::LddMRhlRa => rom.push(0x32),
            Instruction::LdiRaMRhl => rom.push(0x2A),
            Instruction::LddRaMRhl => rom.push(0x3A),
            Instruction::LdRspRhl  => rom.push(0xF9),
            Instruction::LdRhlRspI8 (expr) => {
                rom.push(0xF8);
                rom.push(expr.get_byte(ident_to_address)?);
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

    /// Returns how many bytes the instruction takes up
    pub fn len(&self, start_address: u16) -> u16 {
        match self {
            Instruction::AdvanceAddress (advance_address) => advance_address - start_address,
            Instruction::EmptyLine       => 0,
            Instruction::Label (_)       => 0,
            Instruction::Db (bytes)      => bytes.len() as u16,
            Instruction::Nop             => 1,
            Instruction::Stop            => 1,
            Instruction::Halt            => 1,
            Instruction::Di              => 1,
            Instruction::Ei              => 1,
            Instruction::Ret (_)         => 1,
            Instruction::Reti            => 1,
            Instruction::Call (_,_)      => 3,
            Instruction::JpI16 (_,_)     => 3,
            Instruction::JpRhl           => 1,
            Instruction::Jr (_,_)        => 2,
            Instruction::IncR16 (_)      => 1,
            Instruction::IncR8 (_)       => 1,
            Instruction::IncMRhl         => 1,
            Instruction::DecR16 (_)      => 1,
            Instruction::DecR8 (_)       => 1,
            Instruction::DecMRhl         => 1,
            Instruction::LdR16I16 (_, _) => 3,
            Instruction::LdMI16Rsp (_)   => 3,
            Instruction::LdR8I8 (_, _)   => 2,
            Instruction::LdR8R8 (_, _)   => 1,
            Instruction::LdMRbcRa        => 1,
            Instruction::LdMRdeRa        => 1,
            Instruction::LdRaMRbc        => 1,
            Instruction::LdRaMRde        => 1,
            Instruction::LdR8MRhl (_)    => 1,
            Instruction::LdMRhlR8 (_)    => 1,
            Instruction::LdMRhlI8 (_)    => 2,
            Instruction::LdMI16Ra (_)    => 3,
            Instruction::LdRaMI16 (_)    => 3,
            Instruction::LdhRaMI8 (_)    => 2,
            Instruction::LdhMI8Ra (_)    => 2,
            Instruction::LdhRaMRc        => 1,
            Instruction::LdhMRcRa        => 1,
            Instruction::LdiMRhlRa       => 1,
            Instruction::LddMRhlRa       => 1,
            Instruction::LdiRaMRhl       => 1,
            Instruction::LddRaMRhl       => 1,
            Instruction::LdRhlRspI8 (_)  => 2,
            Instruction::LdRspRhl        => 1,
            Instruction::Push (_)        => 1,
            Instruction::Pop  (_)        => 1,
        }
    }
}
