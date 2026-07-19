# blight-synth

blight-synth is an experimental composition environment and modular real-time sound engine built in Rust. The current repository includes a tracker-based composition source and egui debug interface while a host-independent engine and future Svelte/TypeScript frontend are being designed.

## Documentation and roadmap

- Start at [`docs/README.md`](docs/README.md) for the committed, Obsidian-compatible knowledge base.
- Coding agents should read [`AGENTS.md`](AGENTS.md) for the targeted context-loading and parallel-work protocol.
- GitHub issue [#144](https://github.com/jpalvarezl/blight-synth/issues/144) is the canonical roadmap; [`docs/work/burndown.md`](docs/work/burndown.md) is its generated offline snapshot.
- [`docs/spec/current-product.md`](docs/spec/current-product.md) records the current product direction and open composition questions.
- [`docs/architecture/product-topology.md`](docs/architecture/product-topology.md) documents standalone and optional-plugin boundaries, while [ADR 0001](docs/decisions/0001-product-and-host-priorities.md) records the accepted decision.
- [`docs/architecture/crate-dependency-graph.md`](docs/architecture/crate-dependency-graph.md) records the current CI-enforced M0 crate and feature boundaries.

GitHub Issues own live task status. Specifications, architecture contracts, and accepted decisions live in `docs/`; code and tests describe current implementation behavior.

## Project Structure

- `dsp/` — Portable DSP data and processing primitives: instruments, voices, effects, factories, and immutable sample data. It has no file/platform loader dependencies.
- `engine/` — Host-independent instrument runtime, planar mixer, and master-effects renderer. It owns no composition documents, devices, files, network sockets, or UI.
- `audio_backend/` — Tracker/offline adapters plus an optional default `standalone` feature containing CPAL, command transport, OSC, metering, and the temporary current-thread Tokio runtime.
- `sequencer/` — Current tracker document, timing, and `Song -> Chain -> Phrase` composition model.
- `tracker_gui/` — Current egui debug/reference interface; it does not dictate the future composition UI.
- `utils/` — Music theory utilities such as notes and scales.
- `os_dls/` — macOS DLS parsing and sample-resource support.
- `assets/` — Data files for notes and other resources.
- `scripts/` — Validation, smoke-test, and documentation tooling.

## Development checks

The hardware-free CI baseline has direct local equivalents:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo test -p audio_backend --no-default-features --all-targets
python3 scripts/check_architecture.py
python3 scripts/check_rt_logging.py
python3 scripts/docs/check_docs.py
```

See [`scripts/README.md`](scripts/README.md) for roadmap-generator and manual audio/OSC checks. TypeScript checks will be added when the production `gui/` workspace exists.

## Offline song rendering and golden regression tests

Render the supported synth/drum reference songs without opening an audio device:

```bash
scripts/render_reference_songs.sh
```

Or render one JSON song:

```bash
cargo run -p audio_backend --example render_song -- \
  calibration.json target/offline-renders/calibration.wav
```

On macOS, listen with `afplay target/offline-renders/calibration.wav`. CI compares canonical PCM SHA-256 references for the synth/drum songs. Intentional audio changes use the explicit reference-update workflow documented in [`docs/architecture/offline-render-contract.md`](docs/architecture/offline-render-contract.md); normal tests never rewrite references.

## audio_backend Architecture

The `audio_backend` crate is responsible for all audio processing and device management. Its architecture is modular and consists of the following main components:

```mermaid
graph TB
    %% External Input
    UI[User/GUI Input] -->|Commands| BA[BlightAudio<br/>NRT Thread]
    
    %% Command Flow
    BA -->|Command Queue<br/>Lock-free SPSC| AP[AudioProcessor<br/>RT Thread]
    
    %% Command Processing
    AP -->|Commands| CMD[Command Processor]
    CMD -->|Instrument/Master Commands| ENG[Engine]
    CMD -->|Transport/Song Commands| P[Player/Sequencer]
    
    %% Song Data Flow
    P -->|Read Song Data| SD[(Song Data<br/>- Arrangement<br/>- Chains<br/>- Phrases<br/>- Events)]
    P -->|Tracker Operations| TA[TrackerEngineAdapter]
    TA -->|Engine Commands / Render| ENG
    
    %% Instrument Management
    ENG -->|Manages| INST[Instruments]
    
    %% Instrument Types
    INST --> MO[MonophonicOscillator<br/>Single Voice]
    INST --> PO[PolyphonicOscillator<br/>Multiple Voices]
    INST --> PERC[Percussion]
    
    %% Percussion Instruments
    PERC --> KD[KickDrum<br/>- Oscillator<br/>- Pitch Envelope<br/>- ADSR]
    PERC --> SD2[SnareDrum<br/>- Noise<br/>- Oscillator<br/>- ADSR]
    PERC --> HH[HiHat<br/>- Noise<br/>- Filter<br/>- ADSR]
    
    %% Voice Processing
    PO -->|Voice Pool| V[Voice<br/>- Note ID<br/>- Frequency<br/>- Velocity]
    MO --> V
    KD --> V
    SD2 --> V
    HH --> V
    
    %% Synthesis Components
    V --> SC[Synth Components]
    SC --> SN[SynthNode]
    SC --> ENV[Envelopes]
    
    %% SynthNode Types
    SN --> OSC[OscillatorNode<br/>- Sine<br/>- Square<br/>- Saw<br/>- Triangle<br/>- Pulse]
    SN --> DRUM[DrumNodes<br/>- KickDrumNode<br/>- SnareDrumNode<br/>- HiHatNode]
    SN --> SP[SamplePlayerNode]
    
    %% Envelope Types
    ENV --> ADSR[ADSR Envelope<br/>- Attack<br/>- Decay<br/>- Sustain<br/>- Release]
    ENV --> PENV[Pitch Envelope<br/>- Depth<br/>- Time Parameters]
    
    %% Audio Processing Pipeline
    SC -->|Generate| MB[Mono Buffer]
    ADSR -->|Amplitude| MB
    PENV -->|Pitch Mod| MB
    
    %% Effects Chain
    MB -->|Process| MEC[MonoEffectChain<br/>Per-Voice Effects]
    MEC --> MEF[MonoEffects<br/>- Gain<br/>- Delay<br/>- Filter<br/>- Reverb<br/>- Distortion<br/>- BitCrusher]
    
    %% Stereo Processing
    MEF -->|Panning| SB[Stereo Buffers<br/>Left + Right]
    SB -->|Sum all voices| MIX[Mixer]
    
    %% Master Effects
    MIX -->|Master Bus| SEC[StereoEffectChain<br/>Master Effects]
    SEC --> SEF[StereoEffects<br/>- Gain<br/>- Delay<br/>- Reverb<br/>- Compressor]
    
    %% Output
    SEF -->|Process| FB[Final Buffers]
    FB -->|Interleave| OB[Output Buffer]
    OB -->|CPAL Stream| AUDIO[Audio Device/Speaker]
    
    %% Resource Management (NRT)
    BA -.->|Creates| VF[VoiceFactory]
    BA -.->|Creates| IF[InstrumentFactory]
    BA -.->|Creates| EF[EffectFactory]
    BA -.->|Manages| RM[ResourceManager<br/>- Samples<br/>- Wavetables]
    
    %% State Management
    AP -->|State Query| SM[StateManager<br/>- Voice States<br/>- Effect States]
    SM -.->|Reports| BA
    
    style BA fill:#f9f,stroke:#333,stroke-width:2px
    style AP fill:#9ff,stroke:#333,stroke-width:2px
    style AUDIO fill:#9f9,stroke:#333,stroke-width:2px
    style V fill:#ff9,stroke:#333,stroke-width:2px
    style ADSR fill:#faf,stroke:#333,stroke-width:2px
    style PENV fill:#faf,stroke:#333,stroke-width:2px
```

## Main Dependencies

- [cpal](https://github.com/RustAudio/cpal): Cross-platform audio I/O in Rust.

## Details

- Audio streaming is managed by the `audio_backend` crate, which sets up and runs the audio stream using `cpal`.
- The synthesis engine (within `audio_backend`) supports multiple waveforms and envelopes, and is designed for extensibility.
- The `sequencer` provides timing and pattern-based composition capabilities like traditional trackers.

### Audio backend API (quick start)

- The standalone/tracker queue has domain-specific payloads:
  - `Command::Transport(TransportCmd)` — adapter transport.
  - `Command::Sequencer(SequencerCmd)` — song loading/playback.
  - `Command::Instrument(InstrumentCmd)` — instrument creation, notes, synth control, and instrument-owned effects.
  - `Command::Mixer(MixerCmd)` — master mixer/effect pipeline only.
- `InstrumentCmd` and `MixerCmd` are owned by `engine` and compatibility-re-exported by `audio_backend`.
- Subcommands convert into the queue envelope with `.into()`.

Direct instrument control through the current standalone adapter:

```rust
let mut audio = audio_backend::BlightAudio::new().unwrap();
let instrument_id = 1;
let instrument = audio
    .get_instrument_factory()
    .create_simple_oscillator(instrument_id, 0.0);

audio.send_command(audio_backend::InstrumentCmd::AddInstrument { instrument }.into());
// Current adapter rendering is transport-gated; M1 will separate live rendering from tracker transport.
audio.send_command(audio_backend::TransportCmd::PlayLastSong.into());
audio.send_command(
    audio_backend::InstrumentCmd::NoteOn {
        instrument_id,
        note: 60,
        velocity: 127,
    }
    .into(),
);
audio.send_command(audio_backend::InstrumentCmd::NoteOff { instrument_id }.into());
```

Tracker mode (sequencer-driven):

```rust
use std::sync::Arc;
use audio_backend::{SequencerCmd, TransportCmd};
let song = Arc::new(sequencer::models::Song::new("My Song"));
let mut audio = audio_backend::BlightAudio::with_song(song.clone()).unwrap();
audio.send_command(SequencerCmd::PlaySong { song }.into());
audio.send_command(TransportCmd::StopSong.into());
```
