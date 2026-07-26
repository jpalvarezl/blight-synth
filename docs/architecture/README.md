---
title: Architecture Index
summary: Routing page for cross-domain contracts and target architecture.
status: current
updated: 2026-07-26
---

# Architecture

## Read first

- [Product and host topology](product-topology.md) — accepted standalone-first target matrix, diagrams, and authority.
- [System boundaries](system-boundaries.md) — target dependency direction and ownership.
- [M0 crate dependency graph](crate-dependency-graph.md) — current enforced crate/feature dependencies and compatibility shims.
- [Offline render contract](offline-render-contract.md) — canonical PCM, golden regression policy, and reference updates.
- [Real-time audio contract](realtime-contract.md) — proposed callback rules, ownership, overload, reclamation, and current violations.
- [Device host boundary (draft)](device-host-boundary.md) — reusable in-process device host vs OSC standalone transport adapter (ADR 0002).
- [Event-source contract (draft)](event-source-contract.md) — producer-side clock mapping, bounded current-block events, and separate NRT lookahead (ADR 0003).
- [RT allocation audit](rt-allocation-audit.md) — test-only heap allocation/deallocation instrumentation for prepared Engine processing.
- [Current product specification](../spec/current-product.md) — what architecture is serving.
- [ADR index](../decisions/README.md) — accepted decisions.

## Contract ownership

| Contract | Owning issue | Status |
|---|---|---|
| Workspace and host-independent engine boundary | [#130](https://github.com/jpalvarezl/blight-synth/issues/130) | Draft |
| Engine lifecycle | [#132](https://github.com/jpalvarezl/blight-synth/issues/132) | Draft |
| Real-time safety | [#133](https://github.com/jpalvarezl/blight-synth/issues/133) | Draft |
| Sample-accurate events | [#134](https://github.com/jpalvarezl/blight-synth/issues/134) | Draft |
| Composition event sources | [#145](https://github.com/jpalvarezl/blight-synth/issues/145) | Proposed (ADR 0003) |
| Parameter manifest | [#121](https://github.com/jpalvarezl/blight-synth/issues/121) | Draft |
| Routing graph | [#136](https://github.com/jpalvarezl/blight-synth/issues/136) | Draft |
| Versioned state | [#138](https://github.com/jpalvarezl/blight-synth/issues/138) | Draft |
| Device host vs OSC standalone split | [#185](https://github.com/jpalvarezl/blight-synth/issues/185) | Proposed (ADR 0002) |

Do not treat draft diagrams as implemented fact. Domain pages list current code entry points separately.
