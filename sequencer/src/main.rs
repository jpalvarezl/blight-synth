// use sequencer::models::*;
use sequencer::cli::CliArgs;
use sequencer::models::{EffectType, Pattern, PatternEvent, Sequencer};

fn main() {
    let args = CliArgs::parse_arguments();

    let sequence_events = vec![
        PatternEvent {
            note: 60,
            volume: 100,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
        PatternEvent {
            note: 60,
            volume: 0,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        },
        PatternEvent {
            note: 60,
            volume: 127,
            effect: EffectType::Arpeggio,
            effect_param: 1,
        },
    ];

    let sequencer = Sequencer {
        sequences: vec![Pattern::from_events(sequence_events)],
    };

    args.write_file(&sequencer).expect("Failed to write file");
}
