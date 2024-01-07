use note::{Note, NoteInner};
use std::collections::HashMap;

pub mod note;
pub mod scales;

pub fn load_notes() -> HashMap<String, Note> {
    let mut notes = HashMap::new();
    let notes_json = include_str!("../../assets/notes.json");
    let notes_as_vec: Vec<NoteInner> =
        serde_json::from_str(notes_json).expect("Could not load music notes data");

    for note in notes_as_vec {
        notes.insert(note.note_label.clone(), note.into());
    }

    return notes;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_notes() {
        let notes = load_notes();
        assert_eq!(notes.len(), 153);
    }

    #[test]
    fn get_notes_for_cromatic_scale() {
        let notes = load_notes();
        let cromatic_scale = scales::get_notes_for_cromatic_scale_in_octave(4);
        let mut cromatic_scale_notes = Vec::new();
        for note in cromatic_scale {
            cromatic_scale_notes.push(notes.get(&note).unwrap().clone());
        }
        assert_eq!(cromatic_scale_notes[0].note_label, "C4");
        assert_eq!(cromatic_scale_notes[1].note_label, "C#4");
        assert_eq!(cromatic_scale_notes[2].note_label, "D4");
        assert_eq!(cromatic_scale_notes[3].note_label, "D#4");
        assert_eq!(cromatic_scale_notes[4].note_label, "E4");
        assert_eq!(cromatic_scale_notes[5].note_label, "F4");
        assert_eq!(cromatic_scale_notes[6].note_label, "F#4");
        assert_eq!(cromatic_scale_notes[7].note_label, "G4");
        assert_eq!(cromatic_scale_notes[8].note_label, "G#4");
        assert_eq!(cromatic_scale_notes[9].note_label, "A4");
        assert_eq!(cromatic_scale_notes[10].note_label, "A#4");
        assert_eq!(cromatic_scale_notes[11].note_label, "B4");
    }
}
