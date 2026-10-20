# #247 — Session-manifold signature lane: feed persistent geometric state into the graph engine's existing signature channel

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T03:52:57Z
- **Updated:** 2026-08-03T17:03:07Z
- **Closed:** 2026-08-03T17:03:07Z
- **Labels:** enhancement
- **Assignees:** auser, Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/247
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
The graph engine already has a geometric routing lane — `predict_distribution(context_tokens, signature: Option<&[u8]>, ...)`:
- When the suffix DFA fails, it Hamming-matches the 288-bit signature against per-node ROUT prototypes/masks to re-enter the graph (`uor-r4-graph-runtime/src/engine.rs:568-619`, "Geometric Routing Fallback").
- Signature proximity adds a score bonus on emissions (`engine.rs:714-738`).

But today the lane carries no new information: serving passes a signature derived from the **same 8-token window as the context itself** (`src/chat.rs:325-333`: `bundle_window_plain` → `sig_plain` over `beam_history`; same in `src/tless_uor.rs:205-218`), and `certify` passes `None` (`certify.rs:326`) so the lane is never measured. Meanwhile the persistent 512-d session manifold state (unit hypersphere, evolved every turn via `evolve_brain_state`, persisted in `manifold_cache_rust.json`) feeds only audit telemetry and the last-resort geometric fallback — never the primary engine.

### Proposal
Quantize the session brain state into a 288-bit graded/VSA signature (server-side, floats permitted — signature *construction* is outside the runtime contract) and feed it through the existing signature channel, replacing or blending with the context-derived signature. Start bias-only (sig_bonus path — no routing behavior change), then evaluate the ROUT-fallback path. This gives the primary engine its first conversation-level signal — the architecture's missing long-range binding — with the consumption side already implemented and the mul-free kernel untouched.

### Open design points
- Quantization of the hypersphere state → bit signature (same address-space review discipline as #243: this defines what "semantically near" means for session states).
- Whether ROUT prototypes need a compile-time story for session-space matching, or whether emission-bias-only is the right first milestone.
- Blend policy between context-signature and session-signature (concatenate lanes vs weighted mask).

### DoD
- A/B on multi-turn prompts: identical 8-token context, different session histories ⇒ measurably different outputs (impossible today by construction).
- No runtime-contract violation (P-4 scan green; signature built server-side).
- Evaluation extended so the signature lane is measured, not `None` (coordinates with #234 and #237 — today's harness is blind to this lane).

*Depends on content-bearing geometric state (#245). Measurement honesty depends on #242 and #237. From the two-lineage reintegration plan.*


## Comments (4)

### comment 5148238714 — 2026-07-31T23:14:01Z — auser

Implemented the first measured session-signature milestone in PR #315: https://github.com/UOR-Foundation/uor-r4/pull/315

The server now quantizes evolved 512D manifold state into a deterministic 288-bit signature and feeds it into the existing graph emission-bias lane. Context signatures remain the only ROUT fallback input until calibration. Direct chat gets an order-sensitive history fallback. The same-eight-token/different-history A/B guard, integer-only runtime regression, docs, clippy, no-std checks, and full workspace suite all pass.

### comment 5153614482 — 2026-08-01T21:52:52Z — auser

## Status audit (2026-08-01): first milestone merged, evaluation DoD item remains

PR #315 (merge 0b8c252) delivered the bias-only lane: session-manifold signatures quantized server-side feed the graph engine's sig_bonus path, with an architecture-level regression guard (differing session signatures under byte-identical context slices). Verified on origin/main.

DoD status: (1) A/B differing outputs — covered at architecture level; general quality A/B deferred to a pinned multi-turn fixture; (2) no runtime-contract violation — met (P-4 green, server-side floats only); (3) **evaluation extended so the lane is measured, not None — still open**: certify path / held-out calibration pending, session signatures kept out of ROUT fallback until then. Issue stays open on item 3.

### comment 5155114526 — 2026-08-02T03:55:36Z — auser

Follow-up merged via PR #344 (merge 967c0297928e9c78665bafc34df0d5657c9f2811). Added `cargo bench -p uor-r4-graph-runtime --bench session_signature` to measure context-only vs session-bias vs alternate-history predictions on held-out positions. Local probe on the pinned 200k SmolLM2 bundle: 17 positions, session scores changed on 16/17 (94.12%), token choices unchanged in this small sample. The harness preserves context-only ROUT fallback and makes no quality-improvement claim. Issue remains open for the pinned multi-turn quality/calibration decision.

### comment 5169077627 — 2026-08-03T16:32:21Z — Casey-allard

## Work plan for the remaining DoD item (pinned multi-turn fixture + calibration decision) — in progress

Surface audit against main @ 73ab76b. Four gaps to close, then the decision:

1. **No multi-turn fixture exists.** The #344 bench substitutes story-prefix "history" and a reversed-prefix synthetic alternate (`benches/session_signature.rs:65-86`) — a document, not a conversation. Plan: a pinned in-source fixture of deterministic multi-turn transcripts (the `PARITY_PROMPTS` precedent, chat-formatted via `scenarios.rs`), each turn evolving the 512-d manifold through the real `evolve_state` path.
2. **The bench exercises the wrong quantizer** — `session_signature_from_tokens` (client fallback), while the shipped server lane uses `session_signature_from_state` over the manifold. The fixture harness will run `from_state`, so the measurement finally exercises the deployed representation.
3. **Gate C has no session lane** (`evaluate_gate_c` builds context-only sigs, score.rs:1949) — the calibration therefore lands as a bench-derived measured row with the #350 entropy-bucketed corrections/regressions accounting, not a Gate C change.
4. **The decision is two lines wide by design**: ROUT admission is one guard (`engine.rs:667-668`, context-only by policy) plus one inverted assertion (`r4g1_runtime_test.rs:242-245`, "session lane must not change ROUT fallback yet"). If the fixture measurement shows session corrections beating regressions across entropy buckets by a pinned-floor margin (the bdd.rs floor style), the same PR flips both; if not, the lane stays bias-only and the negative is recorded here.

Sequencing note: this runs after the #364 campaign items (#379/#380/#381 filed today); measurement discipline applies — the fixture harness is cheap (bench-scale), the decision threshold is declared before the run.



---
_Comment/review ids in this file: 5148238714, 5153614482, 5155114526, 5169077627_

