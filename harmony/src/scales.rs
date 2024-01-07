const CROMATIC: &'static [&'static str] = &[
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

pub fn get_notes_for_cromatic_scale_in_octave(octave: u8) -> Vec<String> {
    let mut notes = Vec::new();
    for note in CROMATIC {
        notes.push(format!("{}{}", note, octave));
    }
    return notes;
}
