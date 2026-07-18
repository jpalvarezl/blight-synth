use std::{collections::BTreeMap, path::Path, sync::Arc};

use anyhow::{bail, Context, Result};
use sequencer::{cli::FileFormat, models::Song, project::open_song_from_file};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{build_song_hydration_commands, Player, SequencerCmd, TransportCmd};

/// Versioned golden-render profile values, not device/runtime defaults.
///
/// 48 kHz is the project reference rate; 256 frames is a representative bounded
/// callback partition; 120 seconds is a CI safety ceiling well above current
/// reference-song durations. Changing these values intentionally invalidates
/// the render references.
pub const CANONICAL_SAMPLE_RATE: u32 = 48_000;
pub const CANONICAL_BLOCK_SIZE: usize = 256;
pub const CANONICAL_MAX_FRAMES: usize = CANONICAL_SAMPLE_RATE as usize * 120;
const MAX_ENGINE_BLOCK_SIZE: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfflineRenderConfig {
    pub sample_rate: u32,
    pub block_size: usize,
    pub max_frames: usize,
}

impl OfflineRenderConfig {
    pub const fn canonical() -> Self {
        Self {
            sample_rate: CANONICAL_SAMPLE_RATE,
            block_size: CANONICAL_BLOCK_SIZE,
            max_frames: CANONICAL_MAX_FRAMES,
        }
    }

    fn validate(self) -> Result<Self> {
        if self.sample_rate == 0 {
            bail!("offline sample rate must be greater than zero");
        }
        if self.block_size == 0 || self.block_size > MAX_ENGINE_BLOCK_SIZE {
            bail!(
                "offline block size must be in 1..={MAX_ENGINE_BLOCK_SIZE}, got {}",
                self.block_size
            );
        }
        if self.max_frames == 0 {
            bail!("offline maximum frame count must be greater than zero");
        }
        Ok(self)
    }
}

impl Default for OfflineRenderConfig {
    fn default() -> Self {
        Self::canonical()
    }
}

#[derive(Debug)]
pub struct OfflineRender {
    sample_rate: u32,
    left: Vec<f32>,
    right: Vec<f32>,
}

impl OfflineRender {
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn frame_count(&self) -> usize {
        self.left.len()
    }

    pub fn left(&self) -> &[f32] {
        &self.left
    }

    pub fn right(&self) -> &[f32] {
        &self.right
    }

    pub fn canonical_pcm(&self) -> Vec<i16> {
        let mut pcm = Vec::with_capacity(self.frame_count() * 2);
        for (&left, &right) in self.left.iter().zip(&self.right) {
            pcm.push(quantize_pcm16(left));
            pcm.push(quantize_pcm16(right));
        }
        pcm
    }

    pub fn pcm_sha256(&self) -> String {
        let pcm = self.canonical_pcm();
        let mut hasher = Sha256::new();
        for sample in pcm {
            hasher.update(sample.to_le_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    pub fn reference(&self) -> OfflineRenderReference {
        let (peak_left, rms_left, clipped_left) = channel_statistics(&self.left);
        let (peak_right, rms_right, clipped_right) = channel_statistics(&self.right);
        OfflineRenderReference {
            sample_rate: self.sample_rate,
            channels: 2,
            frames: self.frame_count(),
            pcm_sha256: self.pcm_sha256(),
            peak_left,
            peak_right,
            rms_left,
            rms_right,
            clipped_samples: clipped_left + clipped_right,
        }
    }

    /// Wrap canonical PCM in a WAV container for listening.
    ///
    /// CPAL only streams buffers to audio devices; it does not encode offline
    /// files. Hound remains confined to this host/I/O crate and is not an
    /// `engine` or `dsp` dependency.
    pub fn write_wav(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec)
            .with_context(|| format!("failed to create {}", path.display()))?;
        for sample in self.canonical_pcm() {
            writer.write_sample(sample)?;
        }
        writer.finalize()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfflineRenderReference {
    pub sample_rate: u32,
    pub channels: u16,
    pub frames: usize,
    pub pcm_sha256: String,
    pub peak_left: f32,
    pub peak_right: f32,
    pub rms_left: f32,
    pub rms_right: f32,
    pub clipped_samples: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfflineGoldenManifest {
    pub format_version: u32,
    pub baseline_kind: String,
    pub canonical_platform: String,
    pub known_limitations: Vec<String>,
    pub config: OfflineRenderConfig,
    pub songs: BTreeMap<String, OfflineRenderReference>,
}

impl OfflineGoldenManifest {
    pub fn characterization(
        config: OfflineRenderConfig,
        songs: BTreeMap<String, OfflineRenderReference>,
    ) -> Self {
        Self {
            format_version: 1,
            baseline_kind: "characterization".to_string(),
            canonical_platform: current_platform(),
            known_limitations: vec![
                "#132 transport-independent rendering and release/effect tails".to_string(),
                "#134 sample-accurate event scheduling".to_string(),
                "#136 mixer gain staging and clipping".to_string(),
            ],
            config,
            songs,
        }
    }
}

pub fn current_platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

pub fn load_json_song(path: &Path) -> Result<Song> {
    open_song_from_file(&path.to_path_buf(), &FileFormat::Json)
        .with_context(|| format!("failed to load song from {}", path.display()))
}

pub fn render_json_song(path: &Path, config: OfflineRenderConfig) -> Result<OfflineRender> {
    let song = load_json_song(path)?;
    render_song(&song, config)
}

pub fn render_song(song: &Song, config: OfflineRenderConfig) -> Result<OfflineRender> {
    let config = config.validate()?;
    let hydration_commands = build_song_hydration_commands(song, config.sample_rate as f32)?;
    let song = Arc::new(song.clone());
    let mut player = Player::new(song.clone(), config.sample_rate as f64);
    player.handle_command(SequencerCmd::LoadSong { song }.into());
    for command in hydration_commands {
        player.handle_command(command);
    }
    player.handle_command(TransportCmd::PlayLastSong.into());

    let initial_capacity = config.max_frames.min(config.sample_rate as usize * 60);
    let mut rendered = OfflineRender {
        sample_rate: config.sample_rate,
        left: Vec::with_capacity(initial_capacity),
        right: Vec::with_capacity(initial_capacity),
    };

    let mut block_left = vec![0.0; config.block_size];
    let mut block_right = vec![0.0; config.block_size];
    while player.is_playing() {
        if rendered.frame_count() >= config.max_frames {
            bail!(
                "offline render exceeded maximum of {} frames",
                config.max_frames
            );
        }
        let frame_count = config
            .block_size
            .min(config.max_frames - rendered.frame_count());
        let left = &mut block_left[..frame_count];
        let right = &mut block_right[..frame_count];
        left.fill(0.0);
        right.fill(0.0);
        player.process(left, right, config.sample_rate as f32, frame_count);
        if left
            .iter()
            .chain(right.iter())
            .any(|sample| !sample.is_finite())
        {
            bail!("offline render produced a non-finite sample");
        }
        rendered.left.extend_from_slice(left);
        rendered.right.extend_from_slice(right);
    }

    if rendered.left.iter().all(|sample| *sample == 0.0)
        && rendered.right.iter().all(|sample| *sample == 0.0)
    {
        bail!("offline render produced only silence");
    }

    Ok(rendered)
}

fn quantize_pcm16(sample: f32) -> i16 {
    let sample = sample.clamp(-1.0, 1.0);
    if sample >= 0.0 {
        (sample * i16::MAX as f32).round() as i16
    } else {
        (sample * -(i16::MIN as f32)).round() as i16
    }
}

fn channel_statistics(channel: &[f32]) -> (f32, f32, usize) {
    if channel.is_empty() {
        return (0.0, 0.0, 0);
    }
    let mut peak = 0.0_f32;
    let mut sum_squares = 0.0_f64;
    let mut clipped = 0;
    for &sample in channel {
        peak = peak.max(sample.abs());
        sum_squares += f64::from(sample) * f64::from(sample);
        if sample.abs() > 1.0 {
            clipped += 1;
        }
    }
    let rms = (sum_squares / channel.len() as f64).sqrt() as f32;
    (peak, rms, clipped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_offline_render_limits() {
        let song = Song::new("limit test");
        let error = render_song(
            &song,
            OfflineRenderConfig {
                sample_rate: 48_000,
                block_size: 256,
                max_frames: 256,
            },
        )
        .expect_err("default song should exceed one block");
        assert!(error.to_string().contains("exceeded maximum"));
    }

    #[test]
    fn pcm_quantization_has_explicit_endpoints() {
        assert_eq!(quantize_pcm16(-2.0), i16::MIN);
        assert_eq!(quantize_pcm16(-1.0), i16::MIN);
        assert_eq!(quantize_pcm16(0.0), 0);
        assert_eq!(quantize_pcm16(1.0), i16::MAX);
        assert_eq!(quantize_pcm16(2.0), i16::MAX);
    }

    #[test]
    fn pcm_quantization_rounds_regular_samples_symmetrically() {
        assert_eq!(quantize_pcm16(0.5), 16_384);
        assert_eq!(quantize_pcm16(-0.5), -16_384);
        assert_eq!(quantize_pcm16(0.25), 8_192);
        assert_eq!(quantize_pcm16(-0.25), -8_192);
    }

    #[test]
    fn canonical_pcm_is_stereo_interleaved_little_endian() {
        let render = OfflineRender {
            sample_rate: 48_000,
            left: vec![0.5, 0.25],
            right: vec![-0.5, -0.25],
        };

        let pcm = render.canonical_pcm();
        assert_eq!(pcm, [16_384, -16_384, 8_192, -8_192]);
        let bytes = pcm
            .iter()
            .flat_map(|sample| sample.to_le_bytes())
            .collect::<Vec<_>>();
        assert_eq!(bytes, [0x00, 0x40, 0x00, 0xC0, 0x00, 0x20, 0x00, 0xE0]);
    }
}
