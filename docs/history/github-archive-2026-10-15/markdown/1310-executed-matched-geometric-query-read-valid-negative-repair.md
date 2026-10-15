# #1310 — Executed matched geometric query read: valid negative; repair two projection defects (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-20T17:25:19Z
- **Updated:** 2026-10-09T13:17:29Z
- **Closed:** 2026-09-20T17:33:48Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1310
- **Merged:** 2026-09-20T17:33:48Z
- **Merge commit:** 7ea3744ce190c6acd708275e4121a6354b301023
- **Base / Head:** main / UOR-Foundation:codex/geometric-query-read
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Executes the complete bounded task in `docs/integration/deepseek-geometric-query-step-2026-09-20.md` from refreshed `main` at PR #1309 (`92240a86`).

## What ran

Frozen corrected E (`565cf98a…`) reproduced exactly at **7.170815718** bits/target on the 36-document / 288-window / 17,342-target open-dev panel. Three equally parameterized arms (separate write/query maps, two reader accesses, residual bound 4096) were fitted for 512 batch-8 updates each with a 64-update R/W-only warm-up, exported as a new versioned `CPX3` artifact (reader shift 4 + output shift 3 = 7), reloaded, and measured with **0 reload/parity mismatches over all 17,342 positions**.

| Arm | Dev micro CE | Gain over E, nominal 95% paired |
| --- | ---: | --- |
| E, corrected empirical incumbent | 7.170815718 | — |
| Q, joint `q*b` older/query read | 7.141505532 | +0.029310 [0.022866, 0.036005] |
| S, **separable** older + query read | 7.032227740 | +0.138588 [0.125831, 0.150844] |
| L, local-only matched read | 7.101049449 | +0.069766 [0.058944, 0.080294] |

## Decision — valid NEGATIVE, no candidate promoted

| Screen | Result |
| --- | --- |
| Practical `CE_E − CE_Q ≥ 0.10`, lower bound > 0 | **FAIL** (0.029310 < 0.10) |
| Matched vs S | **FAIL** — S is **0.109278** better than Q |
| Matched vs L | **FAIL** — L is **0.040456** better than Q |
| Older content under the exact donor map | **FAIL** — −0.003830 [−0.012235, +0.004366] |
| Query-route use (`b = e` neutralisation) | **FAIL** — +0.000322 [−0.002913, +0.003358] |
| E reproduced | **PASS** |
| Instrument checks | **PASS** |

Instrument checks: read-disabled equals E exactly in every arm; L is byte-identical under the donor map and older-order reversal; reload parity is exact; an on-disk split-at-4 checkpoint/resume equals the uninterrupted run in next-batch loss, all losses and artifact bytes. S's query channel *is* strongly used (+0.190660 [0.173466, 0.207857]); Q's is not distinguishable from zero. Learning moved 1,549–2,241 write codes and 1,544–1,581 query codes per arm, so this is an arrangement result, not an optimizer stall. Every arm's six retained-prompt generations still degenerate into short cycles.

## Two projection defects repaired with focused tests (no calibration rerun)

1. `learner/head_projection.rs::project_row` now sweeps the nearest-code seed at **every** admissible shift including `s0`. Regression `nearest_seed_is_swept_at_s0_on_a_correlated_gram`: `G=[[1,.9],[.9,1]]`, `w=[.6,.6]`, `s0=0` — untouched `[1,1]` `J=.608`, swept `[0,1]` `J=.088`; restoring the old guard makes the test fail `[1,1]` vs `[0,1]` (confirmed). The complete originally specified projection remains **NOT_RUN**; no old report bytes changed.
2. `bin/head-projection.rs` writes `singularity: "NOT_MEASURED"` with `singular_inputs_supported: true` instead of the hardcoded `singular: true`. No rank was computed.

## New source

- `crates/uor-r4-core/src/native_geometric/learner/query_read.rs` — `CPX3` artifact with fixed declared shifts, matched Q/S/L arms, the declared analytic surrogate gradient checked against continuous finite differences, separate A/B credit and softmax Jacobians, bounded parent-score cache, resumable `CPQK` checkpoint. `CPX2` bytes are rejected by magic and never reinterpreted.
- `crates/uor-r4-core/src/bin/query-read.rs` — pinned-input verification, population reproduction, fits, reloaded evaluation, matched controls, generation and cost.

13 focused module tests plus the 8 head-projection tests pass; `cargo fmt --check`, the touched-package offline check and `scripts/check_claim_wording.py` are clean. Unrelated pre-existing failures in `learner::lowbit_attention`, `learner::lowbit_core` and `learner::geometric_attention` were observed in a broad run and are **not** caused by this change (those modules are untouched).

## Resources

Standing-authorized **+7200000 ms** extension to **178100000 ms** was recorded in the live ledger **before** consumption. This step charges **5900000 ms** once (four runner executions including two superseded probes and one superseded complete run, compile/test cycles including a 636.5 s broad learner run, and a documentation/delivery allocation). New cumulative **176,138,565 / 178,100,000 ms**, remaining 1,961,435 ms.

Retained: `.uor-models/realtext-prior-2026-09-20/query-read-2` — sealed, verified, 39 files, 35,737,680 bytes. Superseded `query-read-1` preserved sealed. Peak RSS 763,002,880 B; complete run 992.7 s; 128 MiB storage margin intact; no deletion, corpus download or paid/external compute. **Physical energy UNAVAILABLE.**

## Next handoff

Continue from the **separable** read S (the strongest arm against E so far) and test **selective write/reset**, **exact occurrence/version access** or a **history-capacity** question on it, against a new independent behaviour panel. Do not rebuild the multiplicative read or start a projection/width/precision sweep. `#973`, `#820`, `#963`, `#964` remain open.

**References #973, #820, #963, #964 — does not close any of them.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

