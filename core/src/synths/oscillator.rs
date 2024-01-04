use std::ops::Mul;

pub enum Waveform {
    Sine,
    Square,
    Saw,
    Triangle,
}

pub struct Oscillator {
    // pub sample_rate: u32,
    pub waveform: Waveform,
    // pub current_sample_index: u32,
    pub frequency_hz: f32,
}

impl Oscillator {

    // pub(crate) fn set_waveform(&mut self, waveform: Waveform) {
    //     self.waveform = waveform;
    // }

    // fn is_multiple_of_freq_above_nyquist(&self, multiple: f32) -> bool {
    //     self.frequency_hz * multiple > self.sample_rate / 2.0
    // }

    // fn generative_waveform(&mut self, harmonic_index_increment: i32, gain_exponent: f32) -> f32 {
    //     let mut output = 0.0;
    //     let mut i = 1;
    //     while !self.is_multiple_of_freq_above_nyquist(i as f32) {
    //         let gain = 1.0 / (i as f32).powf(gain_exponent);
    //         output += gain * self.calculate_sine_output_from_freq(self.frequency_hz * i as f32);
    //         i += harmonic_index_increment;
    //     }
    //     output
    // }

    // fn square_wave(&mut self) -> f32 {
    //     self.generative_waveform(2, 1.0)
    // }

    // fn saw_wave(&mut self) -> f32 {
    //     self.generative_waveform(1, 1.0)
    // }

    // fn triangle_wave(&mut self) -> f32 {
    //     self.generative_waveform(2, 2.0)
    // }

    fn sine_wave(&self, time: f32) -> f32 {
        // 2πft
        (std::f32::consts::TAU * self.frequency_hz * time as f32).sin()
    }

    // "time" is the "current sample index" divided by the "sample rate"
    pub(crate) fn tick(&self, time: f32) -> f32 {
        // self.advance_sample();
        // println!("Oscillator tick. Sample rate: {}, time: {}", self.sample_rate, time);
        match self.waveform {
            Waveform::Sine => self.sine_wave(time),
            // Waveform::Square => self.square_wave(),
            // Waveform::Saw => self.saw_wave(),
            // Waveform::Triangle => self.triangle_wave(),
            _ => self.sine_wave(time),
        }
    }
}
