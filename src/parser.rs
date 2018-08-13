use byteorder::{LittleEndian, WriteBytesExt};
use failure::Error;
use failure::bail;
use nom::*;
use nom::types::CompleteStr;

use crate::instruction::*;

static IDENT: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890_";
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

named!(parse_binary_operator<CompleteStr, BinaryOperator>,
    alt!(
        value!(BinaryOperator::Add, tag_no_case!("+")) |
        value!(BinaryOperator::Sub, tag_no_case!("-")) |
        value!(BinaryOperator::Mul, tag_no_case!("*")) |
        value!(BinaryOperator::Div, tag_no_case!("/")) |
        value!(BinaryOperator::Rem, tag_no_case!("%"))
    )
);

fn u16_to_vec(input: u16) -> Vec<u8> {
    let mut result = vec!();
    result.write_u16::<LittleEndian>(input).unwrap();
    result
}

// Pulled some of parse_expr into this parser to avoid infinite recursion on the left binary expr
named!(parse_expr_no_recurse<CompleteStr, Expr>,
    alt!(
        do_parse!(
            tag_no_case!("(") >>
            opt!(is_a!(WHITESPACE)) >>
            left: parse_expr_no_recurse >>
            opt!(is_a!(WHITESPACE)) >>
            operator: parse_binary_operator >>
            opt!(is_a!(WHITESPACE)) >>
            right: parse_expr >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!(")") >>
            (Expr::binary(left, operator, right))
        ) |
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

named!(parse_expr<CompleteStr, Expr>,
    alt!(
        do_parse!(
            left: parse_expr_no_recurse >>
            opt!(is_a!(WHITESPACE)) >>
            operator: parse_binary_operator >>
            opt!(is_a!(WHITESPACE)) >>
            right: parse_expr >>
            (Expr::binary(left, operator, right))
        ) |
        parse_expr_no_recurse
    )
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

named!(deref_hl<CompleteStr, CompleteStr>,
    do_parse!(
        tag_no_case!("[") >>
        opt!(is_a!(WHITESPACE)) >>
        a : tag_no_case!("hl") >>
        opt!(is_a!(WHITESPACE)) >>
        tag_no_case!("]") >>
        (a)
    )
);

named!(parse_string<CompleteStr, Vec<u8> >,
    delimited!(
        tag!("\""),
        do_parse!(
            value: is_not!("\r\n\"") >>
            (value.as_bytes().to_vec())
        ),
        tag!("\"")
    )
);

named!(instruction<CompleteStr, Instruction>,
    alt!(
        // label
        do_parse!(
            label: is_a!(IDENT) >>
            tag!(":") >>
            end_line >>
            (Instruction::Label (label.to_string()))
        ) |

        // direct bytes
        do_parse!(
            tag_no_case!("db") >>
            is_a!(WHITESPACE) >>
            value: separated_nonempty_list!(
                is_a!(WHITESPACE),
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
            is_a!(WHITESPACE) >>
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
            is_a!(WHITESPACE) >>
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
            is_a!(WHITESPACE) >>
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
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            reg1: parse_reg_u8 >>
            is_a!(WHITESPACE) >>
            reg2: parse_reg_u8 >>
            end_line >>
            (Instruction::LdR8R8 (reg1, reg2))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            reg1: parse_reg_u8 >>
            is_a!(WHITESPACE) >>
            reg2: parse_expr >>
            end_line >>
            (Instruction::LdR8I8 (reg1, reg2))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("sp") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("hl") >>
            end_line >>
            (Instruction::LdRspRhl)
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("[") >>
            opt!(is_a!(WHITESPACE)) >>
            expr: parse_expr >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("]") >>
            is_a!(WHITESPACE) >>
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
            is_a!(WHITESPACE) >>
            tag_no_case!("a") >>
            end_line >>
            (instruction)
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("a") >>
            is_a!(WHITESPACE) >>
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
            is_a!(WHITESPACE) >>
            tag_no_case!("a") >>
            end_line >>
            (Instruction::LdiMRhlRa)
        ) |
        do_parse!(
            tag_no_case!("ldd") >>
            is_a!(WHITESPACE) >>
            deref_hl >>
            is_a!(WHITESPACE) >>
            tag_no_case!("a") >>
            end_line >>
            (Instruction::LddMRhlRa)
        ) |
        do_parse!(
            tag_no_case!("ldi") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("a") >>
            is_a!(WHITESPACE) >>
            deref_hl >>
            end_line >>
            (Instruction::LdiRaMRhl)
        ) |
        do_parse!(
            tag_no_case!("ldd") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("a") >>
            is_a!(WHITESPACE) >>
            deref_hl >>
            end_line >>
            (Instruction::LddRaMRhl)
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            deref_hl >>
            is_a!(WHITESPACE) >>
            reg: parse_reg_u8 >>
            end_line >>
            (Instruction::LdMRhlR8 (reg))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            deref_hl >>
            is_a!(WHITESPACE) >>
            expr: parse_expr >>
            end_line >>
            (Instruction::LdMRhlI8 (expr))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            reg: parse_reg_u8 >>
            is_a!(WHITESPACE) >>
            deref_hl >>
            end_line >>
            (Instruction::LdR8MRhl (reg))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("a") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("[") >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("0xFF00") >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("+") >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("c") >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("]") >>
            end_line >>
            (Instruction::LdhRaMRc)
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("[") >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("0xFF00") >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("+") >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("c") >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("]") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("a") >>
            end_line >>
            (Instruction::LdhMRcRa)
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("[") >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("0xFF00") >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("+") >>
            opt!(is_a!(WHITESPACE)) >>
            expr: parse_expr >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("]") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("a") >>
            end_line >>
            (Instruction::LdhMI8Ra (expr))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("a") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("[") >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("0xFF00") >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("+") >>
            opt!(is_a!(WHITESPACE)) >>
            expr: parse_expr >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("]") >>
            end_line >>
            (Instruction::LdhRaMI8 (expr))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("hl") >>
            is_a!(WHITESPACE) >>
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
            tag_no_case!("[") >>
            opt!(is_a!(WHITESPACE)) >>
            expr: parse_expr >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("]") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("a") >>
            end_line >>
            (Instruction::LdMI16Ra (expr))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("a") >>
            is_a!(WHITESPACE) >>
            tag_no_case!("[") >>
            opt!(is_a!(WHITESPACE)) >>
            expr: parse_expr >>
            opt!(is_a!(WHITESPACE)) >>
            tag_no_case!("]") >>
            end_line >>
            (Instruction::LdRaMI16 (expr))
        ) |
        do_parse!(
            tag_no_case!("ld") >>
            is_a!(WHITESPACE) >>
            reg: parse_reg_u16 >>
            is_a!(WHITESPACE) >>
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

        // line containing only whitespace/empty
        value!(Instruction::EmptyLine, end_line)
    )
);

named!(end_line<CompleteStr, ()>,
    do_parse!(
        // ignore trailing whitespace
        opt!(is_a!(WHITESPACE)) >>

        // ignore comments
        opt!(do_parse!(
            is_a!(";") >>
            opt!(is_not!("\r\n")) >>
            (())
        )) >>

        // does the line truely end?
        peek!(is_a!("\r\n")) >>
        (())
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

pub fn parse_asm(text: &str) -> Result<Vec<Option<Instruction>>, Error> {
    // Ensure a trailing \n is included TODO: Avoid this copy, should be able to handle this in the parser combinator
    let mut text = String::from(text);
    if text.chars().last().map(|x| x != '\n').unwrap_or(false) {
        text.push('\n');
    }

    // The completeByteSlice disables nom's streaming features, this stops the combinators from returning Incomplete
    match instructions(CompleteStr(&text)) {
        Ok(instructions) => Ok(instructions.1),
        Err(err)         => bail!("{}", err), // Convert error to text immediately to avoid lifetime issues
    }
}
