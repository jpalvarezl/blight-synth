use std::{collections::HashMap, sync::Arc};

use crate::{SampleData, SampleId};


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

    /// Retrieves a sample by its ID.
    pub fn get_sample(&self, sample_id: SampleId) -> Option<Arc<SampleData>> {
        self.samples.get(&sample_id).cloned()
    }
}