# #909 — research(#908): end-to-end deployed causal run for the injected-candidate skip-mix lane — floor cleared

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-21T22:52:59Z
- **Updated:** 2026-10-09T13:09:10Z
- **Closed:** 2026-08-21T23:14:51Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/909
- **Merged:** 2026-08-21T23:14:51Z
- **Merge commit:** d876766854efd2724c8d8dc42f6e0c63c0de290b
- **Base / Head:** main / UOR-Foundation:issue-908-skipmix-endtoend
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

The S1-redesign injected-candidate skip-mix lane (#906) cleared its deployed **fidelity** spot-check (58/87), but fidelity is not promotion. Per the #822 §6-4 re-entry decision the frozen **20‰ end-to-end causal floor** is the S1 promotion gate. This PR measures the lane's causal effect on deployed held-out top-1 against that floor and lands the evidence.

**Outcome: floor CLEARED.** On the attested `smollm2-360m-broad-clean` bundle (72,130 held-out positions), teacher-free:

| arm | top-1 | rate (‰) |
|---|---|---|
| base (no SKMX/PSIB) | 19,372 | 268.57 [265.34, 271.81] |
| **skip (injected lane)** | **21,424** | **297.02 [293.68, 300.35]** |
| null (label-shuffle) | 2,281 | 31.62 [30.35, 32.90] |

- **paired skip − base = +28.45‰ [25.57, 31.32]** — the paired 95% lower bound (25.57‰) clears the frozen 20‰ floor.
- **conditioning-specificity null collapses to −236.95‰** — the signal is lane-specific.
- As predicted, the deployed +28.45‰ is ~half the off-serving +56.2‰ (toy base); the strong deployed base absorbs the rest, but the lowered lane still clears the floor.

## Scope and non-goals

Deployed `R4Engine` causal measurement only. **Teacher-free** (deployed base vs deployed skip-mix scored against recorded `t_argmax`; no live teacher). The skip-mix lane remains **dormant** — no `model/ids.toml` row, no `CONFORMANCE.md` change, no wire-format/compile-time change, no recompile of any released bundle. Turning the lane on (RF-31 registration + serving/compile-pipeline wiring) is the promotion-conditional follow-on, out of scope here.

## Method (isolates exactly the lane's contribution)

Three graphs are emitted from the SAME recompiled sections, differing only in the two optional lane sections: **base** (empty SKMX/PSIB → absent-section identity, asserted), **skip** (real fitted tables), **null** (tables fitted on a corpus whose TRAIN targets are deterministically rotated — the conditioning-specificity control). Every held-out position is replayed through each engine's `predict_decision_candidates_with_skipmix`. Stats use the phase-0 `ci95_permille` / `paired_delta_permille` conventions.

## Run contract / cheap instrument

The run contract is on #908. The AGENTS.md binding reachability probe (6,000-position subsample) ran FIRST and passed: ceiling 549‰ (≫ 20‰), subsample +35.67‰ [25.76, 45.58]; the full run launched per the pre-declared gate.

## Acceptance criteria status

- Deployed causal delta vs frozen 20‰ floor — **CLEARED** (lower bound 25.57‰).
- Non-vacuous null / negative control — **PASS** (null collapses −236.95‰; the absent-section base is byte-identical by assertion).
- Reproducible from pinned inputs — **PASS** (result_cid `blake3:e32e4e33…` reproduced across two runs; counts pinned).
- Recommendation to maintainer — **PROMOTE** (S1 verdict itself is the maintainer decision on #822).

## Tests / verification

- New `--ignored` harness `crates/uor-r4-api/tests/skipmix_endtoend_causal_908.rs`; base/skip/null counts (19,372 / 21,424 / 2,281), reachability decomposition (39,360 / 6,651 / 4,599), floor-cleared and null-collapse verdicts pinned as regression assertions; both `probe` and `full` phases green.
- Change-scoped gates green: `cargo fmt --check`, `cargo clippy -p uor-r4-api --all-targets --all-features -- -D warnings`, `python3 scripts/check_claim_wording.py`, `cargo test -p uor-r4-api --offline`. The diff is test + docs only (no production code, manifests, or generated files), so the workspace-wide no_std/wasm/deny/xtask gates are unaffected; the merge queue re-verifies the full ladder.

## Evidence artifacts

`docs/skipmix_endtoend_causal_908.md` (record) and `docs/skipmix_endtoend_causal_908_result.json` (CID-bound, result_cid `blake3:e32e4e33…`).

## Compatibility / migration / conformance effects

None. Lane dormant; no serving-behavior change, no format/compile change, no `CONFORMANCE.md` change. `RF` capability IDs unchanged under this PR.

## Claim-boundary changes

The result is an **Empirical Criterion** (`docs/formal_vocabulary.md`) at deployed-serving scope. It supplies the evidence for the S1 promotion decision; it does not itself promote the stage.

## Negative / unavailable results

None outstanding — the primary result is positive. The off-serving +56.2‰ figure is confirmed to overestimate the deployed delta (recorded), but the deployed delta still clears the floor.

## Intentionally excluded

RF-31 registration, wiring `fit_skipmix_tables` into the production compile path, and routing the serving decision path through the lane — all promotion-conditional, tracked as the activation follow-on.

Closes #908


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

