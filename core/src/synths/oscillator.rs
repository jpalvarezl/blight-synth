use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub enum Waveform {
    Sine,
    Square,
    Saw,
    Triangle,
    Silence,
}

impl Display for Waveform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let waveform = match self {
            Waveform::Sine => "Sine",
            Waveform::Square => "Square",
            Waveform::Saw => "Saw",
            Waveform::Triangle => "Triangle",
            Waveform::Silence => "Silence",
        };
        write!(f, "{}", waveform)
    }
}

#[derive(Debug)]
pub struct Oscillator {
    pub waveform: Waveform,
    pub frequency_hz: f32,
}

impl Oscillator {

    fn triangle_wave(&self, time: f32) -> f32 {
        // 2πft
        (std::f32::consts::TAU * self.frequency_hz * time).sin().asin()
    }

    fn sawtooth_wave(&self, time: f32) -> f32 {
        // 2πft
        (std::f32::consts::TAU * self.frequency_hz * time).sin().atan()
    }

    fn square_wave(&self, time: f32) -> f32 {
        // 2πft
        (std::f32::consts::TAU * self.frequency_hz * time).sin().signum()
    }

    fn sine_wave(&self, time: f32) -> f32 {
        // 2πft
        (std::f32::consts::TAU * self.frequency_hz * time).sin()
    }

    // "time" is the "current sample index" divided by the "sample rate"
    pub(crate) fn tick(&self, time: f32) -> f32 {
        // self.advance_sample();
        match self.waveform {
            Waveform::Sine => self.sine_wave(time),
            Waveform::Square => self.square_wave(time),
            Waveform::Saw => self.sawtooth_wave(time),
            Waveform::Triangle => self.triangle_wave(time),
            Waveform::Silence => 0.0,
        }
    }
}
