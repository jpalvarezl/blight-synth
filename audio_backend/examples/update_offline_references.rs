use std::{collections::BTreeMap, path::PathBuf};

use anyhow::{bail, Context, Result};
use audio_backend::{
    render_json_song, OfflineGoldenManifest, OfflineRenderConfig, OfflineRenderReference,
};

const SONGS: [&str; 2] = ["calibration.json", "ending_theme_no_effect.json"];

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.as_slice() != ["--update-reference"] {
        bail!(
            "this command rewrites reviewed audio references; run explicitly with --update-reference"
        );
    }

    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("audio_backend must be inside the workspace")
        .to_path_buf();
    let output_dir = workspace.join("target/offline-renders");
    std::fs::create_dir_all(&output_dir)?;
    let config = OfflineRenderConfig::canonical();
    let mut references = BTreeMap::<String, OfflineRenderReference>::new();

    for song_name in SONGS {
        let song_path = workspace.join(song_name);
        let first = render_json_song(&song_path, config)
            .with_context(|| format!("failed to render {song_name}"))?;
        let second = render_json_song(&song_path, config)
            .with_context(|| format!("failed repeat render for {song_name}"))?;
        let reference = first.reference();
        if reference != second.reference() {
            bail!("{song_name} was not deterministic across repeated renders");
        }
        let wav_path = output_dir.join(song_name.replace(".json", ".wav"));
        first.write_wav(&wav_path)?;
        println!(
            "{song_name}: {} frames, {} -> {}",
            reference.frames,
            reference.pcm_sha256,
            wav_path.display()
        );
        references.insert(song_name.to_string(), reference);
    }

    let manifest = OfflineGoldenManifest::characterization(config, references);
    let manifest_path = workspace.join("audio_backend/tests/golden/offline_render_manifest.json");
    std::fs::create_dir_all(
        manifest_path
            .parent()
            .expect("golden manifest must have a parent"),
    )?;
    let mut json = serde_json::to_string_pretty(&manifest)?;
    json.push('\n');
    std::fs::write(&manifest_path, json)?;
    println!("updated {}", manifest_path.display());
    println!("listen to every WAV before committing the reference update");
    Ok(())
}
