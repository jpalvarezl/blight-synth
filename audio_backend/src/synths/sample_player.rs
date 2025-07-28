use crate::SynthNode;

pub struct SamplePlayerNode {}

impl SamplePlayerNode {
    pub fn new() -> Self {
        Self {}
    }
}

impl SynthNode for SamplePlayerNode {
    fn process(&mut self, output_buffer: &mut [f32], sample_rate: f32) {
        todo!()
    }

    fn note_on(&mut self, note: u8, velocity: u8) {
        todo!()
    }

    fn note_off(&mut self) {
        todo!()
    }

    fn is_active(&self) -> bool {
        todo!()
    }
}