# #1379 — M1 Part B-prime: KVAR panel — a gated overwrite addressed memory extends the horizon (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-24T10:25:58Z
- **Updated:** 2026-10-09T13:17:36Z
- **Closed:** 2026-09-24T10:27:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1379
- **Merged:** 2026-09-24T10:27:02Z
- **Merge commit:** 552d847d49fb263966165004b835f2f53cccaae1
- **Base / Head:** main / UOR-Foundation:codex/kvar-20260924
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820.

Executes **M1 Part B′** — keyed variable-lag retrieval (KVAR) with rebinding — the amended milestone from the K3
architecture gate. Result record:
[`docs/integration/kvar-recall-result-2026-09-24.md`](docs/integration/kvar-recall-result-2026-09-24.md);
pre-registration [`docs/integration/kvar-plan-2026-09-24.md`](docs/integration/kvar-plan-2026-09-24.md).
Evidence root `/Users/casey.allard/uor-r4-investigations/kvar-20260924` (`run4` authoritative). Standalone synthetic
harness; `transferable_lexical.rs` and `tl_execution.rs` byte-unchanged.

## What was built

A new self-contained bin `crates/uor-r4-core/src/bin/kvar-recall.rs` (+1813 lines): the KVAR generator, an order-2
count control, a hand-coded overwrite-table positive control, and a single trainable model with gate/store flags
covering the current recurrence (a), gate-only (b), and gate + token-addressed overwrite store (c). Float-trained
with Adam; **served** in an integer/select form (hard thresholds, table reads, shift-add — no softmax/float).

## Result (primary = plan-compliant hard-select readout, §5.1)

Panel: V=64, `K∈{4,8}`, `lag∈{4,16,64}`, every key rebound, answer = most recent value; train 600 / held-out 204.
Controls: order-2 count **at chance** (0/204; chance 1/64); hand-coded overwrite table **1.0000** (204/204).

| arm | served acc | acc CI95 | bits/query | bits vs count |
|---|---|---|---|---|
| (a) current recurrence | 0.0098 | [0.0000, 0.0245] | 6.11 | −0.11 |
| (b) hard gate only | 0.0196 | [0.0049, 0.0392] | 6.10 | −0.10 |
| **(c) gate + overwrite store** | **0.8235** | **[0.7696, 0.8725]** | **2.251** | **+3.749** |
| (c) seed 2 | **0.7157** | **[0.6520, 0.7745]** | **2.763** | **+3.237** |

**Decision: `ACCEPT_MEMORY_MECHANISM`** — the gated token-addressed overwrite store extends the horizon where the
plain and gate-only recurrences are at chance; this is the memory mechanism the reviews predicted would be the lever
and it answers M1 Part A's "lever is state/horizon". Accuracy degrades with lag (0.94 at lag 4 → 0.59–0.71 at lag
64) but stays far above chance.

## Recorded honestly

`run3` first evaluated an **additive** readout (`wo@h + (mem<<shift)`) and rejected; that was a **deviation** from the
pre-registration §5.1, which requires the argmax/compare-select read. Promoting the plan-compliant hard select to
primary (`run4`) is the correct evaluation; the additive form is retained only as a labelled diagnostic. Gates are
trained soft and served hard — a declared train/serve realization.

## Limits

Synthetic authored panel only; no language/capability/geometric/energy claim. **`(f)` the equal-cost ordinary matched
control is `NOT_RUN`**, so "beats an ordinary equal-cost gated memory" is not yet established (`(b)` is the current
comparator). **`(h)` the geometry gate is `NOT_RUN`** — the vectors/Hamiltonians hypothesis is not tested here.
`(d)`, `(e)` and the equal-work stateful n-gram are `NOT_RUN`.

## Verification

`cargo fmt --check` clean; offline release build of `kvar-recall`; `cargo test -p uor-r4-core --bin kvar-recall`
**8 passed/0 failed** (incl. generator rebinding, overwrite control, count-at-chance, served store read, and three
finite-difference gradient checks); `scripts/check_claim_wording.py` passed. Ledger charged `+900,000 ms`.

## Next

Re-train (c) end-to-end with the hard-select readout as the objective, then run the deferred `(f)` equal-cost ordinary
gated memory and `(h)` geometric parameterisation on the same frozen panel to test the geometry gate.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

