#[derive(PartialEq, Debug)]
pub enum Instruction {
    EmptyLine, // Keeping track of empty lines makes it easier to refer errors back to a line number
    AdvanceAddress (u32),
    Label (String),
    Db (Vec<u8>),
    Nop,
    Stop,
    Halt,
    Di,
    Ei,
}

impl Instruction {
    pub fn bytes(&self) -> Vec<u8> {
        match self {
            Instruction::AdvanceAddress (_) => vec!(),
            Instruction::EmptyLine  => vec!(),
            Instruction::Label (_)  => vec!(),
            Instruction::Db (bytes) => bytes.clone(),
            Instruction::Nop        => vec!(0x00),
            Instruction::Stop       => vec!(0x10),
            Instruction::Halt       => vec!(0x76),
            Instruction::Di         => vec!(0xF3),
            Instruction::Ei         => vec!(0xFB),
        }
    }
}
