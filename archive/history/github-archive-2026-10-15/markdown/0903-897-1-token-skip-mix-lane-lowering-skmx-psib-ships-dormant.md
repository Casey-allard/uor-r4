# #903 — #897: 1-token skip-mix lane lowering (SKMX/PSIB) -- ships dormant

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-21T18:04:28Z
- **Updated:** 2026-10-09T14:16:56Z
- **Closed:** 2026-08-21T18:26:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/903
- **Merged:** 2026-08-21T18:26:44Z
- **Merge commit:** 1b84770e6daca060cff4e19899796daf3ec7e8e1
- **Base / Head:** main / UOR-Foundation:issue-897-skipmix-lowering
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Implements the #897 S1-redesign lowering of the confirmed 1-token skip-mix scorer (phase-0 verdict `SELECT-1-token`, `docs/skipmix_confirm_897_result.json`) onto the deployed `R4Engine`, per Casey's sign-off on the posted design spec:

- **decision point A**: a real fixed-capacity open-addressed hash table (SKMX) for the primary `(content_token, last_window_token)` joint table, plus an unconditioned Ψ-bag fallback (PSIB) -- not the sorted-array default.
- **decision point B**: extends the REAL release emitter (`ScoredGraphSections` / `emit_scored_r4g1`), not a side converter.

A bounded post-implementation spot-check (task 6, #886-style) found the deployed lane does not yet meet a predeclared fidelity bar, so **the lane ships dormant** -- correctly implemented, fully tested, fully servable end-to-end, but not wired into `model/ids.toml` and with no `CONFORMANCE.md` change. This PR is infrastructure, not a serving-behavior change.

## What's in this PR (6 commits)

1. **`format`**: SKMX (multiply-free ARX-hashed, bounded-probe, open-addressed) and PSIB (sorted/binary-search) wire sections in `uor-r4-graph-format`, both `SectionId::OPTIONAL_BIT` (absent-section identity). 9 unit tests (round-trip, corruption rejection, canonicalization order-independence, a 200-key collision exercise).
2. **`compiler`**: `skipmix_fit::fit_skipmix_tables` -- fits the confirmed key family from a corpus's TRAIN split (`induction::split_positions`), reproducing the exact fold/formula the phase-0 harness validated.
3. **`format`**: `from_trusted_bytes` cheap-reconstruction methods on `SkipmixTable`/`PsiBagTable` -- O(1) header re-checks (not the full O(capacity) `parse()` scan) so `R4Engine`'s lifetime-free, owned-`Vec<u8>` architecture can reconstruct a fresh borrowed view every inference step without per-token revalidation cost. 4 new unit tests.
4. **`runtime`**: `predict_decision_candidates_with_skipmix[_witness]` on `R4Engine` -- the #836-PSTATE-shaped contribution+argmax combinator, reused unchanged per the design spec. Joint-table-row presence gates the Ψ-bag fallback independent of candidate match (tested explicitly). 10 new unit tests; absent-section byte-identical to `predict_decision_candidates`.
5. **`certify`**: extends `ScoredGraphSections`/`emit_scored_r4g1` with `skipmix_rows`/`psi_bag_rows` fields, mirroring the #399 FWDA precedent (empty rows -> section not emitted at all, not merely empty-but-present). Fixed 7 external call sites the new required fields broke. 3 new round-trip tests.
6. **research**: the deployed-fidelity spot-check (below).

## Spot-check result (task 6; full record in `docs/skipmix_lane_897_spotcheck.md`)

Reproduced the phase-0 `skipmix` arm's favorable minimal pairs exactly (87/87, reproduction control) and its baseline-follow degeneracy control (0). Fit + emitted via the REAL release emitter into a real bundle, loaded into `R4Engine`, replayed the deployed lane on all 174 favorable positions.

| quantity | value |
|---|---|
| positions evaluated | 174 |
| served / abstained / unservable | 174 / 0 / 0 |
| deployed-lane follow | 41 / 87 (47.1%) |
| predeclared "faithful" threshold | >= 53/87 (60%, the #886 bar) |

**Verdict: LOWERING-FIDELITY GAP.** Unlike #886, this lowering's re-emission goes through the real production emitter, so the emission-path confound #886 had to caveat around does not apply -- 100% servability confirms the gap is a clean read on the lowering itself (plausibly integer `ScoreQ` quantization + the per-key top-64 cap + the decided-candidate-list bound), not an artifact.

## Consequence: no end-to-end run, no promotion

Per the same discipline #836/#886 followed: the reachability-gate probe and the #833-shaped end-to-end causal run are **not launched** against a lowering already shown sub-ceiling on its own best cases. RF-31 registration (conditional on promotion) is likewise not pursued. This PR therefore contains infrastructure only:

- No `model/ids.toml` change.
- No `CONFORMANCE.md` change.
- No serving-behavior change (every new code path is additive and gated on the new optional SKMX/PSIB sections; absent them, behavior is byte-identical to before this PR).

## Tests / verification

- All 6 commits' own unit/integration tests (format, compiler, runtime, certify) plus the new spot-check harness (`crates/uor-r4-api/tests/skipmix_lane_spotcheck_897.rs`, `--ignored`, needs the real #833 bundle).
- Full local gate ladder, all green: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --no-fail-fast --offline` (full suite, 0 failures), `cargo check -p uor-r4-graph-format --no-default-features` (+ `--features alloc`), `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib`, `cargo deny check bans`, `cargo run -q -p xtask -- validate` (R1+R4+R5+gnaf), `python3 scripts/check_claim_wording.py`.

## Negative / unavailable results

The headline result of this PR is negative: the lowering does not (yet) meet its own predeclared fidelity bar, so it ships dormant rather than promoted. Recorded honestly in `docs/skipmix_lane_897_spotcheck.md` and `docs/skipmix_lane_897_result.json`, with an explicit "Implications" section for the maintainer.

## Intentionally excluded

- Wiring `skipmix_fit` into the production `score` CLI compile pipeline (design spec section 7: a post-promotion step).
- The reachability-gate probe, the #833 end-to-end causal run, and RF-31 registration (all conditional on the spot-check clearing; it did not).

Closes #897.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

