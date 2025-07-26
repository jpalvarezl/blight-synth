/// A trait for any sound-generating algorithm (e.g., oscillator, sampler).
/// A `SynthNode` is monophonic and represents the raw sound source.
pub trait SynthNode: Send + Sync {
    /// Renders the next block of audio into the provided mono buffer.
    fn process(&mut self, output_buffer: &mut [f32], sample_rate: f32);

    /// Triggers the note on event, typically setting the frequency.
    fn note_on(&mut self, note: u8, velocity: u8);

    /// Triggers the note off event. For some nodes, this may do nothing.
    fn note_off(&mut self);

    /// Indicates if the sound-generating node is still active.
    /// For continuous sources like oscillators, this should always return `true`.
    /// For finite sources like sample players, this should return `false`
    /// once the sample has finished playing.
    fn is_active(&self) -> bool;
}