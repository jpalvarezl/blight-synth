//! Compact identities used by the prepared DSP and engine runtime.
//!
//! Each identity has its own type so unrelated engine objects cannot be mixed:
//!
//! ```compile_fail
//! use dsp::id::{EffectId, InstrumentId};
//!
//! let instrument = InstrumentId::from_raw(7);
//! let _effect: EffectId = instrument;
//! ```
//!
//! Raw constructors are intentionally explicit. There are no conversions
//! between ID domains.

macro_rules! define_id {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(u32);

        impl $name {
            /// Creates an ID from its current raw runtime value.
            #[inline]
            #[must_use]
            pub const fn from_raw(raw: u32) -> Self {
                Self(raw)
            }

            /// Returns this ID's current raw runtime value.
            #[inline]
            #[must_use]
            pub const fn raw(self) -> u32 {
                self.0
            }
        }
    };
}

define_id!(VoiceId, "Identity of one prepared DSP voice.");
define_id!(SampleId, "Identity of one decoded sample resource.");
define_id!(InstrumentId, "Identity of one engine instrument instance.");
define_id!(EffectChainId, "Identity of one prepared effect chain.");
define_id!(EffectId, "Identity of one effect instance or slot.");
define_id!(EnvelopeId, "Identity of one envelope within a synth voice.");

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

/// A complete note-on payload, bundling the note's stable identity, the MIDI
/// pitch to render, and the velocity so they always travel together and cannot
/// be mismatched at a call site.
///
/// `pitch` is the MIDI note number to render; the microtonal/tuning axis lives
/// here and is owned by the future event contract (#134/#145). `id` is the
/// stable voice identity (see [`NoteId`]) that lets a polyphonic instrument
/// address individual voices and target note-off precisely. `velocity` is the
/// MIDI note-on velocity forwarded to the underlying voice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoteEvent {
    /// Stable identity of the sounding note.
    pub id: NoteId,
    /// MIDI pitch to render (microtonal/tuning axis, future #134/#145).
    pub pitch: u8,
    /// MIDI note-on velocity.
    pub velocity: u8,
}

impl NoteEvent {
    /// Builds a note-on event for hosts without a richer identity source by
    /// deriving the identity from the pitch via [`NoteId::from_pitch`].
    #[inline]
    pub const fn from_pitch(pitch: u8, velocity: u8) -> Self {
        Self {
            id: NoteId::from_pitch(pitch),
            pitch,
            velocity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashSet, mem};

    macro_rules! assert_id_contract {
        ($type:ty) => {{
            const ID: $type = <$type>::from_raw(42);
            assert_eq!(ID.raw(), 42);
            assert_eq!(mem::size_of::<$type>(), mem::size_of::<u32>());
            assert_eq!(mem::align_of::<$type>(), mem::align_of::<u32>());

            let lower = <$type>::from_raw(1);
            let higher = <$type>::from_raw(2);
            assert!(lower < higher);
            let mut ids = HashSet::new();
            ids.insert(lower);
            assert!(ids.contains(&lower));
        }};
    }

    #[test]
    fn runtime_ids_are_compact_copy_ordered_hashable_numeric_values() {
        assert_id_contract!(VoiceId);
        assert_id_contract!(SampleId);
        assert_id_contract!(InstrumentId);
        assert_id_contract!(EffectChainId);
        assert_id_contract!(EffectId);
        assert_id_contract!(EnvelopeId);
    }
}
