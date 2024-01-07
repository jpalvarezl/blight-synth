use serde::Deserialize;

// pub struct Note {
//     pub pitch: Pitch,
//     pub accidental: Accidental,
//     pub octave: u8,
//     pub frequency: f32,
//     pub note_label: String,
// }

// TODO make this type only used for deserialization and exposed public type with better types
// i.e.: frequency should be a float, not a string
#[derive(Debug, PartialEq, Clone, Eq, Hash, Deserialize)]
pub struct Note {
    pub pitch: Pitch,
    pub accidental: Accidental,
    pub octave: u8,
    pub frequency: String, // Hash can't be derived for float types. Use String instead?
    pub note_label: String,
}

#[derive(Debug, PartialEq, Clone, Eq ,Hash, Deserialize)]
pub enum Accidental {
    Sharp,
    Flat,
    Natural,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash, Deserialize)]
pub enum Pitch {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

// TODO remove
impl Note {
    pub fn new() -> Self {
        Self {
            pitch: Pitch::C,
            accidental: Accidental::Natural,
            octave: 4,
            frequency: "440".to_string(),
            note_label: "C4".to_string(),
        }
    }
}

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.pitch, self.accidental, self.octave)
    }
}

impl std::fmt::Display for Accidental {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Accidental::Sharp => write!(f, "#"),
            Accidental::Flat => write!(f, "b"),
            Accidental::Natural => write!(f, ""),
        }
    }
}

impl std::fmt::Display for Pitch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pitch::A => write!(f, "A"),
            Pitch::B => write!(f, "B"),
            Pitch::C => write!(f, "C"),
            Pitch::D => write!(f, "D"),
            Pitch::E => write!(f, "E"),
            Pitch::F => write!(f, "F"),
            Pitch::G => write!(f, "G"),
        }
    }
}
