pub struct Note {
    pitch: Pitch,
    accidental: Accidental,
    octave: u8,
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
