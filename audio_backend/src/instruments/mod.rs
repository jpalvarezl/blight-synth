mod hihat;
mod monophonic_osc;
mod polyphonic_osc;
mod synth_nodes;
mod kick_drum;

pub use hihat::*;
pub use monophonic_osc::*;
pub use polyphonic_osc::*;
pub use synth_nodes::*;
pub use kick_drum::*;

use crate::{id::NoteId, SynthNode, Voice};

struct VoiceSlot<S: SynthNode> {
    inner: Voice<S>,
    note_id: Option<NoteId>, // This is the MIDI value of the note we use for identifying which voice is playing it.
}
