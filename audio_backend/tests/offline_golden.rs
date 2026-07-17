use std::{collections::BTreeMap, path::PathBuf};

use audio_backend::{
    current_platform, load_json_song, render_json_song, render_song, OfflineGoldenManifest,
    OfflineRenderReference,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("audio_backend must be inside the workspace")
        .to_path_buf()
}

fn load_manifest() -> OfflineGoldenManifest {
    let path = workspace_root().join("audio_backend/tests/golden/offline_render_manifest.json");
    let contents = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_json::from_str(&contents)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
}

#[test]
fn repository_songs_match_reviewed_offline_references() {
    let manifest = load_manifest();
    assert_eq!(manifest.format_version, 1);
    assert_eq!(manifest.baseline_kind, "characterization");
    assert_eq!(manifest.songs.len(), 3);

    let mut actual_references = BTreeMap::<String, OfflineRenderReference>::new();
    for (song_name, expected) in &manifest.songs {
        let path = workspace_root().join(song_name);
        let first = render_json_song(&path, manifest.config)
            .unwrap_or_else(|error| panic!("failed to render {song_name}: {error:#}"));
        let second = render_json_song(&path, manifest.config)
            .unwrap_or_else(|error| panic!("failed repeat render for {song_name}: {error:#}"));
        let actual = first.reference();

        assert_eq!(
            actual,
            second.reference(),
            "{song_name} is nondeterministic"
        );
        assert_eq!(actual.sample_rate, manifest.config.sample_rate);
        assert_eq!(actual.channels, 2);
        assert_eq!(
            actual.frames, expected.frames,
            "{song_name} duration changed"
        );
        assert!(actual.peak_left > 0.0 || actual.peak_right > 0.0);
        assert_eq!(actual.pcm_sha256.len(), 64);

        if current_platform() == manifest.canonical_platform {
            assert_eq!(
                &actual, expected,
                "{song_name} audio changed; inspect rendered WAVs and run the explicit \
                 update_offline_references -- --update-reference command only when intentional"
            );
        } else {
            assert_eq!(actual.clipped_samples, expected.clipped_samples);
            assert_metric_close(song_name, "peak_left", actual.peak_left, expected.peak_left);
            assert_metric_close(
                song_name,
                "peak_right",
                actual.peak_right,
                expected.peak_right,
            );
            assert_metric_close(song_name, "rms_left", actual.rms_left, expected.rms_left);
            assert_metric_close(song_name, "rms_right", actual.rms_right, expected.rms_right);
        }
        actual_references.insert(song_name.clone(), actual);
    }

    assert_eq!(
        actual_references.keys().collect::<Vec<_>>(),
        manifest.songs.keys().collect::<Vec<_>>()
    );
}

fn assert_metric_close(song: &str, metric: &str, actual: f32, expected: f32) {
    const TOLERANCE: f32 = 1.0e-5;
    assert!(
        (actual - expected).abs() <= TOLERANCE,
        "{song} {metric} changed: expected {expected}, got {actual}"
    );
}

#[test]
fn changing_a_song_note_changes_its_pcm_reference() {
    let manifest = load_manifest();
    let song_name = "calibration.json";
    let mut song =
        load_json_song(&workspace_root().join(song_name)).expect("load calibration song");
    song.phrase_bank[0].events[0].note += 1;

    let changed = render_song(&song, manifest.config).expect("render changed song");

    assert_ne!(
        changed.pcm_sha256(),
        manifest.songs[song_name].pcm_sha256,
        "a note change must invalidate the end-to-end PCM reference"
    );
}
