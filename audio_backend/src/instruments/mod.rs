mod hihat;
mod kick_drum;
mod monophonic_osc;
mod polyphonic_osc;
mod snare_drum;
mod synth_nodes;

pub use hihat::*;
pub use kick_drum::*;
pub use monophonic_osc::*;
pub use polyphonic_osc::*;
pub use snare_drum::*;
pub use synth_nodes::*;

use crate::{id::NoteId, SynthNode, Voice};

struct VoiceSlot<S: SynthNode> {
    inner: Voice<S>,
    note_id: Option<NoteId>, // This is the MIDI value of the note we use for identifying which voice is playing it.
}
