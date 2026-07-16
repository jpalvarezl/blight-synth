use crate::{AudioBackendError, Result};
use dsp::{id::SampleId, SampleData};
#[cfg(target_os = "macos")]
use log::info;
use std::{collections::HashMap, sync::Arc};

/// ResourceManager handles and identifies audio samples and other resources which can be identified by a unique ID.
pub struct ResourceManager {
    samples: HashMap<SampleId, Arc<SampleData>>,
    sample_names: HashMap<SampleId, String>,
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            samples: HashMap::new(),
            sample_names: HashMap::new(),
        }
    }

    /// Adds a sample to the resource manager.
    pub fn add_sample(&mut self, sample_id: SampleId, sample: SampleData) {
        self.samples.insert(sample_id, Arc::new(sample));
    }

    pub fn add_sample_from_file<P: AsRef<std::path::Path>>(
        &mut self,
        sample_id: SampleId,
        path: P,
    ) -> Result<()> {
        let path_ref = path.as_ref();
        let name = path_ref
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed")
            .to_string();
        let sample = load_wav_file(path)?;
        self.sample_names.insert(sample_id, name);
        self.add_sample(sample_id, sample);
        Ok(())
    }

    /// Retrieves a sample by its ID.
    pub fn get_sample_unsafe(&self, sample_id: SampleId) -> Arc<SampleData> {
        let sample = self.samples.get(&sample_id).expect("Sample not found");
        sample.clone()
    }

    /// Returns all sample names as a HashMap of SampleId to name.
    pub fn get_sample_names(&self) -> &HashMap<SampleId, String> {
        &self.sample_names
    }

    /// Returns a sample by ID, or None if not found.
    pub fn get_sample(&self, sample_id: SampleId) -> Option<Arc<SampleData>> {
        self.samples.get(&sample_id).cloned()
    }

    /// Loads all samples from the macOS DLS file. Returns the count loaded. Sample Ids range from 0 - 494
    #[cfg(target_os = "macos")]
    pub fn load_macos_dls_samples(&mut self) -> Result<usize> {
        let dls_file = os_dls::load_mac_os_default()
            .map_err(|e| AudioBackendError(format!("Failed to load macOS DLS file: {}", e)))?;

        let samples = dls_file
            .samples()
            .map_err(|e| AudioBackendError(format!("Failed to parse DLS samples: {}", e)))?;

        let count = samples.len();

        for (index, sample) in samples.into_iter().enumerate() {
            let name = sample.name().unwrap_or("unnamed").to_string();
            info!("Loaded DLS sample {}: {}", index, name);
            let sample_data = SampleData {
                data: sample.to_f32_samples(),
                sample_rate: sample.sample_rate() as f32,
                channels: sample.channels(),
                // Extract loop information from DLS sample
                loop_start: sample.loop_info().map(|l| l.start),
                loop_end: sample.loop_info().map(|l| l.end),
            };
            let sample_id = index as SampleId;
            self.sample_names.insert(sample_id, name);
            self.add_sample(sample_id, sample_data);
        }

        Ok(count)
    }
}

/// Loads a WAV file and returns the sample data as Vec<f32> and playback info
fn load_wav_file<P: AsRef<std::path::Path>>(path: P) -> Result<SampleData> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    let sample_rate = spec.sample_rate as f32;
    let channels = spec.channels;

    // Convert all samples to f32
    let data: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().map(|s| s.unwrap_or(0.0)).collect(),
        hound::SampleFormat::Int => {
            let max_value = (1i64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.map(|v| v as f32 / max_value).unwrap_or(0.0))
                .collect()
        }
    };

    Ok(SampleData {
        data,
        sample_rate,
        channels,
        loop_start: None, // WAV files don't have loop info (could add SMPL chunk parsing later)
        loop_end: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_wav_path() -> std::path::PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "blight-synth-resource-{}-{timestamp}.wav",
            std::process::id()
        ))
    }

    #[test]
    fn stores_and_retrieves_decoded_sample_data() {
        let mut resources = ResourceManager::new();
        resources.add_sample(
            7,
            SampleData {
                data: vec![0.25, -0.25],
                sample_rate: 48_000.0,
                channels: 1,
                loop_start: None,
                loop_end: None,
            },
        );

        let sample = resources.get_sample(7).expect("sample must exist");
        assert_eq!(sample.data, [0.25, -0.25]);
        assert_eq!(sample.sample_rate, 48_000.0);
        assert!(resources.get_sample(8).is_none());
    }

    #[test]
    fn loads_wav_files_in_the_non_realtime_resource_adapter() {
        let path = temporary_wav_path();
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 22_050,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(&path, spec).expect("create temporary WAV");
        for sample in [0_i16, i16::MAX, i16::MIN] {
            writer.write_sample(sample).expect("write WAV sample");
        }
        writer.finalize().expect("finalize temporary WAV");

        let mut resources = ResourceManager::new();
        let result = resources.add_sample_from_file(11, &path);
        std::fs::remove_file(&path).expect("remove temporary WAV");
        result.expect("load temporary WAV");

        let sample = resources.get_sample(11).expect("loaded sample must exist");
        assert_eq!(sample.sample_rate, 22_050.0);
        assert_eq!(sample.channels, 1);
        assert_eq!(sample.data.len(), 3);
        assert_eq!(sample.data[0], 0.0);
        assert!((sample.data[1] - (i16::MAX as f32 / 32_768.0)).abs() < f32::EPSILON);
        assert_eq!(sample.data[2], -1.0);
        assert_eq!(
            resources.get_sample_names().get(&11).map(String::as_str),
            path.file_stem().and_then(|name| name.to_str())
        );
    }
}
