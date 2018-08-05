use byteorder::{LittleEndian, WriteBytesExt};
use failure::Error;
use failure::bail;
use nom::*;
use nom::types::CompleteStr;

use crate::instruction::Instruction;

static IDENTIFIER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890_-";
static HEX:        &str = "1234567890ABCDEFabcdef";
static DEC:        &str = "1234567890";
static WHITESPACE: &str = " \t";

fn is_hex(input: char) -> bool {
    HEX.contains(input)
}

fn is_dec(input: char) -> bool {
    DEC.contains(input)
}

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
            (u8::from_str_radix(value.as_ref(), 10).unwrap())
        )
    )
);

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
            (u16::from_str_radix(value.as_ref(), 10).unwrap())
        )
    )
);

fn u16_to_db(input: u16) -> Instruction {
    let mut result = vec!();
    result.write_u16::<LittleEndian>(input).unwrap();
    Instruction::Db(result)
}

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

named!(instruction<CompleteStr, Instruction >,
    alt!(
        // label
        do_parse!(
            label: is_a!(IDENTIFIER) >>
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
        map!(
            do_parse!(
                tag_no_case!("dw") >>
                is_a!(WHITESPACE) >>
                value: parse_u16 >>
                end_line >>
                (value)
            ),
            u16_to_db
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
        terminated!(value!(Instruction::Stop,  tag_no_case!("stop")), end_line) |
        terminated!(value!(Instruction::Nop,   tag_no_case!("nop")),  end_line) |
        terminated!(value!(Instruction::Halt,  tag_no_case!("halt")), end_line) |
        terminated!(value!(Instruction::Di,    tag_no_case!("di")),   end_line) |
        terminated!(value!(Instruction::Ei,    tag_no_case!("ei")),   end_line) |
        terminated!(value!(Instruction::Ret,   tag_no_case!("ret")),  end_line) |
        terminated!(value!(Instruction::Reti,  tag_no_case!("reti")), end_line) |

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
