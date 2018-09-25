//! Parse asm files into an AST.

use byteorder::{LittleEndian, WriteBytesExt};
use failure::Error;
use failure::bail;
use nom::*;
use nom::types::CompleteStr;

use crate::ast::*;

static IDENT:      &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890_";
static HEX:        &str = "1234567890ABCDEFabcdef";
static DEC:        &str = "1234567890";
static WHITESPACE: &str = " \t";

fn is_hex(input: char) -> bool {
    HEX.contains(input)
}

fn is_dec(input: char) -> bool {
    DEC.contains(input)
}

// TODO: Replace with parse_constant in db and dw, advance_address
named!(parse_u8<CompleteStr, u8>,
    alt!(
        // hexadecimal
        do_parse!(
            tag!("0x") >>
            value: take_while_m_n!(1, 2, is_hex) >>
            (u8::from_str_radix(value.as_ref(), 16).unwrap())
        ) |
        // decimal
        do_parse!(
            value: take_while_m_n!(1, 3, is_dec) >>
            (u8::from_str_radix(value.as_ref(), 10).unwrap()) // TODO: Handle 255 < x < 1000
        )
    )
);

// TODO: Replace with parse_constant in db and dw, advance_address
named!(parse_u16<CompleteStr, u16>,
    alt!(
        // hexadecimal
        do_parse!(
            tag!("0x") >>
            value: take_while_m_n!(1, 4, is_hex) >>
            (u16::from_str_radix(value.as_ref(), 16).unwrap())
        ) |
        // decimal
        do_parse!(
            value: take_while_m_n!(1, 5, is_dec) >>
            (u16::from_str_radix(value.as_ref(), 10).unwrap()) // TODO: Handle 65535 < x < 100000
        )
    )
);

named!(parse_constant<CompleteStr, i64>,
    alt!(
        // hexadecimal
        do_parse!(
            tag!("0x") >>
            value: take_while_m_n!(1, 16, is_hex) >> // TODO: Make this endless, we should really handle all the num to big to parse errors in one case
            (i64::from_str_radix(value.as_ref(), 16).unwrap())
        ) |
        // decimal
        do_parse!(
            value: take_while_m_n!(1, 20, is_dec) >> // TODO: Make this endless, we should really handle all the num to big to parse errors in one case
            (i64::from_str_radix(value.as_ref(), 10).unwrap())
        )
    )
);

fn u16_to_vec(input: u16) -> Vec<u8> {
    let mut result = vec!();
    result.write_u16::<LittleEndian>(input).unwrap();
    result
}

named!(primary_expr<CompleteStr, Expr>,
    alt!(
        delimited!(char!('('), parse_expr, char!(')')) |
        do_parse!(
            value: parse_constant >>
            (Expr::Const(value))
        ) |
        do_parse!(
            ident: is_a!(IDENT) >>
            (Expr::Ident(ident.to_string()))
        )
    )
);

named!(unary_expr<CompleteStr, Expr>,
    alt!(
        do_parse!(
            op: alt!(
                value!(UnaryOperator::Minus, char!('-'))
            ) >>
            expr: unary_expr >>
            (Expr::unary(expr, op))
        ) |
        primary_expr
    )
);

named!(mult_expr<CompleteStr, Expr>,
    do_parse!(
        left: unary_expr >>
        expr: alt!(
            do_parse!(
                opt!(is_a!(WHITESPACE)) >>
                op: alt!(
                    value!(BinaryOperator::Mul, char!('*')) |
                    value!(BinaryOperator::Div, char!('/')) |
                    value!(BinaryOperator::Rem, char!('%'))
                ) >>
                opt!(is_a!(WHITESPACE)) >>
                right: mult_expr >>
                (Expr::binary(left.clone(), op, right))
            ) |
            value!(left)
        ) >>
        (expr)
    )
);

named!(add_expr<CompleteStr, Expr>,
    do_parse!(
        left: mult_expr >>
        expr: alt!(
            do_parse!(
                opt!(is_a!(WHITESPACE)) >>
                op: alt!(
                    value!(BinaryOperator::Add, char!('+')) |
                    value!(BinaryOperator::Sub, char!('-'))
                ) >>
                opt!(is_a!(WHITESPACE)) >>
                right: add_expr >>
                (Expr::binary(left.clone(), op, right))
            ) |
            value!(left)
        ) >>
        (expr)
    )
);

named!(bit_and_expr<CompleteStr, Expr>,
    do_parse!(
        left: add_expr >>
        expr: alt!(
            do_parse!(
                opt!(is_a!(WHITESPACE)) >>
                char!('&') >>
                opt!(is_a!(WHITESPACE)) >>
                right: bit_and_expr >>
                (Expr::binary(left.clone(), BinaryOperator::And, right))
            ) |
            value!(left)
        ) >>
        (expr)
    )
);

named!(bit_xor_expr<CompleteStr, Expr>,
    do_parse!(
        left: bit_and_expr >>
        expr: alt!(
            do_parse!(
                opt!(is_a!(WHITESPACE)) >>
                char!('^') >>
                opt!(is_a!(WHITESPACE)) >>
                right: bit_xor_expr >>
                (Expr::binary(left.clone(), BinaryOperator::Xor, right))
            ) |
            value!(left)
        ) >>
        (expr)
    )
);

named!(bit_or_expr<CompleteStr, Expr>,
    do_parse!(
        left: bit_xor_expr >>
        expr: alt!(
            do_parse!(
                opt!(is_a!(WHITESPACE)) >>
                char!('|') >>
                opt!(is_a!(WHITESPACE)) >>
                right: bit_or_expr >>
                (Expr::binary(left.clone(), BinaryOperator::Or, right))
            ) |
            value!(left)
        ) >>
        (expr)
    )
);

named!(parse_expr<CompleteStr, Expr>,
    do_parse!(expr: bit_or_expr >> (expr))
);


named!(parse_reg_u8<CompleteStr, Reg8>,
    alt!(
        value!(Reg8::A, tag_no_case!("a")) |
        value!(Reg8::B, tag_no_case!("b")) |
        value!(Reg8::C, tag_no_case!("c")) |
        value!(Reg8::D, tag_no_case!("d")) |
        value!(Reg8::E, tag_no_case!("e")) |
        value!(Reg8::H, tag_no_case!("h")) |
        value!(Reg8::L, tag_no_case!("l"))
    )
);

named!(parse_reg_u16<CompleteStr, Reg16>,
    alt!(
        value!(Reg16::BC, tag_no_case!("bc")) |
        value!(Reg16::DE, tag_no_case!("de")) |
        value!(Reg16::HL, tag_no_case!("hl")) |
        value!(Reg16::SP, tag_no_case!("sp"))
    )
);

named!(parse_reg_u16_push<CompleteStr, Reg16Push>,
    alt!(
        value!(Reg16Push::BC, tag_no_case!("bc")) |
        value!(Reg16Push::DE, tag_no_case!("de")) |
        value!(Reg16Push::HL, tag_no_case!("hl")) |
        value!(Reg16Push::AF, tag_no_case!("af"))
    )
);

named!(parse_flag<CompleteStr, Flag>,
    alt!(
        value!(Flag::Z,  tag_no_case!("z")) |
        value!(Flag::NZ, tag_no_case!("nz")) |
        value!(Flag::C,  tag_no_case!("c")) |
        value!(Flag::NC, tag_no_case!("nc"))
    )
);

named!(comma_sep<CompleteStr, ()>,
    do_parse!(
        // ignore trailing whitespace
        opt!(is_a!(WHITESPACE)) >>
        char!(',') >>
        opt!(is_a!(WHITESPACE)) >>
        (())
    )
);

named!(end_line<CompleteStr, ()>,
    do_parse!(
        // ignore trailing whitespace
        opt!(is_a!(WHITESPACE)) >>

        // ignore comments
        opt!(do_parse!(
            char!(';') >>
            opt!(is_not!("\r\n")) >>
            (())
        )) >>

        // does the line truely end?
        peek!(is_a!("\r\n")) >>
        (())
    )
);

// rgbds seems to use an "a" in add, sub etc. fairly unpredictably.
// So I just made it optional instead of enforcing arbitrary rules.
named!(opt_reg_a<CompleteStr, ()>,
    do_parse!(
        is_a!(WHITESPACE) >>
        opt!(do_parse!(
            tag_no_case!("a") >>
            comma_sep >>
            (())
        )) >>
        (())
    )
);

// proper backtracking for optional register a followed by a real u8 register
named!(reg_a_u8<CompleteStr, Reg8>,
    alt!(
        do_parse!(
            tag_no_case!("a") >>
            comma_sep >>
            reg: parse_reg_u8 >>
            (reg)
        ) |
        parse_reg_u8
    )
);

named!(deref_hl<CompleteStr, CompleteStr>,
    do_parse!(
        char!('[') >>
        opt!(is_a!(WHITESPACE)) >>
        a : tag_no_case!("hl") >>
        opt!(is_a!(WHITESPACE)) >>
        char!(']') >>
        (a)
    )
);

named!(parse_string<CompleteStr, Vec<u8> >,
    delimited!(
        char!('"'),
        do_parse!(
            value: is_not!("\r\n\"") >>
            (value.as_bytes().to_vec())
        ),
        char!('"')
    )
);

named!(instruction<CompleteStr, Instruction>,
    alt!(
        // label
        do_parse!(
            label: is_a!(IDENT) >>
            char!(':') >>
            end_line >>
            (Instruction::Label (label.to_string()))
        ) |

        // equ
        do_parse!(
            label: is_a!(IDENT) >>
            is_a!(WHITESPACE) >>
            tag_no_case!("EQU") >>
            is_a!(WHITESPACE) >>
            expr: parse_expr >>
            end_line >>
            (Instruction::Equ (label.to_string(), expr))
        ) |

        // direct bytes
        do_parse!(
            tag_no_case!("db") >>
            is_a!(WHITESPACE) >>
            value: separated_nonempty_list!(
                comma_sep,
                alt!(
                    parse_string |
                    do_parse!(
                        value: parse_u8 >>
                        (vec!(value))
                    )
                )
            ) >>
            end_line >>
            (Instruction::Db (value.iter().flatten().cloned().collect()))
        ) |

        // direct words
        do_parse!(
            tag_no_case!("dw") >>
            is_a!(WHITESPACE) >>
            value: parse_u16 >>
            end_line >>
            (Instruction::Db (u16_to_vec(value)))
        ) |

        // advance address
        do_parse!(
            tag_no_case!("advance_address") >>
            is_a!(WHITESPACE) >>
            value: parse_u16 >>
            end_line >>
            (Instruction::AdvanceAddress (value))
        ) |

        // instructions
        terminated!(value!(Instruction::Stop, tag_no_case!("stop")), end_line) |
        terminated!(value!(Instruction::Nop,  tag_no_case!("nop")),  end_line) |
        terminated!(value!(Instruction::Halt, tag_no_case!("halt")), end_line) |
        terminated!(value!(Instruction::Di,   tag_no_case!("di")),   end_line) |
        terminated!(value!(Instruction::Ei,   tag_no_case!("ei")),   end_line) |
        terminated!(value!(Instruction::Reti, tag_no_case!("reti")), end_line) |
        terminated!(value!(Instruction::Rrca, tag_no_case!("rrca")), end_line) |
        terminated!(value!(Instruction::Rra,  tag_no_case!("rra")), end_line) |
        terminated!(value!(Instruction::Cpl,  tag_no_case!("cpl")), end_line) |
        terminated!(value!(Instruction::Ccf,  tag_no_case!("ccf")), end_line) |
        terminated!(value!(Instruction::Rlca, tag_no_case!("rlca")), end_line) |
        terminated!(value!(Instruction::Rla,  tag_no_case!("rla")), end_line) |
        terminated!(value!(Instruction::Daa,  tag_no_case!("daa")), end_line) |
        terminated!(value!(Instruction::Scf,  tag_no_case!("scf")), end_line) |
        terminated!(value!(Instruction::Ret (Flag::Always),  tag_no_case!("ret")),  end_line) |
        do_parse!(
            tag_no_case!("ret") >>
            is_a!(WHITESPACE) >>
            flag: parse_flag >>
            end_line >>
            (Instruction::Ret (flag))
        ) |
        do_parse!(
            tag_no_case!("call") >>
            is_a!(WHITESPACE) >>
            flag: parse_flag >>
            comma_sep >>
            expr: parse_expr >>
            end_line >>
            (Instruction::Call (flag, expr))
        ) |
        do_parse!(
            tag_no_case!("call") >>
            is_a!(WHITESPACE) >>
            expr: parse_expr >>
            end_line >>
            (Instruction::Call (Flag::Always, expr))
        ) |
        do_parse!(
            tag_no_case!("jp") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("hl") >>
            end_line >>
            (Instruction::JpRhl)
        ) |
        do_parse!(
            tag_no_case!("jp") >>
            is_a!(WHITESPACE) >>
            expr: parse_expr >>
            end_line >>
            (Instruction::JpI16 (Flag::Always, expr))
        ) |
        do_parse!(
            tag_no_case!("jp") >>
            is_a!(WHITESPACE) >>
            flag: parse_flag >>
            comma_sep >>
            expr: parse_expr >>
            end_line >>
            (Instruction::JpI16 (flag, expr))
        ) |
        do_parse!(
            tag_no_case!("jr") >>
            is_a!(WHITESPACE) >>
            expr: parse_expr >>
            end_line >>
            (Instruction::Jr (Flag::Always, expr))
        ) |
        do_parse!(
            tag_no_case!("jr") >>
            is_a!(WHITESPACE) >>
            flag: parse_flag >>
            comma_sep >>
            expr: parse_expr >>
            end_line >>
            (Instruction::Jr (flag, expr))
        ) |
        do_parse!(
            tag_no_case!("inc") >>
            is_a!(WHITESPACE) >>
            instruction: alt!(
                do_parse!(reg: parse_reg_u16 >> (Instruction::IncR16 (reg))) |
                do_parse!(reg: parse_reg_u8  >> (Instruction::IncR8  (reg))) |
                value!(Instruction::IncMRhl, deref_hl)
            ) >>
            end_line >>
            (instruction)
        ) |
        do_parse!(
            tag_no_case!("dec") >>
            is_a!(WHITESPACE) >>
            instruction: alt!(
                do_parse!(reg: parse_reg_u16 >> (Instruction::DecR16 (reg))) |
                do_parse!(reg: parse_reg_u8  >> (Instruction::DecR8  (reg))) |
                value!(Instruction::DecMRhl, deref_hl)
            ) >>
            end_line >>
            (instruction)
        ) |
        do_parse!(
            tag_no_case!("add") >>
            is_a!(WHITESPACE) >>
            reg: reg_a_u8 >>
            end_line >>
            (Instruction::AddR8 (reg))
        ) |
        do_parse!(
            tag_no_case!("add") >>
            opt_reg_a >>
            deref_hl >>
            end_line >>
            (Instruction::AddMRhl)
        ) |
        do_parse!(
            tag_no_case!("add") >>
            opt_reg_a >>
            expr: parse_expr >>
            end_line >>
            (Instruction::AddI8 (expr))
        ) |
        do_parse!(
            tag_no_case!("add") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("hl") >>
            comma_sep >>
            reg: parse_reg_u16 >>
            end_line >>
            (Instruction::AddRhlR16 (reg))
        ) |
        do_parse!(
            tag_no_case!("add") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("sp") >>
            comma_sep >>
            expr: parse_expr >>
            end_line >>
            (Instruction::AddRspI8 (expr))
        ) |
        do_parse!(
            tag_no_case!("sub") >>
            is_a!(WHITESPACE) >>
            reg: reg_a_u8 >>
            end_line >>
            (Instruction::SubR8 (reg))
        ) |
        do_parse!(
            tag_no_case!("sub") >>
            opt_reg_a >>
            deref_hl >>
            end_line >>
            (Instruction::SubMRhl)
        ) |
        do_parse!(
            tag_no_case!("sub") >>
            opt_reg_a >>
            expr: parse_expr >>
            end_line >>
            (Instruction::SubI8 (expr))
        ) |
        do_parse!(
            tag_no_case!("and") >>
            is_a!(WHITESPACE) >>
            reg: reg_a_u8 >>
            end_line >>
            (Instruction::AndR8 (reg))
        ) |
        do_parse!(
            tag_no_case!("and") >>
            opt_reg_a >>
            deref_hl >>
            end_line >>
            (Instruction::AndMRhl)
        ) |
        do_parse!(
            tag_no_case!("and") >>
            opt_reg_a >>
            expr: parse_expr >>
            end_line >>
            (Instruction::AndI8 (expr))
        ) |
        do_parse!(
            tag_no_case!("or") >>
            is_a!(WHITESPACE) >>
            reg: reg_a_u8 >>
            end_line >>
            (Instruction::OrR8 (reg))
        ) |
        do_parse!(
            tag_no_case!("or") >>
            opt_reg_a >>
            deref_hl >>
            end_line >>
            (Instruction::OrMRhl)
        ) |
        do_parse!(
            tag_no_case!("or") >>
            opt_reg_a >>
            expr: parse_expr >>
            end_line >>
            (Instruction::OrI8 (expr))
        ) |
        do_parse!(
            tag_no_case!("adc") >>
            is_a!(WHITESPACE) >>
            reg: reg_a_u8 >>
            end_line >>
            (Instruction::AdcR8 (reg))
        ) |
        do_parse!(
            tag_no_case!("adc") >>
            opt_reg_a >>
            deref_hl >>
            end_line >>
            (Instruction::AdcMRhl)
        ) |
        do_parse!(
            tag_no_case!("adc") >>
            opt_reg_a >>
            expr: parse_expr >>
            end_line >>
            (Instruction::AdcI8 (expr))
        ) |
        do_parse!(
            tag_no_case!("sbc") >>
            is_a!(WHITESPACE) >>
            reg: reg_a_u8 >>
            end_line >>
            (Instruction::SbcR8 (reg))
        ) |
        do_parse!(
            tag_no_case!("sbc") >>
            opt_reg_a >>
            deref_hl >>
            end_line >>
            (Instruction::SbcMRhl)
        ) |
        do_parse!(
            tag_no_case!("sbc") >>
            opt_reg_a >>
            expr: parse_expr >>
            end_line >>
            (Instruction::SbcI8 (expr))
        ) |
        do_parse!(
            tag_no_case!("xor") >>
            is_a!(WHITESPACE) >>
            reg: reg_a_u8 >>
            end_line >>
            (Instruction::XorR8 (reg))
        ) |
        do_parse!(
            tag_no_case!("xor") >>
            opt_reg_a >>
            deref_hl >>
            end_line >>
            (Instruction::XorMRhl)
        ) |
        do_parse!(
            tag_no_case!("xor") >>
            opt_reg_a >>
            expr: parse_expr >>
            end_line >>
            (Instruction::XorI8 (expr))
        ) |
        do_parse!(
            tag_no_case!("cp") >>
            is_a!(WHITESPACE) >>
            reg: reg_a_u8 >>
            end_line >>
            (Instruction::CpR8 (reg))
        ) |
        do_parse!(
            tag_no_case!("cp") >>
            opt_reg_a >>
            deref_hl >>
            end_line >>
            (Instruction::CpMRhl)
        ) |
        do_parse!(
            tag_no_case!("cp") >>
            opt_reg_a >>
            expr: parse_expr >>
            end_line >>
            (Instruction::CpI8 (expr))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            reg1: parse_reg_u8 >>
            comma_sep >>
            reg2: parse_reg_u8 >>
            end_line >>
            (Instruction::LdR8R8 (reg1, reg2))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            reg1: parse_reg_u8 >>
            comma_sep >>
            reg2: parse_expr >>
            end_line >>
            (Instruction::LdR8I8 (reg1, reg2))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("sp") >>
            comma_sep >>
            tag_no_case!("hl") >>
            end_line >>
            (Instruction::LdRspRhl)
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            char!('[') >>
            opt!(is_a!(WHITESPACE)) >>
            expr: parse_expr >>
            opt!(is_a!(WHITESPACE)) >>
            char!(']') >>
            comma_sep >>
            tag_no_case!("sp") >>
            end_line >>
            (Instruction::LdMI16Rsp (expr))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            instruction: alt!(
                value!(Instruction::LdMRbcRa, tag_no_case!("[bc]")) |
                value!(Instruction::LdMRdeRa, tag_no_case!("[de]"))
            ) >>
            comma_sep >>
            tag_no_case!("a") >>
            end_line >>
            (instruction)
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("a") >>
            comma_sep >>
            instruction: alt!(
                value!(Instruction::LdRaMRbc, tag_no_case!("[bc]")) |
                value!(Instruction::LdRaMRde, tag_no_case!("[de]"))
            ) >>
            end_line >>
            (instruction)
        ) |
        do_parse!(
            tag_no_case!("ldi") >>
            is_a!(WHITESPACE) >>
            deref_hl >>
            comma_sep >>
            tag_no_case!("a") >>
            end_line >>
            (Instruction::LdiMRhlRa)
        ) |
        do_parse!(
            tag_no_case!("ldd") >>
            is_a!(WHITESPACE) >>
            deref_hl >>
            comma_sep >>
            tag_no_case!("a") >>
            end_line >>
            (Instruction::LddMRhlRa)
        ) |
        do_parse!(
            tag_no_case!("ldi") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("a") >>
            comma_sep >>
            deref_hl >>
            end_line >>
            (Instruction::LdiRaMRhl)
        ) |
        do_parse!(
            tag_no_case!("ldd") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("a") >>
            comma_sep >>
            deref_hl >>
            end_line >>
            (Instruction::LddRaMRhl)
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            deref_hl >>
            comma_sep >>
            reg: parse_reg_u8 >>
            end_line >>
            (Instruction::LdMRhlR8 (reg))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            deref_hl >>
            comma_sep >>
            expr: parse_expr >>
            end_line >>
            (Instruction::LdMRhlI8 (expr))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            reg: parse_reg_u8 >>
            comma_sep >>
            deref_hl >>
            end_line >>
            (Instruction::LdR8MRhl (reg))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("a") >>
            comma_sep >>
            char!('[') >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("0xFF00") >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("+") >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("c") >>
            opt!(is_a!(WHITESPACE)) >>
            char!(']') >>
            end_line >>
            (Instruction::LdhRaMRc)
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            char!('[') >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("0xFF00") >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("+") >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("c") >>
            opt!(is_a!(WHITESPACE)) >>
            char!(']') >>
            comma_sep >>
            tag_no_case!("a") >>
            end_line >>
            (Instruction::LdhMRcRa)
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            char!('[') >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("0xFF00") >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("+") >>
            opt!(is_a!(WHITESPACE)) >>
            expr: parse_expr >>
            opt!(is_a!(WHITESPACE)) >>
            char!(']') >>
            comma_sep >>
            tag_no_case!("a") >>
            end_line >>
            (Instruction::LdhMI8Ra (expr))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("a") >>
            comma_sep >>
            char!('[') >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("0xFF00") >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("+") >>
            opt!(is_a!(WHITESPACE)) >>
            expr: parse_expr >>
            opt!(is_a!(WHITESPACE)) >>
            char!(']') >>
            end_line >>
            (Instruction::LdhRaMI8 (expr))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("hl") >>
            comma_sep >>
            tag_no_case!("sp") >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("+") >>
            opt!(is_a!(WHITESPACE)) >>
            expr: parse_expr >>
            end_line >>
            (Instruction::LdRhlRspI8 (expr))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            char!('[') >>
            opt!(is_a!(WHITESPACE)) >>
            expr: parse_expr >>
            opt!(is_a!(WHITESPACE)) >>
            char!(']') >>
            comma_sep >>
            tag_no_case!("a") >>
            end_line >>
            (Instruction::LdMI16Ra (expr))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("a") >>
            comma_sep >>
            char!('[') >>
            opt!(is_a!(WHITESPACE)) >>
            expr: parse_expr >>
            opt!(is_a!(WHITESPACE)) >>
            char!(']') >>
            end_line >>
            (Instruction::LdRaMI16 (expr))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            reg: parse_reg_u16 >>
            comma_sep >>
            expr: parse_expr >>
            end_line >>
            (Instruction::LdR16I16 (reg, expr))
        ) |
        do_parse!(
            tag_no_case!("push") >>
            is_a!(WHITESPACE) >>
            reg: parse_reg_u16_push >>
            end_line >>
            (Instruction::Push (reg))
        ) |
        do_parse!(
            tag_no_case!("pop") >>
            is_a!(WHITESPACE) >>
            reg: parse_reg_u16_push >>
            end_line >>
            (Instruction::Pop (reg))
        ) |
        do_parse!(
            tag_no_case!("rlc") >>
            is_a!(WHITESPACE) >>
            reg: parse_reg_u8 >>
            end_line >>
            (Instruction::RlcR8 (reg))
        ) |
        do_parse!(
            tag_no_case!("rlc") >>
            is_a!(WHITESPACE) >>
            deref_hl >>
            end_line >>
            (Instruction::RlcMRhl)
        ) |
        do_parse!(
            tag_no_case!("rrc") >>
            is_a!(WHITESPACE) >>
            reg: parse_reg_u8 >>
            end_line >>
            (Instruction::RrcR8 (reg))
        ) |
        do_parse!(
            tag_no_case!("rrc") >>
            is_a!(WHITESPACE) >>
            deref_hl >>
            end_line >>
            (Instruction::RrcMRhl)
        ) |
        do_parse!(
            tag_no_case!("rl") >>
            is_a!(WHITESPACE) >>
            reg: parse_reg_u8 >>
            end_line >>
            (Instruction::RlR8 (reg))
        ) |
        do_parse!(
            tag_no_case!("rl") >>
            is_a!(WHITESPACE) >>
            deref_hl >>
            end_line >>
            (Instruction::RlMRhl)
        ) |
        do_parse!(
            tag_no_case!("rr") >>
            is_a!(WHITESPACE) >>
            reg: parse_reg_u8 >>
            end_line >>
            (Instruction::RrR8 (reg))
        ) |
        do_parse!(
            tag_no_case!("rr") >>
            is_a!(WHITESPACE) >>
            deref_hl >>
            end_line >>
            (Instruction::RrMRhl)
        ) |
        do_parse!(
            tag_no_case!("sla") >>
            is_a!(WHITESPACE) >>
            reg: parse_reg_u8 >>
            end_line >>
            (Instruction::SlaR8 (reg))
        ) |
        do_parse!(
            tag_no_case!("sla") >>
            is_a!(WHITESPACE) >>
            deref_hl >>
            end_line >>
            (Instruction::SlaMRhl)
        ) |
        do_parse!(
            tag_no_case!("sra") >>
            is_a!(WHITESPACE) >>
            reg: parse_reg_u8 >>
            end_line >>
            (Instruction::SraR8 (reg))
        ) |
        do_parse!(
            tag_no_case!("sra") >>
            is_a!(WHITESPACE) >>
            deref_hl >>
            end_line >>
            (Instruction::SraMRhl)
        ) |
        do_parse!(
            tag_no_case!("swap") >>
            is_a!(WHITESPACE) >>
            reg: parse_reg_u8 >>
            end_line >>
            (Instruction::SwapR8 (reg))
        ) |
        do_parse!(
            tag_no_case!("swap") >>
            is_a!(WHITESPACE) >>
            deref_hl >>
            end_line >>
            (Instruction::SwapMRhl)
        ) |
        do_parse!(
            tag_no_case!("srl") >>
            is_a!(WHITESPACE) >>
            reg: parse_reg_u8 >>
            end_line >>
            (Instruction::SrlR8 (reg))
        ) |
        do_parse!(
            tag_no_case!("srl") >>
            is_a!(WHITESPACE) >>
            deref_hl >>
            end_line >>
            (Instruction::SrlMRhl)
        ) |
        do_parse!(
            tag_no_case!("bit") >>
            is_a!(WHITESPACE) >>
            expr: parse_expr >>
            comma_sep >>
            reg: parse_reg_u8 >>
            end_line >>
            (Instruction::BitBitR8 (expr, reg))
        ) |
        do_parse!(
            tag_no_case!("bit") >>
            is_a!(WHITESPACE) >>
            expr: parse_expr >>
            comma_sep >>
            deref_hl >>
            end_line >>
            (Instruction::BitBitMRhl (expr))
        ) |
        do_parse!(
            tag_no_case!("res") >>
            is_a!(WHITESPACE) >>
            expr: parse_expr >>
            comma_sep >>
            reg: parse_reg_u8 >>
            end_line >>
            (Instruction::ResBitR8 (expr, reg))
        ) |
        do_parse!(
            tag_no_case!("res") >>
            is_a!(WHITESPACE) >>
            expr: parse_expr >>
            comma_sep >>
            deref_hl >>
            end_line >>
            (Instruction::ResBitMRhl (expr))
        ) |
        do_parse!(
            tag_no_case!("set") >>
            is_a!(WHITESPACE) >>
            expr: parse_expr >>
            comma_sep >>
            reg: parse_reg_u8 >>
            end_line >>
            (Instruction::SetBitR8 (expr, reg))
        ) |
        do_parse!(
            tag_no_case!("set") >>
            is_a!(WHITESPACE) >>
            expr: parse_expr >>
            comma_sep >>
            deref_hl >>
            end_line >>
            (Instruction::SetBitMRhl (expr))
        ) |

        // line containing only whitespace/empty
        value!(Instruction::EmptyLine, end_line)
    )
);

named!(instructions<CompleteStr, Vec<Option<Instruction>> >,
    many0!(
        terminated!(
            do_parse!(
                // ignore preceding whitespace
                opt!(is_a!(WHITESPACE)) >>

                // if an instruction fails to parse, it becomes a None and we handle the error later
                instruction: opt!(instruction) >>

                // If the instruction is None, then we need to clean up the unparsed line.
                opt!(is_not!("\r\n")) >>
                (instruction)
            ),
            line_ending
        )
    )
);

/// Parses the text in the provided &str into a Vec<Option<Instruction>>
/// Instructions are None when that line fails to parse.
pub fn parse_asm(text: &str) -> Result<Vec<Option<Instruction>>, Error> {
    // Ensure a trailing \n is included TODO: Avoid this copy, should be able to handle this in the parser combinator
    let mut text = String::from(text);
    if text.chars().last().map(|x| x != '\n').unwrap_or(false) {
        text.push('\n');
    }

    // The CompleteStr disables nom's streaming features, this stops the combinators from returning Incomplete
    match instructions(CompleteStr(&text)) {
        Ok(instructions) => Ok(instructions.1),
        Err(err)         => bail!("{}", err), // Convert error to text immediately to avoid lifetime issues
    }
}
