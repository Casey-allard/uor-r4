# #972 — A1P: bind paired-H4 R4 heatmap identifiability

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-27T15:48:36Z
- **Updated:** 2026-10-09T14:17:23Z
- **Closed:** 2026-08-27T15:49:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/972
- **Merged:** 2026-08-27T15:49:53Z
- **Merge commit:** e422fa2a3e4cd31ae7edc56182316a545db3af21
- **Base / Head:** main / UOR-Foundation:issue-970-candidate-relative-readout
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #970

## Public contract correction

- Keeps the six visible A1R queries unchanged as regression/root-cause evidence only.
- Binds independent, disjoint, κ-pinned construction and sealed-validation fixtures before the replacement readout.
- Replaces the superseded single-`D` class key with the ordered paired-H4 witness `X=C(H,c)`, `Y=C(P_c,c)`, relative projection `D=X*Y^-1`, and exact signed `(1,i)` R4 heatmap in `Z[phi]`.
- Maps only exact endpoints `sin=+/-1, cos=0 -> activation 1` and `sin=0, cos=+/-1 -> activation 0`; the activation bit is not a candidate label. Nonlandmarks stay exact and typed-null rows abstain.
- Carries the immutable zeta grid, ordered multiplicity-preserving n-lets, exact golden radial maps, and the typed `sqrt(2) <-> 2i <-> [0,2]` marker as structural transport. It does not invent a zeta/n-let-to-phi exponent rule.

Contract κ: `blake3:2daacf538c022fab9580d1e124af6c18d0b06da04604fbc962a01bda57f08a98`.

## Bounded evidence

The target-free structural census enumerates all 120×120 ordered operand pairs:

- 14,400/14,400 pairs;
- 120/120 relative rows, multiplicity 120 each;
- 45 exact heatmap classes;
- 480 typed-null pairs;
- census κ `blake3:dca725c0ec6060166bcd0023df956e1ff029661b5fa7800ccb9f20808712b796`.

Fixture identities:

- construction `blake3:fb5f27fc1107f527d616f32affa8eba1746a2f60cfdb95ddbb21a0e493299652`;
- validation `blake3:ecbe8b404e7542d801ff4b4e66c91a41f90158d84efa484dc4edb53aff38b602`.

Across 36 candidate decisions, the paired heatmap yields 14 classes. Construction is 12/12 covered and pure; validation coverage and the no-class-splitting ceiling are 10/12; strict construction transfer is 0/6; eight classes require incompatible outcomes.

The unchanged old-six shortest-Cayley regression remains 0/6 strict selections, 6/6 ties, and 6/6 abstentions under inherited report κ `blake3:f0db7a5d5c81d51ebf3b4bf8a2715c4960ec16b14161e8bf7598d7b98c48c881`.

The existing-additive comparator forms two construction-impure classes, covers 12/12 validation decisions, has a 6/12 oracle ceiling, and transfers 0/6.

## Terminal result

`RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q`

The predeclared identifiability hard gate stopped before scalar search, scorer compilation, validation selection, downstream control execution, or placement. Every downstream row is `NOT_RUN_IDENTIFIABILITY_HARD_STOP`; `NOT_RUN` is not PASS. No selector or trace seam is emitted because no readout was authorized.

Corrected report κ: `blake3:5f9239150dea8c0c27c4dfa6ad2e4d0068bc3d18afc127b315c0ec358ceddb3f`, reproduced by byte-identical double execution.

This is a bounded exact-heatmap readout-identifiability negative. It does not negate paired H4, zeta phases, n-lets, golden radial transport, or typed geometry; it does not establish attention, inference, generation, correctness, reasoning, or product readiness. Protected delivery makes #969 the next unassigned multichannel A1Q stage; #953 remains blocked by #969.

## Focused verification

- `cargo test -p uor-r4-core --offline --test recursive_geometric_attention_970 -- --nocapture` — PASS, double execution, 115.43 s test runtime
- `cargo test -p uor-r4-core --lib a1p_contract_tests --offline` — PASS (2)
- `cargo test -p uor-r4-core --offline --test recursive_geometric_attention_967` — PASS (2)
- `cargo test -p uor-r4-model-source --lib exact_executor_contract_binds_complete_local_source_closure --offline` — PASS
- focused core/test and host CLI compilation — PASS
- WASM library check — PASS (existing target-specific warnings only)
- `cargo fmt --all --check` — PASS
- `git diff --check` — PASS
- `python3 scripts/check_claim_wording.py` — PASS

The broad workspace suite, corpus/model/teacher runs, generation canaries, and product QA were intentionally NOT_RUN; protected CI carries exhaustive integration.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

