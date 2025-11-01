use crate::MonoEffect;

pub struct MoogLadder {
    cutoff: f32,
    resonance: f32,
    sample_rate: f32,
    g: f32,
    y: [f32; 5], // stages y1..y4 + input
}

impl MoogLadder {
    /// Create a new Moog Ladder filter effect.
    /// `cutoff` is in Hz, min value is 20 Hz and max is ~ sample_rate / 2. So around 20k
    /// `resonance` is typically between 0.0 and 4.0
    pub fn new(sample_rate: f32, cutoff: f32, resonance: f32) -> Self {
        let mut f = Self {
            cutoff,
            resonance,
            sample_rate,
            g: 0.0,
            y: [0.0; 5],
        };
        f.update_coefficients();
        f
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff;
        self.update_coefficients();
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance.clamp(0.0, 4.0);
    }

    fn update_coefficients(&mut self) {
        let f = (std::f32::consts::PI * self.cutoff / self.sample_rate).tanh();
        self.g = f;
    }

    fn process_private(&mut self, input: f32) -> f32 {
        let k = self.resonance;
        let x = (input - k * self.y[4]).tanh();

        // Four cascaded one-pole filters
        self.y[0] += self.g * (x - self.y[0]);
        self.y[1] += self.g * (self.y[0] - self.y[1]);
        self.y[2] += self.g * (self.y[1] - self.y[2]);
        self.y[3] += self.g * (self.y[2] - self.y[3]);
        self.y[4] = self.y[3]; // Output

        self.y[4]
    }
}

impl MonoEffect for MoogLadder {
    fn process(&mut self, buffer: &mut [f32], _sample_rate: f32) {
        for sample in buffer.iter_mut() {
            *sample = self.process_private(*sample);
        }
    }

    fn set_parameter(&mut self, param_index: u32, value: f32) {
        match param_index {
            0 => self.set_cutoff(value),
            1 => self.set_resonance(value),
            _ => {}
        }
    }
}
