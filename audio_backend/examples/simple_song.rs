use std::{sync::Arc, thread, time::Duration};

use audio_backend::{id::InstrumentId, instruments::Waveform, BlightAudio};
use sequencer::models::{
    Chain, EffectType, Event, NoteSentinelValues, Phrase, Song, SongRow, EMPTY_CHAIN_SLOT,
};

pub fn main() {
    let lead_instrument_id = InstrumentId::from_raw(0);
    let bass_instrument_id = InstrumentId::from_raw(1);
    match &mut BlightAudio::with_song(Arc::new(load_song(lead_instrument_id, bass_instrument_id))) {
        Ok(audio) => {
            let _ = audio.send_command(
                audio_backend::InstrumentCmd::AddInstrument {
                    instrument: audio
                        .get_instrument_factory()
                        .create_simple_oscillator(lead_instrument_id, 0.0),
                }
                .into(),
            );

            let _ = audio.send_command(
                audio_backend::InstrumentCmd::AddInstrument {
                    instrument: audio
                        .get_instrument_factory()
                        .create_oscillator_with_waveform(
                            bass_instrument_id,
                            0.0,
                            Waveform::NesTriangle,
                        ),
                }
                .into(),
            );
            let _ = audio.send_command(audio_backend::TransportCmd::PlayLastSong.into());
            thread::sleep(Duration::from_millis(20000));
        }
        Err(e) => {
            eprintln!("Failed to initialize BlightAudio: {}", e);
        }
    }
}

pub fn load_song(lead_instrument_id: InstrumentId, bass_instrument_id: InstrumentId) -> Song {
    let project_lead_id =
        u8::try_from(lead_instrument_id.raw()).expect("example lead ID fits project u8");
    let project_bass_id =
        u8::try_from(bass_instrument_id.raw()).expect("example bass ID fits project u8");
    let phrase_1 = vec![
        Event {
            note: 60,
            volume: 100,
            instrument_id: project_lead_id,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
        Event {
            note: 63,
            volume: 0,
            instrument_id: project_lead_id,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
        Event {
            note: 66,
            volume: 127,
            instrument_id: project_lead_id,
            effect: EffectType::Arpeggio,
            effect_param: 1,
        },
        Event {
            note: NoteSentinelValues::NoteOff as u8,
            volume: 0,
            instrument_id: project_lead_id,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
    ];

    let phrase_2 = vec![
        Event {
            note: 66,
            volume: 100,
            instrument_id: project_lead_id,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
        Event {
            note: 64,
            volume: 0,
            instrument_id: project_lead_id,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
        Event {
            note: 60,
            volume: 127,
            instrument_id: project_lead_id,
            effect: EffectType::Arpeggio,
            effect_param: 1,
        },
        Event {
            note: NoteSentinelValues::NoteOff as u8,
            volume: 0,
            instrument_id: project_lead_id,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
    ];

    let event_1 = Event {
        note: 40,
        volume: 100,
        instrument_id: project_bass_id,
        effect: EffectType::Arpeggio,
        effect_param: 0,
    };
    let event_2 = Event {
        note: 43,
        volume: 100,
        instrument_id: project_bass_id,
        effect: EffectType::Arpeggio,
        effect_param: 0,
    };
    let phrase_3 = (0..16)
        .map(|i| if i % 2 == 0 { event_1 } else { event_2 })
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

    song
}
