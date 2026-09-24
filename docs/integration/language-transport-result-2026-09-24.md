# Language-conditioned transport and complete compiled execution — September 24, 2026 UTC

This record completes the workstream a prior agent (Codex) left mid-turn on
`codex/language-transport-20260924` (base `48106e7c`, PR #1373; this work is one commit ahead at `aac7b1fd`
plus the three completed files). It is an **exposed, component-scoped result**. The terminal objective is
unchanged: fully transformerless geometric language modelling with exact addressed evidence and learned
geometric operations replacing floating-point matrix multiplication at serving.

Plan: [language-transport-plan-2026-09-24.md](language-transport-plan-2026-09-24.md). Evidence root:
`/Users/casey.allard/uor-r4-investigations/language-transport-complete-20260924` (12 sealed attempts; ignored
payloads, not in a Git clone).

## What was added

- `learner/language_transport.rs` — learned integer token/bigram-feature → Q8-action map. Ownership is never
  inferred from distance: an absent owner is rejected, and a requested but absent owned relation is rejected
  rather than matched to an approximate neighbour; `action ≥ 8` is rejected. A `LQT1` byte format round-trips.
- `learner/tl_execution.rs` — compiled complete native execution (four ternary maps compiled once; precomputed
  embedding bases and per-event offsets; caller-owned buffers).
- `bin/support/language_continuation.rs` — runner modes `geometry`/`cost`/`evaluate`/`train`/`integrate`;
  `span_lengths` + a `fresh_span` panel; a bounded `UOR_LANGUAGE_MAX_SPAN` **wired into the training span draw**
  (the pending script had left it inert).

## Results (sealed; independently re-parsed by an evidence auditor)

### Learned source selection — exact, bounded, typed
- `geometry`: **256/256** fit, **0/256** under a wrong instruction, **256/256** absent-owner rejection,
  **64/256** with the action disabled. Selector `LQT1` sha256 `76ded49d…1109`.
  - The disabled control forces the **identity** action, which selects the owned key equal to the query — true
    only for the `current` role, i.e. exactly **¼ of rows by construction**, not a random 25%. No baseline is
    stated in the artifact; the **0/256 wrong-instruction** result is the sensitivity control.
- `integrate`: **32/32** selections equal the expected source index; **32/32** missing-relation and **32/32**
  absent-owner rejections. In every row `source_index == expected_source_index`, so the later `correct 16/64`
  is a **generation** failure, not a selection failure.

### Compiled execution — faithful, and faster only at the tested scope
- `cost`: **156** grounded native/compiled comparisons identical in tokens, actions and final state digest.
- `integrate`: native/compiled equality is **asserted on the 64 main rollouts** (mismatch → error). The 64
  *changed-payload* rollouts run the compiled path only and are compared to expected actions — they are **not**
  a second parity population. (`evaluate`'s generated output is compiled-only.)
- Median per-token time **1364.16 → 122.03 µs (11.18×)** on 9 alternating rounds, dev windows, empty owned set,
  model/plan/tokenization excluded. This is not a whole-application, quality-matched or energy result.
- **Multiply-free is scoped to a static own-symbol census:** `Map::apply/run/read/choose/workspace` carry no
  multiply mnemonic; construction (`Execution::compile`) contains 6; `start_control`/`advance` each carry one
  `madd` that the source supports as constant-stride `row*h_dim` address arithmetic. **Not** a transitive
  whole-path certificate.
- `plan_payload_bytes 396476` is the logical compiled-plan figure for the four maps **excluding the embedding
  table and all biases** — a partial figure, not the model's total plan.

### Span curriculum (`moments` vs `reset`, 512 steps, `UOR_LANGUAGE_MAX_SPAN=8`)
| arm | historical gates (temporal/class/feedback/held-out) | span by_length (1/2/3/5/8 of 16) | fresh span (13/21/34 of 16) | repository bits | tune bits (train) |
|---|---|---|---|---|---|
| retained parent | 32/32, 4/4, 16/16, 3/3 | 16/0/0/0/0 = 16/80 | 0/48 | 6.343260 | — |
| `moments` | 32/32, 4/4, 16/16, 3/3 | 16/16/9/16/16 = **73/80** | **48/48** | 6.485238 | 9.13001 |
| `reset` | 32/32, 4/4, 16/16, 3/3 | 16/16/12/16/16 = **76/80** | 44/48 | 6.651418 | 9.19406 |

- The span panel is a **deterministic forced-feedback diagnostic over the four authored class values**, not
  general spans. The two arms are data-matched to each other (same seed) but **not** to the earlier
  `max_span=3` root: the span draw consumes the shared global RNG (`len`, then `len` source draws, then `len`
  feedback draws), so changing `max_span` shifts every later sample. The declared **separate span RNG is not
  implemented**. "Recovery" is also not uniform (L2 10→16, but L3 10→9 / 10→12), and the prior root recorded
  L5/L8 = 4/16, 4/16.
- **Both arms are worse than the parent on repository bits** (+0.142 / +0.308); the four "retained" gates are
  authored and were already fitted in the parent.

### Exact resume
A 64-step checkpoint resumed in a fresh process to 128 steps is **byte-identical** to the uninterrupted control
(`bd63cbbd…7841`). Determinism check only; single training seed.

## Negative, at its exact scope
`evaluate.generated` / `integrate.correct` reproduce exact output only for payloads of **two BPE tokens**
(`37`, `42` — which are *outside* the fit range `0..31`); payloads of **three or four BPE tokens** (`105`, `208`,
`317`, `512`, `1024`, `2048`) fail to emit the correct copy/stop action sequence (`integrate.correct 16/64`).
Because the two passing values are out-of-range and still pass, **value coverage/identity is not the binding
constraint** — the collapse is a **multi-token copy/stop-length curriculum gap** (`len ≥ 3`), with a possible
secondary template-variant effect that the passing rows argue against. It does **not** bear on the selector
(selection 32/32, fit 256/256) or on geometric transport.

## Limits and binding
- No general-language, dialogue, coding, geometric-superiority or energy claim. The Q8 learner's established
  tie with an ordinary signed-permutation control is unchanged; nothing here revives a unique geometric edge.
- **Binding gap (process):** the sealed receipts bind `source_sha256 = sha256(principal_continuation.rs)` (the
  unchanged dispatcher); `language_transport.rs` and `tl_execution.rs` are bound nowhere, and the executable
  hash appears only in the unsealed `execution-symbol-census.json`. Delivery must bind the changed files.
- Changed-file identities: `language_continuation.rs` `41864ab7…527a`, `language_transport.rs`
  `0e16986e…7a68`, `tl_execution.rs` `71e7888b…06ed`; binary `d046f03c…d94a`; parent
  `causal-candidate.tlx` `1ac70065…955f`.
- `max_span` was wired in this completion; the pending script left it inert. Implementing the separate span RNG
  remains open. Only successful compiled calls are allocation-free.

## Resources
Runner wall 1007.9 s across 12 runs; peak model RSS ≤ 162 MiB; two threads, one cargo process at a time;
evidence root 177 MB; net session storage ≈ 1.45 GiB. Charge **+3,000,000 ms**; cumulative **449,439,320 /
455,500,000 ms**. No paid/external compute; owner checkout, model parents and prior roots preserved.

## One recommended next milestone
**Make the span curriculum a matched experiment, then fix the multi-token copy gap.** Implement the declared
separate span RNG so `max_span ∈ {3, 8}` arms are exactly data-matched and re-run that small grid; then add
in-range 3-token payloads and a length-only control to separate "length ≥ 3" from "out-of-range value" (the
current data cannot), extending the copy/stop-length curriculum and re-measuring `evaluate.generated` and
`integrate.correct` with the selector and compiled path fixed. No new architecture is required.
