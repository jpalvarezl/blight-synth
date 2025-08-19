mod monophonic_osc;
mod synth_nodes;

pub use monophonic_osc::*;
pub use synth_nodes::*;

use crate::{SynthNode, Voice};

struct VoiceSlot<S: SynthNode> {
    synth_node: Voice<S>,
    // note_id: NoteId,
}
