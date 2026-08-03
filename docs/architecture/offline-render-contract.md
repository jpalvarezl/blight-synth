---
title: Offline Render and Golden Reference Contract
summary: Canonical hardware-free render settings, regression policy, and intentional reference-update workflow.
status: current
updated: 2026-08-02
issues: [132, 134, 155, 164, 204]
---

# Offline Render and Golden Reference Contract

Offline song renders are the end-to-end regression gate for JSON loading, tracker interpretation, hydration, engine command dispatch, synthesized instruments, effects, mixing, and PCM encoding. The initial references characterize current behavior; they do not declare every current timing/tail behavior correct forever.

## Canonical render

| Property | Value |
|---|---:|
| Sample rate | 48,000 Hz |
| Block size | 256 stereo frames |
| Maximum duration | 120 seconds |
| Hashed format | Interleaved signed PCM16 little-endian |
| Dither/normalization | None |
| Instrument mix order | Ascending stable `InstrumentId` |
| Random sources | Fixed implementation seeds |

SHA-256 covers only canonical PCM bytes, not WAV headers or filesystem metadata. The manifest also records frame count, per-channel peak/RMS, and pre-quantization clipping count. The engine uses a preallocated, sorted `Vec<InstrumentSlot>` so render order is explicit and cache-friendly; #137 owns the final hard-capacity/overflow policy.

CPAL is intentionally absent from this path because it streams to real devices rather than encoding offline files. Hound is used only in the host/I/O layer to wrap the already-rendered canonical PCM in a WAV container for listening.

## Characterization policy

The committed manifest is marked `characterization` and records known limitations:

- #132 — the offline harness still needs an explicit lifecycle/post-transport
  tail-duration policy;
- #136 — mixer gain staging and clipping remain characterization behavior.

Issue #204 removed the former sample-accuracy and stopped-rendering limitations:
tracker/live events now use exact current-block offsets, and the engine continues
to render voices/effects while tracker transport is stopped. The canonical song
harness still ends after the complete block in which non-looping transport ends;
it does not yet append an independently configured tail duration.

An unrelated change must not alter a reference. An intentional timing, synthesis, envelope, effect, routing, or mixer correction may update references, but the PR must explain the change and include/listen to generated WAVs. The gate is strict about unexplained changes, not about preserving known bugs.

## Reference update

Normal tests never rewrite expected output. The only supported update command is explicit:

```bash
cargo run -p audio_backend --example update_offline_references -- --update-reference
```

It renders every supported reference song twice, rejects nondeterminism, writes review WAVs under `target/offline-renders/`, and updates `audio_backend/tests/golden/offline_render_manifest.json`. Historical files that no longer satisfy the current schema are excluded rather than silently retrofitted.

Before committing an update:

1. inspect the old/new hash, frame count, peak, RMS, and clipping count;
2. listen to every changed WAV;
3. link the issue that intentionally changes audio behavior;
4. include the manifest diff in review.

## Issue #204 reference update

The #204 integration intentionally replaces callback-start tracker attacks with
sample-offset event application. It also adopts the half-open timing rule: the
song-ending tick that previously counted at a block end is retained for the
next block at offset zero. Both canonical references therefore gain one
256-frame block, and segmented note/release placement intentionally changes PCM,
peak/RMS, and clipping metrics. The update tool generated deterministic review
WAVs under `target/offline-renders/`; generation is not a claim that a human
audition occurred.

## Platform policy

The manifest records the platform where it was updated. Exact PCM/reference equality is required on that canonical platform. CI demonstrated that synthesized drum PCM can differ at the byte level between macOS arm64 and Linux x86-64 even when frame count, clipping count, peak, and RMS are identical. Non-canonical platforms therefore require repeated-render determinism plus exact structure/clipping and tight peak/RMS tolerances. Add a separate reviewed platform hash before claiming byte-identical cross-platform PCM; do not silently replace the canonical reference.

## Known baseline observations

The first baseline exposes pre-quantization clipping in synthesized multi-instrument songs. That is retained as diagnostic evidence, not normalized away. Gain staging and mixer behavior should be corrected deliberately under the routing/mixer work, with an explained reference update.
