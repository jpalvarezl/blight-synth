use crate::Envelope;

pub struct PitchEnvelope {
   adsr: Envelope,
   start_freq: f32,
   end_freq: f32,
}

impl PitchEnvelope {
   pub fn new(start_freq: f32, end_freq: f32, adsr: Envelope) -> Self {
       Self {
           adsr,
           start_freq,
           end_freq,
       }
   }

   pub fn note_on(&mut self) {
       self.adsr.gate(true);
   }

   pub fn note_off(&mut self) {
       self.adsr.gate(false);
   }

    #[inline(always)]
    pub fn next_freq(&mut self) -> f32 {
        let env_val = self.adsr.process();
        // Interpolate between start and end
        self.start_freq * (1.0 - env_val) + self.end_freq * env_val
    }
   pub fn is_active(&self) -> bool {
       self.adsr.is_active()
   }
    
}