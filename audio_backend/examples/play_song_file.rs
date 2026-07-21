use std::{path::PathBuf, thread, time::Duration};

use anyhow::{Context, Result};
use audio_backend::{load_song_file_into_audio, BlightAudio, TransportCmd};

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

    let mut audio = BlightAudio::new()?;
    let song = load_song_file_into_audio(&mut audio, &path)?;

    println!(
        "Loaded '{}' from {}: {} instruments, {} arrangement rows",
        song.name,
        path.display(),
        song.instrument_bank.len(),
        song.arrangement.len()
    );

    log::info!("sending PlayLastSong");
    let _ = audio.send_command(TransportCmd::PlayLastSong.into());
    println!("Playing for {duration_seconds}s...");
    thread::sleep(Duration::from_secs(duration_seconds));

    log::info!("sending StopSong");
    let _ = audio.send_command(TransportCmd::StopSong.into());
    println!("Stopped.");

    Ok(())
}
