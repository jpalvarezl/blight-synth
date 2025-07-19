// use sequencer::models::*;
use sequencer::{cli::CliArgs, models::{EffectType, Pattern, PatternEvent, Sequencer}};

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
        sequences: vec![Pattern {
            name: "Sequence 1".into(),
            instrument_id: 1,
            events: sequence_events,
        }],
    };

    args.write_file(&sequencer).expect("Failed to write file");
}
