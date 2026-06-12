---
tags: [osc, review, history]
sources: ["https://github.com/jpalvarezl/blight-synth/pull/125"]
last-updated: 2025-05-11
---

# PR #125 Review Feedback (resolved)

Copilot review on PR #125 was written against an **older** design (`SharedAudioState`/`state.rs` atomics) that was dropped. Resolution in commit `7a74f83`:

| Comment | Verdict | Fix |
|---|---|---|
| NaN `clamp` in `set_master_gain` | moot | `state.rs` removed; OSC is a [[entities/osc-server|transport adapter]] with no master-gain atomic |
| `send_addr` stored as `String`, re-parsed per send | valid | resolve once at `bind_to`, store `SocketAddr` |
| `/param/set` log says `expected [string, float]` but ints accepted | valid | message → `expected [string, float or int]` |
| `/param/set` log says `expected float` but ints accepted | valid | message → `expected float or int` |
| `tokio` features `"full"` too broad | valid | narrowed to `["rt-multi-thread", "macros", "net", "signal"]` (dropped `bytes`/`parking_lot` from tokio dep tree) |

Note: meter streaming (#103) later re-added the `"time"` tokio feature. All 5 review threads replied to + resolved.
