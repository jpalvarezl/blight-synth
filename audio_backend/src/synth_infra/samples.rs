pub struct SampleData {
    pub data: Vec<f32>, // Assuming samples are stored as f32 audio data
    // pub sample_id: SampleId, // Sample ID of the audio data
    pub sample_rate: f32, // Sample rate of the audio data
    pub channels: u16,    // Number of audio channels
}
