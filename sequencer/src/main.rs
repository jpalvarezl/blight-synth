// use sequencer::models::*;
use sequencer::{cli::CliArgs, models::{Sequence, SequenceEvent, Sequencer}};

fn main() {
    let args = CliArgs::parse_arguments();

    let sequence_events = vec![
        SequenceEvent::NoteOn {
            midi_value: 60,
            velocity: 100,
            start_time: 0,
        },
        SequenceEvent::NoteOff { midi_value: 60 },
        SequenceEvent::CC {
            controller: 1,
            value: 127,
            start_time: 500,
        },
    ];

    let sequencer = Sequencer {
        sequences: vec![Sequence {
            name: "Sequence 1".into(),
            instrument_id: 1,
            events: sequence_events,
        }],
    };

    args.write_file(&sequencer).expect("Failed to write file");
}
