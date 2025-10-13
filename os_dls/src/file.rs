use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::parser::DlsParser;
use crate::sample::Sample;

/// The default location of the General MIDI sound bank on macOS
pub const MACOS_GS_INSTRUMENTS_PATH: &str = 
    "/System/Library/Components/CoreAudio.component/Contents/Resources/gs_instruments.dls";

/// Represents a loaded DLS file
pub struct DlsFile {
    /// Raw file data
    data: Vec<u8>,
    /// Path to the file
    path: String,
}

impl DlsFile {
    /// Opens a DLS file from the specified path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        let mut file = File::open(&path)
            .map_err(|e| format!("Failed to open DLS file at {}: {}", path_str, e))?;
        
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .map_err(|e| format!("Failed to read DLS file at {}: {}", path_str, e))?;
        
        Ok(DlsFile {
            data,
            path: path_str,
        })
    }
    
    /// Opens the default macOS General MIDI sound bank
    pub fn open_macos_default() -> Result<Self, String> {
        Self::open(MACOS_GS_INSTRUMENTS_PATH)
    }
    
    /// Returns the size of the DLS file in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }
    
    /// Returns the path to the DLS file
    pub fn path(&self) -> &str {
        &self.path
    }
    
    /// Extract all samples from the DLS file
    /// 
    /// This parses the DLS file structure and returns a vector of Sample objects,
    /// each containing the audio data and metadata for an individual sample.
    pub fn samples(&self) -> Result<Vec<Sample>, String> {
        let parser = DlsParser::new(self.data.clone());
        parser.extract_samples()
    }
    
    /// List all sample names available in the DLS file
    /// 
    /// Returns a vector of tuples containing (index, name) for each sample.
    /// The index can be used with `get_sample_by_id()` to retrieve a specific sample.
    /// Samples without names will have "unnamed" as their name.
    pub fn list_sample_names(&self) -> Result<Vec<(usize, String)>, String> {
        let samples = self.samples()?;
        Ok(samples
            .iter()
            .enumerate()
            .map(|(idx, sample)| {
                let name = sample
                    .name()
                    .unwrap_or("unnamed")
                    .to_string();
                (idx, name)
            })
            .collect())
    }
    
    /// Get a specific sample by its name
    /// 
    /// # Arguments
    /// * `name` - The name of the sample to retrieve
    /// 
    /// # Returns
    /// * `Ok(Sample)` - The sample with the matching name
    /// * `Err(String)` - If the sample is not found or parsing fails
    /// 
    /// # Example
    /// ```no_run
    /// use os_dls::load_mac_os_default;
    /// 
    /// let dls = load_mac_os_default().expect("Failed to load DLS file");
    /// let piano = dls.get_sample_by_name("PIANO36").expect("Sample not found");
    /// println!("Sample rate: {} Hz", piano.sample_rate());
    /// ```
    pub fn get_sample_by_name(&self, name: &str) -> Result<Sample, String> {
        let samples = self.samples()?;
        samples
            .into_iter()
            .find(|sample| {
                sample.name()
                    .map(|n| n == name)
                    .unwrap_or(false)
            })
            .ok_or_else(|| format!("Sample with name '{}' not found", name))
    }
    
    /// Get a specific sample by its index/ID
    /// 
    /// # Arguments
    /// * `id` - The zero-based index of the sample to retrieve
    /// 
    /// # Returns
    /// * `Ok(Sample)` - The sample at the specified index
    /// * `Err(String)` - If the index is out of bounds or parsing fails
    /// 
    /// # Example
    /// ```no_run
    /// use os_dls::load_mac_os_default;
    /// 
    /// let dls = load_mac_os_default().expect("Failed to load DLS file");
    /// let first_sample = dls.get_sample_by_id(0).expect("Sample not found");
    /// println!("First sample: {}", first_sample.name().unwrap_or("unnamed"));
    /// ```
    pub fn get_sample_by_id(&self, id: usize) -> Result<Sample, String> {
        let samples = self.samples()?;
        samples
            .get(id)
            .cloned()
            .ok_or_else(|| format!("Sample with ID {} not found (total samples: {})", id, samples.len()))
    }
}

/// Loads the default macOS General MIDI sound bank
pub fn load_mac_os_default() -> Result<DlsFile, String> {
    DlsFile::open_macos_default()
}