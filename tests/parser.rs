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
    ; the following lines has whitespace afterwards
    nop        
stop    
whitespace_following: 
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
        Instruction::EmptyLine,
        Instruction::Nop,
        Instruction::Stop,
        Instruction::Label(String::from("whitespace_following")),
    ]);
}

#[test]
fn test_simple_instructions() {
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

#[test]
fn test_db_dw() {
    let text = r#"
    db 0
    db 255
    db 42
    db 0x42
    db 0x0
    db 0xB
    db 0x00
    db 0xFF
    db 0x04 0x13
    db 0 1
    db 0 0 1 2 3 4
    db 0 0x1 2 0x3 5 0x4

    dw 0
    dw 413
    dw 65535
    dw 0x0
    dw 0xE
    dw 0x13
    dw 0x413
    dw 0x0000
    dw 0xFFFF
    dw 0x1337
"#;
    assert_eq!(parse_asm(text).unwrap().as_slice(),
    &[
        Instruction::EmptyLine,
        Instruction::Db (vec!(0)),
        Instruction::Db (vec!(255)),
        Instruction::Db (vec!(42)),
        Instruction::Db (vec!(0x42)),
        Instruction::Db (vec!(0x00)),
        Instruction::Db (vec!(0x0B)),
        Instruction::Db (vec!(0x00)),
        Instruction::Db (vec!(0xFF)),
        Instruction::Db (vec!(0x04, 0x13)),
        Instruction::Db (vec!(0, 1)),
        Instruction::Db (vec!(0, 0, 1, 2, 3, 4)),
        Instruction::Db (vec!(0, 1, 2, 3, 5, 4)),
        Instruction::EmptyLine,
        Instruction::Db (vec!(0x00, 0x00)),
        Instruction::Db (vec!(0x9d, 0x01)),
        Instruction::Db (vec!(0xFF, 0xFF)),
        Instruction::Db (vec!(0x00, 0x00)),
        Instruction::Db (vec!(0x0E, 0x00)),
        Instruction::Db (vec!(0x13, 0x00)),
        Instruction::Db (vec!(0x13, 0x04)),
        Instruction::Db (vec!(0x00, 0x00)),
        Instruction::Db (vec!(0xFF, 0xFF)),
        Instruction::Db (vec!(0x37, 0x13)),
    ]);
}
