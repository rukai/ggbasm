pub enum Instruction {
    Label (String),
    Nop,
    Stop,
}

impl Instruction {
    pub fn bytes(&self) -> Vec<u8> {
        vec!()
    }
}

//impl Instruction {
//    pub fn bytes(&self) -> Option<InstructionBytes> {
//        match self {
//            Label => None,
//            Nop   => InstructionBytes::new(0x00),
//            Stop  => InstructionBytes::new(0x01)
//        }
//    }
//}
//
//pub struct InstructionBytes {
//    byte1: u8,
//    byte2: Option<u8>,
//}
//
//impl InstructionBytes {
//    pub fn new(byte1: u8) -> InstructionBytes {
//        InstructionBytes {
//            byte1,
//            byte2: None
//        }
//    }
//
//    pub fn new2(byte1: u8, byte2: u8) -> InstructionBytes {
//        InstructionBytes {
//            byte1,
//            byte2: Some(byte)
//        }
//    }
//}
