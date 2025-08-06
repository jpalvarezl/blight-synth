#![cfg(feature = "tracker")]

use std::{sync::Arc, thread, time::Duration};

use audio_backend::{BlightAudio, TrackerCommand};
use sequencer::models::{Chain, EffectType, Event, Phrase, Song, SongRow, EMPTY_CHAIN_SLOT};

pub fn main() {
    match &mut BlightAudio::new(Arc::new(load_song())) {
        Ok(audio) => {
            audio.send_command(TrackerCommand::PlaySong {
                song: Arc::new(load_song()),
            });
            thread::sleep(Duration::from_millis(20000));
        }
        Err(e) => {
            eprintln!("Failed to initialize BlightAudio: {}", e);
        }
    }
}

pub fn load_song() -> Song {
    let phrase_1 = vec![
        Event {
            instrument_id: 1,
            note: 60,
            volume: 100,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
        Event {
            instrument_id: 1,
            note: 63,
            volume: 0,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
        Event {
            instrument_id: 1,
            note: 66,
            volume: 127,
            effect: EffectType::Arpeggio,
            effect_param: 1,
        },
    ];

    let phrase_2 = vec![
        Event {
            instrument_id: 1,
            note: 66,
            volume: 100,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
        Event {
            instrument_id: 1,
            note: 64,
            volume: 0,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
        Event {
            instrument_id: 1,
            note: 60,
            volume: 127,
            effect: EffectType::Arpeggio,
            effect_param: 1,
        },
    ];

    let event_1 = Event {
        instrument_id: 0,
        note: 40,
        volume: 100,
        effect: EffectType::Arpeggio,
        effect_param: 0,
    };
    let event_2 = Event {
        instrument_id: 0,
        note: 43,
        volume: 0,
        effect: EffectType::Arpeggio,
        effect_param: 0,
    };
    let phrase_3 = (0..16)
        .map(|i| {
            if i % 2 == 0 {
                event_1.clone()
            } else {
                event_2.clone()
            }
        })
        .collect::<Vec<_>>();

    let mut song = Song::new("Test song");
    song.phrase_bank = vec![
        Phrase::from_events(phrase_1),
        Phrase::from_events(phrase_2),
        Phrase::from_events(phrase_3),
    ];
    song.chain_bank = vec![
        Chain::new([0, 0, 0, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1]),
        Chain::new([2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2]),
    ];
    song.arrangement = vec![SongRow::new([
        0,
        1,
        EMPTY_CHAIN_SLOT,
        EMPTY_CHAIN_SLOT,
        EMPTY_CHAIN_SLOT,
        EMPTY_CHAIN_SLOT,
        EMPTY_CHAIN_SLOT,
        EMPTY_CHAIN_SLOT,
    ])];

    return song;
}
