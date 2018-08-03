use failure::Error;
use failure::bail;
use nom::*;
use nom::types::CompleteStr;

use crate::instruction::Instruction;

static IDENTIFIER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890_-";
static WHITESPACE: &str = " \t";

named!(instructions<CompleteStr, Vec<Instruction> >,
    many0!(
        terminated!(
            do_parse!(
                opt!(is_a!(WHITESPACE)) >>
                instruction: alt!(
                    // label
                    do_parse!(
                        label: is_a!(IDENTIFIER) >>
                        tag!(":") >>
                        (Instruction::Label (label.to_string()))
                    ) |

                    // instructions
                    value!(Instruction::Stop,  tag_no_case!("stop")) |
                    value!(Instruction::Nop,   tag_no_case!("nop")) |
                    value!(Instruction::Halt,  tag_no_case!("halt")) |
                    value!(Instruction::Di,    tag_no_case!("di")) |
                    value!(Instruction::Ei,    tag_no_case!("ei")) |

                    // line containing only whitespace/empty
                    value!(Instruction::EmptyLine, is_a!(WHITESPACE)) |

                    // Gracefully handle unimplemented instructions TODO: make this an error
                    value!(Instruction::EmptyLine, not_line_ending)
                ) >>
                opt!(do_parse!(
                    opt!(is_a!(WHITESPACE)) >>
                    is_a!(";") >>
                    is_not!("\r\n") >>
                    (0)
                )) >>
                (instruction)
            ),
            line_ending
        )
    )
);

pub fn parse_asm(text: &str) -> Result<Vec<Instruction>, Error> {
    // Ensure a trailing \n is included
    // TODO: Avoid this copy, should be able to handle this in the parser combinator
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
