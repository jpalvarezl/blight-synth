#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Note {
    pub pitch: Pitch,
    pub accidental: Accidental,
    pub octave: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Accidental {
    Sharp,
    Flat,
    Natural,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pitch {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

impl Note {
    pub fn new() -> Self {
        Self {
            pitch: Pitch::C,
            accidental: Accidental::Natural,
            octave: 4,
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
