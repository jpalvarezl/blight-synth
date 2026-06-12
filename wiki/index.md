---
tags: [index]
sources: []
last-updated: 2025-01-15
---

# blight-synth Wiki

Modular Rust synthesizer (Cargo workspace + Tauri/egui GUI). See [[concepts/overview]].

## Entities
## Entities

- [[entities/blight-audio]] — NRT audio backend API
- [[entities/audio-processor]] — RT audio callback processor
- [[entities/meter-state]] — lock-free stereo metering
- [[entities/osc-server]] — UDP OSC server
- [[entities/dsp-core-bin]] — standalone `dsp-core` binary
## Concepts
## Concepts

- [[concepts/overview]]
- [[concepts/m1-dsp-core-standalone]] — M1 milestone (standalone DSP core + OSC)
- [[concepts/osc-address-space]] — OSC addresses & sockets
- [[concepts/rt-nrt-metering]] — RT→NRT lock-free metering
- [[concepts/osc-review-feedback-125]] — resolved PR #125 review notes
## Sources
## Sources

- [[sources/osc-validation-scripts]] — OSC/DSP-core validation scripts
