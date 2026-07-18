use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use audio_backend::{render_json_song, OfflineRenderConfig};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let Some(input) = args.next() else {
        bail!(
            "usage: cargo run -p audio_backend --example render_song -- <song.json> <output.wav>"
        );
    };
    let Some(output) = args.next() else {
        bail!(
            "usage: cargo run -p audio_backend --example render_song -- <song.json> <output.wav>"
        );
    };
    if args.next().is_some() {
        bail!("render_song accepts exactly one input and one output path");
    }

    let input = PathBuf::from(input);
    let output = PathBuf::from(output);
    let render = render_json_song(&input, OfflineRenderConfig::canonical())
        .with_context(|| format!("failed to render {}", input.display()))?;
    render.write_wav(&output)?;
    let reference = render.reference();
    println!("rendered {}", input.display());
    println!("  output: {}", output.display());
    println!("  frames: {}", reference.frames);
    println!("  PCM SHA-256: {}", reference.pcm_sha256);
    println!(
        "  peak L/R: {:.6} / {:.6}",
        reference.peak_left, reference.peak_right
    );
    println!(
        "  RMS L/R: {:.6} / {:.6}",
        reference.rms_left, reference.rms_right
    );
    Ok(())
}
