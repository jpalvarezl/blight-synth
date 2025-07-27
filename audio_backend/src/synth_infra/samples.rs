use crate::SampleId;

pub struct SampleData {
    pub data: Vec<f32>, // Assuming samples are stored as f32 audio data
    pub sample_rate: u32, // Sample rate of the audio data
}