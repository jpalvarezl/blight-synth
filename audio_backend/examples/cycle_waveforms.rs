use std::thread;

use audio_backend::{
    id::{InstrumentId, VoiceId},
    instruments::Waveform,
    BlightAudio, InstrumentCmd, SynthCmd, TransportCmd,
};

fn main() {
    // This is a placeholder for the main function.
    // The actual implementation will depend on how you want to use the BlightAudio API.
    match &mut BlightAudio::new() {
        Ok(audio) => {
            println!("BlightAudio initialized successfully!");
            // You can now use `audio` to send commands, etc.

            let instrument_id = InstrumentId::from_raw(0);
            let voice_id = VoiceId::from_raw(0);
            let _ = audio.send_command(
                audio_backend::InstrumentCmd::AddInstrument {
                    instrument: audio
                        .get_instrument_factory()
                        .create_simple_oscillator(instrument_id, 0.0),
                }
                .into(),
            );
            let _ = audio.send_command(TransportCmd::PlayLastSong.into()); // There is no active song but this triggers the pumping of data into the audio thread
            let _ = audio.send_command(
                InstrumentCmd::NoteOn {
                    instrument_id,
                    note: 60,
                    velocity: 127,
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(1000));
            let _ = audio.send_command(
                InstrumentCmd::PassOnSynthCmd {
                    instrument_id,
                    synth_cmd: SynthCmd::SetWaveform {
                        voice_id,
                        waveform: Waveform::Sawtooth,
                    },
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(1000));
            let _ = audio.send_command(
                InstrumentCmd::PassOnSynthCmd {
                    instrument_id,
                    synth_cmd: SynthCmd::SetWaveform {
                        voice_id,
                        waveform: Waveform::Square,
                    },
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(1000));
            let _ = audio.send_command(
                InstrumentCmd::PassOnSynthCmd {
                    instrument_id,
                    synth_cmd: SynthCmd::SetWaveform {
                        voice_id,
                        waveform: Waveform::Triangle,
                    },
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(1000));
            let _ = audio.send_command(
                InstrumentCmd::PassOnSynthCmd {
                    instrument_id,
                    synth_cmd: SynthCmd::SetWaveform {
                        voice_id,
                        waveform: Waveform::NesTriangle,
                    },
                }
                .into(),
            );
            thread::sleep(std::time::Duration::from_millis(1000));
        }
        Err(e) => eprintln!("Failed to initialize BlightAudio: {}", e),
    };
}
