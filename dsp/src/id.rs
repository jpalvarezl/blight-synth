pub type VoiceId = u32;
pub type SampleId = u32;
pub type InstrumentId = u32;
pub type EffectChainId = u32;
pub type EffectId = u32;
pub type EnvelopeId = u32;

/// Stable identity for a single sounding note/event, distinct from the MIDI
/// pitch that a voice renders.
///
/// Separating identity from pitch is what lets a polyphonic instrument address
/// individual voices: two overlapping notes at the same pitch carry different
/// [`NoteId`]s and therefore occupy different voices, and a targeted note-off
/// releases only the voice that owns the matching identity rather than every
/// sounding voice. Hosts that do not yet have a richer event source (#145) —
/// such as the monophonic tracker path — derive a per-pitch identity through
/// [`NoteId::from_pitch`].
///
/// The wrapped counter is deliberately a plain integer so the type is `Copy`
/// and comparisons are branch-free and allocation-free on the audio thread.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NoteId(pub u64);

impl NoteId {
    /// Derives a stable identity from a MIDI pitch for hosts that address at
    /// most one sounding note per pitch and have no richer event identity yet.
    #[inline]
    pub const fn from_pitch(pitch: u8) -> Self {
        NoteId(pitch as u64)
    }

    /// Returns the raw identity value.
    #[inline]
    pub const fn get(self) -> u64 {
        self.0
    }
}
