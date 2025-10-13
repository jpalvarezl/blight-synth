//! # os_dls
//!
//! A library for extracting individual audio samples from operating system DLS (Downloadable Sounds) files.
//! The primary purpose is to unfold the macOS gs_instruments.dls file and provide access to individual samples
//! for audio playback.
//!
//! ## Features
//!
//! - Load DLS files from the macOS system location
//! - Extract individual sample data for playback
//! - List available samples by name and ID
//! - Load samples by name or ID
//! - Access sample metadata (name, sample rate, bit depth, etc.)
//! - Get raw PCM audio data for each sample
//! - Convert samples to normalized f32 format for audio processing
//!
//! ## Example
//!
//! ```no_run
//! use os_dls::load_mac_os_default;
//!
//! let dls_file = load_mac_os_default().expect("Failed to load DLS file");
//!
//! // List all available samples
//! let sample_names = dls_file.list_sample_names().expect("Failed to list samples");
//! println!("Found {} samples", sample_names.len());
//!
//! // Load a specific sample by name
//! let piano = dls_file.get_sample_by_name("PIANO36").expect("Sample not found");
//! println!("Sample: {} - {} Hz, {} bits",
//!     piano.name().unwrap_or("unnamed"),
//!     piano.sample_rate(),
//!     piano.bits_per_sample()
//! );
//!
//! // Or load by ID
//! let first_sample = dls_file.get_sample_by_id(0).expect("Sample not found");
//!
//! // Get all samples at once
//! let samples = dls_file.samples().expect("Failed to parse samples");
//! ```

mod file;
mod parser;
mod sample;

pub use file::{DlsFile, MACOS_GS_INSTRUMENTS_PATH, load_mac_os_default};
pub use sample::Sample;
