// use sequencer::models::*;
use sequencer::cli::CliArgs;
use sequencer::models::{EMPTY_PHRASE_SLOT, EffectType, Event, Phrase, Song, SongRow};

fn main() {
    let args = CliArgs::parse_arguments();

    let phrase = vec![
        Event {
            note: 60,
            volume: 100,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
        Event {
            note: 60,
            volume: 0,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
        Event {
            note: 60,
            volume: 127,
            effect: EffectType::Arpeggio,
            effect_param: 1,
        },
    ];

    let mut song = Song::new("Test song");
    song.phrase_bank = vec![Phrase::from_events(phrase)];
    song.arrangement = vec![SongRow::new([
        0,
        1,
        2,
        EMPTY_PHRASE_SLOT,
        EMPTY_PHRASE_SLOT,
        EMPTY_PHRASE_SLOT,
        EMPTY_PHRASE_SLOT,
        EMPTY_PHRASE_SLOT,
    ])];

    args.write_file(&song).expect("Failed to write file");
}
