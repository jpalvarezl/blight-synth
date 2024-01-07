use std::collections::HashMap;
use note::Note;

pub mod note;

pub fn load_notes() -> HashMap<Note, f64> {
    let mut notes = HashMap::new();

    let notes_json = include_str!("../../assets/notes.json");

    print!("{}", notes_json);
    return notes;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_notes() {
        let notes = load_notes();
        assert_eq!(notes.len(), 12);
    }
}
