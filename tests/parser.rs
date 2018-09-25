use ggbasm::parser::parse_asm;
use ggbasm::ast::*;

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

0this_complicated_id3ntifi3r:

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
xor 13   ; instructions ending with expression need to handle spaces at the end
xor a    ; instructions ending with registers need to handle spaces at the end
xor [hl] ; instructions ending with [hl] need to handle spaces at the end
xor a,42 ; minimal spaces
xor a, 42 ; regular spaces
xor     a   ,    42 ; lots of spaces
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::Nop,
        Instruction::Label(String::from("label")),
        Instruction::Nop,
        Instruction::EmptyLine,
        Instruction::Label(String::from("0this_complicated_id3ntifi3r")),
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
        Instruction::XorI8 (Expr::Const(13)),
        Instruction::XorR8 (Reg8::A),
        Instruction::XorMRhl,
        Instruction::XorI8 (Expr::Const(42)),
        Instruction::XorI8 (Expr::Const(42)),
        Instruction::XorI8 (Expr::Const(42)),
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
    rrca
    rra
    cpl
    ccf
    rlca
    rla
    daa
    scf
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::Nop,
        Instruction::Stop,
        Instruction::Halt,
        Instruction::Di,
        Instruction::Ei,
        Instruction::Rrca,
        Instruction::Rra,
        Instruction::Cpl,
        Instruction::Ccf,
        Instruction::Rlca,
        Instruction::Rla,
        Instruction::Daa,
        Instruction::Scf,
    ));
}

#[test]
fn test_exprs_simple() {
    let text = r#"
    jp foo_bar
    jp -foo
    jp foo + bar
    jp foo - bar
    jp foo * bar
    jp foo / bar
    jp foo % bar
    jp foo & bar
    jp foo | bar
    jp foo ^ bar

    jp foo-42
    jp 413*1111
    jp foo /0x40
    jp foo% bar

    jp z, foo_bar
    jp z, foo + bar
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::JpI16 (Flag::Always, Expr::Ident (String::from("foo_bar"))),
        Instruction::JpI16 (Flag::Always, Expr::unary(Expr::Ident(String::from("foo")), UnaryOperator::Minus)),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Add, Expr::Ident(String::from("bar")))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Sub, Expr::Ident(String::from("bar")))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Mul, Expr::Ident(String::from("bar")))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Div, Expr::Ident(String::from("bar")))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Rem, Expr::Ident(String::from("bar")))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::And, Expr::Ident(String::from("bar")))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Or,  Expr::Ident(String::from("bar")))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Xor, Expr::Ident(String::from("bar")))),
        Instruction::EmptyLine,
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Sub, Expr::Const(42))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::Const(413),                 BinaryOperator::Mul, Expr::Const(1111))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Div, Expr::Const(0x40))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Rem, Expr::Ident(String::from("bar")))),
        Instruction::EmptyLine,
        Instruction::JpI16 (Flag::Z, Expr::Ident (String::from("foo_bar"))),
        Instruction::JpI16 (Flag::Z, Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Add, Expr::Ident(String::from("bar")))),
    ));
}

#[test]
fn test_exprs_complex() {
    let text = r#"
    jp (foo + bar)
    jp (foo + bar) + baz
    jp foo + (bar + baz)
    jp foo * (bar + baz)
    jp (foo + bar) * baz
    jp ((foo + bar) * baz)
    jp (foo * -bar)
    jp ((-foo + bar) * baz)
    jp foo + bar * baz
    jp foo + bar / baz
    jp foo + bar % baz
    jp foo * bar + baz
    jp foo / bar + baz
    jp foo % bar + baz
    jp foo % bar ^ baz
    jp foo - bar & baz
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Add, Expr::Ident(String::from("bar")))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Add, Expr::Ident(String::from("bar"))), BinaryOperator::Add, Expr::Ident(String::from("baz")))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Add, Expr::binary(Expr::Ident(String::from("bar")), BinaryOperator::Add, Expr::Ident(String::from("baz"))))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Mul, Expr::binary(Expr::Ident(String::from("bar")), BinaryOperator::Add, Expr::Ident(String::from("baz"))))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Add, Expr::Ident(String::from("bar"))), BinaryOperator::Mul, Expr::Ident(String::from("baz")))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Add, Expr::Ident(String::from("bar"))), BinaryOperator::Mul, Expr::Ident(String::from("baz")))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Mul, Expr::unary(Expr::Ident(String::from("bar")), UnaryOperator::Minus))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::binary(Expr::unary(Expr::Ident(String::from("foo")), UnaryOperator::Minus), BinaryOperator::Add, Expr::Ident(String::from("bar"))), BinaryOperator::Mul, Expr::Ident(String::from("baz")))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Add, Expr::binary(Expr::Ident(String::from("bar")), BinaryOperator::Mul, Expr::Ident(String::from("baz"))))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Add, Expr::binary(Expr::Ident(String::from("bar")), BinaryOperator::Div, Expr::Ident(String::from("baz"))))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Add, Expr::binary(Expr::Ident(String::from("bar")), BinaryOperator::Rem, Expr::Ident(String::from("baz"))))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Mul, Expr::Ident(String::from("bar"))), BinaryOperator::Add, Expr::Ident(String::from("baz")))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Div, Expr::Ident(String::from("bar"))), BinaryOperator::Add, Expr::Ident(String::from("baz")))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Rem, Expr::Ident(String::from("bar"))), BinaryOperator::Add, Expr::Ident(String::from("baz")))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Rem, Expr::Ident(String::from("bar"))), BinaryOperator::Xor, Expr::Ident(String::from("baz")))),
        Instruction::JpI16 (Flag::Always, Expr::binary(Expr::binary(Expr::Ident(String::from("foo")), BinaryOperator::Sub, Expr::Ident(String::from("bar"))), BinaryOperator::And, Expr::Ident(String::from("baz")))),
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
    call Z, foobar
    call NZ, 0x1337
    call C, 0
    call NC, 42
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
fn test_equ() {
    let text = r#"
    foo equ bar
    bar EQU 0xFF
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::Equ (String::from("foo"), Expr::Ident(String::from("bar"))),
        Instruction::Equ (String::from("bar"), Expr::Const(0xFF)),
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
    db 0x04, 0x13
    db 0, 1
    db 0, 0, 1, 2, 3, 4
    db 0, 0x1, 2, 0x3, 5, 0x4
    db "a"
    db "Hello World!"
    db "hi", 0x13, 37
    db 4, 13, "hammers"
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
    jp nz, foo_bar
    jp z, 413
    jp nc, 1111
    jp c, 42
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
    jr nz, foo_bar
    jr z, 255
    jr nc, 11
    jr c, 42
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
fn test_add() {
    let text = r#"
    add 0xFF
    add [hl]
    add a
    add b
    add c
    add d
    add e
    add h
    add l

    add a, 0xFF
    add a, [hl]
    add a, a
    add a, b
    add a, c
    add a, d
    add a, e
    add a, h
    add a, l

    add hl, bc
    add hl, de
    add hl, hl
    add hl, sp
    add sp, 2
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::AddI8 (Expr::Const(0xFF)),
        Instruction::AddMRhl,
        Instruction::AddR8 (Reg8::A),
        Instruction::AddR8 (Reg8::B),
        Instruction::AddR8 (Reg8::C),
        Instruction::AddR8 (Reg8::D),
        Instruction::AddR8 (Reg8::E),
        Instruction::AddR8 (Reg8::H),
        Instruction::AddR8 (Reg8::L),
        Instruction::EmptyLine,
        Instruction::AddI8 (Expr::Const(0xFF)),
        Instruction::AddMRhl,
        Instruction::AddR8 (Reg8::A),
        Instruction::AddR8 (Reg8::B),
        Instruction::AddR8 (Reg8::C),
        Instruction::AddR8 (Reg8::D),
        Instruction::AddR8 (Reg8::E),
        Instruction::AddR8 (Reg8::H),
        Instruction::AddR8 (Reg8::L),
        Instruction::EmptyLine,
        Instruction::AddRhlR16 (Reg16::BC),
        Instruction::AddRhlR16 (Reg16::DE),
        Instruction::AddRhlR16 (Reg16::HL),
        Instruction::AddRhlR16 (Reg16::SP),
        Instruction::AddRspI8 (Expr::Const(2)),
    ));
}

#[test]
fn test_sub() {
    let text = r#"
    sub 0xFF
    sub [hl]
    sub a
    sub b
    sub c
    sub d
    sub e
    sub h
    sub l

    sub a, 0xFF
    sub a, [hl]
    sub a, a
    sub a, b
    sub a, c
    sub a, d
    sub a, e
    sub a, h
    sub a, l
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::SubI8 (Expr::Const(0xFF)),
        Instruction::SubMRhl,
        Instruction::SubR8 (Reg8::A),
        Instruction::SubR8 (Reg8::B),
        Instruction::SubR8 (Reg8::C),
        Instruction::SubR8 (Reg8::D),
        Instruction::SubR8 (Reg8::E),
        Instruction::SubR8 (Reg8::H),
        Instruction::SubR8 (Reg8::L),
        Instruction::EmptyLine,
        Instruction::SubI8 (Expr::Const(0xFF)),
        Instruction::SubMRhl,
        Instruction::SubR8 (Reg8::A),
        Instruction::SubR8 (Reg8::B),
        Instruction::SubR8 (Reg8::C),
        Instruction::SubR8 (Reg8::D),
        Instruction::SubR8 (Reg8::E),
        Instruction::SubR8 (Reg8::H),
        Instruction::SubR8 (Reg8::L),
    ));
}

#[test]
fn test_and() {
    let text = r#"
    and 0xFF
    and [hl]
    and a
    and b
    and c
    and d
    and e
    and h
    and l

    and a, 0xFF
    and a, [hl]
    and a, a
    and a, b
    and a, c
    and a, d
    and a, e
    and a, h
    and a, l
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::AndI8 (Expr::Const(0xFF)),
        Instruction::AndMRhl,
        Instruction::AndR8 (Reg8::A),
        Instruction::AndR8 (Reg8::B),
        Instruction::AndR8 (Reg8::C),
        Instruction::AndR8 (Reg8::D),
        Instruction::AndR8 (Reg8::E),
        Instruction::AndR8 (Reg8::H),
        Instruction::AndR8 (Reg8::L),
        Instruction::EmptyLine,
        Instruction::AndI8 (Expr::Const(0xFF)),
        Instruction::AndMRhl,
        Instruction::AndR8 (Reg8::A),
        Instruction::AndR8 (Reg8::B),
        Instruction::AndR8 (Reg8::C),
        Instruction::AndR8 (Reg8::D),
        Instruction::AndR8 (Reg8::E),
        Instruction::AndR8 (Reg8::H),
        Instruction::AndR8 (Reg8::L),
    ));
}

#[test]
fn test_or() {
    let text = r#"
    or 0xFF
    or [hl]
    or a
    or b
    or c
    or d
    or e
    or h
    or l

    or a, 0xFF
    or a, [hl]
    or a, a
    or a, b
    or a, c
    or a, d
    or a, e
    or a, h
    or a, l
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::OrI8 (Expr::Const(0xFF)),
        Instruction::OrMRhl,
        Instruction::OrR8 (Reg8::A),
        Instruction::OrR8 (Reg8::B),
        Instruction::OrR8 (Reg8::C),
        Instruction::OrR8 (Reg8::D),
        Instruction::OrR8 (Reg8::E),
        Instruction::OrR8 (Reg8::H),
        Instruction::OrR8 (Reg8::L),
        Instruction::EmptyLine,
        Instruction::OrI8 (Expr::Const(0xFF)),
        Instruction::OrMRhl,
        Instruction::OrR8 (Reg8::A),
        Instruction::OrR8 (Reg8::B),
        Instruction::OrR8 (Reg8::C),
        Instruction::OrR8 (Reg8::D),
        Instruction::OrR8 (Reg8::E),
        Instruction::OrR8 (Reg8::H),
        Instruction::OrR8 (Reg8::L),
    ));
}

#[test]
fn test_adc() {
    let text = r#"
    adc 0xFF
    adc [hl]
    adc a
    adc b
    adc c
    adc d
    adc e
    adc h
    adc l

    adc a, 0xFF
    adc a, [hl]
    adc a, a
    adc a, b
    adc a, c
    adc a, d
    adc a, e
    adc a, h
    adc a, l
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::AdcI8 (Expr::Const(0xFF)),
        Instruction::AdcMRhl,
        Instruction::AdcR8 (Reg8::A),
        Instruction::AdcR8 (Reg8::B),
        Instruction::AdcR8 (Reg8::C),
        Instruction::AdcR8 (Reg8::D),
        Instruction::AdcR8 (Reg8::E),
        Instruction::AdcR8 (Reg8::H),
        Instruction::AdcR8 (Reg8::L),
        Instruction::EmptyLine,
        Instruction::AdcI8 (Expr::Const(0xFF)),
        Instruction::AdcMRhl,
        Instruction::AdcR8 (Reg8::A),
        Instruction::AdcR8 (Reg8::B),
        Instruction::AdcR8 (Reg8::C),
        Instruction::AdcR8 (Reg8::D),
        Instruction::AdcR8 (Reg8::E),
        Instruction::AdcR8 (Reg8::H),
        Instruction::AdcR8 (Reg8::L),
    ));
}

#[test]
fn test_sbc() {
    let text = r#"
    sbc 0xFF
    sbc [hl]
    sbc a
    sbc b
    sbc c
    sbc d
    sbc e
    sbc h
    sbc l

    sbc a, 0xFF
    sbc a, [hl]
    sbc a, a
    sbc a, b
    sbc a, c
    sbc a, d
    sbc a, e
    sbc a, h
    sbc a, l
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::SbcI8 (Expr::Const(0xFF)),
        Instruction::SbcMRhl,
        Instruction::SbcR8 (Reg8::A),
        Instruction::SbcR8 (Reg8::B),
        Instruction::SbcR8 (Reg8::C),
        Instruction::SbcR8 (Reg8::D),
        Instruction::SbcR8 (Reg8::E),
        Instruction::SbcR8 (Reg8::H),
        Instruction::SbcR8 (Reg8::L),
        Instruction::EmptyLine,
        Instruction::SbcI8 (Expr::Const(0xFF)),
        Instruction::SbcMRhl,
        Instruction::SbcR8 (Reg8::A),
        Instruction::SbcR8 (Reg8::B),
        Instruction::SbcR8 (Reg8::C),
        Instruction::SbcR8 (Reg8::D),
        Instruction::SbcR8 (Reg8::E),
        Instruction::SbcR8 (Reg8::H),
        Instruction::SbcR8 (Reg8::L),
    ));
}

#[test]
fn test_xor() {
    let text = r#"
    xor 0xFF
    xor [hl]
    xor a
    xor b
    xor c
    xor d
    xor e
    xor h
    xor l

    xor a, 0xFF
    xor a, [hl]
    xor a, a
    xor a, b
    xor a, c
    xor a, d
    xor a, e
    xor a, h
    xor a, l
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::XorI8 (Expr::Const(0xFF)),
        Instruction::XorMRhl,
        Instruction::XorR8 (Reg8::A),
        Instruction::XorR8 (Reg8::B),
        Instruction::XorR8 (Reg8::C),
        Instruction::XorR8 (Reg8::D),
        Instruction::XorR8 (Reg8::E),
        Instruction::XorR8 (Reg8::H),
        Instruction::XorR8 (Reg8::L),
        Instruction::EmptyLine,
        Instruction::XorI8 (Expr::Const(0xFF)),
        Instruction::XorMRhl,
        Instruction::XorR8 (Reg8::A),
        Instruction::XorR8 (Reg8::B),
        Instruction::XorR8 (Reg8::C),
        Instruction::XorR8 (Reg8::D),
        Instruction::XorR8 (Reg8::E),
        Instruction::XorR8 (Reg8::H),
        Instruction::XorR8 (Reg8::L),
    ));
}

#[test]
fn test_cp() {
    let text = r#"
    cp 0xFF
    cp [hl]
    cp a
    cp b
    cp c
    cp d
    cp e
    cp h
    cp l

    cp a, 0xFF
    cp a, [hl]
    cp a, a
    cp a, b
    cp a, c
    cp a, d
    cp a, e
    cp a, h
    cp a, l
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::CpI8 (Expr::Const(0xFF)),
        Instruction::CpMRhl,
        Instruction::CpR8 (Reg8::A),
        Instruction::CpR8 (Reg8::B),
        Instruction::CpR8 (Reg8::C),
        Instruction::CpR8 (Reg8::D),
        Instruction::CpR8 (Reg8::E),
        Instruction::CpR8 (Reg8::H),
        Instruction::CpR8 (Reg8::L),
        Instruction::EmptyLine,
        Instruction::CpI8 (Expr::Const(0xFF)),
        Instruction::CpMRhl,
        Instruction::CpR8 (Reg8::A),
        Instruction::CpR8 (Reg8::B),
        Instruction::CpR8 (Reg8::C),
        Instruction::CpR8 (Reg8::D),
        Instruction::CpR8 (Reg8::E),
        Instruction::CpR8 (Reg8::H),
        Instruction::CpR8 (Reg8::L),
    ));
}

#[test]
fn test_ld_r8_r8() {
    let text = r#"
    ld a, a
    ld a, b
    ld a, c
    ld a, d
    ld a, e
    ld a, h
    ld a, l
    ld b, a
    ld b, b
    ld b, c
    ld b, d
    ld b, e
    ld b, h
    ld b, l
    ld c, a
    ld c, b
    ld c, c
    ld c, d
    ld c, e
    ld c, h
    ld c, l
    ld d, a
    ld d, b
    ld d, c
    ld d, d
    ld d, e
    ld d, h
    ld d, l
    ld e, a
    ld e, b
    ld e, c
    ld e, d
    ld e, e
    ld e, h
    ld e, l
    ld h, a
    ld h, b
    ld h, c
    ld h, d
    ld h, e
    ld h, h
    ld h, l
    ld l, a
    ld l, b
    ld l, c
    ld l, d
    ld l, e
    ld l, h
    ld l, l
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
    ld BC, 0x0413
    ld BC, something
    ld DE, 0x413
    ld HL, 0x413
    ld SP, 0x413

    ld [0x3535], sp
    ld [ 0x3535 ], sp
    ld [0x3535  ], sp
    ld [    0x3535], sp
    ld [    0x3535  ], sp

    ld a, 0xFF
    ld b, foo
    ld c, 0x10
    ld d, 42
    ld e, 42
    ld h, 42
    ld l, 42

    ld [bc], a
    ld [de], a
    ld a, [bc]
    ld a, [de]

    ldi [hl], a
    ldd [hl], a
    ldi a, [hl]
    ldd a, [hl]

    ld [hl], 42

    ld a, [hl]
    ld b, [hl]
    ld c, [hl]
    ld d, [hl]
    ld e, [hl]
    ld h, [hl]
    ld l, [hl]

    ld [hl], a
    ld [hl], b
    ld [hl], c
    ld [hl], d
    ld [hl], e
    ld [hl], h
    ld [hl], l

    ld [0xFF00 + 42], a
    ld [0xFF00+42], a
    ld a, [0xFF00 + 42]
    ld a, [0xFF00+42]

    ld [0xFF00 + c], a
    ld [0xFF00+c], a
    ld [  0xFF00   +   c   ], a
    ld a, [0xFF00 + c]
    ld a, [0xFF00+c]

    ld hl, sp+13
    ld hl, sp + 13
    ld sp, hl
    ld [0x413], a
    ld a, [0x0413]
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

#[test]
fn test_rlc() {
    let text = r#"
    rlc a
    rlc b
    rlc c
    rlc d
    rlc e
    rlc h
    rlc l
    rlc [hl]
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::RlcR8   (Reg8::A),
        Instruction::RlcR8   (Reg8::B),
        Instruction::RlcR8   (Reg8::C),
        Instruction::RlcR8   (Reg8::D),
        Instruction::RlcR8   (Reg8::E),
        Instruction::RlcR8   (Reg8::H),
        Instruction::RlcR8   (Reg8::L),
        Instruction::RlcMRhl,
    ));
}

#[test]
fn test_rrc() {
    let text = r#"
    rrc a
    rrc b
    rrc c
    rrc d
    rrc e
    rrc h
    rrc l
    rrc [hl]
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::RrcR8   (Reg8::A),
        Instruction::RrcR8   (Reg8::B),
        Instruction::RrcR8   (Reg8::C),
        Instruction::RrcR8   (Reg8::D),
        Instruction::RrcR8   (Reg8::E),
        Instruction::RrcR8   (Reg8::H),
        Instruction::RrcR8   (Reg8::L),
        Instruction::RrcMRhl,
    ));
}

#[test]
fn test_rl() {
    let text = r#"
    rl a
    rl b
    rl c
    rl d
    rl e
    rl h
    rl l
    rl [hl]
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::RlR8   (Reg8::A),
        Instruction::RlR8   (Reg8::B),
        Instruction::RlR8   (Reg8::C),
        Instruction::RlR8   (Reg8::D),
        Instruction::RlR8   (Reg8::E),
        Instruction::RlR8   (Reg8::H),
        Instruction::RlR8   (Reg8::L),
        Instruction::RlMRhl,
    ));
}

#[test]
fn test_rr() {
    let text = r#"
    rr a
    rr b
    rr c
    rr d
    rr e
    rr h
    rr l
    rr [hl]
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::RrR8   (Reg8::A),
        Instruction::RrR8   (Reg8::B),
        Instruction::RrR8   (Reg8::C),
        Instruction::RrR8   (Reg8::D),
        Instruction::RrR8   (Reg8::E),
        Instruction::RrR8   (Reg8::H),
        Instruction::RrR8   (Reg8::L),
        Instruction::RrMRhl,
    ));
}

#[test]
fn test_sla() {
    let text = r#"
    sla a
    sla b
    sla c
    sla d
    sla e
    sla h
    sla l
    sla [hl]
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::SlaR8   (Reg8::A),
        Instruction::SlaR8   (Reg8::B),
        Instruction::SlaR8   (Reg8::C),
        Instruction::SlaR8   (Reg8::D),
        Instruction::SlaR8   (Reg8::E),
        Instruction::SlaR8   (Reg8::H),
        Instruction::SlaR8   (Reg8::L),
        Instruction::SlaMRhl,
    ));
}

#[test]
fn test_sra() {
    let text = r#"
    sra a
    sra b
    sra c
    sra d
    sra e
    sra h
    sra l
    sra [hl]
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::SraR8   (Reg8::A),
        Instruction::SraR8   (Reg8::B),
        Instruction::SraR8   (Reg8::C),
        Instruction::SraR8   (Reg8::D),
        Instruction::SraR8   (Reg8::E),
        Instruction::SraR8   (Reg8::H),
        Instruction::SraR8   (Reg8::L),
        Instruction::SraMRhl,
    ));
}

#[test]
fn test_swap() {
    let text = r#"
    swap a
    swap b
    swap c
    swap d
    swap e
    swap h
    swap l
    swap [hl]
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::SwapR8   (Reg8::A),
        Instruction::SwapR8   (Reg8::B),
        Instruction::SwapR8   (Reg8::C),
        Instruction::SwapR8   (Reg8::D),
        Instruction::SwapR8   (Reg8::E),
        Instruction::SwapR8   (Reg8::H),
        Instruction::SwapR8   (Reg8::L),
        Instruction::SwapMRhl,
    ));
}

#[test]
fn test_srl() {
    let text = r#"
    srl a
    srl b
    srl c
    srl d
    srl e
    srl h
    srl l
    srl [hl]
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::SrlR8   (Reg8::A),
        Instruction::SrlR8   (Reg8::B),
        Instruction::SrlR8   (Reg8::C),
        Instruction::SrlR8   (Reg8::D),
        Instruction::SrlR8   (Reg8::E),
        Instruction::SrlR8   (Reg8::H),
        Instruction::SrlR8   (Reg8::L),
        Instruction::SrlMRhl,
    ));
}

#[test]
fn test_bit_bit_r8() {
    let text = r#"
    bit 2, a
    bit 2, b
    bit 2, c
    bit 2, d
    bit 2, e
    bit 2, h
    bit 2, l
    bit 2, [hl]
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::BitBitR8   (Expr::Const(2), Reg8::A),
        Instruction::BitBitR8   (Expr::Const(2), Reg8::B),
        Instruction::BitBitR8   (Expr::Const(2), Reg8::C),
        Instruction::BitBitR8   (Expr::Const(2), Reg8::D),
        Instruction::BitBitR8   (Expr::Const(2), Reg8::E),
        Instruction::BitBitR8   (Expr::Const(2), Reg8::H),
        Instruction::BitBitR8   (Expr::Const(2), Reg8::L),
        Instruction::BitBitMRhl (Expr::Const(2)),
    ));
}

#[test]
fn test_res_bit_r8() {
    let text = r#"
    res 2, a
    res 2, b
    res 2, c
    res 2, d
    res 2, e
    res 2, h
    res 2, l
    res 2, [hl]
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::ResBitR8   (Expr::Const(2), Reg8::A),
        Instruction::ResBitR8   (Expr::Const(2), Reg8::B),
        Instruction::ResBitR8   (Expr::Const(2), Reg8::C),
        Instruction::ResBitR8   (Expr::Const(2), Reg8::D),
        Instruction::ResBitR8   (Expr::Const(2), Reg8::E),
        Instruction::ResBitR8   (Expr::Const(2), Reg8::H),
        Instruction::ResBitR8   (Expr::Const(2), Reg8::L),
        Instruction::ResBitMRhl (Expr::Const(2)),
    ));
}

#[test]
fn test_set_bit_r8() {
    let text = r#"
    set 2, a
    set 2, b
    set 2, c
    set 2, d
    set 2, e
    set 2, h
    set 2, l
    set 2, [hl]
"#;
    let result: Vec<Instruction> = parse_asm(text).unwrap().into_iter().map(|x| x.unwrap()).collect();
    assert_eq!(result, vec!(
        Instruction::EmptyLine,
        Instruction::SetBitR8   (Expr::Const(2), Reg8::A),
        Instruction::SetBitR8   (Expr::Const(2), Reg8::B),
        Instruction::SetBitR8   (Expr::Const(2), Reg8::C),
        Instruction::SetBitR8   (Expr::Const(2), Reg8::D),
        Instruction::SetBitR8   (Expr::Const(2), Reg8::E),
        Instruction::SetBitR8   (Expr::Const(2), Reg8::H),
        Instruction::SetBitR8   (Expr::Const(2), Reg8::L),
        Instruction::SetBitMRhl (Expr::Const(2)),
    ));
}
