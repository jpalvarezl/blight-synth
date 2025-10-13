/// Represents an individual audio sample extracted from a DLS file.
/// This struct contains both metadata and the raw PCM audio data,
/// ready for audio playback.
#[derive(Debug, Clone)]
pub struct Sample {
    /// Sample name (if available)
    name: Option<String>,

    /// Raw PCM audio data
    audio_data: Vec<u8>,

    /// Sample rate in Hz (e.g., 22050, 44100)
    sample_rate: u32,

    /// Number of audio channels (1 = mono, 2 = stereo)
    channels: u16,

    /// Bits per sample (typically 8 or 16)
    bits_per_sample: u16,

    /// MIDI Unity Note - the MIDI note number at which this sample plays at its original pitch
    /// For example, 60 = Middle C (C4)
    unity_note: Option<u8>,

    /// Fine tune adjustment in cents (-100 to +100)
    fine_tune: Option<i16>,
}

impl Sample {
    /// Create a new Sample
    pub(crate) fn new(
        name: Option<String>,
        audio_data: Vec<u8>,
        sample_rate: u32,
        channels: u16,
        bits_per_sample: u16,
        unity_note: Option<u8>,
        fine_tune: Option<i16>,
    ) -> Self {
        Self {
            name,
            audio_data,
            sample_rate,
            channels,
            bits_per_sample,
            unity_note,
            fine_tune,
        }
    }

    /// Get the sample name (if available)
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Get the raw PCM audio data
    ///
    /// For 8-bit samples, each byte is an unsigned sample value (0-255)
    /// For 16-bit samples, pairs of bytes represent signed 16-bit values (little-endian)
    pub fn audio_data(&self) -> &[u8] {
        &self.audio_data
    }

    /// Get the sample rate in Hz
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get the number of channels (1 = mono, 2 = stereo)
    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// Get bits per sample (8 or 16)
    pub fn bits_per_sample(&self) -> u16 {
        self.bits_per_sample
    }

    /// Get the MIDI unity note (the note at which the sample plays at original pitch)
    /// Returns None if not specified
    pub fn unity_note(&self) -> Option<u8> {
        self.unity_note
    }

    /// Get the fine tune adjustment in cents
    /// Returns None if not specified
    pub fn fine_tune(&self) -> Option<i16> {
        self.fine_tune
    }

    /// Get the duration of the sample in seconds
    pub fn duration_seconds(&self) -> f64 {
        let bytes_per_sample = (self.bits_per_sample / 8) as u32;
        let total_samples =
            self.audio_data.len() as u32 / (bytes_per_sample * self.channels as u32);
        total_samples as f64 / self.sample_rate as f64
    }

    /// Get the size of the audio data in bytes
    pub fn size(&self) -> usize {
        self.audio_data.len()
    }

    /// Check if the sample is mono
    pub fn is_mono(&self) -> bool {
        self.channels == 1
    }

    /// Check if the sample is stereo
    pub fn is_stereo(&self) -> bool {
        self.channels == 2
    }

    /// Convert 8-bit unsigned samples to f32 samples in the range [-1.0, 1.0]
    /// This is useful for audio playback systems that expect normalized float samples
    pub fn to_f32_samples(&self) -> Vec<f32> {
        match self.bits_per_sample {
            8 => {
                // 8-bit samples are unsigned (0-255), center at 128
                self.audio_data
                    .iter()
                    .map(|&sample| (sample as f32 - 128.0) / 128.0)
                    .collect()
            }
            16 => {
                // 16-bit samples are signed little-endian
                self.audio_data
                    .chunks_exact(2)
                    .map(|chunk| {
                        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                        sample as f32 / 32767.0
                    })
                    .collect()
            }
            _ => {
                eprintln!("Warning: Unsupported bit depth: {}", self.bits_per_sample);
                Vec::new()
            }
        }
    }

    /// Get the MIDI note name for the unity note
    pub fn unity_note_name(&self) -> Option<String> {
        self.unity_note.map(|note| {
            let note_names = [
                "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
            ];
            let octave = (note / 12) as i32 - 1;
            let note_name = note_names[(note % 12) as usize];
            format!("{}{}", note_name, octave)
        })
    }
}
