#![cfg(feature = "tracker")]

use std::{sync::Arc, thread, time::Duration};

use audio_backend::{id::InstrumentId, BlightAudio, TrackerCommand};
use sequencer::models::{
    Chain, EffectType, Event, NoteSentinelValues, Phrase, Song, SongRow, EMPTY_CHAIN_SLOT,
};

pub fn main() {
    env_logger::init();

    let lead_instrument_id: InstrumentId = 1;
    match &mut BlightAudio::new(Arc::new(load_song(lead_instrument_id))) {
        Ok(audio) => {
            audio.send_command(TrackerCommand::AddTrackInstrument {
                instrument_id: lead_instrument_id,
                instrument: audio.get_instrument_factory().create_polyphonic_oscillator(
                    lead_instrument_id,
                    0.0,
                    5,
                ),
            });
            audio.send_command(TrackerCommand::PlayLastSong);
            thread::sleep(Duration::from_millis(5000));
        }
        Err(e) => {
            eprintln!("Failed to initialize BlightAudio: {}", e);
        }
    }
}

pub fn load_song(lead_instrument_id: InstrumentId) -> Song {
    let phrase_1 = vec![
        Event {
            note: 60,
            volume: 100,
            instrument_id: lead_instrument_id as u8,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
        Event {
            note: 63,
            volume: 0,
            instrument_id: lead_instrument_id as u8,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
        Event {
            note: 66,
            volume: 127,
            instrument_id: lead_instrument_id as u8,
            effect: EffectType::Arpeggio,
            effect_param: 1,
        },
        Event {
            note: 69,
            volume: 100,
            instrument_id: lead_instrument_id as u8,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
        Event {
            note: 72,
            volume: 0,
            instrument_id: lead_instrument_id as u8,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
    ];

    let phrase_2 = vec![Event {
        note: NoteSentinelValues::NoteOff as u8,
        volume: 100,
        instrument_id: lead_instrument_id as u8,
        effect: EffectType::Arpeggio,
        effect_param: 0,
    }];

    let mut song = Song::new("Test song");
    song.phrase_bank = vec![Phrase::from_events(phrase_1), Phrase::from_events(phrase_2)];
    song.chain_bank = vec![Chain::from_phrases([0, 1])];
    song.arrangement = vec![SongRow::new([
        0,
        EMPTY_CHAIN_SLOT,
        EMPTY_CHAIN_SLOT,
        EMPTY_CHAIN_SLOT,
        EMPTY_CHAIN_SLOT,
        EMPTY_CHAIN_SLOT,
        EMPTY_CHAIN_SLOT,
        EMPTY_CHAIN_SLOT,
    ])];
    song.initial_bpm = 120;
    song.initial_speed = 8;

    return song;
}
