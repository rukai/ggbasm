pub enum ColorSupport {
    Unsupported,
    SupportedBackwardsCompatible,
    SupportedNotBackwardsCompatible,
}

impl ColorSupport {
    pub fn byte(&self) -> u8 {
        match self {
            ColorSupport::Unsupported                     => 0x00,
            ColorSupport::SupportedBackwardsCompatible    => 0x80,
            ColorSupport::SupportedNotBackwardsCompatible => 0xC0,
        }
    }

    pub fn is_supported(&self) -> bool {
        match self {
            ColorSupport::Unsupported                     => false,
            ColorSupport::SupportedBackwardsCompatible    => true,
            ColorSupport::SupportedNotBackwardsCompatible => true,
        }
    }
}

pub enum CartridgeType {
    RomOnly,
    Mbc1,
    Mbc1Ram,
    Mbc1RamBattery,
    Mbc2,
    Mbc2Battery,
    RomRam,
    RomRamBattery,
    Mmm01,
    Mmm01Ram,
    Mmm01RamBattery,
    Mbc3TimerBattery,
    Mbc3TimerRamBattery,
    Mbc3,
    Mbc3Ram,
    Mbc3RamBattery,
    Mbc5,
    Mbc5Ram,
    Mbc5RamBattery,
    Mbc5Rumble,
    Mbc5RumbleRam,
    Mbc5RumbleRamBattery,
    PocketCamera,
    HuC3,
    HuC1RamBattery,
    Unknown (u8)
}

impl CartridgeType {
    pub fn byte(&self) -> u8 {
        match self {
            CartridgeType::RomOnly              => 0x00,
            CartridgeType::Mbc1                 => 0x01,
            CartridgeType::Mbc1Ram              => 0x02,
            CartridgeType::Mbc1RamBattery       => 0x03,
            CartridgeType::Mbc2                 => 0x05,
            CartridgeType::Mbc2Battery          => 0x06,
            CartridgeType::RomRam               => 0x08,
            CartridgeType::RomRamBattery        => 0x09,
            CartridgeType::Mmm01                => 0x0B,
            CartridgeType::Mmm01Ram             => 0x0C,
            CartridgeType::Mmm01RamBattery      => 0x0D,
            CartridgeType::Mbc3TimerBattery     => 0x0F,
            CartridgeType::Mbc3TimerRamBattery  => 0x10,
            CartridgeType::Mbc3                 => 0x11,
            CartridgeType::Mbc3Ram              => 0x12,
            CartridgeType::Mbc3RamBattery       => 0x13,
            CartridgeType::Mbc5                 => 0x19,
            CartridgeType::Mbc5Ram              => 0x1A,
            CartridgeType::Mbc5RamBattery       => 0x1B,
            CartridgeType::Mbc5Rumble           => 0x1C,
            CartridgeType::Mbc5RumbleRam        => 0x1D,
            CartridgeType::Mbc5RumbleRamBattery => 0x1E,
            CartridgeType::PocketCamera         => 0xFC,
            CartridgeType::HuC3                 => 0xFE,
            CartridgeType::HuC1RamBattery       => 0xFF,
            CartridgeType::Unknown (value)      => *value,
        }
    }

    pub fn variant(value: u8) -> CartridgeType {
        match value {
            0x00 => CartridgeType::RomOnly,
            0x01 => CartridgeType::Mbc1,
            0x02 => CartridgeType::Mbc1Ram,
            0x03 => CartridgeType::Mbc1RamBattery,
            0x05 => CartridgeType::Mbc2,
            0x06 => CartridgeType::Mbc2Battery,
            0x08 => CartridgeType::RomRam,
            0x09 => CartridgeType::RomRamBattery,
            0x0B => CartridgeType::Mmm01,
            0x0C => CartridgeType::Mmm01Ram,
            0x0D => CartridgeType::Mmm01RamBattery,
            0x0F => CartridgeType::Mbc3TimerBattery,
            0x10 => CartridgeType::Mbc3TimerRamBattery,
            0x11 => CartridgeType::Mbc3,
            0x12 => CartridgeType::Mbc3Ram,
            0x13 => CartridgeType::Mbc3RamBattery,
            0x19 => CartridgeType::Mbc5,
            0x1A => CartridgeType::Mbc5Ram,
            0x1B => CartridgeType::Mbc5RamBattery,
            0x1C => CartridgeType::Mbc5Rumble,
            0x1D => CartridgeType::Mbc5RumbleRam,
            0x1E => CartridgeType::Mbc5RumbleRamBattery,
            0xFC => CartridgeType::PocketCamera,
            0xFE => CartridgeType::HuC3,
            0xFF => CartridgeType::HuC1RamBattery,
            a    => CartridgeType::Unknown (a)
        }
    }
}

pub enum RamType {
    None,
    Mbc2,
    Some2KB,
    Some8KB,
    Some32KB,
}

impl RamType {
    pub fn byte(&self) -> u8 {
        match self {
            RamType::None     => 0,
            RamType::Mbc2     => 0,
            RamType::Some2KB  => 1,
            RamType::Some8KB  => 2,
            RamType::Some32KB => 3,
        }
    }
}

pub struct Header {
    /// 11 bytes
    pub title:          String,
    pub color_support:  ColorSupport,
    /// 2 bytes
    pub licence:        String,
    pub sgb_support:    bool,
    pub cartridge_type: CartridgeType,
    pub ram_type:       RamType,
    pub japanese:       bool,
    pub version_number: u8,
}

impl Header {
    pub fn write(&self, rom: &mut Vec<u8>, rom_size_factor: u8) {
        rom.extend(LOGO.iter());
        let title = self.title.as_bytes();
        rom.extend(title);
        if title.len() < 0x10 {
            for _ in 0..0xF - title.len() {
                rom.push(0x00);
            }
            rom.push(self.color_support.byte());
        }

        rom.extend(self.licence.as_bytes());
        for _ in 0..0x2 - self.licence.as_bytes().len() {
            rom.push(0x00);
        }
        rom.push(if self.sgb_support { 0x03 } else { 0x00 });
        rom.push(self.cartridge_type.byte());
        rom.push(rom_size_factor);
        rom.push(self.ram_type.byte());
        rom.push(if self.japanese { 0x00 } else { 0x01 });
        rom.push(0x33); // we are using the new licence, so set old licence accordingly
        rom.push(self.version_number);

        let mut checksum: u8 = 0;
        for byte in &rom[0x0134..0x014D] {
            checksum = checksum.wrapping_sub(*byte);
            checksum = checksum.wrapping_sub(1);
        }
        rom.push(checksum);

        // Global checksum, gameboy doesnt care about these
        rom.push(0x00);
        rom.push(0x00);
    }
}

static LOGO: [u8; 0x30] = [0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00,
                           0x83, 0x00, 0x0C, 0x00, 0x0D, 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89,
                           0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99, 0xBB,
                           0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F,
                           0xBB, 0xB9, 0x33, 0x3E];
