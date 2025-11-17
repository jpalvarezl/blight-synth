use std::thread;

use audio_backend::{BlightAudio, InstrumentCmd, SequencerCmd, TransportCmd};

pub fn main() {
    match &mut BlightAudio::new() {
        Ok(audio) => {
            println!("BlightAudio initialized successfully!");
            let instrument_id = 0;
            audio.send_command(
                SequencerCmd::AddTrackInstrument {
                    instrument: audio
                        .get_instrument_factory()
                        .create_dfam(instrument_id, 0.0),
                }
                .into(),
            );
            audio.send_command(TransportCmd::PlayLastSong.into());
            audio.send_command(
                InstrumentCmd::NoteOn {
                    instrument_id,
                    note: 60,
                    velocity: 127,
                }
                .into(),
            );
            audio.send_command(
                SequencerCmd::AddEffectToInstrument {
                    instrument_id,
                    effect: audio.get_effect_factory().create_mono_gain(1.0),
                }
                .into(),
            );

            // Play a very short note - 200ms
            thread::sleep(std::time::Duration::from_millis(200));
            audio.send_command(InstrumentCmd::NoteOff { instrument_id }.into());

            // Wait to hear the release decay
            thread::sleep(std::time::Duration::from_millis(1000));
            audio.send_command(
                SequencerCmd::AddEffectToInstrument {
                    instrument_id,
                    effect: audio.get_effect_factory().create_mono_gain(0.1),
                }
                .into(),
            );
            audio.send_command(
                InstrumentCmd::NoteOn {
                    instrument_id,
                    note: 60,
                    velocity: 100,
                }
                .into(),
            );

            thread::sleep(std::time::Duration::from_millis(500));
            audio.send_command(
                SequencerCmd::AddEffectToInstrument {
                    instrument_id,
                    effect: audio.get_effect_factory().create_mono_gain(0.5),
                }
                .into(),
            );
            audio.send_command(
                InstrumentCmd::NoteOn {
                    instrument_id,
                    note: 64,
                    velocity: 100,
                }
                .into(),
            );

            thread::sleep(std::time::Duration::from_millis(500));
            audio.send_command(
                SequencerCmd::AddEffectToInstrument {
                    instrument_id,
                    effect: audio.get_effect_factory().create_mono_gain(7.0),
                }
                .into(),
            );
            audio.send_command(
                InstrumentCmd::NoteOn {
                    instrument_id,
                    note: 67,
                    velocity: 100,
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(500));

            // Stop all notes
            audio.send_command(InstrumentCmd::NoteOff { instrument_id }.into());

            // Listen to the release tail
            thread::sleep(std::time::Duration::from_millis(1000));
        }
        Err(e) => eprintln!("Failed to initialize BlightAudio: {}", e),
    };
}
