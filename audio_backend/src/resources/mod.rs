use std::{collections::HashMap, sync::Arc};

use crate::{id::SampleId, Result, SampleData};

/// ResourceManager handles and identifies audio samples and other resources which can be identified by a unique ID.
pub struct ResourceManager {
    samples: HashMap<SampleId, Arc<SampleData>>,
    // instruments: HashMap<InstrumentId, Arc<InstrumentData>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            samples: HashMap::new(),
            // instruments: HashMap::new(),
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
        let sample = load_wav_file(path)?;
        self.add_sample(sample_id, sample);
        Ok(())
    }

    /// Retrieves a sample by its ID.
    pub fn get_sample(&self, sample_id: SampleId) -> Option<Arc<SampleData>> {
        self.samples.get(&sample_id).cloned()
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
