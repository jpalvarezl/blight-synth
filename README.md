# blight-synth

blight-synth is a modular synthesizer application built in Rust, featuring a dedicated audio backend and a graphical frontend interface. The project is organized into several Rust crates and a frontend GUI (built with Tauri), enabling real-time audio synthesis and user interaction.

## Project Structure

- `audio_backend/` — Core audio engine. Handles audio device management, synthesis, streaming, and processing. Written in Rust.
- `sequencer/` — Sequencing and timing engine for pattern-based music composition. Written in Rust.
- `utils/` — Music theory utilities (notes, scales, etc.) for use by the synth engine. Written in Rust.
- `frontend/` — Graphical User Interface (GUI) for operating the synth, built with Tauri. (Details are in the folder; not covered here.)
- `assets/` — Data files for notes and other resources.
- `scripts/` — Utility scripts (e.g., for generating note data).

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
    CMD -->|Instrument Commands| IM[InstrumentManager]
    CMD -->|Sequencer Commands| P[Player/Sequencer]
    
    %% Song Data Flow
    P -->|Read Song Data| SD[(Song Data<br/>- Arrangement<br/>- Chains<br/>- Phrases<br/>- Events)]
    P -->|Note Events| TS[TrackerSynthesizer]
    
    %% Instrument Management
    TS -->|Note On/Off/Modify| IM
    IM -->|Manages| INST[Instruments]
    
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

- Unified command enum with domain subtypes:
  - Command::Transport(TransportCmd)
  - Command::Sequencer(SequencerCmd)
  - Command::Synth(SynthCmd)
  - Command::Mixer(MixerCmd)
- You can send subcommands directly using From, e.g. SynthCmd::PlayNote { ... }.into().

Instrument mode (no tracker feature at runtime):

```rust
// create engine
let mut audio = audio_backend::BlightAudio::new().unwrap();
// play a note
use audio_backend::SynthCmd;
audio.send_command(SynthCmd::PlayNote { voice: audio.get_voice_factory().create_voice(0, audio_backend::InstrumentDefinition::Oscillator, 0.0), note: 60, velocity: 127 }.into());
// stop
audio.send_command(SynthCmd::StopNote { voice_id: 0 }.into());
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

Feature flags
- Default features enable tracker integration. Non-tracker examples use `--no-default-features`.
- Example:
  - cargo run -p audio_backend --example cycle_waveforms --no-default-features
