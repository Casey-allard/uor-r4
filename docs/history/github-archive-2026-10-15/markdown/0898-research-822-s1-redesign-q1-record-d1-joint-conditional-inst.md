# #898 — research(#822): S1 redesign — Q1 record + D1 joint-conditional instrument (SELECT: backed-off mix)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-21T11:47:41Z
- **Updated:** 2026-10-09T14:17:17Z
- **Closed:** 2026-08-21T12:09:33Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/898
- **Merged:** 2026-08-21T12:09:32Z
- **Merge commit:** 9d2dbe194a3a18e4ce03e55fd8f1d7b7c66bf3be
- **Base / Head:** main / UOR-Foundation:issue-822-d1-joint-conditional
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

S1's REVISE verdict (2026-08-21) moved the stage into representation/compiler redesign under the approved RFC (`docs/s1_redesign_rfc_822.md`, PR #896). This PR executes the RFC's first two mandated steps and records their outcomes:

- **Q1 (§4-D3, reading-only):** the recorded `t_argmax` labels consumed the full article-token prefix (full causal attention, positions `0..=pos`), hard-capped at `--sequence-length 128` — not the deployed 8-token window. Corpus-byte confirmation: `span_start` 0..=127; 2,556/2,994 stories at the cap; mean label prefix 62.7 tokens. Consequence: the measured suffix-locality is a property of the compiled key space; the label-side conditioning ceiling is 128 tokens.
- **D1 (§4-D1 / §6-2):** the joint-conditional instrument ran under the contract posted to #822 before the run. Verdict under the pre-registered rule: **SELECT (backed-off mix)** — mix 277.2‰ vs base 246.6‰, paired delta **+30.6‰ (95% CI [+28.6, +32.5])**: the first arm in the S1 record whose paired lower bound clears the frozen 20‰ causal floor and the Q3-adopted 25‰ lowering-track opening bar, off-serving. The strict joint arm is real but sparsity-bound (+6.8‰ [4.8, 8.9]; support on exactly the 46,128 known-suffix positions). Both planted nulls collapse (swap −33.2‰; keyshuf −116.3‰), certifying that the (token, suffix) pairing carries the signal.

## Scope and non-goals

Reference-only / off-serving-path instrument + records + one doc-comment correction. No deployed behavior changes; no format changes; no new capability (no `model/ids.toml` / `CONFORMANCE.md` changes); no gate moves (the 20‰ floor and the #838/#841 bars are untouched). The lowering itself is NOT in this PR — it is filed as #897, trigger-gated on the RFC §6-4 decision after D2.

## Changes

- `crates/uor-r4-api/tests/joint_conditional_run_822.rs` (new, `#[ignore]` heavy): pre-registered arms (base/psi/cr reproduction; joint PRIMARY; mix fallback; joint-narrow; d4pos/d4skip comparison; swap + keyshuf planted nulls), reproduction gates against `docs/conditional_residuals_834_result.json`, coverage reporting, double-run determinism check, CID-bound JSON record writer.
- `docs/s1_redesign_instruments_822.md` (new): Q1 answer + D1 contract + D1 outcome (append-only record).
- `docs/joint_conditional_822_result.json` (new): CID-bound run record (result_cid `blake3:41be22f0…`).
- `docs/RESEARCH.md`: S1 section extended with the Q1/D1 entries (the standing five-arm claims are representation-scoped and remain true as written).
- `crates/uor-r4-graph-compiler/src/observation_text.rs`: module-doc correction — the text path prepends no BOS (the wording belonged to the autoregressive driver). Comment-only.

## Acceptance-criteria / contract status

- Run contract posted to #822 before the run: done (issue comment). Reproduction gates: PASS, all exact (base 246.6 / Ψ 264.1 / CR 262.8 / 4,722 pairs / follows 10/13/0). Anti-vacuity + null teeth: PASS (both nulls change >33k predictions and sit far below the real arms). Pre-registered verdict rule applied unmodified: SELECT (backed-off mix) via the declared joint→mix consultation order.
- Positive-branch obligations: lowering track filed (#897, sub-issue of #822) with the key-granularity design question named; D2 remains next per §6-3.

## Tests and verification

- `cargo test -p uor-r4-api --release --test joint_conditional_run_822 -- --ignored --nocapture` → ok (35.9 s; full output mirrored in the record doc).
- `cargo check -p uor-r4-api --tests`; `cargo clippy -p uor-r4-api -p uor-r4-graph-compiler --all-targets -- -D warnings`; `cargo fmt --check`; `python3 scripts/check_claim_wording.py` → all clean.
- The new test is `#[ignore]` (needs the local #833 bundle) and self-skips without the bundle (κ-test convention); CI's default suites are unaffected.

## Compatibility, conformance, claim boundaries

No wire, format, or runtime surface changes. Claim boundary: adds an off-serving **Empirical** result (first floor-clearing arm) recorded with formal-vocabulary labels; no serving claim is made — deployed-path claims remain governed by the frozen gates and would change only via #897's future end-to-end measurement.

## Negative / unavailable results

`d4pos` (distance-bucketed unigrams) measured −28.7‰ [−31.8, −25.7] — recorded NEGATIVE (order structure without conditioning hurts). The strict `joint` arm alone is sub-floor (+6.8‰) — recorded as sparsity-bound, with the coverage stats that explain why (mean 2.85/7.41 supported tokens; zero support on novel suffixes).

References #822


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

