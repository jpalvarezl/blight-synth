use serde::Deserialize;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Note {
    pub pitch: Pitch,
    pub accidental: Accidental,
    pub octave: u8,
    pub frequency: String,
    pub note_label: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct NoteInner {
    pub pitch: Pitch,
    pub accidental: Accidental,
    pub octave: u8,
    pub frequency: f32,
    pub note_label: String,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash, Deserialize)]
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

/// Convert a MIDI note number to a note label string, e.g. 60 -> "C4"
pub fn midi_to_note_label(midi: u8) -> Option<String> {
    if midi < 12 || midi > 127 {
        return None;
    }
    let chromatic = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = (midi / 12) as i8 - 1;
    let note_index = (midi % 12) as usize;
    Some(format!("{}{}", chromatic[note_index], octave))
}

/// Convert a MIDI note number to a frequency in Hz (A440 standard)
pub fn midi_to_frequency(midi: u8) -> f32 {
    440.0 * 2f32.powf((midi as f32 - 69.0) / 12.0)
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

impl From<NoteInner> for Note {
    fn from(note: NoteInner) -> Self {
        Self {
            pitch: note.pitch,
            accidental: note.accidental,
            octave: note.octave,
            frequency: note.frequency.to_string(),
            note_label: note.note_label,
        }
    }
}
