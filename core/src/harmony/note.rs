pub struct Note {
    pub pitch: Pitch,
    pub accidental: Accidental,
    pub octave: u8,
}

pub enum Accidental {
    Sharp,
    Flat,
    Natural,
}

pub enum Pitch {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}
