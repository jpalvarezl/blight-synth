use note::{Note, NoteInner};
use std::collections::HashMap;

pub mod note;

pub fn load_notes() -> HashMap<String, Note> {
    let mut notes = HashMap::new();
    let notes_json = include_str!("../../assets/notes.json");
    let notes_as_vec: Vec<NoteInner> = serde_json::from_str(notes_json).unwrap();

    for note in notes_as_vec {
        notes.insert(note.note_label.clone(), note.into());
    }

    print!("{}", notes_json);
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
}
