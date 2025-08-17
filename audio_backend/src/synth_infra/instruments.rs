use crate::id::VoiceId;

/// A trait for a complete instrument, which is responsible for managing
/// its own voices and polyphony according to its specific behavior.
pub trait InstrumentTrait: Send + Sync {
    /// Handles a note-on event for this instrument.
    /// It will decide whether to create a new voice, re-trigger an existing one, etc.
    fn note_on(&mut self, note: u8, velocity: u8, voice_id: VoiceId);

    /// Handles a note-off event for a specific voice ID.
    fn note_off(&mut self, voice_id: VoiceId);

    /// Processes all active voices for this instrument, adding their
    /// output to the main stereo buffers.
    fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32]);

    // We can also add methods for parameter changes, etc.
    // fn set_parameter(&mut self, index: u32, value: f32);
}
