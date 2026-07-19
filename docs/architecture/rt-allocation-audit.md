---
title: RT Allocation Audit Harness
summary: Test-only allocator instrumentation for detecting heap allocation and destruction during prepared Engine processing.
status: current
updated: 2026-07-19
issues: [133, 172, 174]
---

# RT Allocation Audit Harness

`engine/tests/rt_allocations.rs` installs a test-binary-local global allocator wrapper around `System`. Thread-local counters are enabled only around the operation under measurement, so test setup, factory construction, and assertion formatting are excluded. Measurement guards restore prior tracking/counters on scope exit (including unwinding); nested scopes are tested and contribute their counts to the outer scope.

## Current representative path

The zero-heap test prepares an oscillator and buffers before measurement, warms any lazy process-global state, then measures:

```text
InstrumentCmd::NoteOn
InstrumentCmd::PassOnSynthCmd(SetWaveform)
Engine::process(256 stereo frames)
InstrumentCmd::NoteOff
```

The test requires zero:

- allocations;
- reallocations;
- deallocations.

This covers a representative prepared note/parameter/render path without CPAL or a device.

## Harness self-test

A deliberately allocating test instrument creates and drops a `Vec` during `process`. The harness must report both allocation and deallocation. This prevents a broken/no-op allocator counter from making the zero-heap test pass falsely.

## Usage

```bash
cargo test -p engine --test rt_allocations -- --nocapture
```

The test executable has its own global allocator and does not alter allocator behavior in production crates or other test binaries.

## Limitations and extension

- Counters are thread-local; they measure the thread executing the callback simulation. Callback code must not delegate work to another thread, which is separately prohibited by the RT contract.
- The current test covers steady-state prepared operations, not structural replacement or retirement. #174 must add swap/retire/drop-thread probes.
- A passing representative path does not prove every DSP node/effect is allocation-free. Add focused prepared fixtures when new node categories or callback operations are introduced.
- Lazy global initialization must be warmed before measurement or explicitly classified as a violation if it can happen in a real first callback.
- Debug callback diagnostics should be disabled when using this harness to make RT-performance claims; #175 enforces release compile-out and logging policy.
