use std::thread;

use audio_backend::{id::InstrumentId, BlightAudio, EngineCmd, MixerCmd, SequencerCmd};

pub fn main() {
    match &mut BlightAudio::new() {
        Ok(audio) => {
            println!("BlightAudio initialized successfully!");
            audio.send_command(
                MixerCmd::AddMasterEffect {
                    effect: audio.get_effect_factory().create_stereo_gain(0.1),
                }
                .into(),
            );

            let inst_id: InstrumentId = 1;
            let instrument = audio
                .get_instrument_factory()
                .create_simple_oscillator(inst_id, 0.0);
            audio.send_command(SequencerCmd::AddTrackInstrument { instrument }.into());
            audio.send_command(
                EngineCmd::NoteOn {
                    instrument_id: inst_id,
                    note: 60,
                    velocity: 127,
                }
                .into(),
            );

            // Play a very short note - 200ms
            thread::sleep(std::time::Duration::from_millis(200));
            audio.send_command(
                EngineCmd::NoteOff {
                    instrument_id: inst_id,
                }
                .into(),
            );

            // Wait to hear the release decay
            thread::sleep(std::time::Duration::from_millis(3000));

            // Play a chord
            let inst2: InstrumentId = 2;
            let inst3: InstrumentId = 3;
            let i2 = audio
                .get_instrument_factory()
                .create_simple_oscillator(inst2, 0.0);
            let i3 = audio
                .get_instrument_factory()
                .create_simple_oscillator(inst3, 0.0);
            audio.send_command(SequencerCmd::AddTrackInstrument { instrument: i2 }.into());
            audio.send_command(SequencerCmd::AddTrackInstrument { instrument: i3 }.into());
            audio.send_command(
                EngineCmd::NoteOn {
                    instrument_id: inst_id,
                    note: 60,
                    velocity: 100,
                }
                .into(),
            );
            audio.send_command(
                EngineCmd::NoteOn {
                    instrument_id: inst2,
                    note: 64,
                    velocity: 100,
                }
                .into(),
            );
            audio.send_command(
                EngineCmd::NoteOn {
                    instrument_id: inst3,
                    note: 67,
                    velocity: 100,
                }
                .into(),
            );

            thread::sleep(std::time::Duration::from_millis(500));

            // Stop all notes
            audio.send_command(
                EngineCmd::NoteOff {
                    instrument_id: inst_id,
                }
                .into(),
            );
            audio.send_command(
                EngineCmd::NoteOff {
                    instrument_id: inst2,
                }
                .into(),
            );
            audio.send_command(
                EngineCmd::NoteOff {
                    instrument_id: inst3,
                }
                .into(),
            );

            // Listen to the release tail
            thread::sleep(std::time::Duration::from_millis(5000));
        }
        Err(e) => eprintln!("Failed to initialize BlightAudio: {}", e),
    };
}
