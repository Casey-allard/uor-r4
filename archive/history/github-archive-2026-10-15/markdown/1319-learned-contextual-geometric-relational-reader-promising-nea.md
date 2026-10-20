# #1319 — Learned contextual geometric relational reader: promising near miss (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-20T22:53:13Z
- **Updated:** 2026-10-09T13:17:53Z
- **Closed:** 2026-09-20T22:55:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1319
- **Merged:** 2026-09-20T22:55:42Z
- **Merge commit:** bfa13a918e4909ce679bb852d6c0acfcbd288e0e
- **Base / Head:** main / UOR-Foundation:codex/relational-reader
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Principal review correction:** [PR #1320](https://github.com/UOR-Foundation/uor-r4/pull/1320) preserves this prototype's synthetic improvement but withdraws the categorical/group-attribution conclusion. Categorical training and evaluation use different relation addresses; the task allows a source-class shortcut. The reported loss interval is a soft-policy diagnostic, natural-text CE was not measured, and generation/cost/control defects remain. Original failed decision and raw evidence are retained. See the [review](https://github.com/UOR-Foundation/uor-r4/blob/59e6df2037c47507752b0e0b2147bf269dcbaeec/docs/integration/relational-reader-review-2026-09-20.md) and [single next prompt](https://github.com/UOR-Foundation/uor-r4/blob/59e6df2037c47507752b0e0b2147bf269dcbaeec/docs/integration/deepseek-competitive-reader-step-2026-09-20.md).

---

Original run report (historical claims must be read with the correction above):

Executes [deepseek-relational-reader-step-2026-09-20.md](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/deepseek-relational-reader-step-2026-09-20.md) from refreshed `main` at PR #1318 (`647483ab`). The [design note](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/relational-reader-design-2026-09-20.md) was written **before** any scoring.

## Mechanism

```
q = Q(x_{i-1})          learned 120-valued descriptor, initialised from the frozen S write slots
k = Q(x_{j-1})          the same learned map
r = inverse(q) * k      directed relative element, common-left invariant
s = bias + rank[r] + sum_e w_e*phi_e + sb[a]
a = NoRead or (exact occurrence reference, bounded strength from {0.0625, 1, 8} nats)
```

Selection **and** strength are fitted against the **actual one-step action loss** of the observed next token, `delta(a) = log(1 + p*(exp(a)-1)) - a*1[payload == target]` — observable offline, no reward model. The descriptor itself is learned by a bounded discrete coordinate search on that objective. Only **8 of 120** rank entries are non-zero and **50 of 4,096** roots moved.

**Shared pool.** Every arm gets the same candidates: a candidate is admitted when it shares **any** exact context token (`cur`, `prev2`, `prev`) with the query, newest-first, bounded to 24 of a 128-token ring. Every arm also receives the exact indicators, so widening the pool does not blind the exact-recurrence comparator. No gold source.

## Fresh held-out split (940 positions, 233 covered, frozen before scoring)

| arm — same pool, actions, dose, objective | covered read precision | reads | no-read | E[action loss] nats | real CE Δ bits |
| --- | ---: | ---: | ---: | ---: | ---: |
| local (always NoRead) | — | 0 | 1.000 | 0.000 | 0.000 |
| exact-recurrence (no relation) | 0.755 | 940 | 0.000 | −0.807 | −1.082 |
| categorical control (arbitrary code, same table size) | 0.709 | 858 | 0.087 | −0.722 | −0.995 |
| **relational (learned descriptor)** | **0.873** | 832 | 0.115 | **−1.024** | **−1.500** |

Paired by sequence, 2,000 draws: precision **+0.0987 [+0.0418, +0.1603]**; expected action loss **−0.2172 [−0.2976, −0.1342] nats**.

## Decision — promising near miss, not promoted

The predeclared rule required a precision lead ≥0.15 **and** a loss improvement ≥0.01 nats. The loss criterion passes decisively; precision is above zero with an interval excluding zero but **below its bar**, so the label stays **not positive**, unchanged.

Two findings make the miss informative rather than noise:

- **The geometry carries the gain, not the extra table.** The matched categorical control — the same 120-entry table on an arbitrary code, fitted identically — scores **0.709, worse than the no-relation arm's 0.755**. Adding a large contextual table without the group structure actively hurts.
- **Abstention emerges** at ~11% in the relational and categorical arms; the unconstrained exact arm reads on 100% and pays for it.

**Disagreement with the sketch, with a reason.** The prompt's decision rule privileges raw read precision, but a few points of precision on a near-chance baseline is a weak instrument while a 0.22-nat expected-loss improvement with the whole interval below zero is not. The successor should state its practical margin on the **calibrated** quantity. The original criterion's outcome is preserved unchanged.

## Audit instruments repaired

Payload-correct ranking counter · a **real** future-token intervention (444 earlier positions compared, 0 changed decisions) · a **matched fit** as the comparison arm instead of post-hoc weight zeroing · paired intervals against it · a local cost arm with **no reader work** (1,263.9 µs vs 1,260.1 µs per prediction) · the **SHA256 of the executable bytes** rather than a hex re-encoding · complete export/reload parity (940/940 on action *and* emitted argmax, 0 mismatches).

**One control is mis-specified and diagnosed, not hidden.** `independent_s_query_only_reconstruction` reports 940/940 mismatches because `QueryHard::int_logits(prev, cur, Some((b, e)))` evaluates `u(b) + u(e)` while `z_local` is `E + u(b)`: the difference is exactly the identity-row constant at every position. The baseline definition is unaffected; the control needs to subtract `u(e)`. `instrument_checks_pass` is reported **false** for this reason.

## Cost, tests, delivery

Uncached declared path, no token-pair cache: **1,263.9 µs** per prediction with no reader work and **1,260.1 µs** for the full reader — within noise. Parent 454,788 + local artifact 53,555 + relational artifact **8,844** bytes. Peak RSS 34,471,936 B; wall 35.2 s; **energy UNAVAILABLE**.

**8** focused `relational` module tests pass (common-left invariance, the action-loss closed form and derived optimum, shared admission with the candidate bound, NoRead versus a doomed read, a learned bounded strength, the descriptor search never increasing the objective, checkpoint round trip with foreign-identity rejection, matched comparator dropping only the relation term). `cargo fmt --check`, the touched-package offline check and `scripts/check_claim_wording.py` are clean.

Sealed, verified root: `.uor-models/realtext-prior-2026-09-20/relational-reader-1`. **No limit extension was required or recorded**: the projection fit the existing 191,300,000 ms limit. 900,000 ms charged once; cumulative **182,138,565 / 191,300,000 ms**.

## Next handoff

1. **Re-run the same frozen panels with the corrected parity control and a prospective margin stated on the calibrated expected action loss.**
2. **Carry the learned relation into one dependent two-hop read with a learned scheduler** — changing only the first retrieved relation must change the later answer, including a derived output absent from all source payloads.

Do not widen the ring or candidate bound, sweep widths/precision/corpus, reopen the frozen S attribution, or start a reset-only campaign. `#973`/`#820`/`#963`/`#964` remain open.

**References #973, #820, #963, #964 — does not close any of them.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

