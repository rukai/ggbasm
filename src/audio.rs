//! Generate audio data

use failure::{Error, bail};

/// ## Channel 1 format:
///
/// TODO
///
/// ## Channel 2 format:
///
/// *   AB:C:DD:E:FG:HI
///
/// Key:
///
/// *   A:  Note                    A-G (natural), a-g (sharp)
/// *   B:  Octave                  1-8
/// *   C:  Duty                    0-3
/// *   DD: length                  0-3F
/// *   E:  envelope initial volume 0-F
/// *   F:  envelope argument       0-7
/// *   G:  envelope increase       Y/N
/// *   H:  enable length           Y/N
/// *   I:  initial                 Y/N
///
/// For example:
///
/// *   2 D6:2:10:7:4Y:NY
///
/// ## Channel 3 format:
///
/// TODO
///
/// ## Channel 4 format:
///
/// TODO
///
/// Control lines
/// *   wait AA - wait AA frames before continuing
/// *   start   - force start the song at this point, used for quick testing
///
///
///
/// TODO: Stack channels side by side like this:
/// CHANNEL1         CHANNEL2         CHANNEL3      CHANNEL4
/// D6:2:10:7:4Y:NY  D6:2:10:7:4Y:NY  ...           ...
///                  D6:2:10:7:4Y:NY
///                  D6:2:10:7:4Y:NY
/// D6:2:10:7:4Y:NY                 
///                  D6:2:10:7:4Y:NY
/// D6:2:10:7:4Y:NY  D6:2:10:7:4Y:NY
///
/// TODO: Maybe syntax highlighting could help make this more readable
///
/// TODO: Provide a way to insert the music player instructions
pub fn generate_sound(lines: Vec<AudioLine>) -> Vec<u8> {
    // Each entry in the table has 8 bytes:
    //
    // Byte 1 - Length and Duty: FF16
    // Byte 2 - Envelope:        FF17
    // Byte 3 - Frequency Low:   FF18
    // Byte 4 - Frequency High:  FF19
    // Byte 5 - frame delay until play next entry
    // Byte 6 - ?!?!?!?
    // Byte 7 - ?!?!?!?
    // Byte 8 - ?!?!?!?
    let mut result = vec!();
    for line in lines {
        match line {
            AudioLine::Channel1 => { }
            AudioLine::Channel2 (state) => {
                let frequency = note_to_frequency(state.octave, &state.note, state.sharp);
                let length = 0x3f - state.length; // make length start at 0 and higher values mean longer length.

                let ff16 = ((state.duty << 6 & 0b11000000))
                           | length          & 0b00111111;
                let ff17 = (state.envelope_initial_volume << 4)
                    | (if state.envelope_increase { 1 } else { 0 } << 3)
                    | (state.envelope_argument & 0b00000111);
                let ff18 = (frequency & 0xFF) as u8;
                let ff19 = ((frequency >> 8) as u8 & 0b00000111)
                    | if state.enable_length { 1 } else { 0 } << 6
                    | if state.initial       { 1 } else { 0 } << 7;
                let delay = 0x09;

                result.push(ff16);
                result.push(ff17);
                result.push(ff18);
                result.push(ff19);
                result.push(delay);
                result.push(0x00);
                result.push(0x00);
                result.push(0x00);
            }
            AudioLine::Channel3 => { }
            AudioLine::Channel4 => { }
            AudioLine::Wait (_) => { }
            AudioLine::Start => { }
        }
    }
    result
}

pub fn parse_audio_file(text: &str) -> Result<Vec<AudioLine>, Error> {
    let mut result = vec!();
    for line in text.lines() {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.len() == 0 {
            continue;
        }
        if tokens[0].to_lowercase() == "wait" {
            if let Some(value) = tokens.get(1) {
                if let Ok(value) = value.parse() {
                    result.push(AudioLine::Wait (value));
                } else {
                    bail!("wait instruction argument is not an integer");
                }
            } else {
                bail!("wait instruction needs an argument");
            }
        } else if tokens[0].to_lowercase() == "start" {
            result.push(AudioLine::Start);
        }
        else {
            let line: Vec<char> = line.chars().collect();
            // TODO
            //if line.len() < 32 {
            //    bail!("Line is too short for channel");
            //}

            let note = match line[0] {
                'a' | 'A' => Note::A,
                'b' | 'B' => Note::B,
                'c' | 'C' => Note::C,
                'd' | 'D' => Note::D,
                'e' | 'E' => Note::E,
                'f' | 'F' => Note::F,
                'g' | 'G' => Note::G,
                _   => bail!("Invalid character for note"),
            };

            let sharp = line[0].is_lowercase();

            let octave = match line[1].to_string().parse() {
                Ok(value) => value,
                Err(_)    => bail!("Invalid character for octave"),
            };

            let duty = match line[3].to_string().parse() {
                Ok(value) => value,
                Err(_)    => bail!("Invalid character for duty"),
            };

            let length = match u8::from_str_radix(line[5..7].iter().cloned().collect::<String>().as_ref(), 16) {
                Ok(value) => value,
                Err(_)    => bail!("Invalid character for length"),
            };

            let envelope_initial_volume = match u8::from_str_radix(line[8].to_string().as_ref(), 16) {
                Ok(value) => value,
                Err(_)    => bail!("Invalid character for envelope initial volume"),
            };

            let envelope_argument = match line[10].to_string().parse() {
                Ok(value) => value,
                Err(_)    => bail!("Invalid character for envelope argument"),
            };

            let envelope_increase = match line[11] {
                'Y' => true,
                'N' => false,
                _   => bail!("Invalid character for envelope increase"),
            };

            let enable_length = match line[13] {
                'Y' => true,
                'N' => false,
                _   => bail!("Invalid character for enable length"),
            };

            let initial = match line[14] {
                'Y' => true,
                'N' => false,
                _   => bail!("Invalid character for initial"),
            };

            result.push(AudioLine::Channel2 (Channel2State {
                note, sharp, octave, duty, length,
                envelope_initial_volume,
                envelope_argument,
                envelope_increase,
                enable_length,
                initial,
            }));
        }
    }
    Ok(result)
}

pub enum AudioLine {
    Channel1,
    Channel2 (Channel2State),
    Channel3,
    Channel4,
    Wait (u8),
    Start,
}

pub enum Note {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

impl Note {
    pub fn to_string(&self) -> String {
        match self {
            Note::A => String::from("A"),
            Note::B => String::from("B"),
            Note::C => String::from("C"),
            Note::D => String::from("D"),
            Note::E => String::from("E"),
            Note::F => String::from("F"),
            Note::G => String::from("G"),
        }
    }
}

pub struct Channel2State {
    pub note:                    Note,
    pub sharp:                   bool,
    pub octave:                  u8,
    pub duty:                    u8,
    pub length:                  u8,
    pub envelope_initial_volume: u8,
    pub envelope_argument:       u8,
    pub envelope_increase:       bool,
    pub enable_length:           bool,
    pub initial:                 bool,
}

fn note_to_frequency(octave: u8, note: &Note, sharp: bool) -> u16 {
    match (octave, note, sharp) {
        (3, Note::C, false)  => 44,
        (3, Note::C, true)   => 156,
        (3, Note::D, false)  => 262,
        (3, Note::D, true)   => 363,
        (3, Note::E, false)  => 457,
        (3, Note::F, false)  => 547,
        (3, Note::F, true)   => 631,
        (3, Note::G, false)  => 710,
        (3, Note::G, true)   => 786,
        (3, Note::A, false)  => 854,
        (3, Note::A, true)   => 923,
        (3, Note::B, false)  => 986,
        (4, Note::C, false)  => 1046,
        (4, Note::C, true)   => 1102,
        (4, Note::D, false)  => 1155,
        (4, Note::D, true)   => 1205,
        (4, Note::E, false)  => 1253,
        (4, Note::F, false)  => 1297,
        (4, Note::F, true)   => 1339,
        (4, Note::G, false)  => 1379,
        (4, Note::G, true)   => 1417,
        (4, Note::A, false)  => 1452,
        (4, Note::A, true)   => 1486,
        (4, Note::B, false)  => 1517,
        (5, Note::C, false)  => 1546,
        (5, Note::C, true)   => 1575,
        (5, Note::D, false)  => 1602,
        (5, Note::D, true)   => 1627,
        (5, Note::E, false)  => 1650,
        (5, Note::F, false)  => 1673,
        (5, Note::F, true)   => 1694,
        (5, Note::G, false)  => 1714,
        (5, Note::G, true)   => 1732,
        (5, Note::A, false)  => 1750,
        (5, Note::A, true)   => 1767,
        (5, Note::B, false)  => 1783,
        (6, Note::C, false)  => 1798,
        (6, Note::C, true)   => 1812,
        (6, Note::D, false)  => 1825,
        (6, Note::D, true)   => 1837,
        (6, Note::E, false)  => 1849,
        (6, Note::F, false)  => 1860,
        (6, Note::F, true)   => 1871,
        (6, Note::G, false)  => 1881,
        (6, Note::G, true)   => 1890,
        (6, Note::A, false)  => 1899,
        (6, Note::A, true)   => 1907,
        (6, Note::B, false)  => 1915,
        (7, Note::C, false)  => 1923,
        (7, Note::C, true)   => 1930,
        (7, Note::D, false)  => 1936,
        (7, Note::D, true)   => 1943,
        (7, Note::E, false)  => 1949,
        (7, Note::F, false)  => 1954,
        (7, Note::F, true)   => 1959,
        (7, Note::G, false)  => 1964,
        (7, Note::G, true)   => 1969,
        (7, Note::A, false)  => 1974,
        (7, Note::A, true)   => 1978,
        (7, Note::B, false)  => 1982,
        (8, Note::C, false)  => 1985,
        (8, Note::C, true)   => 1988,
        (8, Note::D, false)  => 1992,
        (8, Note::D, true)   => 1995,
        (8, Note::E, false)  => 1998,
        (8, Note::F, false)  => 2001,
        (8, Note::F, true)   => 2004,
        (8, Note::G, false)  => 2006,
        (8, Note::G, true)   => 2009,
        (8, Note::A, false)  => 2011,
        (8, Note::A, true)   => 2013,
        (8, Note::B, false)  => 2015,
        (octave, note, false) => panic!("Invalid note: {}{}", octave, note.to_string().to_uppercase()),
        (octave, note, true)  => panic!("Invalid note: {}{}", octave, note.to_string().to_lowercase())
    }
}
