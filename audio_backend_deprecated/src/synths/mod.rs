mod oscillator;
pub mod synthesizer;

mod adsr;
pub mod voice;
pub mod voice_manager;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Waveform {
    Sine,
    Square,
    Triangle,
    Sawtooth,
}
