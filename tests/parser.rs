use ggbasm::parser::parse_asm;
use ggbasm::instruction::Instruction;

#[test]
fn test_empty() {
    assert_eq!(parse_asm("").unwrap().as_slice(), &[]);
}

#[test]
fn test_single_newline() {
    assert_eq!(parse_asm("\n").unwrap().as_slice(),
    &[
        Instruction::EmptyLine
    ]);
}

#[test]
fn test_two_newline() {
    assert_eq!(parse_asm("\n\n").unwrap().as_slice(),
    &[
        Instruction::EmptyLine,
        Instruction::EmptyLine,
    ]);
}

#[test]
fn test_two_newline_and_space() {
    assert_eq!(parse_asm("\n   \n").unwrap().as_slice(),
    &[
        Instruction::EmptyLine,
        Instruction::EmptyLine,
    ]);
}

#[test]
fn test_final_newline_missing() {
    assert_eq!(parse_asm("nop\nnop\nnop").unwrap().as_slice(),
    &[
        Instruction::Nop,
        Instruction::Nop,
        Instruction::Nop,
    ]);
}

#[test]
fn test_final_newline_included() {
    assert_eq!(parse_asm("nop\nnop\nnop\n").unwrap().as_slice(),
    &[
        Instruction::Nop,
        Instruction::Nop,
        Instruction::Nop,
    ]);
}

#[test]
fn test_whacky_line_handling() {
    let text = r#" nop
label:
    nop

0this_-complicated_id3ntifi3r:

nop

    that:
foo:

; comment at start of line
;
     ; comment after whitespace
        nop ; comment after command
   label:; comment after label
nop ; comment after command
label2: ; some very important message

  stop    ; another comment
"#;
    assert_eq!(parse_asm(text).unwrap().as_slice(),
    &[
        Instruction::Nop,
        Instruction::Label(String::from("label")),
        Instruction::Nop,
        Instruction::EmptyLine,
        Instruction::Label(String::from("0this_-complicated_id3ntifi3r")),
        Instruction::EmptyLine,
        Instruction::Nop,
        Instruction::EmptyLine,
        Instruction::Label(String::from("that")),
        Instruction::Label(String::from("foo")),
        Instruction::EmptyLine,
        Instruction::EmptyLine,
        Instruction::EmptyLine,
        Instruction::EmptyLine,
        Instruction::Nop,
        Instruction::Label(String::from("label")),
        Instruction::Nop,
        Instruction::Label(String::from("label2")),
        Instruction::EmptyLine,
        Instruction::Stop,
    ]);
}

#[test]
fn test_instructions() {
    let text = r#"
    nop
    stop
    halt
    di
    ei
"#;
    assert_eq!(parse_asm(text).unwrap().as_slice(),
    &[
        Instruction::EmptyLine,
        Instruction::Nop,
        Instruction::Stop,
        Instruction::Halt,
        Instruction::Di,
        Instruction::Ei,
    ]);
}
