use std::sync::{Arc, Mutex};

// --- Enum to Select Active Waveform ---
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveWaveform {
    Sine,
    Square,
    Saw,
    Triangle,
}

// --- Struct for Shared Synthesizer Control State ---
#[derive(Debug, Clone)]
pub struct SynthControl {
    pub frequency: f32,
    pub waveform: ActiveWaveform,
    pub amplitude: f32, // Added amplitude control
}

#[derive(Clone)] // Clone is cheap due to Arc
pub struct Synthesizer {
    pub control: Arc<Mutex<SynthControl>>,
}

impl Synthesizer {
    /// Creates a new Synthesizer controller.
    pub fn new(initial_frequency: f32, initial_waveform: ActiveWaveform, initial_amplitude: f32) -> Self {
        Synthesizer {
            control: Arc::new(Mutex::new(SynthControl {
                frequency: initial_frequency,
                waveform: initial_waveform,
                amplitude: initial_amplitude.clamp(0.0, 1.0), // Clamp amplitude
            })),
        }
    }

    /// Sets the synthesizer's frequency.
    pub fn set_frequency(&self, frequency: f32) {
        // Lock, update, drop lock automatically
        if let Ok(mut control) = self.control.lock() {
             control.frequency = frequency.max(0.0);
        } else {
            eprintln!("Error setting frequency: Mutex poisoned");
        }
    }

    /// Sets the synthesizer's active waveform.
    pub fn set_waveform(&self, waveform: ActiveWaveform) {
        if let Ok(mut control) = self.control.lock() {
            control.waveform = waveform;
        } else {
             eprintln!("Error setting waveform: Mutex poisoned");
        }
    }

    /// Sets the synthesizer's amplitude (clamped between 0.0 and 1.0).
    pub fn set_amplitude(&self, amplitude: f32) {
        if let Ok(mut control) = self.control.lock() {
            control.amplitude = amplitude.clamp(0.0, 1.0);
        } else {
            eprintln!("Error setting amplitude: Mutex poisoned");
        }
    }

    // Internal helper to get a clone of the control Arc for the audio thread
    pub fn get_control_clone(&self) -> Arc<Mutex<SynthControl>> {
        Arc::clone(&self.control)
    }
}