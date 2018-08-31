//! Generate audio data.
//!
//! Normally you only need to use the high level RomBuilder methods:
//! RomBuilder::add_audio_data and RomBuilder::add_audio_player.
//! So check those out first.
//!
//! The audio player that plays the generated audio can be found at:
//! [audio_player.asm](https://github.com/rukai/ggbasm/blob/master/src/audio_player.asm)

use failure::{Error, bail};
use crate::ast::{Instruction, Expr};

/// Processes `Vec<AudioLine>` into `Vec<Instruction>` that can be played by the audio player
/// Despite returning Instruction, the only variants used are Db* and Label.
pub fn generate_audio_data(lines: Vec<AudioLine>) -> Result<Vec<Instruction>, Error> {
    // Bail if a clean exit is impossible
    let mut bad_label  = None;
    let mut clean_exit = false;
    for line in &lines {
        match line {
            AudioLine::Disable       => { clean_exit = true }
            AudioLine::PlayFrom (_)  => { clean_exit = true }
            AudioLine::Label (label) => {
                clean_exit = false;
                bad_label = Some(label.clone());
            }
            _ => { }
        }
    }
    if !clean_exit {
        if let Some(bad_label) = bad_label {
            bail!("It is impossible to cleanly exit from label \"{}\". Please ensure `disable` or `playfrom song_label` is used at least once after this label.", bad_label);
        }
        else {
            bail!("Audio has no labels so there is no way to use it.");
        }
    }

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
                let rest = 0x09;

                let mut chan2 = vec!();
                chan2.push(0x16);
                chan2.push(ff16);

                chan2.push(0x17);
                chan2.push(ff17);

                chan2.push(0x18);
                chan2.push(ff18);

                chan2.push(0x19);
                chan2.push(ff19);

                // stop processing and rest
                chan2.push(0xFF);
                chan2.push(rest);
                result.push(Instruction::Db(chan2));
            }
            AudioLine::Channel3 => { }
            AudioLine::Channel4 => { }
            AudioLine::Rest (_) => { }
            AudioLine::Disable  => result.push(Instruction::Db (vec!(0xFC))),
            AudioLine::PlayFrom (label) => {
                result.push(Instruction::Db (vec!(0xFE)));
                result.push(Instruction::DbExpr16 (Expr::Ident (label)));
            }
            AudioLine::Label (label) => result.push(Instruction::Label (label)),
        }
    }

    Ok(result)
}

/// Parses `&str` into `Vec<AudioLine>`
/// Returns `Err` if the text does not conform to the audio text format.
///
/// Documentation on the input format is given for RomBuilder::add_audio_data.
/// Each AudioLine cooresponds to a line in the input file. Empty lines are skipped.
pub fn parse_audio_text(text: &str) -> Result<Vec<AudioLine>, Error> {
    let mut result = vec!();
    for line in text.lines() {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.len() == 0 {
            continue;
        }
        if tokens[0].to_lowercase() == "rest" {
            if let Some(value) = tokens.get(1) {
                if let Ok(value) = value.parse() {
                    result.push(AudioLine::Rest (value));
                } else {
                    bail!("rest instruction argument is not an integer");
                }
            } else {
                bail!("rest instruction needs an argument");
            }
        } else if tokens[0].to_lowercase() == "playfrom" {
            if tokens.len() == 2 {
                result.push(AudioLine::PlayFrom (tokens[1].to_string()));
            } else {
                bail!("Expected 1 argument for playfrom, however there is {} arguments", tokens.len());
            }
        } else if tokens[0].to_lowercase() == "label" {
            if tokens.len() == 2 {
                result.push(AudioLine::Label (tokens[1].to_string()));
            } else {
                bail!("Expected 1 argument for label, however there is {} arguments", tokens.len());
            }
        } else if tokens[0].to_lowercase() == "disable" {
            result.push(AudioLine::Disable);
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

/// Represents a line from the audio file
pub enum AudioLine {
    Channel1, // TODO: Combine into SetRegisters(Option<Channel1State>, Option<Channel2State>, Option<Channel3State>, Option<Channel4State>),
    Channel2 (Channel2State),
    Channel3,
    Channel4,
    Label (String),
    PlayFrom (String),
    Rest (u8),
    Disable,
}

/// Represents a Note to be played by a channel
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

/// Represents the state of channel 2
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

/// Converts an octave, note and sharp into the 16 bit value the gameboy uses for frequency.
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
