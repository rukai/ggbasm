//! Parse asm files into an AST.

use anyhow::{bail, Error};
use byteorder::{LittleEndian, WriteBytesExt};
use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not, tag, tag_no_case, take_while_m_n};
use nom::character::complete::{char, line_ending};
use nom::combinator::{map, opt, peek, value};
use nom::error::VerboseError;
use nom::multi::{many0, separated_list1};
use nom::sequence::{delimited, terminated};
use nom::IResult;

use crate::ast::*;

static IDENT: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890_";
static HEX: &str = "1234567890ABCDEFabcdef";
static DEC: &str = "1234567890";
static WHITESPACE: &str = " \t";

fn is_hex(input: char) -> bool {
    HEX.contains(input)
}

fn is_dec(input: char) -> bool {
    DEC.contains(input)
}

fn parse_u8_hex(i: &str) -> IResult<&str, u8, VerboseError<&str>> {
    let (i, _) = tag("0x")(i)?;
    let (i, value) = take_while_m_n(1, 2, is_hex)(i)?;
    let value = u8::from_str_radix(value, 16).unwrap();
    Ok((i, value))
}

fn parse_u8_dec(i: &str) -> IResult<&str, u8, VerboseError<&str>> {
    let (i, value) = take_while_m_n(1, 3, is_dec)(i)?;
    let value = value.parse().unwrap(); // TODO: Handle 255 < x < 1000
    Ok((i, value))
}

// TODO: Replace with parse_constant in db and dw, advance_address
fn parse_u8(i: &str) -> IResult<&str, u8, VerboseError<&str>> {
    alt((parse_u8_hex, parse_u8_dec))(i)
}

fn parse_u16_hex(i: &str) -> IResult<&str, u16, VerboseError<&str>> {
    let (i, _) = tag("0x")(i)?;
    let (i, value) = take_while_m_n(1, 4, is_hex)(i)?;
    let value = u16::from_str_radix(value, 16).unwrap();
    Ok((i, value))
}

fn parse_u16_dec(i: &str) -> IResult<&str, u16, VerboseError<&str>> {
    let (i, value) = take_while_m_n(1, 5, is_dec)(i)?;
    let value = value.parse().unwrap(); // TODO: Handle 65535 < x < 100000
    Ok((i, value))
}

// TODO: Replace with parse_constant in db and dw, advance_address
fn parse_u16(i: &str) -> IResult<&str, u16, VerboseError<&str>> {
    alt((parse_u16_hex, parse_u16_dec))(i)
}

fn parse_constant_hex(i: &str) -> IResult<&str, i64, VerboseError<&str>> {
    let (i, _) = tag("0x")(i)?;
    let (i, value) = take_while_m_n(1, 16, is_hex)(i)?; // TODO: Make this endless, we should really handle all the num to big to parse errors in one case
    let value = i64::from_str_radix(value, 16).unwrap();
    Ok((i, value))
}

fn parse_constant_dec(i: &str) -> IResult<&str, i64, VerboseError<&str>> {
    let (i, value) = take_while_m_n(1, 20, is_dec)(i)?; // TODO: Make this endless, we should really handle all the num to big to parse errors in one case
    let value = value.parse().unwrap(); // TODO: Handle 65535 < x < 100000
    Ok((i, value))
}

fn parse_constant(i: &str) -> IResult<&str, i64, VerboseError<&str>> {
    alt((parse_constant_hex, parse_constant_dec))(i)
}

fn u16_to_vec(input: u16) -> Vec<u8> {
    let mut result = vec![];
    result.write_u16::<LittleEndian>(input).unwrap();
    result
}

fn primary_expr(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    alt((
        delimited(char('('), parse_expr, char(')')),
        map(parse_constant, Expr::Const),
        map(is_a(IDENT), |ident: &str| Expr::Ident(ident.to_string())),
    ))(i)
}

fn unary_expr_inner(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let (i, op) = value(UnaryOperator::Minus, char('-'))(i)?;
    let (i, expr) = unary_expr(i)?;
    Ok((i, Expr::unary(expr, op)))
}

fn unary_expr(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    alt((unary_expr_inner, primary_expr))(i)
}

fn mult_expr_inner(i: &str) -> IResult<&str, (BinaryOperator, Expr), VerboseError<&str>> {
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, op) = alt((
        value(BinaryOperator::Mul, char('*')),
        value(BinaryOperator::Div, char('/')),
        value(BinaryOperator::Rem, char('%')),
    ))(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, right) = mult_expr(i)?;
    Ok((i, (op, right)))
}

fn mult_expr(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let (i, left) = unary_expr(i)?;
    let left2 = left.clone();
    alt((
        map(mult_expr_inner, move |(op, right)| {
            Expr::binary(left2.clone(), op, right)
        }),
        move |i| Ok((i, left.clone())),
    ))(i)
}

fn add_expr_inner(i: &str) -> IResult<&str, (BinaryOperator, Expr), VerboseError<&str>> {
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, op) = alt((
        value(BinaryOperator::Add, char('+')),
        value(BinaryOperator::Sub, char('-')),
    ))(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, right) = add_expr(i)?;
    Ok((i, (op, right)))
}

fn add_expr(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let (i, left) = mult_expr(i)?;
    let left2 = left.clone();
    alt((
        map(add_expr_inner, move |(op, right)| {
            Expr::binary(left2.clone(), op, right)
        }),
        move |i| Ok((i, left.clone())),
    ))(i)
}

fn bit_and_expr_inner(i: &str) -> IResult<&str, (BinaryOperator, Expr), VerboseError<&str>> {
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, op) = value(BinaryOperator::And, char('&'))(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, right) = bit_and_expr(i)?;
    Ok((i, (op, right)))
}

fn bit_and_expr(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let (i, left) = add_expr(i)?;
    let left2 = left.clone();
    alt((
        map(bit_and_expr_inner, move |(op, right)| {
            Expr::binary(left2.clone(), op, right)
        }),
        move |i| Ok((i, left.clone())),
    ))(i)
}

fn bit_xor_expr_inner(i: &str) -> IResult<&str, (BinaryOperator, Expr), VerboseError<&str>> {
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, op) = value(BinaryOperator::Xor, char('^'))(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, right) = bit_xor_expr(i)?;
    Ok((i, (op, right)))
}

fn bit_xor_expr(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let (i, left) = bit_and_expr(i)?;
    let left2 = left.clone();
    alt((
        map(bit_xor_expr_inner, move |(op, right)| {
            Expr::binary(left2.clone(), op, right)
        }),
        move |i| Ok((i, left.clone())),
    ))(i)
}

fn bit_or_expr_inner(i: &str) -> IResult<&str, (BinaryOperator, Expr), VerboseError<&str>> {
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, op) = value(BinaryOperator::Or, char('|'))(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, right) = bit_or_expr(i)?;
    Ok((i, (op, right)))
}

fn bit_or_expr(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let (i, left) = bit_xor_expr(i)?;
    let left2 = left.clone();
    alt((
        map(bit_or_expr_inner, move |(op, right)| {
            Expr::binary(left2.clone(), op, right)
        }),
        move |i| Ok((i, left.clone())),
    ))(i)
}

fn parse_expr(i: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    bit_or_expr(i)
}

fn parse_reg_u8(i: &str) -> IResult<&str, Reg8, VerboseError<&str>> {
    alt((
        value(Reg8::A, tag_no_case("a")),
        value(Reg8::B, tag_no_case("b")),
        value(Reg8::C, tag_no_case("c")),
        value(Reg8::D, tag_no_case("d")),
        value(Reg8::E, tag_no_case("e")),
        value(Reg8::H, tag_no_case("h")),
        value(Reg8::L, tag_no_case("l")),
    ))(i)
}

fn parse_reg_u16(i: &str) -> IResult<&str, Reg16, VerboseError<&str>> {
    alt((
        value(Reg16::BC, tag_no_case("bc")),
        value(Reg16::DE, tag_no_case("de")),
        value(Reg16::HL, tag_no_case("hl")),
        value(Reg16::SP, tag_no_case("sp")),
    ))(i)
}

fn parse_reg_u16_push(i: &str) -> IResult<&str, Reg16Push, VerboseError<&str>> {
    alt((
        value(Reg16Push::BC, tag_no_case("bc")),
        value(Reg16Push::DE, tag_no_case("de")),
        value(Reg16Push::HL, tag_no_case("hl")),
        value(Reg16Push::AF, tag_no_case("af")),
    ))(i)
}

fn parse_flag(i: &str) -> IResult<&str, Flag, VerboseError<&str>> {
    alt((
        value(Flag::Z, tag_no_case("z")),
        value(Flag::NZ, tag_no_case("nz")),
        value(Flag::C, tag_no_case("c")),
        value(Flag::NC, tag_no_case("nc")),
    ))(i)
}

fn comma_sep(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    // ignore trailing whitespace
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = char(',')(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    Ok((i, ()))
}

fn comment(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    let (i, _) = char(';')(i)?;
    let (i, _) = opt(is_not("\r\n"))(i)?;

    Ok((i, ()))
}

fn end_line(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    // ignore trailing whitespace
    let (i, _) = opt(is_a(WHITESPACE))(i)?;

    // ignore comments
    let (i, _) = opt(comment)(i)?;

    // does the line truely end?
    peek(is_a("\r\n"))(i)?;

    Ok((i, ()))
}

fn reg_a(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    let (i, _) = tag_no_case("a")(i)?;
    let (i, _) = comma_sep(i)?;
    Ok((i, ()))
}

// rgbds seems to use an "a" in add, sub etc. fairly unpredictably.
// So I just made it optional instead of enforcing arbitrary rules.
fn opt_reg_a(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = opt(reg_a)(i)?;
    Ok((i, ()))
}

fn reg_a_u8_inner(i: &str) -> IResult<&str, Reg8, VerboseError<&str>> {
    let (i, _) = tag_no_case("a")(i)?;
    let (i, _) = comma_sep(i)?;
    parse_reg_u8(i)
}

// proper backtracking for optional register a followed by a real u8 register
fn reg_a_u8(i: &str) -> IResult<&str, Reg8, VerboseError<&str>> {
    alt((reg_a_u8_inner, parse_reg_u8))(i)
}

fn deref_hl(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    let (i, _) = char('[')(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = tag_no_case("hl")(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = char(']')(i)?;
    Ok((i, ()))
}

fn parse_string(i: &str) -> IResult<&str, Vec<u8>, VerboseError<&str>> {
    delimited(
        char('"'),
        map(is_not("\r\n\""), |value: &str| value.as_bytes().to_vec()),
        char('"'),
    )(i)
}

fn label(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, label) = is_a(IDENT)(i)?;
    let (i, _) = char(':')(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::Label(label.to_string())))
}

fn equ(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, label) = is_a(IDENT)(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = tag_no_case("EQU")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::Equ(label.to_string(), expr)))
}

fn direct_bytes(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("db")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, value) = separated_list1(
        comma_sep,
        alt((parse_string, map(parse_u8, |value| vec![value]))),
    )(i)?;
    let (i, _) = end_line(i)?;
    Ok((
        i,
        Instruction::Db(value.iter().flatten().cloned().collect()),
    ))
}

fn direct_words(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("dw")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, value) = parse_u16(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::Db(u16_to_vec(value))))
}

fn advance_address(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("advance_address")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, value) = parse_u16(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::AdvanceAddress(value)))
}

fn instruction_ret(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ret")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, flag) = parse_flag(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::Ret(flag)))
}

fn instruction_call_flag(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("call")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, flag) = parse_flag(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::Call(flag, expr)))
}

fn instruction_call_always(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("call")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::Call(Flag::Always, expr)))
}

fn instruction_jprhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("jp")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = tag_no_case("hl")(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::JpRhl))
}

fn instruction_jpi16_always(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("jp")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::JpI16(Flag::Always, expr)))
}

fn instruction_jpi16_flag(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("jp")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, flag) = parse_flag(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::JpI16(flag, expr)))
}

fn instruction_jr_always(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("jr")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::Jr(Flag::Always, expr)))
}

fn instruction_jr_flag(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("jr")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, flag) = parse_flag(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::Jr(flag, expr)))
}

fn instruction_inc(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("inc")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, instruction) = alt((
        map(parse_reg_u16, Instruction::IncR16),
        map(parse_reg_u8, Instruction::IncR8),
        value(Instruction::IncMRhl, deref_hl),
    ))(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, instruction))
}

fn instruction_dec(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("dec")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, instruction) = alt((
        map(parse_reg_u16, Instruction::DecR16),
        map(parse_reg_u8, Instruction::DecR8),
        value(Instruction::DecMRhl, deref_hl),
    ))(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, instruction))
}

fn instruction_addr8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("add")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = reg_a_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::AddR8(reg)))
}

fn instruction_addmrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("add")(i)?;
    let (i, _) = opt_reg_a(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::AddMRhl))
}

fn instruction_addi8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("add")(i)?;
    let (i, _) = opt_reg_a(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::AddI8(expr)))
}

fn instruction_addrhlr16(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("add")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = tag_no_case("hl")(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, reg) = parse_reg_u16(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::AddRhlR16(reg)))
}

fn instruction_addrspi8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("add")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = tag_no_case("sp")(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::AddRspI8(expr)))
}

fn instruction_subr8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("sub")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = reg_a_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::SubR8(reg)))
}

fn instruction_submrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("sub")(i)?;
    let (i, _) = opt_reg_a(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::SubMRhl))
}

fn instruction_subi8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("sub")(i)?;
    let (i, _) = opt_reg_a(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::SubI8(expr)))
}

fn instruction_andr8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("and")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = reg_a_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::AndR8(reg)))
}

fn instruction_andmrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("and")(i)?;
    let (i, _) = opt_reg_a(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::AndMRhl))
}

fn instruction_andi8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("and")(i)?;
    let (i, _) = opt_reg_a(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::AndI8(expr)))
}

fn instruction_orr8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("or")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = reg_a_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::OrR8(reg)))
}

fn instruction_ormrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("or")(i)?;
    let (i, _) = opt_reg_a(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::OrMRhl))
}

fn instruction_ori8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("or")(i)?;
    let (i, _) = opt_reg_a(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::OrI8(expr)))
}

fn instruction_adcr8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("adc")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = reg_a_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::AdcR8(reg)))
}

fn instruction_adcmrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("adc")(i)?;
    let (i, _) = opt_reg_a(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::AdcMRhl))
}

fn instruction_adci8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("adc")(i)?;
    let (i, _) = opt_reg_a(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::AdcI8(expr)))
}

fn instruction_sbcr8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("sbc")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = reg_a_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::SbcR8(reg)))
}

fn instruction_sbcmrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("sbc")(i)?;
    let (i, _) = opt_reg_a(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::SbcMRhl))
}

fn instruction_sbci8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("sbc")(i)?;
    let (i, _) = opt_reg_a(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::SbcI8(expr)))
}

fn instruction_xorr8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("xor")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = reg_a_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::XorR8(reg)))
}

fn instruction_xormrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("xor")(i)?;
    let (i, _) = opt_reg_a(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::XorMRhl))
}

fn instruction_xori8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("xor")(i)?;
    let (i, _) = opt_reg_a(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::XorI8(expr)))
}

fn instruction_cpr8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("cp")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = reg_a_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::CpR8(reg)))
}

fn instruction_cpmrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("cp")(i)?;
    let (i, _) = opt_reg_a(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::CpMRhl))
}

fn instruction_cpi8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("cp")(i)?;
    let (i, _) = opt_reg_a(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::CpI8(expr)))
}

fn instruction_ldr8r8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ld")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg1) = parse_reg_u8(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, reg2) = parse_reg_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::LdR8R8(reg1, reg2)))
}

fn instruction_ldr8i8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ld")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = parse_reg_u8(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::LdR8I8(reg, expr)))
}

fn instruction_ldrsprhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ld")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = tag_no_case("sp")(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, _) = tag_no_case("hl")(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::LdRspRhl))
}

fn instruction_ldmi16rsp(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ld")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = char('[')(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = char(']')(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, _) = tag_no_case("sp")(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::LdMI16Rsp(expr)))
}

fn instruction_ldmr16ra(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ld")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, instruction) = alt((
        value(Instruction::LdMRbcRa, tag_no_case("[bc]")),
        value(Instruction::LdMRdeRa, tag_no_case("[de]")),
    ))(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, _) = tag_no_case("a")(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, instruction))
}

fn instruction_ldramr16(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ld")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = tag_no_case("a")(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, instruction) = alt((
        value(Instruction::LdRaMRbc, tag_no_case("[bc]")),
        value(Instruction::LdRaMRde, tag_no_case("[de]")),
    ))(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, instruction))
}

fn instruction_ldimrhlra(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ldi")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, _) = tag_no_case("a")(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::LdiMRhlRa))
}

fn instruction_lddmrhlra(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ldd")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, _) = tag_no_case("a")(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::LddMRhlRa))
}

fn instruction_ldiramrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ldi")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = tag_no_case("a")(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::LdiRaMRhl))
}

fn instruction_lddramrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ldd")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = tag_no_case("a")(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::LddRaMRhl))
}

fn instruction_ldmrhlr8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ld")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, reg) = parse_reg_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::LdMRhlR8(reg)))
}

fn instruction_ldmrhli8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ld")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::LdMRhlI8(expr)))
}

fn instruction_ldr8mrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ld")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = parse_reg_u8(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::LdR8MRhl(reg)))
}

fn instruction_ldhramrc(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ld")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = tag_no_case("a")(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, _) = char('[')(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = tag_no_case("0xFF00")(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = tag_no_case("+")(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = tag_no_case("c")(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = char(']')(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::LdhRaMRc))
}

fn instruction_ldhmrcra(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ld")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = char('[')(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = tag_no_case("0xFF00")(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = tag_no_case("+")(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = tag_no_case("c")(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = char(']')(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, _) = tag_no_case("a")(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::LdhMRcRa))
}

fn instruction_ldhmi8ra(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ld")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = char('[')(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = tag_no_case("0xFF00")(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = tag_no_case("+")(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = char(']')(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, _) = tag_no_case("a")(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::LdhMI8Ra(expr)))
}

fn instruction_ldhrami8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ld")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = tag_no_case("a")(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, _) = char('[')(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = tag_no_case("0xFF00")(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = tag_no_case("+")(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = char(']')(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::LdhRaMI8(expr)))
}

fn instruction_ldrhlrspi8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ld")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = tag_no_case("hl")(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, _) = tag_no_case("sp")(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = tag_no_case("+")(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::LdRhlRspI8(expr)))
}

fn instruction_ldmi16ra(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ld")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = char('[')(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = char(']')(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, _) = tag_no_case("a")(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::LdMI16Ra(expr)))
}

fn instruction_ldrami16(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ld")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = tag_no_case("a")(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, _) = char('[')(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = opt(is_a(WHITESPACE))(i)?;
    let (i, _) = char(']')(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::LdRaMI16(expr)))
}

fn instruction_ldr16i16(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("ld")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = parse_reg_u16(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::LdR16I16(reg, expr)))
}

fn instruction_push(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("push")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = parse_reg_u16_push(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::Push(reg)))
}

fn instruction_pop(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("pop")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = parse_reg_u16_push(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::Pop(reg)))
}

fn instruction_rlcr8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("rlc")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = parse_reg_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::RlcR8(reg)))
}

fn instruction_rlcmrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("rlc")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::RlcMRhl))
}

fn instruction_rrcr8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("rrc")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = parse_reg_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::RrcR8(reg)))
}

fn instruction_rrcmrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("rrc")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::RrcMRhl))
}

fn instruction_rlr8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("rl")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = parse_reg_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::RlR8(reg)))
}

fn instruction_rlmrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("rl")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::RlMRhl))
}

fn instruction_rrr8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("rr")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = parse_reg_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::RrR8(reg)))
}

fn instruction_rrmrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("rr")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::RrMRhl))
}

fn instruction_slar8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("sla")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = parse_reg_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::SlaR8(reg)))
}

fn instruction_slamrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("sla")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::SlaMRhl))
}

fn instruction_srar8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("sra")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = parse_reg_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::SraR8(reg)))
}

fn instruction_sramrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("sra")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::SraMRhl))
}

fn instruction_swapr8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("swap")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = parse_reg_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::SwapR8(reg)))
}

fn instruction_swapmrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("swap")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::SwapMRhl))
}

fn instruction_srlr8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("srl")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, reg) = parse_reg_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::SrlR8(reg)))
}

fn instruction_srlmrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("srl")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::SrlMRhl))
}

fn instruction_bitbitr8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("bit")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, reg) = parse_reg_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::BitBitR8(expr, reg)))
}

fn instruction_bitbitmrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("bit")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::BitBitMRhl(expr)))
}

fn instruction_resbitr8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("res")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, reg) = parse_reg_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::ResBitR8(expr, reg)))
}

fn instruction_resbitmrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("res")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::ResBitMRhl(expr)))
}

fn instruction_setbitr8(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("set")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, reg) = parse_reg_u8(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::SetBitR8(expr, reg)))
}

fn instruction_setbitmrhl(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    let (i, _) = tag_no_case("set")(i)?;
    let (i, _) = is_a(WHITESPACE)(i)?;
    let (i, expr) = parse_expr(i)?;
    let (i, _) = comma_sep(i)?;
    let (i, _) = deref_hl(i)?;
    let (i, _) = end_line(i)?;
    Ok((i, Instruction::SetBitMRhl(expr)))
}

fn instruction(i: &str) -> IResult<&str, Instruction, VerboseError<&str>> {
    alt((
        label,
        equ,
        direct_bytes,
        direct_words,
        advance_address,
        // instructions
        alt((
            terminated(value(Instruction::Stop, tag_no_case("stop")), end_line),
            terminated(value(Instruction::Nop, tag_no_case("nop")), end_line),
            terminated(value(Instruction::Halt, tag_no_case("halt")), end_line),
            terminated(value(Instruction::Di, tag_no_case("di")), end_line),
            terminated(value(Instruction::Ei, tag_no_case("ei")), end_line),
            terminated(value(Instruction::Reti, tag_no_case("reti")), end_line),
            terminated(value(Instruction::Rrca, tag_no_case("rrca")), end_line),
            terminated(value(Instruction::Rra, tag_no_case("rra")), end_line),
            terminated(value(Instruction::Cpl, tag_no_case("cpl")), end_line),
            terminated(value(Instruction::Ccf, tag_no_case("ccf")), end_line),
            terminated(value(Instruction::Rlca, tag_no_case("rlca")), end_line),
            terminated(value(Instruction::Rla, tag_no_case("rla")), end_line),
            terminated(value(Instruction::Daa, tag_no_case("daa")), end_line),
            terminated(value(Instruction::Scf, tag_no_case("scf")), end_line),
            terminated(
                value(Instruction::Ret(Flag::Always), tag_no_case("ret")),
                end_line,
            ),
        )),
        alt((
            instruction_ret,
            instruction_call_flag,
            instruction_call_always,
            instruction_jprhl,
            instruction_jpi16_always,
            instruction_jpi16_flag,
            instruction_jr_always,
            instruction_jr_flag,
            instruction_inc,
            instruction_dec,
            instruction_addr8,
            instruction_addmrhl,
            instruction_addi8,
            instruction_addrhlr16,
            instruction_addrspi8,
            instruction_subr8,
            instruction_submrhl,
            instruction_subi8,
            instruction_andr8,
            instruction_andmrhl,
            instruction_andi8,
        )),
        alt((
            instruction_orr8,
            instruction_ormrhl,
            instruction_ori8,
            instruction_adcr8,
            instruction_adcmrhl,
            instruction_adci8,
            instruction_sbcr8,
            instruction_sbcmrhl,
            instruction_sbci8,
            instruction_xorr8,
            instruction_xormrhl,
            instruction_xori8,
            instruction_cpr8,
            instruction_cpmrhl,
            instruction_cpi8,
        )),
        alt((
            instruction_ldr8r8,
            instruction_ldr8i8,
            instruction_ldrsprhl,
            instruction_ldmi16rsp,
            instruction_ldmr16ra,
            instruction_ldramr16,
            instruction_ldimrhlra,
            instruction_lddmrhlra,
            instruction_ldiramrhl,
            instruction_lddramrhl,
            instruction_ldmrhlr8,
            instruction_ldmrhli8,
            instruction_ldr8mrhl,
            instruction_ldhramrc,
            instruction_ldhmrcra,
            instruction_ldhmi8ra,
            instruction_ldhrami8,
            instruction_ldrhlrspi8,
            instruction_ldmi16ra,
            instruction_ldrami16,
            instruction_ldr16i16,
        )),
        alt((
            instruction_push,
            instruction_pop,
            instruction_rlcr8,
            instruction_rlcmrhl,
            instruction_rrcr8,
            instruction_rrcmrhl,
            instruction_rlr8,
            instruction_rlmrhl,
            instruction_rrr8,
            instruction_rrmrhl,
            instruction_slar8,
            instruction_slamrhl,
            instruction_srar8,
            instruction_sramrhl,
            instruction_swapr8,
            instruction_swapmrhl,
            instruction_srlr8,
            instruction_srlmrhl,
        )),
        alt((
            instruction_bitbitr8,
            instruction_bitbitmrhl,
            instruction_resbitr8,
            instruction_resbitmrhl,
            instruction_setbitr8,
            instruction_setbitmrhl,
        )),
        // line containing only whitespace/empty
        value(Instruction::EmptyLine, end_line),
    ))(i)
}

fn instruction_option(i: &str) -> IResult<&str, Option<Instruction>, VerboseError<&str>> {
    // ignore preceding whitespace
    let (i, _) = opt(is_a(WHITESPACE))(i)?;

    // if an instruction fails to parse, it becomes a None and we handle the error later
    let (i, instruction) = opt(instruction)(i)?;

    // If the instruction is None, then we need to clean up the unparsed line.
    let (i, _) = opt(is_not("\r\n"))(i)?;
    Ok((i, instruction))
}

fn instructions(i: &str) -> IResult<&str, Vec<Option<Instruction>>, VerboseError<&str>> {
    many0(terminated(instruction_option, line_ending))(i)
}

/// Parses the text in the provided &str into a Vec<Option<Instruction>>
/// Instructions are None when that line fails to parse.
pub fn parse_asm(text: &str) -> Result<Vec<Option<Instruction>>, Error> {
    // Ensure a trailing \n is included TODO: Avoid this copy, should be able to handle this in the parser combinator
    let mut text = String::from(text);
    if text.chars().last().map(|x| x != '\n').unwrap_or(false) {
        text.push('\n');
    }

    match instructions(&text) {
        Ok(instructions) => Ok(instructions.1),
        Err(err) => bail!("{:?}", err), // Convert error to text immediately to avoid lifetime issues
    }
}
