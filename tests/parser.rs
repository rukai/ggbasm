use ggbasm::parser::parse_asm;
use ggbasm::instruction::*;

#[test]
fn test_empty() {
    assert_eq!(parse_asm("").unwrap().as_slice(), &[]);
}

#[test]
fn test_single_newline() {
    let result: Vec<Instruction> = parse_asm("\n").unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result.as_slice(),
    &[
        Instruction::EmptyLine
    ]);
}

#[test]
fn test_two_newline() {
    let result: Vec<Instruction> = parse_asm("\n\n").unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result.as_slice(),
    &[
        Instruction::EmptyLine,
        Instruction::EmptyLine,
    ]);
}

#[test]
fn test_two_newline_and_space() {
    let result: Vec<Instruction> = parse_asm("\n   \n").unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result.as_slice(),
    &[
        Instruction::EmptyLine,
        Instruction::EmptyLine,
    ]);
}

#[test]
fn test_final_newline_missing() {
    let result: Vec<Instruction> = parse_asm("nop\nnop\nnop").unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result.as_slice(),
    &[
        Instruction::Nop,
        Instruction::Nop,
        Instruction::Nop,
    ]);
}

#[test]
fn test_final_newline_included() {
    let result: Vec<Instruction> = parse_asm("nop\nnop\nnop\n").unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result.as_slice(),
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
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result.as_slice(),
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
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result.as_slice(),
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
fn test_ret() {
    let text = r#"
    ret Z
    ret NZ
    ret C
    ret NC
    ret
    reti
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result.as_slice(),
    &[
        Instruction::EmptyLine,
        Instruction::RetFlag (Flag::Z),
        Instruction::RetFlag (Flag::NZ),
        Instruction::RetFlag (Flag::C),
        Instruction::RetFlag (Flag::NC),
        Instruction::Ret,
        Instruction::Reti,
    ]);
}

#[test]
fn test_call() {
    let text = r#"
    call Z foobar
    call NZ 0x1337
    call C 0
    call NC 42
    call 413
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result.as_slice(),
    &[
        Instruction::EmptyLine,
        Instruction::CallFlag (Flag::Z, ExprU16::Ident(String::from("foobar"))),
        Instruction::CallFlag (Flag::NZ, ExprU16::U16(0x1337)),
        Instruction::CallFlag (Flag::C, ExprU16::U16(0)),
        Instruction::CallFlag (Flag::NC, ExprU16::U16(42)),
        Instruction::Call (ExprU16::U16(413)),
    ]);
}

#[test]
fn test_db() {
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
    db "a"
    db "Hello World!"
    db "hi" 0x13 37
    db 4 13 "hammers"
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result.as_slice(),
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
        Instruction::Db (vec!(0x61)),
        Instruction::Db (vec!(0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21)),
        Instruction::Db (vec!(0x68, 0x69, 0x13, 37)),
        Instruction::Db (vec!(4, 13, 0x68, 0x61, 0x6d, 0x6d, 0x65, 0x72, 0x73)),
    ]);
}

#[test]
fn test_dw() {
    let text = r#"
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
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result.as_slice(),
    &[
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

#[test]
fn test_advance_address() {
    let text = r#"
    advance_address 0
    advance_address 0x0
    advance_address 413
    advance_address 0x1337
    advance_address 0xFFFF
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result.as_slice(),
    &[
        Instruction::EmptyLine,
        Instruction::AdvanceAddress (0),
        Instruction::AdvanceAddress (0x0),
        Instruction::AdvanceAddress (413),
        Instruction::AdvanceAddress (0x1337),
        Instruction::AdvanceAddress (0xFFFF),
    ]);
}

#[test]
fn test_invalid_instruction() {
    let text = r#"
    nop

    foobar

    nop
    stop that
    nop
    stopthat
    nop
a b c d
"#;
    assert_eq!(parse_asm(text).unwrap().as_slice(),
    &[
        Some(Instruction::EmptyLine),
        Some(Instruction::Nop),
        Some(Instruction::EmptyLine),
        None,
        Some(Instruction::EmptyLine),
        Some(Instruction::Nop),
        None,
        Some(Instruction::Nop),
        None,
        Some(Instruction::Nop),
        None,
    ]);
}

#[test]
fn test_jp() {
    let text = r#"
    jp 0x150
    jp foo_bar
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result.as_slice(),
    &[
        Instruction::EmptyLine,
        Instruction::Jp (ExprU16::U16 (0x0150)),
        Instruction::Jp (ExprU16::Ident (String::from("foo_bar"))),
    ]);
}

#[test]
fn test_inc_dec() {
    let text = r#"
    inc BC
    inc DE
    inc HL
    inc SP
    inc A
    inc B
    inc C
    inc D
    inc E
    inc H
    inc L
    inc [hl]
    dec BC
    dec DE
    dec HL
    dec SP
    dec A
    dec B
    dec C
    dec D
    dec E
    dec H
    dec L
    dec [hl]
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result.as_slice(),
    &[
        Instruction::EmptyLine,
        Instruction::IncR16 (Reg16::BC),
        Instruction::IncR16 (Reg16::DE),
        Instruction::IncR16 (Reg16::HL),
        Instruction::IncR16 (Reg16::SP),
        Instruction::IncR8  (Reg8::A),
        Instruction::IncR8  (Reg8::B),
        Instruction::IncR8  (Reg8::C),
        Instruction::IncR8  (Reg8::D),
        Instruction::IncR8  (Reg8::E),
        Instruction::IncR8  (Reg8::H),
        Instruction::IncR8  (Reg8::L),
        Instruction::IncM8,
        Instruction::DecR16 (Reg16::BC),
        Instruction::DecR16 (Reg16::DE),
        Instruction::DecR16 (Reg16::HL),
        Instruction::DecR16 (Reg16::SP),
        Instruction::DecR8  (Reg8::A),
        Instruction::DecR8  (Reg8::B),
        Instruction::DecR8  (Reg8::C),
        Instruction::DecR8  (Reg8::D),
        Instruction::DecR8  (Reg8::E),
        Instruction::DecR8  (Reg8::H),
        Instruction::DecR8  (Reg8::L),
        Instruction::DecM8,
    ]);
}

#[test]
fn test_ld() {
    let text = r#"
    ld BC 0x0413
    ld BC something
    ld DE 0x413
    ld HL 0x413
    ld SP 0x413
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result.as_slice(),
    &[
        Instruction::EmptyLine,
        Instruction::LdReg16Immediate (Reg16::BC, ExprU16::U16 (0x0413)),
        Instruction::LdReg16Immediate (Reg16::BC, ExprU16::Ident (String::from("something"))),
        Instruction::LdReg16Immediate (Reg16::DE, ExprU16::U16 (0x0413)),
        Instruction::LdReg16Immediate (Reg16::HL, ExprU16::U16 (0x0413)),
        Instruction::LdReg16Immediate (Reg16::SP, ExprU16::U16 (0x0413)),
    ]);
}

#[test]
fn test_push_pop() {
    let text = r#"
    push BC
    push DE
    push HL
    push AF
    pop BC
    pop DE
    pop HL
    pop AF
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result.as_slice(),
    &[
        Instruction::EmptyLine,
        Instruction::Push (Reg16Push::BC),
        Instruction::Push (Reg16Push::DE),
        Instruction::Push (Reg16Push::HL),
        Instruction::Push (Reg16Push::AF),
        Instruction::Pop  (Reg16Push::BC),
        Instruction::Pop  (Reg16Push::DE),
        Instruction::Pop  (Reg16Push::HL),
        Instruction::Pop  (Reg16Push::AF),
    ]);
}
