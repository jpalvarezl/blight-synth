use std::{sync::Arc, thread, time::Duration};

use audio_backend::{
    id::{EffectId, InstrumentId},
    BlightAudio,
};
use sequencer::models::{
    Chain, EffectType, Event, NoteSentinelValues, Phrase, Song, SongRow, EMPTY_CHAIN_SLOT,
};

pub fn main() {
    env_logger::init();

    let lead_instrument_id = InstrumentId::from_raw(1);
    let reverb_id = EffectId::from_raw(0);
    match &mut BlightAudio::with_song(Arc::new(load_song(lead_instrument_id))) {
        Ok(audio) => {
            let max_voices = 5;
            let _ = audio.send_command(
                audio_backend::InstrumentCmd::AddInstrument {
                    instrument: audio.get_instrument_factory().create_polyphonic_oscillator(
                        lead_instrument_id,
                        0.0,
                        max_voices,
                    ),
                }
                .into(),
            );

            let reverbs =
                (0..max_voices).map(|_| audio.get_effect_factory().create_mono_reverb(reverb_id));
            let _ = audio.send_command(
                audio_backend::InstrumentCmd::AddVoiceEffects {
                    instrument_id: lead_instrument_id,
                    effects: reverbs.collect(),
                }
                .into(),
            );
            let _ = audio.send_command(audio_backend::TransportCmd::PlayLastSong.into());
            thread::sleep(Duration::from_millis(5000));
        }
        Err(e) => {
            eprintln!("Failed to initialize BlightAudio: {}", e);
        }
    }
}

pub fn load_song(lead_instrument_id: InstrumentId) -> Song {
    let project_instrument_id =
        u8::try_from(lead_instrument_id.raw()).expect("example instrument ID fits project u8");
    let phrase_1 = vec![
        Event {
            note: 60,
            volume: 100,
            instrument_id: project_instrument_id,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
        Event {
            note: 63,
            volume: 0,
            instrument_id: project_instrument_id,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
        Event {
            note: 66,
            volume: 127,
            instrument_id: project_instrument_id,
            effect: EffectType::Arpeggio,
            effect_param: 1,
        },
        Event {
            note: 69,
            volume: 100,
            instrument_id: project_instrument_id,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
        Event {
            note: 72,
            volume: 0,
            instrument_id: project_instrument_id,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
    ];

    let phrase_2 = vec![Event {
        note: NoteSentinelValues::NoteOff as u8,
        volume: 100,
        instrument_id: project_instrument_id,
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

    song
}
