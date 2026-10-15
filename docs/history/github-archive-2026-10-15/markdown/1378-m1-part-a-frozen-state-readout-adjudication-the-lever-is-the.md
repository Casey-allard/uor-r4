# #1378 — M1 Part A: frozen-state readout adjudication — the lever is the state/horizon, not the readout (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-24T09:00:33Z
- **Updated:** 2026-10-09T13:17:15Z
- **Closed:** 2026-09-24T09:01:45Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1378
- **Merged:** 2026-09-24T09:01:44Z
- **Merge commit:** f0a7fc4a931c7a3602eee81ca1515dfe9e2973d1
- **Base / Head:** main / UOR-Foundation:codex/escape-bigram-20260924
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820.

Executes **Part A** of the owner-adopted [`escape-bigram-class-plan-2026-09-24.md`](docs/integration/escape-bigram-class-plan-2026-09-24.md):
the **frozen-state readout adjudication** — a read-only, float-only diagnostic on the artifact's own recorded served
states. Result record: [`frozen-state-readout-adjudication-result-2026-09-24.md`](docs/integration/frozen-state-readout-adjudication-result-2026-09-24.md).
Evidence root `/Users/casey.allard/uor-r4-investigations/escape-bigram-20260924` (`partA-run6` authoritative).

## What was added

A runner mode `--readout-adjudication` (in `crates/uor-r4-core/src/bin/ordinary-lexical.rs`, +1833 lines) that reuses
the merged probe machinery (`record_served_states`, `ProbeSet`, `fit_float_probe`/`fit_ridge_readout`) and fits
linear / rank-K quadratic / MLP / count-augmented readouts on the recorded states, **document-held-out**. It is
inert by default and read-only. `transferable_lexical.rs`, `tl_execution.rs` and the D0-b kernel are byte-unchanged.

## Result (held-out bits/target, artifact `warm-decay-candidate.tlx`)

| arm | bits/target |
|---|---|
| `A` artifact readout (K-class) | 5.7555 |
| `U` unigram | 8.5690 |
| `C` tuned order-2 count prior | 4.8041 |
| (i) converged float linear | 5.7048 |
| (ii) rank-K quadratic sketch | 7.7023 |
| (iii) 2-layer MLP | 6.9112 |
| **(v) count-augmented linear** | **4.7819** |

- **(v) − C = −0.0223 [−0.0295, −0.0149]** → the state adds **+0.022 bits** over a servable count table.
- (i) − A = −0.0507 [−0.0016, +0.1037]; (iii) − (i) = **+1.2064** (no nonlinear readout gain).
- Instrument control: artifact-initialised probe reproduces `A` to **2.7e-15**.

**Decision (branch1-substantive true): the state's local information is subsumed by a servable order-2 count table,
so no readout work — linear, quadratic or MLP — is justified; the lever is the state/horizon (M1 Part B).** This
corroborates the merged count-blend negative and the direction decision, and does not revive the addressed local
`(prev,cur)` read.

## Verification

`cargo fmt --check` clean; offline release build; `cargo test --offline -p uor-r4-core --bin ordinary-lexical`
**30 passed / 0 failed / 1 ignored** (incl. `adjudication_split_is_disjoint_and_planted_signal_recovers`);
`scripts/check_claim_wording.py` passed. Read-only: no training, no artifact writes. Ledger charged `+1,500,000 ms`.

## Caveats and limits

8 held-out documents (arm (i)'s gain over `A` is not significant at that cluster count); arm (ii) is a declared-seed
closed-form sketch; `A` is not a document-held-out comparator. A feature-form measurement on one artifact — **no**
language, capability, geometric or energy claim.

## Next

**M1 Part B** — a gate × capacity arm on a synthetic induction panel where the count control is at chance, with the
owner's vectors/Hamiltonians lead attached as a **learned parameterisation of the state/gate**, tested against a
matched ordinary parameterisation at equal parameters.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

