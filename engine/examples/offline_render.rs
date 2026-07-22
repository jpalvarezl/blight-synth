use dsp::InstrumentFactory;
use engine::{Engine, InstrumentCmd};

const SAMPLE_RATE: f32 = 48_000.0;
const BLOCK_SIZE: usize = 256;
const BLOCK_COUNT: usize = 100;

fn main() {
    let instrument_id = 1;
    let factory = InstrumentFactory::new(SAMPLE_RATE);
    let mut engine = Engine::new();
    let _ = engine.handle_command(
        InstrumentCmd::AddInstrument {
            instrument: factory.create_simple_oscillator(instrument_id, 0.0),
        }
        .into(),
    );
    let _ = engine.handle_command(
        InstrumentCmd::NoteOn {
            instrument_id,
            note: 60,
            velocity: 127,
        }
        .into(),
    );

    let mut peak = 0.0_f32;
    let mut checksum = 0.0_f64;
    for _ in 0..BLOCK_COUNT {
        let mut left = [0.0; BLOCK_SIZE];
        let mut right = [0.0; BLOCK_SIZE];
        engine.process(&mut left, &mut right, SAMPLE_RATE);
        for (&left, &right) in left.iter().zip(&right) {
            peak = peak.max(left.abs()).max(right.abs());
            checksum += f64::from(left) + f64::from(right);
        }
    }

    let _ = engine.handle_command(InstrumentCmd::NoteOff { instrument_id }.into());
    println!(
        "rendered {} stereo frames without a host; peak={peak:.6}, checksum={checksum:.6}",
        BLOCK_SIZE * BLOCK_COUNT
    );
}
