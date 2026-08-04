use std::thread;

use audio_backend::{
    id::{EffectId, InstrumentId},
    BlightAudio, InstrumentCmd, MixerCmd,
};

pub fn main() {
    match &mut BlightAudio::new() {
        Ok(audio) => {
            println!("BlightAudio initialized successfully!");
            let stereo_gain_id = EffectId::from_raw(0);
            let _ = audio.send_command(
                MixerCmd::AddMasterEffect {
                    effect: audio
                        .get_effect_factory()
                        .create_stereo_gain(stereo_gain_id, 10f32),
                }
                .into(),
            );

            let inst_id = InstrumentId::from_raw(1);
            let _ = audio.send_command(
                InstrumentCmd::AddInstrument {
                    instrument: audio
                        .get_instrument_factory()
                        .create_simple_oscillator(inst_id, 0.0),
                }
                .into(),
            );
            let _ = audio.send_command(audio_backend::TransportCmd::PlayLastSong.into());

            let _ = audio.send_command(
                InstrumentCmd::NoteOn {
                    instrument_id: inst_id,
                    note: 60,
                    velocity: 127,
                }
                .into(),
            );

            // Play a very short note - 200ms
            thread::sleep(std::time::Duration::from_millis(200));
            let _ = audio.send_command(
                InstrumentCmd::NoteOff {
                    instrument_id: inst_id,
                }
                .into(),
            );

            // Wait to hear the release decay
            thread::sleep(std::time::Duration::from_millis(1000));

            let _ = audio.send_command(
                MixerCmd::SetMasterEffectParameter {
                    effect_id: stereo_gain_id,
                    param_index: 0,
                    value: 6f32,
                }
                .into(),
            );
            // Play a chord
            let inst2 = InstrumentId::from_raw(2);
            let inst3 = InstrumentId::from_raw(3);
            let i2 = audio
                .get_instrument_factory()
                .create_simple_oscillator(inst2, 0.0);
            let i3 = audio
                .get_instrument_factory()
                .create_simple_oscillator(inst3, 0.0);
            let _ = audio.send_command(InstrumentCmd::AddInstrument { instrument: i2 }.into());
            let _ = audio.send_command(InstrumentCmd::AddInstrument { instrument: i3 }.into());
            let _ = audio.send_command(
                InstrumentCmd::NoteOn {
                    instrument_id: inst_id,
                    note: 60,
                    velocity: 100,
                }
                .into(),
            );
            let _ = audio.send_command(
                InstrumentCmd::NoteOn {
                    instrument_id: inst2,
                    note: 64,
                    velocity: 100,
                }
                .into(),
            );
            let _ = audio.send_command(
                InstrumentCmd::NoteOn {
                    instrument_id: inst3,
                    note: 67,
                    velocity: 100,
                }
                .into(),
            );

            thread::sleep(std::time::Duration::from_millis(500));

            // Stop all notes
            let _ = audio.send_command(
                InstrumentCmd::NoteOff {
                    instrument_id: inst_id,
                }
                .into(),
            );
            let _ = audio.send_command(
                InstrumentCmd::NoteOff {
                    instrument_id: inst2,
                }
                .into(),
            );
            let _ = audio.send_command(
                InstrumentCmd::NoteOff {
                    instrument_id: inst3,
                }
                .into(),
            );

            // Listen to the release tail
            thread::sleep(std::time::Duration::from_millis(1000));
        }
        Err(e) => eprintln!("Failed to initialize BlightAudio: {}", e),
    };
}
