use std::thread;

use audio_backend::{BlightAudio, EffectCmd, InstrumentCmd, SequencerCmd, SynthCmd, TransportCmd};

pub fn main() {
    match &mut BlightAudio::new() {
        Ok(audio) => {
            println!("BlightAudio initialized successfully!");
            let instrument_id = 0;
            let effect_id = 0;
            audio.send_command(
                SequencerCmd::AddTrackInstrument {
                    instrument: audio
                        .get_instrument_factory()
                        .create_dfam(instrument_id, 0.0),
                }
                .into(),
            );
            audio.send_command(
                SequencerCmd::AddEffectToInstrument {
                    instrument_id,
                    effect: audio.get_effect_factory().create_mono_gain(effect_id, 1.0),
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

            // Play a very short note - 200ms
            thread::sleep(std::time::Duration::from_millis(200));
            audio.send_command(InstrumentCmd::NoteOff { instrument_id }.into());

            // Wait to hear the release decay
            thread::sleep(std::time::Duration::from_millis(1000));
            audio.send_command(
                InstrumentCmd::PassOnSynthCmd {
                    instrument_id,
                    synth_cmd: SynthCmd::EffectCommand {
                        effect_id,
                        command: EffectCmd::SetParameter {
                            param_index: 0,
                            value: -20f32,
                        },
                    },
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
                InstrumentCmd::PassOnSynthCmd {
                    instrument_id,
                    synth_cmd: SynthCmd::EffectCommand {
                        effect_id,
                        command: EffectCmd::SetParameter {
                            param_index: 0,
                            value: -10f32,
                        },
                    },
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
                InstrumentCmd::PassOnSynthCmd {
                    instrument_id,
                    synth_cmd: SynthCmd::EffectCommand {
                        effect_id,
                        command: EffectCmd::SetParameter {
                            param_index: 0,
                            value: 6f32,
                        },
                    },
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
