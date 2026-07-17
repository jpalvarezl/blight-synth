use std::{thread, time::Duration};

use audio_backend::{BlightAudio, EnvelopeCmd, InstrumentCmd, SynthCmd, TransportCmd};

fn main() {
    match &mut BlightAudio::new() {
        Ok(audio) => {
            let instrument_id = 0;
            audio.send_command(
                audio_backend::InstrumentCmd::AddInstrument {
                    instrument: audio
                        .get_instrument_factory()
                        .create_simple_oscillator(instrument_id, 0.0),
                }
                .into(),
            );
            audio.send_command(
                InstrumentCmd::PassOnSynthCmd {
                    instrument_id,
                    synth_cmd: SynthCmd::EnvelopeCommand {
                        envelope_id: None,
                        command: EnvelopeCmd::SetAttack { attack: 0.4 },
                    },
                }
                .into(),
            );
            audio.send_command(
                InstrumentCmd::PassOnSynthCmd {
                    instrument_id,
                    synth_cmd: SynthCmd::EnvelopeCommand {
                        envelope_id: None,
                        command: EnvelopeCmd::SetDecay { decay: 0.1 },
                    },
                }
                .into(),
            );
            audio.send_command(
                InstrumentCmd::PassOnSynthCmd {
                    instrument_id,
                    synth_cmd: SynthCmd::EnvelopeCommand {
                        envelope_id: None,
                        command: EnvelopeCmd::SetSustain { sustain: 0.8 },
                    },
                }
                .into(),
            );
            audio.send_command(
                InstrumentCmd::PassOnSynthCmd {
                    instrument_id,
                    synth_cmd: SynthCmd::EnvelopeCommand {
                        envelope_id: None,
                        command: EnvelopeCmd::SetRelease { release: 0.5 },
                    },
                }
                .into(),
            );
            audio.send_command(TransportCmd::PlayLastSong.into()); // There is no active song but this triggers the pumping of data into the audio thread

            audio.send_command(
                InstrumentCmd::NoteOn {
                    instrument_id,
                    note: 60,
                    velocity: 127,
                }
                .into(),
            );
            thread::sleep(Duration::from_millis(600));
            audio.send_command(InstrumentCmd::NoteOff { instrument_id }.into());
            thread::sleep(Duration::from_millis(1000));

            audio.send_command(
                InstrumentCmd::PassOnSynthCmd {
                    instrument_id,
                    synth_cmd: SynthCmd::EnvelopeCommand {
                        envelope_id: None,
                        command: EnvelopeCmd::SetAttack { attack: 2.0 },
                    },
                }
                .into(),
            );
            audio.send_command(
                InstrumentCmd::PassOnSynthCmd {
                    instrument_id,
                    synth_cmd: SynthCmd::EnvelopeCommand {
                        envelope_id: None,
                        command: EnvelopeCmd::SetDecay { decay: 0.1 },
                    },
                }
                .into(),
            );
            audio.send_command(
                InstrumentCmd::PassOnSynthCmd {
                    instrument_id,
                    synth_cmd: SynthCmd::EnvelopeCommand {
                        envelope_id: None,
                        command: EnvelopeCmd::SetSustain { sustain: 0.8 },
                    },
                }
                .into(),
            );
            audio.send_command(
                InstrumentCmd::PassOnSynthCmd {
                    instrument_id,
                    synth_cmd: SynthCmd::EnvelopeCommand {
                        envelope_id: None,
                        command: EnvelopeCmd::SetRelease { release: 2.0 },
                    },
                }
                .into(),
            );
            audio.send_command(
                InstrumentCmd::NoteOn {
                    instrument_id,
                    note: 60,
                    velocity: 127,
                }
                .into(),
            );
            thread::sleep(Duration::from_millis(2200));
            audio.send_command(InstrumentCmd::NoteOff { instrument_id }.into());
            thread::sleep(Duration::from_millis(4000)); // wait for release to finish so the voice gets evicted from the voice manager

            audio.send_command(
                InstrumentCmd::PassOnSynthCmd {
                    instrument_id,
                    synth_cmd: SynthCmd::EnvelopeCommand {
                        envelope_id: None,
                        command: EnvelopeCmd::SetAttack { attack: 0.01 },
                    },
                }
                .into(),
            );
            audio.send_command(
                InstrumentCmd::PassOnSynthCmd {
                    instrument_id,
                    synth_cmd: SynthCmd::EnvelopeCommand {
                        envelope_id: None,
                        command: EnvelopeCmd::SetDecay { decay: 0.1 },
                    },
                }
                .into(),
            );
            audio.send_command(
                InstrumentCmd::PassOnSynthCmd {
                    instrument_id,
                    synth_cmd: SynthCmd::EnvelopeCommand {
                        envelope_id: None,
                        command: EnvelopeCmd::SetSustain { sustain: 0.8 },
                    },
                }
                .into(),
            );
            audio.send_command(
                InstrumentCmd::PassOnSynthCmd {
                    instrument_id,
                    synth_cmd: SynthCmd::EnvelopeCommand {
                        envelope_id: None,
                        command: EnvelopeCmd::SetRelease { release: 0.1 },
                    },
                }
                .into(),
            );
            audio.send_command(
                InstrumentCmd::NoteOn {
                    instrument_id,
                    note: 60,
                    velocity: 127,
                }
                .into(),
            );
            thread::sleep(Duration::from_millis(200));
            audio.send_command(InstrumentCmd::NoteOff { instrument_id }.into());
            thread::sleep(Duration::from_millis(1000));
        }
        Err(e) => {
            eprintln!("Error initializing audio: {}", e);
        }
    }
}
