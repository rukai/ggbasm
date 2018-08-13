use ggbasm::parser::parse_asm;
use ggbasm::instruction::*;

#[test]
fn test_empty() {
    assert_eq!(parse_asm("").unwrap().as_slice(), &[]);
}

#[test]
fn test_single_newline() {
    let result: Vec<Instruction> = parse_asm("\n").unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(Instruction::EmptyLine));
}

#[test]
fn test_two_newline() {
    let result: Vec<Instruction> = parse_asm("\n\n").unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::EmptyLine,
    ));
}

#[test]
fn test_two_newline_and_space() {
    let result: Vec<Instruction> = parse_asm("\n   \n").unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::EmptyLine,
    ));
}

#[test]
fn test_final_newline_missing() {
    let result: Vec<Instruction> = parse_asm("nop\nnop\nnop").unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::Nop,
        Instruction::Nop,
        Instruction::Nop,
    ));
}

#[test]
fn test_final_newline_included() {
    let result: Vec<Instruction> = parse_asm("nop\nnop\nnop\n").unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::Nop,
        Instruction::Nop,
        Instruction::Nop,
    ));
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
    assert_eq!(result, vec!(
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
    ));
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
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::Nop,
        Instruction::Stop,
        Instruction::Halt,
        Instruction::Di,
        Instruction::Ei,
    ));
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
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::Ret (Flag::Z),
        Instruction::Ret (Flag::NZ),
        Instruction::Ret (Flag::C),
        Instruction::Ret (Flag::NC),
        Instruction::Ret (Flag::Always),
        Instruction::Reti,
    ));
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
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::Call (Flag::Z, Expr::Ident(String::from("foobar"))),
        Instruction::Call (Flag::NZ, Expr::Const(0x1337)),
        Instruction::Call (Flag::C, Expr::Const(0)),
        Instruction::Call (Flag::NC, Expr::Const(42)),
        Instruction::Call (Flag::Always, Expr::Const(413)),
    ));
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
    assert_eq!(result, vec!(
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
    ));
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
    assert_eq!(result, vec!(
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
    ));
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
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::AdvanceAddress (0),
        Instruction::AdvanceAddress (0x0),
        Instruction::AdvanceAddress (413),
        Instruction::AdvanceAddress (0x1337),
        Instruction::AdvanceAddress (0xFFFF),
    ));
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
    assert_eq!(parse_asm(text).unwrap(), vec!(
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
    ));
}

#[test]
fn test_jp() {
    let text = r#"
    jp 0x150
    jp nz foo_bar
    jp z 413
    jp nc 1111
    jp c 42
    jp hl
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::JpI16 (Flag::Always, Expr::Const (0x0150)),
        Instruction::JpI16 (Flag::NZ,     Expr::Ident (String::from("foo_bar"))),
        Instruction::JpI16 (Flag::Z,      Expr::Const (413)),
        Instruction::JpI16 (Flag::NC,     Expr::Const (1111)),
        Instruction::JpI16 (Flag::C,      Expr::Const (42)),
        Instruction::JpRhl,
    ));
}

#[test]
fn test_jr() {
    let text = r#"
    jr 0x42
    jr nz foo_bar
    jr z 255
    jr nc 11
    jr c 42
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::Jr (Flag::Always, Expr::Const (0x42)),
        Instruction::Jr (Flag::NZ,     Expr::Ident (String::from("foo_bar"))),
        Instruction::Jr (Flag::Z,      Expr::Const (255)),
        Instruction::Jr (Flag::NC,     Expr::Const (11)),
        Instruction::Jr (Flag::C,      Expr::Const (42)),
    ));
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
    assert_eq!(result, vec!(
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
        Instruction::IncMRhl,
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
        Instruction::DecMRhl,
    ));
}

#[test]
fn test_ld_r8_r8() {
    let text = r#"
    ld a a
    ld a b
    ld a c
    ld a d
    ld a e
    ld a h
    ld a l
    ld b a
    ld b b
    ld b c
    ld b d
    ld b e
    ld b h
    ld b l
    ld c a
    ld c b
    ld c c
    ld c d
    ld c e
    ld c h
    ld c l
    ld d a
    ld d b
    ld d c
    ld d d
    ld d e
    ld d h
    ld d l
    ld e a
    ld e b
    ld e c
    ld e d
    ld e e
    ld e h
    ld e l
    ld h a
    ld h b
    ld h c
    ld h d
    ld h e
    ld h h
    ld h l
    ld l a
    ld l b
    ld l c
    ld l d
    ld l e
    ld l h
    ld l l
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::LdR8R8 (Reg8::A, Reg8::A),
        Instruction::LdR8R8 (Reg8::A, Reg8::B),
        Instruction::LdR8R8 (Reg8::A, Reg8::C),
        Instruction::LdR8R8 (Reg8::A, Reg8::D),
        Instruction::LdR8R8 (Reg8::A, Reg8::E),
        Instruction::LdR8R8 (Reg8::A, Reg8::H),
        Instruction::LdR8R8 (Reg8::A, Reg8::L),
        Instruction::LdR8R8 (Reg8::B, Reg8::A),
        Instruction::LdR8R8 (Reg8::B, Reg8::B),
        Instruction::LdR8R8 (Reg8::B, Reg8::C),
        Instruction::LdR8R8 (Reg8::B, Reg8::D),
        Instruction::LdR8R8 (Reg8::B, Reg8::E),
        Instruction::LdR8R8 (Reg8::B, Reg8::H),
        Instruction::LdR8R8 (Reg8::B, Reg8::L),
        Instruction::LdR8R8 (Reg8::C, Reg8::A),
        Instruction::LdR8R8 (Reg8::C, Reg8::B),
        Instruction::LdR8R8 (Reg8::C, Reg8::C),
        Instruction::LdR8R8 (Reg8::C, Reg8::D),
        Instruction::LdR8R8 (Reg8::C, Reg8::E),
        Instruction::LdR8R8 (Reg8::C, Reg8::H),
        Instruction::LdR8R8 (Reg8::C, Reg8::L),
        Instruction::LdR8R8 (Reg8::D, Reg8::A),
        Instruction::LdR8R8 (Reg8::D, Reg8::B),
        Instruction::LdR8R8 (Reg8::D, Reg8::C),
        Instruction::LdR8R8 (Reg8::D, Reg8::D),
        Instruction::LdR8R8 (Reg8::D, Reg8::E),
        Instruction::LdR8R8 (Reg8::D, Reg8::H),
        Instruction::LdR8R8 (Reg8::D, Reg8::L),
        Instruction::LdR8R8 (Reg8::E, Reg8::A),
        Instruction::LdR8R8 (Reg8::E, Reg8::B),
        Instruction::LdR8R8 (Reg8::E, Reg8::C),
        Instruction::LdR8R8 (Reg8::E, Reg8::D),
        Instruction::LdR8R8 (Reg8::E, Reg8::E),
        Instruction::LdR8R8 (Reg8::E, Reg8::H),
        Instruction::LdR8R8 (Reg8::E, Reg8::L),
        Instruction::LdR8R8 (Reg8::H, Reg8::A),
        Instruction::LdR8R8 (Reg8::H, Reg8::B),
        Instruction::LdR8R8 (Reg8::H, Reg8::C),
        Instruction::LdR8R8 (Reg8::H, Reg8::D),
        Instruction::LdR8R8 (Reg8::H, Reg8::E),
        Instruction::LdR8R8 (Reg8::H, Reg8::H),
        Instruction::LdR8R8 (Reg8::H, Reg8::L),
        Instruction::LdR8R8 (Reg8::L, Reg8::A),
        Instruction::LdR8R8 (Reg8::L, Reg8::B),
        Instruction::LdR8R8 (Reg8::L, Reg8::C),
        Instruction::LdR8R8 (Reg8::L, Reg8::D),
        Instruction::LdR8R8 (Reg8::L, Reg8::E),
        Instruction::LdR8R8 (Reg8::L, Reg8::H),
        Instruction::LdR8R8 (Reg8::L, Reg8::L),
    ));
}
#[test]
fn test_ld() {
    let text = r#"
    ld BC 0x0413
    ld BC something
    ld DE 0x413
    ld HL 0x413
    ld SP 0x413

    ld [0x3535] sp
    ld [ 0x3535 ] sp
    ld [0x3535  ] sp
    ld [    0x3535] sp
    ld [    0x3535  ] sp

    ld a 0xFF
    ld b foo
    ld c 0x10
    ld d 42
    ld e 42
    ld h 42
    ld l 42

    ld [bc] a
    ld [de] a
    ld a [bc]
    ld a [de]

    ldi [hl] a
    ldd [hl] a
    ldi a [hl]
    ldd a [hl]

    ld [hl] 42

    ld a [hl]
    ld b [hl]
    ld c [hl]
    ld d [hl]
    ld e [hl]
    ld h [hl]
    ld l [hl]

    ld [hl] a
    ld [hl] b
    ld [hl] c
    ld [hl] d
    ld [hl] e
    ld [hl] h
    ld [hl] l

    ld [0xFF00 + 42] a
    ld [0xFF00+42] a
    ld a [0xFF00 + 42]
    ld a [0xFF00+42]

    ld [0xFF00 + c] a
    ld [0xFF00+c] a
    ld [  0xFF00   +   c   ] a
    ld a [0xFF00 + c]
    ld a [0xFF00+c]

    ld hl sp+13
    ld hl sp + 13
    ld sp hl
    ld [0x413] a
    ld a [0x0413]
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::LdR16I16 (Reg16::BC, Expr::Const (0x0413)),
        Instruction::LdR16I16 (Reg16::BC, Expr::Ident (String::from("something"))),
        Instruction::LdR16I16 (Reg16::DE, Expr::Const (0x0413)),
        Instruction::LdR16I16 (Reg16::HL, Expr::Const (0x0413)),
        Instruction::LdR16I16 (Reg16::SP, Expr::Const (0x0413)),
        Instruction::EmptyLine,
        Instruction::LdMI16Rsp (Expr::Const (0x3535)),
        Instruction::LdMI16Rsp (Expr::Const (0x3535)),
        Instruction::LdMI16Rsp (Expr::Const (0x3535)),
        Instruction::LdMI16Rsp (Expr::Const (0x3535)),
        Instruction::LdMI16Rsp (Expr::Const (0x3535)),
        Instruction::EmptyLine,
        Instruction::LdR8I8 (Reg8::A, Expr::Const (0xFF)),
        Instruction::LdR8I8 (Reg8::B, Expr::Ident (String::from("foo"))),
        Instruction::LdR8I8 (Reg8::C, Expr::Const (0x10)),
        Instruction::LdR8I8 (Reg8::D, Expr::Const (42)),
        Instruction::LdR8I8 (Reg8::E, Expr::Const (42)),
        Instruction::LdR8I8 (Reg8::H, Expr::Const (42)),
        Instruction::LdR8I8 (Reg8::L, Expr::Const (42)),
        Instruction::EmptyLine,
        Instruction::LdMRbcRa,
        Instruction::LdMRdeRa,
        Instruction::LdRaMRbc,
        Instruction::LdRaMRde,
        Instruction::EmptyLine,
        Instruction::LdiMRhlRa,
        Instruction::LddMRhlRa,
        Instruction::LdiRaMRhl,
        Instruction::LddRaMRhl,
        Instruction::EmptyLine,
        Instruction::LdMRhlI8 (Expr::Const (42)),
        Instruction::EmptyLine,
        Instruction::LdR8MRhl (Reg8::A),
        Instruction::LdR8MRhl (Reg8::B),
        Instruction::LdR8MRhl (Reg8::C),
        Instruction::LdR8MRhl (Reg8::D),
        Instruction::LdR8MRhl (Reg8::E),
        Instruction::LdR8MRhl (Reg8::H),
        Instruction::LdR8MRhl (Reg8::L),
        Instruction::EmptyLine,
        Instruction::LdMRhlR8 (Reg8::A),
        Instruction::LdMRhlR8 (Reg8::B),
        Instruction::LdMRhlR8 (Reg8::C),
        Instruction::LdMRhlR8 (Reg8::D),
        Instruction::LdMRhlR8 (Reg8::E),
        Instruction::LdMRhlR8 (Reg8::H),
        Instruction::LdMRhlR8 (Reg8::L),
        Instruction::EmptyLine,
        Instruction::LdhMI8Ra (Expr::Const (42)),
        Instruction::LdhMI8Ra (Expr::Const (42)),
        Instruction::LdhRaMI8 (Expr::Const (42)),
        Instruction::LdhRaMI8 (Expr::Const (42)),
        Instruction::EmptyLine,
        Instruction::LdhMRcRa,
        Instruction::LdhMRcRa,
        Instruction::LdhMRcRa,
        Instruction::LdhRaMRc,
        Instruction::LdhRaMRc,
        Instruction::EmptyLine,
        Instruction::LdRhlRspI8 (Expr::Const (13)),
        Instruction::LdRhlRspI8 (Expr::Const (13)),
        Instruction::LdRspRhl,
        Instruction::LdMI16Ra (Expr::Const (0x413)),
        Instruction::LdRaMI16 (Expr::Const (0x413)),
    ));
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
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::Push (Reg16Push::BC),
        Instruction::Push (Reg16Push::DE),
        Instruction::Push (Reg16Push::HL),
        Instruction::Push (Reg16Push::AF),
        Instruction::Pop  (Reg16Push::BC),
        Instruction::Pop  (Reg16Push::DE),
        Instruction::Pop  (Reg16Push::HL),
        Instruction::Pop  (Reg16Push::AF),
    ));
}
