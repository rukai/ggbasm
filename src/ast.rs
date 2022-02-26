//! The AST produced by the parser.
//!
//! You can manually create the types below and give them to the RomBuilder via RomBuilder::add_instructions(instructions)

use std::collections::HashMap;

use anyhow::{bail, Error};
use byteorder::{ByteOrder, LittleEndian};
use thiserror::Error as ThisError;

use crate::constants::*;

/// Assembly uses constant expressions to avoid copying magic numbers around.
/// Expr represents these constant expressions.
///
/// The run method evaluates the constant expression.
/// The get_2bytes, get_byte and get_bit_index evaluate the constant expression but also convert
/// to a specific low level type needed by instructions.
#[derive(Clone, PartialEq, Debug)]
pub enum Expr {
    Ident(String),
    Const(i64),
    Binary(Box<BinaryExpr>),
    Unary(Box<UnaryExpr>),
}

impl Expr {
    pub fn binary(left: Expr, operator: BinaryOperator, right: Expr) -> Expr {
        Expr::Binary(Box::new(BinaryExpr {
            left,
            operator,
            right,
        }))
    }

    pub fn unary(expr: Expr, operator: UnaryOperator) -> Expr {
        Expr::Unary(Box::new(UnaryExpr { expr, operator }))
    }

    pub fn get_2bytes(&self, constants: &HashMap<String, i64>) -> Result<[u8; 2], ExprRunError> {
        let value = self.run(constants)?;
        if value > 0xFFFF {
            Err(ExprRunError::ResultDoesntFit(format!(
                "0x{} > 0xFFFF This is invalid because the value needs to fit in two bytes",
                value
            )))
        } else {
            let mut result = [0, 0];
            LittleEndian::write_u16(&mut result, value as u16);
            Ok(result)
        }
    }

    pub fn get_byte(&self, constants: &HashMap<String, i64>) -> Result<u8, ExprRunError> {
        let value = self.run(constants)?;
        if value > 0xFF {
            Err(ExprRunError::ResultDoesntFit(format!(
                "0x{:x} > 0xFF This is invalid because the value needs to fit in one byte",
                value
            )))
        } else {
            Ok(value as u8)
        }
    }

    pub fn get_bit_index(&self, constants: &HashMap<String, i64>) -> Result<u8, ExprRunError> {
        let value = self.run(constants)?;
        if value > 7 {
            Err(ExprRunError::ResultDoesntFit(format!(
                "{} > 7 This is invalid because the value needs to index bits in a byte.",
                value
            )))
        } else {
            Ok(value as u8)
        }
    }

    pub fn run(&self, constants: &HashMap<String, i64>) -> Result<i64, ExprRunError> {
        match self {
            Expr::Ident(ident) => match constants.get(ident) {
                Some(address) => Ok(*address as i64),
                None => Err(ExprRunError::MissingIdentifier(ident.clone())),
            },
            Expr::Const(value) => Ok(*value),
            Expr::Binary(binary) => {
                let left = binary.left.run(constants)?;
                let right = binary.right.run(constants)?;
                match binary.operator {
                    BinaryOperator::Add => match left.checked_add(right) {
                        Some(value) => Ok(value),
                        None => Err(ExprRunError::ArithmeticError(format!(
                            "Addition overflowed: {:?} + {:?}",
                            binary.left, binary.right
                        ))),
                    },
                    BinaryOperator::Sub => match left.checked_sub(right) {
                        Some(value) => Ok(value),
                        None => Err(ExprRunError::ArithmeticError(format!(
                            "Subtraction underflowed: {:?} - {:?}",
                            binary.left, binary.right
                        ))),
                    },
                    BinaryOperator::Mul => match left.checked_mul(right) {
                        Some(value) => Ok(value),
                        None => Err(ExprRunError::ArithmeticError(format!(
                            "Multiplication overflowed: {:?} * {:?}",
                            binary.left, binary.right
                        ))),
                    },
                    BinaryOperator::Div => {
                        if right == 0 {
                            Err(ExprRunError::ArithmeticError(format!(
                                "Attempted to divide by zero: {:?} / {:?}",
                                binary.left, binary.right
                            )))
                        } else {
                            match left.checked_div(right) {
                                Some(value) => Ok(value),
                                None => Err(ExprRunError::ArithmeticError(format!(
                                    "Division overflowed: {:?} / {:?}",
                                    binary.left, binary.right
                                ))),
                            }
                        }
                    }
                    BinaryOperator::Rem => {
                        if right == 0 {
                            Err(ExprRunError::ArithmeticError(format!(
                                "Attempted to divide by zero (remainder): {:?} % {:?}",
                                binary.left, binary.right
                            )))
                        } else {
                            match left.checked_rem(right) {
                                Some(value) => Ok(value),
                                None => Err(ExprRunError::ArithmeticError(format!(
                                    "Remainder overflowed: {:?} % {:?}",
                                    binary.left, binary.right
                                ))),
                            }
                        }
                    }
                    BinaryOperator::And => Ok(left & right),
                    BinaryOperator::Or => Ok(left | right),
                    BinaryOperator::Xor => Ok(left ^ right),
                }
            }
            Expr::Unary(unary) => match unary.operator {
                UnaryOperator::Minus => {
                    let value = unary.expr.run(constants)?;
                    match value.checked_neg() {
                        Some(value) => Ok(value),
                        None => Err(ExprRunError::ArithmeticError(format!(
                            "Failed to get negative value of: {}",
                            value
                        ))),
                    }
                }
            },
        }
    }
}

#[derive(Debug, ThisError)]
pub enum ExprRunError {
    #[error("Identifier {0} can not be found.")]
    MissingIdentifier(String),
    #[error("Arithmetic error: {0}")]
    ArithmeticError(String),
    #[error("{0}")]
    ResultDoesntFit(String),
}

#[derive(Clone, PartialEq, Debug)]
pub struct BinaryExpr {
    pub left: Expr,
    pub operator: BinaryOperator,
    pub right: Expr,
}

#[derive(Clone, PartialEq, Debug)]
pub struct UnaryExpr {
    pub operator: UnaryOperator,
    pub expr: Expr,
}

#[derive(Clone, PartialEq, Debug)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Xor,
    Or,
}

#[derive(Clone, PartialEq, Debug)]
pub enum UnaryOperator {
    Minus,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Reg16 {
    BC,
    DE,
    HL,
    SP,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Reg16Push {
    BC,
    DE,
    HL,
    AF,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Flag {
    Always,
    Z,
    NZ,
    C,
    NC,
}

/// The main type in the AST, the parser creates an Instruction for each line in a *.asm
///
/// Key:
/// *   R16  - 16 bit register
/// *   R8   - 8 bit register
/// *   Rhl  - the hl register
/// *   Ra   - the a register
/// *   MR16 - 8 bit value in memory pointed at by a 16 bit register
/// *   MRhl - 8 bit value in memory pointed at by the HL register
/// *   MRc  - 8 bit value in memory pointed at by 0xFF + the register C
/// *   I8   - immediate 8 bit value
/// *   I16  - immediate 16 bit value
/// *   Bit  - an index to a bit
#[derive(Clone, PartialEq, Debug)]
pub enum Instruction {
    /// Keeping track of empty lines makes it easier to refer errors back to a line number
    EmptyLine, // TODO: Combine this and the Option returned by the parser into a new enum
    /// the address within the current ROM bank
    AdvanceAddress(u16),
    Equ(String, Expr),
    Label(String),
    Db(Vec<u8>),
    DbExpr8(Expr),
    DbExpr16(Expr),
    Nop,
    Stop,
    Halt,
    Di,
    Ei,
    Rrca,
    Rra,
    Cpl,
    Ccf,
    Rlca,
    Rla,
    Daa,
    Scf,
    Ret(Flag),
    Reti,
    Call(Flag, Expr),
    JpI16(Flag, Expr),
    JpRhl,
    Jr(Flag, Expr),
    IncR16(Reg16),
    IncR8(Reg8),
    IncMRhl,
    DecR16(Reg16),
    DecR8(Reg8),
    DecMRhl,
    AddR8(Reg8),
    AddMRhl,
    AddI8(Expr),
    AddRhlR16(Reg16),
    AddRspI8(Expr),
    SubR8(Reg8),
    SubMRhl,
    SubI8(Expr),
    AndR8(Reg8),
    AndMRhl,
    AndI8(Expr),
    OrR8(Reg8),
    OrMRhl,
    OrI8(Expr),
    AdcR8(Reg8),
    AdcMRhl,
    AdcI8(Expr),
    SbcR8(Reg8),
    SbcMRhl,
    SbcI8(Expr),
    XorR8(Reg8),
    XorMRhl,
    XorI8(Expr),
    CpR8(Reg8),
    CpMRhl,
    CpI8(Expr),
    LdR16I16(Reg16, Expr),
    LdMI16Rsp(Expr),
    LdMRbcRa,
    LdMRdeRa,
    LdRaMRbc,
    LdRaMRde,
    LdR8R8(Reg8, Reg8),
    LdR8I8(Reg8, Expr),
    LdR8MRhl(Reg8),
    LdMRhlR8(Reg8),
    LdMRhlI8(Expr),
    LdMI16Ra(Expr),
    LdRaMI16(Expr),
    LdhRaMI8(Expr),
    LdhMI8Ra(Expr),
    LdhRaMRc,
    LdhMRcRa,
    LdiMRhlRa,
    LddMRhlRa,
    LdiRaMRhl,
    LddRaMRhl,
    LdRhlRspI8(Expr),
    LdRspRhl,
    Push(Reg16Push),
    Pop(Reg16Push),

    // 0xCB prefix
    RlcR8(Reg8),
    RlcMRhl,
    RrcR8(Reg8),
    RrcMRhl,
    RlR8(Reg8),
    RlMRhl,
    RrR8(Reg8),
    RrMRhl,
    SlaR8(Reg8),
    SlaMRhl,
    SraR8(Reg8),
    SraMRhl,
    SwapR8(Reg8),
    SwapMRhl,
    SrlR8(Reg8),
    SrlMRhl,
    BitBitR8(Expr, Reg8),
    BitBitMRhl(Expr),
    ResBitR8(Expr, Reg8),
    ResBitMRhl(Expr),
    SetBitR8(Expr, Reg8),
    SetBitMRhl(Expr),
}

impl Instruction {
    /// Writes the instructions bytes to the passed rom.
    /// If an expr in the instruction uses an identifier than it looks up the value for it in constants.
    /// Will return Err if constants doesn't contain the required label.
    pub fn write_to_rom(
        &self,
        rom: &mut Vec<u8>,
        constants: &HashMap<String, i64>,
    ) -> Result<(), Error> {
        match self {
            Instruction::AdvanceAddress(advance_address) => {
                let address_bank = (rom.len() as u32 % ROM_BANK_SIZE) as u16;
                assert!(
                    *advance_address >= address_bank,
                    "This should be handled by RomBuilder::add_instructions_inner"
                );
                for _ in 0..(advance_address - address_bank) {
                    rom.push(0x00);
                }
            }
            Instruction::EmptyLine => {}
            Instruction::Equ(_, _) => {}
            Instruction::Label(_) => {}
            Instruction::Db(bytes) => rom.extend(bytes.iter()),
            Instruction::DbExpr8(expr) => rom.push(expr.get_byte(constants)?),
            Instruction::DbExpr16(expr) => rom.extend(expr.get_2bytes(constants)?.iter()),
            Instruction::Nop => rom.push(0x00),
            Instruction::Stop => rom.push(0x10),
            Instruction::Halt => rom.extend([0x76, 0x00].iter()),
            Instruction::Di => rom.push(0xF3),
            Instruction::Ei => rom.push(0xFB),
            Instruction::Rrca => rom.push(0x0F),
            Instruction::Rra => rom.push(0x1F),
            Instruction::Cpl => rom.push(0x2F),
            Instruction::Ccf => rom.push(0x3F),
            Instruction::Rlca => rom.push(0x07),
            Instruction::Rla => rom.push(0x17),
            Instruction::Daa => rom.push(0x27),
            Instruction::Scf => rom.push(0x37),
            Instruction::Reti => rom.push(0xD9),
            Instruction::Ret(flag) => match flag {
                Flag::Always => rom.push(0xC9),
                Flag::Z => rom.push(0xC8),
                Flag::C => rom.push(0xC9),
                Flag::NZ => rom.push(0xC0),
                Flag::NC => rom.push(0xD0),
            },
            Instruction::Call(flag, expr) => {
                match flag {
                    Flag::Always => rom.push(0xCD),
                    Flag::Z => rom.push(0xCC),
                    Flag::C => rom.push(0xDC),
                    Flag::NZ => rom.push(0xC4),
                    Flag::NC => rom.push(0xD4),
                }
                rom.extend(expr.get_2bytes(constants)?.iter());
            }
            Instruction::JpI16(flag, expr) => {
                match flag {
                    Flag::Always => rom.push(0xC3),
                    Flag::Z => rom.push(0xCA),
                    Flag::C => rom.push(0xDA),
                    Flag::NZ => rom.push(0xC2),
                    Flag::NC => rom.push(0xD2),
                }
                rom.extend(expr.get_2bytes(constants)?.iter());
            }
            Instruction::JpRhl => rom.push(0xE9),
            Instruction::Jr(flag, expr) => {
                let abs_dest = expr.run(constants)?;
                let rel_dest = abs_dest - rom.len() as i64 - 2; // 2 accounts for the 2 bytes that the make up the jr instruction
                if rel_dest > 0x7F {
                    bail!("0x{} > 0x7F This is invalid because the value needs to fit in a signed byte", rel_dest);
                } else if rel_dest < -0x80 {
                    bail!("0x{} < 0x80 This is invalid because the value needs to fit in a signed byte", rel_dest);
                }
                match flag {
                    Flag::Always => rom.push(0x18),
                    Flag::Z => rom.push(0x28),
                    Flag::C => rom.push(0x38),
                    Flag::NZ => rom.push(0x20),
                    Flag::NC => rom.push(0x30),
                }
                rom.push(rel_dest as u8);
            }
            Instruction::IncR16(reg) => match reg {
                Reg16::BC => rom.push(0x03),
                Reg16::DE => rom.push(0x13),
                Reg16::HL => rom.push(0x23),
                Reg16::SP => rom.push(0x33),
            },
            Instruction::IncR8(reg) => match reg {
                Reg8::A => rom.push(0x3C),
                Reg8::B => rom.push(0x04),
                Reg8::C => rom.push(0x0C),
                Reg8::D => rom.push(0x14),
                Reg8::E => rom.push(0x1C),
                Reg8::H => rom.push(0x24),
                Reg8::L => rom.push(0x2C),
            },
            Instruction::IncMRhl => rom.push(0x034),
            Instruction::DecR16(reg) => match reg {
                Reg16::BC => rom.push(0x0B),
                Reg16::DE => rom.push(0x1B),
                Reg16::HL => rom.push(0x2B),
                Reg16::SP => rom.push(0x3B),
            },
            Instruction::DecR8(reg) => match reg {
                Reg8::A => rom.push(0x3D),
                Reg8::B => rom.push(0x05),
                Reg8::C => rom.push(0x0D),
                Reg8::D => rom.push(0x15),
                Reg8::E => rom.push(0x1D),
                Reg8::H => rom.push(0x25),
                Reg8::L => rom.push(0x2D),
            },
            Instruction::DecMRhl => rom.push(0x035),
            Instruction::AddR8(reg) => match reg {
                Reg8::A => rom.push(0x87),
                Reg8::B => rom.push(0x80),
                Reg8::C => rom.push(0x81),
                Reg8::D => rom.push(0x82),
                Reg8::E => rom.push(0x83),
                Reg8::H => rom.push(0x84),
                Reg8::L => rom.push(0x85),
            },
            Instruction::AddMRhl => rom.push(0x86),
            Instruction::AddI8(expr) => {
                rom.push(0xC6);
                rom.push(expr.get_byte(constants)?);
            }
            Instruction::AddRhlR16(reg) => match reg {
                Reg16::BC => rom.push(0x09),
                Reg16::DE => rom.push(0x19),
                Reg16::HL => rom.push(0x29),
                Reg16::SP => rom.push(0x39),
            },
            Instruction::AddRspI8(expr) => {
                rom.push(0xE8);
                rom.push(expr.get_byte(constants)?);
            }
            Instruction::SubR8(reg) => match reg {
                Reg8::A => rom.push(0x97),
                Reg8::B => rom.push(0x90),
                Reg8::C => rom.push(0x91),
                Reg8::D => rom.push(0x92),
                Reg8::E => rom.push(0x93),
                Reg8::H => rom.push(0x94),
                Reg8::L => rom.push(0x95),
            },
            Instruction::SubMRhl => rom.push(0x96),
            Instruction::SubI8(expr) => {
                rom.push(0xD6);
                rom.push(expr.get_byte(constants)?);
            }
            Instruction::AndR8(reg) => match reg {
                Reg8::A => rom.push(0xA7),
                Reg8::B => rom.push(0xA0),
                Reg8::C => rom.push(0xA1),
                Reg8::D => rom.push(0xA2),
                Reg8::E => rom.push(0xA3),
                Reg8::H => rom.push(0xA4),
                Reg8::L => rom.push(0xA5),
            },
            Instruction::AndMRhl => rom.push(0xA6),
            Instruction::AndI8(expr) => {
                rom.push(0xE6);
                rom.push(expr.get_byte(constants)?);
            }
            Instruction::OrR8(reg) => match reg {
                Reg8::A => rom.push(0xB7),
                Reg8::B => rom.push(0xB0),
                Reg8::C => rom.push(0xB1),
                Reg8::D => rom.push(0xB2),
                Reg8::E => rom.push(0xB3),
                Reg8::H => rom.push(0xB4),
                Reg8::L => rom.push(0xB5),
            },
            Instruction::OrMRhl => rom.push(0xB6),
            Instruction::OrI8(expr) => {
                rom.push(0xF6);
                rom.push(expr.get_byte(constants)?);
            }
            Instruction::AdcR8(reg) => match reg {
                Reg8::A => rom.push(0x8F),
                Reg8::B => rom.push(0x88),
                Reg8::C => rom.push(0x89),
                Reg8::D => rom.push(0x8A),
                Reg8::E => rom.push(0x8B),
                Reg8::H => rom.push(0x8C),
                Reg8::L => rom.push(0x8D),
            },
            Instruction::AdcMRhl => rom.push(0x8E),
            Instruction::AdcI8(expr) => {
                rom.push(0xCE);
                rom.push(expr.get_byte(constants)?);
            }
            Instruction::SbcR8(reg) => match reg {
                Reg8::A => rom.push(0x9F),
                Reg8::B => rom.push(0x98),
                Reg8::C => rom.push(0x99),
                Reg8::D => rom.push(0x9A),
                Reg8::E => rom.push(0x9B),
                Reg8::H => rom.push(0x9C),
                Reg8::L => rom.push(0x9D),
            },
            Instruction::SbcMRhl => rom.push(0x9E),
            Instruction::SbcI8(expr) => {
                rom.push(0xDE);
                rom.push(expr.get_byte(constants)?);
            }
            Instruction::XorR8(reg) => match reg {
                Reg8::A => rom.push(0xAF),
                Reg8::B => rom.push(0xA8),
                Reg8::C => rom.push(0xA9),
                Reg8::D => rom.push(0xAA),
                Reg8::E => rom.push(0xAB),
                Reg8::H => rom.push(0xAC),
                Reg8::L => rom.push(0xAD),
            },
            Instruction::XorMRhl => rom.push(0xAE),
            Instruction::XorI8(expr) => {
                rom.push(0xEE);
                rom.push(expr.get_byte(constants)?);
            }
            Instruction::CpR8(reg) => match reg {
                Reg8::A => rom.push(0xBF),
                Reg8::B => rom.push(0xB8),
                Reg8::C => rom.push(0xB9),
                Reg8::D => rom.push(0xBA),
                Reg8::E => rom.push(0xBB),
                Reg8::H => rom.push(0xBC),
                Reg8::L => rom.push(0xBD),
            },
            Instruction::CpMRhl => rom.push(0xBE),
            Instruction::CpI8(expr) => {
                rom.push(0xFE);
                rom.push(expr.get_byte(constants)?);
            }
            Instruction::LdR16I16(reg, expr) => {
                match reg {
                    Reg16::BC => rom.push(0x01),
                    Reg16::DE => rom.push(0x11),
                    Reg16::HL => rom.push(0x21),
                    Reg16::SP => rom.push(0x31),
                }
                rom.extend(expr.get_2bytes(constants)?.iter());
            }
            Instruction::LdMI16Rsp(expr) => {
                rom.push(0x08);
                rom.extend(expr.get_2bytes(constants)?.iter());
            }
            Instruction::LdR8I8(reg, expr) => {
                match reg {
                    Reg8::A => rom.push(0x3E),
                    Reg8::B => rom.push(0x06),
                    Reg8::C => rom.push(0x0E),
                    Reg8::D => rom.push(0x16),
                    Reg8::E => rom.push(0x1E),
                    Reg8::H => rom.push(0x26),
                    Reg8::L => rom.push(0x2E),
                }
                rom.push(expr.get_byte(constants)?);
            }
            Instruction::LdR8R8(reg_in, reg_out) => {
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
                byte |= Instruction::reg8_to_bits(&reg_out);

                rom.push(byte);
            }
            Instruction::LdMRbcRa => rom.push(0x02),
            Instruction::LdMRdeRa => rom.push(0x12),
            Instruction::LdRaMRbc => rom.push(0x0A),
            Instruction::LdRaMRde => rom.push(0x1A),
            Instruction::LdR8MRhl(reg) => match reg {
                Reg8::A => rom.push(0x7E),
                Reg8::B => rom.push(0x46),
                Reg8::C => rom.push(0x4E),
                Reg8::D => rom.push(0x56),
                Reg8::E => rom.push(0x5E),
                Reg8::H => rom.push(0x66),
                Reg8::L => rom.push(0x6E),
            },
            Instruction::LdMRhlR8(reg) => match reg {
                Reg8::A => rom.push(0x77),
                Reg8::B => rom.push(0x70),
                Reg8::C => rom.push(0x71),
                Reg8::D => rom.push(0x72),
                Reg8::E => rom.push(0x73),
                Reg8::H => rom.push(0x74),
                Reg8::L => rom.push(0x75),
            },
            Instruction::LdMRhlI8(expr) => {
                rom.push(0x36);
                rom.push(expr.get_byte(constants)?);
            }
            Instruction::LdMI16Ra(expr) => {
                rom.push(0xEA);
                rom.extend(expr.get_2bytes(constants)?.iter());
            }
            Instruction::LdRaMI16(expr) => {
                rom.push(0xFA);
                rom.extend(expr.get_2bytes(constants)?.iter());
            }
            Instruction::LdhRaMI8(expr) => {
                rom.push(0xF0);
                rom.push(expr.get_byte(constants)?);
            }
            Instruction::LdhMI8Ra(expr) => {
                rom.push(0xE0);
                rom.push(expr.get_byte(constants)?);
            }
            Instruction::LdhRaMRc => rom.push(0xF2),
            Instruction::LdhMRcRa => rom.push(0xE2),
            Instruction::LdiMRhlRa => rom.push(0x22),
            Instruction::LddMRhlRa => rom.push(0x32),
            Instruction::LdiRaMRhl => rom.push(0x2A),
            Instruction::LddRaMRhl => rom.push(0x3A),
            Instruction::LdRspRhl => rom.push(0xF9),
            Instruction::LdRhlRspI8(expr) => {
                rom.push(0xF8);
                rom.push(expr.get_byte(constants)?);
            }
            Instruction::Push(reg) => match reg {
                Reg16Push::BC => rom.push(0xC5),
                Reg16Push::DE => rom.push(0xD5),
                Reg16Push::HL => rom.push(0xE5),
                Reg16Push::AF => rom.push(0xF5),
            },
            Instruction::Pop(reg) => match reg {
                Reg16Push::BC => rom.push(0xC1),
                Reg16Push::DE => rom.push(0xD1),
                Reg16Push::HL => rom.push(0xE1),
                Reg16Push::AF => rom.push(0xF1),
            },
            Instruction::RlcR8(reg) => {
                rom.push(0xCB);
                match reg {
                    Reg8::A => rom.push(0x07),
                    Reg8::B => rom.push(0x00),
                    Reg8::C => rom.push(0x01),
                    Reg8::D => rom.push(0x02),
                    Reg8::E => rom.push(0x03),
                    Reg8::H => rom.push(0x04),
                    Reg8::L => rom.push(0x05),
                }
            }
            Instruction::RlcMRhl => {
                rom.push(0xCB);
                rom.push(0x06);
            }
            Instruction::RrcR8(reg) => {
                rom.push(0xCB);
                match reg {
                    Reg8::A => rom.push(0x0f),
                    Reg8::B => rom.push(0x08),
                    Reg8::C => rom.push(0x09),
                    Reg8::D => rom.push(0x0a),
                    Reg8::E => rom.push(0x0b),
                    Reg8::H => rom.push(0x0c),
                    Reg8::L => rom.push(0x0d),
                }
            }
            Instruction::RrcMRhl => {
                rom.push(0xCB);
                rom.push(0x0E);
            }
            Instruction::RlR8(reg) => {
                rom.push(0xCB);
                match reg {
                    Reg8::A => rom.push(0x17),
                    Reg8::B => rom.push(0x10),
                    Reg8::C => rom.push(0x11),
                    Reg8::D => rom.push(0x12),
                    Reg8::E => rom.push(0x13),
                    Reg8::H => rom.push(0x14),
                    Reg8::L => rom.push(0x15),
                }
            }
            Instruction::RlMRhl => {
                rom.push(0xCB);
                rom.push(0x16);
            }
            Instruction::RrR8(reg) => {
                rom.push(0xCB);
                match reg {
                    Reg8::A => rom.push(0x1f),
                    Reg8::B => rom.push(0x18),
                    Reg8::C => rom.push(0x19),
                    Reg8::D => rom.push(0x1a),
                    Reg8::E => rom.push(0x1b),
                    Reg8::H => rom.push(0x1c),
                    Reg8::L => rom.push(0x1d),
                }
            }
            Instruction::RrMRhl => {
                rom.push(0xCB);
                rom.push(0x1E);
            }
            Instruction::SlaR8(reg) => {
                rom.push(0xCB);
                match reg {
                    Reg8::A => rom.push(0x27),
                    Reg8::B => rom.push(0x20),
                    Reg8::C => rom.push(0x21),
                    Reg8::D => rom.push(0x22),
                    Reg8::E => rom.push(0x23),
                    Reg8::H => rom.push(0x24),
                    Reg8::L => rom.push(0x25),
                }
            }
            Instruction::SlaMRhl => {
                rom.push(0xCB);
                rom.push(0x26);
            }
            Instruction::SraR8(reg) => {
                rom.push(0xCB);
                match reg {
                    Reg8::A => rom.push(0x2f),
                    Reg8::B => rom.push(0x28),
                    Reg8::C => rom.push(0x29),
                    Reg8::D => rom.push(0x2a),
                    Reg8::E => rom.push(0x2b),
                    Reg8::H => rom.push(0x2c),
                    Reg8::L => rom.push(0x2d),
                }
            }
            Instruction::SraMRhl => {
                rom.push(0xCB);
                rom.push(0x2E);
            }
            Instruction::SwapR8(reg) => {
                rom.push(0xCB);
                match reg {
                    Reg8::A => rom.push(0x37),
                    Reg8::B => rom.push(0x30),
                    Reg8::C => rom.push(0x31),
                    Reg8::D => rom.push(0x32),
                    Reg8::E => rom.push(0x33),
                    Reg8::H => rom.push(0x34),
                    Reg8::L => rom.push(0x35),
                }
            }
            Instruction::SwapMRhl => {
                rom.push(0xCB);
                rom.push(0x36);
            }
            Instruction::SrlR8(reg) => {
                rom.push(0xCB);
                match reg {
                    Reg8::A => rom.push(0x3f),
                    Reg8::B => rom.push(0x38),
                    Reg8::C => rom.push(0x39),
                    Reg8::D => rom.push(0x3a),
                    Reg8::E => rom.push(0x3b),
                    Reg8::H => rom.push(0x3c),
                    Reg8::L => rom.push(0x3d),
                }
            }
            Instruction::SrlMRhl => {
                rom.push(0xCB);
                rom.push(0x3E);
            }
            Instruction::BitBitR8(expr, reg) => {
                rom.push(0xCB);
                let byte = 0x40                                    // 0b11000000
                         | (expr.get_bit_index(constants)? * 0x08) // 0b00111000
                         | Instruction::reg8_to_bits(&reg); // 0b00000111
                rom.push(byte);
            }
            Instruction::BitBitMRhl(expr) => {
                rom.push(0xCB);
                rom.push(0x46 + expr.get_bit_index(constants)? * 0x08);
            }
            Instruction::ResBitR8(expr, reg) => {
                rom.push(0xCB);
                let byte = 0x80                                    // 0b11000000
                         | (expr.get_bit_index(constants)? * 0x08) // 0b00111000
                         | Instruction::reg8_to_bits(&reg); // 0b00000111
                rom.push(byte)
            }
            Instruction::ResBitMRhl(expr) => {
                rom.push(0xCB);
                rom.push(0x86 + expr.get_bit_index(constants)? * 0x08);
            }
            Instruction::SetBitR8(expr, reg) => {
                rom.push(0xCB);
                let byte = 0xC0                                    // 0b11000000
                         | (expr.get_bit_index(constants)? * 0x08) // 0b00111000
                         | Instruction::reg8_to_bits(&reg); // 0b00000111
                rom.push(byte)
            }
            Instruction::SetBitMRhl(expr) => {
                rom.push(0xCB);
                rom.push(0xC6 + expr.get_bit_index(constants)? * 0x08);
            }
        }
        Ok(())
    }

    fn reg8_to_bits(reg: &Reg8) -> u8 {
        match reg {
            Reg8::A => 0x07,
            Reg8::B => 0x00,
            Reg8::C => 0x01,
            Reg8::D => 0x02,
            Reg8::E => 0x03,
            Reg8::H => 0x04,
            Reg8::L => 0x05,
        }
    }

    /// Returns how many bytes the instruction takes up
    pub fn len(&self, start_address: u16) -> u16 {
        match self {
            Instruction::AdvanceAddress(advance_address) => advance_address - start_address,
            Instruction::EmptyLine => 0,
            Instruction::Equ(_, _) => 0,
            Instruction::Label(_) => 0,
            Instruction::Db(bytes) => bytes.len() as u16,
            Instruction::DbExpr8(_) => 1,
            Instruction::DbExpr16(_) => 2,
            Instruction::Nop => 1,
            Instruction::Stop => 1,
            Instruction::Halt => 2,
            Instruction::Di => 1,
            Instruction::Ei => 1,
            Instruction::Rrca => 1,
            Instruction::Rra => 1,
            Instruction::Cpl => 1,
            Instruction::Ccf => 1,
            Instruction::Rlca => 1,
            Instruction::Rla => 1,
            Instruction::Daa => 1,
            Instruction::Scf => 1,
            Instruction::Ret(_) => 1,
            Instruction::Reti => 1,
            Instruction::Call(_, _) => 3,
            Instruction::JpI16(_, _) => 3,
            Instruction::JpRhl => 1,
            Instruction::Jr(_, _) => 2,
            Instruction::IncR16(_) => 1,
            Instruction::IncR8(_) => 1,
            Instruction::IncMRhl => 1,
            Instruction::DecR16(_) => 1,
            Instruction::DecR8(_) => 1,
            Instruction::DecMRhl => 1,
            Instruction::AddR8(_) => 1,
            Instruction::AddMRhl => 1,
            Instruction::AddI8(_) => 2,
            Instruction::AddRhlR16(_) => 1,
            Instruction::AddRspI8(_) => 2,
            Instruction::SubR8(_) => 1,
            Instruction::SubMRhl => 1,
            Instruction::SubI8(_) => 2,
            Instruction::AndR8(_) => 1,
            Instruction::AndMRhl => 1,
            Instruction::AndI8(_) => 2,
            Instruction::OrR8(_) => 1,
            Instruction::OrMRhl => 1,
            Instruction::OrI8(_) => 2,
            Instruction::AdcR8(_) => 1,
            Instruction::AdcMRhl => 1,
            Instruction::AdcI8(_) => 2,
            Instruction::SbcR8(_) => 1,
            Instruction::SbcMRhl => 1,
            Instruction::SbcI8(_) => 2,
            Instruction::XorR8(_) => 1,
            Instruction::XorMRhl => 1,
            Instruction::XorI8(_) => 2,
            Instruction::CpR8(_) => 1,
            Instruction::CpMRhl => 1,
            Instruction::CpI8(_) => 2,
            Instruction::LdR16I16(_, _) => 3,
            Instruction::LdMI16Rsp(_) => 3,
            Instruction::LdR8I8(_, _) => 2,
            Instruction::LdR8R8(_, _) => 1,
            Instruction::LdMRbcRa => 1,
            Instruction::LdMRdeRa => 1,
            Instruction::LdRaMRbc => 1,
            Instruction::LdRaMRde => 1,
            Instruction::LdR8MRhl(_) => 1,
            Instruction::LdMRhlR8(_) => 1,
            Instruction::LdMRhlI8(_) => 2,
            Instruction::LdMI16Ra(_) => 3,
            Instruction::LdRaMI16(_) => 3,
            Instruction::LdhRaMI8(_) => 2,
            Instruction::LdhMI8Ra(_) => 2,
            Instruction::LdhRaMRc => 1,
            Instruction::LdhMRcRa => 1,
            Instruction::LdiMRhlRa => 1,
            Instruction::LddMRhlRa => 1,
            Instruction::LdiRaMRhl => 1,
            Instruction::LddRaMRhl => 1,
            Instruction::LdRhlRspI8(_) => 2,
            Instruction::LdRspRhl => 1,
            Instruction::Push(_) => 1,
            Instruction::Pop(_) => 1,
            Instruction::BitBitR8(_, _) => 2,
            Instruction::BitBitMRhl(_) => 2,
            Instruction::ResBitR8(_, _) => 2,
            Instruction::ResBitMRhl(_) => 2,
            Instruction::SetBitR8(_, _) => 2,
            Instruction::SetBitMRhl(_) => 2,
            Instruction::RlcR8(_) => 2,
            Instruction::RlcMRhl => 2,
            Instruction::RrcR8(_) => 2,
            Instruction::RrcMRhl => 2,
            Instruction::RlR8(_) => 2,
            Instruction::RlMRhl => 2,
            Instruction::RrR8(_) => 2,
            Instruction::RrMRhl => 2,
            Instruction::SlaR8(_) => 2,
            Instruction::SlaMRhl => 2,
            Instruction::SraR8(_) => 2,
            Instruction::SraMRhl => 2,
            Instruction::SwapR8(_) => 2,
            Instruction::SwapMRhl => 2,
            Instruction::SrlR8(_) => 2,
            Instruction::SrlMRhl => 2,
        }
    }
}
