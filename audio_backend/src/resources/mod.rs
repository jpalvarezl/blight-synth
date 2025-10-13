use std::{collections::HashMap, sync::Arc};
use log::info;
use crate::{id::SampleId, Result, SampleData};

/// ResourceManager handles and identifies audio samples and other resources which can be identified by a unique ID.
pub struct ResourceManager {
    samples: HashMap<SampleId, Arc<SampleData>>,
    sample_names: HashMap<SampleId, String>,
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
            .map_err(|e| crate::AudioBackendError(format!("Failed to load macOS DLS file: {}", e)))?;
        
        let samples = dls_file.samples()
            .map_err(|e| crate::AudioBackendError(format!("Failed to parse DLS samples: {}", e)))?;
        
        let count = samples.len();
        
        for (index, sample) in samples.into_iter().enumerate() {
            info!("Loaded DLS sample {}: {}", index, sample.name().unwrap_or("unnamed"));
            let name = sample.name().unwrap_or("unnamed").to_string();
            let sample_data = SampleData {
                data: sample.to_f32_samples(),
                sample_rate: sample.sample_rate() as f32,
                channels: sample.channels(),
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
    })
}


