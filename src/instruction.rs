pub enum Instruction {
    Label (String),
    Nop
}

impl Instruction {
    pub fn bytes(&self) -> Vec<u8> {
        let mut bytes = vec!();
        bytes
    }
}
