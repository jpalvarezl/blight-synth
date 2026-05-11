use std::{path::PathBuf, sync::Arc, thread, time::Duration};

use anyhow::{bail, Context, Result};
use audio_backend::{
    id::InstrumentId, instruments::Waveform as BackendWaveform, BlightAudio, EnvelopeCmd,
    InstrumentCmd, SequencerCmd, SynthCmd, TransportCmd,
};
use sequencer::{
    cli::FileFormat,
    models::{AmpEnvelopeParams, InstrumentData, Song, Waveform},
    project::open_song_from_file,
};

fn main() -> Result<()> {
    env_logger::init();

    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("calibration.json"));
    let duration_seconds = std::env::args()
        .nth(2)
        .map(|s| s.parse::<u64>())
        .transpose()
        .context("duration must be an integer number of seconds")?
        .unwrap_or(10);

    log::info!("loading song from {}", path.display());
    let song = open_song_from_file(&path, &FileFormat::Json)
        .with_context(|| format!("failed to load song from {}", path.display()))?;

    println!(
        "Loaded '{}' from {}: {} instruments, {} arrangement rows",
        song.name,
        path.display(),
        song.instrument_bank.len(),
        song.arrangement.len()
    );

    let mut audio = BlightAudio::with_song(Arc::new(song.clone()))?;
    hydrate_song(&mut audio, &song)?;

    log::info!("sending PlayLastSong");
    audio.send_command(TransportCmd::PlayLastSong.into());
    println!("Playing for {duration_seconds}s...");
    thread::sleep(Duration::from_secs(duration_seconds));

    log::info!("sending StopSong");
    audio.send_command(TransportCmd::StopSong.into());
    println!("Stopped.");

    Ok(())
}

fn hydrate_song(audio: &mut BlightAudio, song: &Song) -> Result<()> {
    for instrument in &song.instrument_bank {
        let instrument_id = instrument.id as InstrumentId;
        log::info!(
            "hydrating instrument {} ({})",
            instrument_id,
            instrument.name
        );
        match &instrument.data {
            InstrumentData::SimpleOscillator(params) => {
                let instrument = audio
                    .get_instrument_factory()
                    .create_oscillator_with_waveform(
                        instrument_id,
                        0.0,
                        map_waveform_to_backend(params.waveform),
                    );
                audio.send_command(SequencerCmd::AddTrackInstrument { instrument }.into());
                send_amp_envelope(audio, instrument_id, &params.amp_envelope);
            }
            InstrumentData::HiHat(params) => {
                let instrument = audio
                    .get_instrument_factory()
                    .create_hihat(instrument_id, 0.0);
                audio.send_command(SequencerCmd::AddTrackInstrument { instrument }.into());
                send_amp_envelope(audio, instrument_id, &params.amp_envelope);
            }
            InstrumentData::KickDrum(params) => {
                let instrument = audio
                    .get_instrument_factory()
                    .create_kick_drum(instrument_id, 0.0);
                audio.send_command(SequencerCmd::AddTrackInstrument { instrument }.into());
                send_amp_envelope(audio, instrument_id, &params.amp_envelope);
            }
            InstrumentData::SnareDrum(params) => {
                let instrument = audio
                    .get_instrument_factory()
                    .create_snare_drum(instrument_id, 0.0);
                audio.send_command(SequencerCmd::AddTrackInstrument { instrument }.into());
                send_amp_envelope(audio, instrument_id, &params.amp_envelope);
            }
            InstrumentData::DFAM(params) => {
                let instrument = audio
                    .get_instrument_factory()
                    .create_dfam(instrument_id, 0.0);
                audio.send_command(SequencerCmd::AddTrackInstrument { instrument }.into());
                send_amp_envelope(audio, instrument_id, &params.amp_envelope);
            }
            unsupported => {
                bail!("unsupported instrument in CLI playback example: {unsupported:?}");
            }
        }
    }

    Ok(())
}

fn send_amp_envelope(
    audio: &mut BlightAudio,
    instrument_id: InstrumentId,
    env: &AmpEnvelopeParams,
) {
    for command in [
        EnvelopeCmd::SetAttack { attack: env.attack },
        EnvelopeCmd::SetDecay { decay: env.decay },
        EnvelopeCmd::SetSustain {
            sustain: env.sustain,
        },
        EnvelopeCmd::SetRelease {
            release: env.release,
        },
    ] {
        audio.send_command(
            InstrumentCmd::PassOnSynthCmd {
                instrument_id,
                synth_cmd: SynthCmd::EnvelopeCommand {
                    envelope_id: Some(0),
                    command,
                },
            }
            .into(),
        );
    }
}

fn map_waveform_to_backend(waveform: Waveform) -> BackendWaveform {
    match waveform {
        Waveform::Sine => BackendWaveform::Sine,
        Waveform::Square => BackendWaveform::Square,
        Waveform::Sawtooth => BackendWaveform::Sawtooth,
        Waveform::Triangle => BackendWaveform::Triangle,
        Waveform::NesTriangle => BackendWaveform::NesTriangle,
    }
}
