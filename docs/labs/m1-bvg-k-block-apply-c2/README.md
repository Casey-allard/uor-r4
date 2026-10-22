# C2 qualification and the Stage T registration state (item #2835)

Item: [`m1-bvg-k-block-apply-c2`](../../roadmap/items/m1-bvg-k-block-apply-c2.md) (`#2835`, M1 W3 BVG-K). Lab: deepseek, Unit: the in-scope record while the Stage T sheet's preconditions are unresolved.

## What is already delivered

- **Stage C2 (block propose, verify and apply) PASS.** Fixture-only GPU controls from 2026-10-19, about 13 minutes for roughly $0.40. Receipt: [`docs/labs/injective-preflight-and-c2-qualify-2026-10-19/evidence/bvgk-c2-qualify-20261019/report/bvgk-qualification.json`](../injective-preflight-and-c2-qualify-2026-10-19/evidence/bvgk-c2-qualify-20261019/report/bvgk-qualification.json), SHA-256 `2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`.
- **Controls that passed:** V0 source identity, RC re-carry parity, UT untouched, JR joint reference, DC decision coverage, V3 block apply equals forced sequential apply, RF refusals and C1R (C1 parity rerun). All 3,419 updates were on synthetic fixture devices; `Device::apply` was never called.
- **The Stage T sheet exists as a DRAFT** and is machine-complete to shape: [`m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md`](../../harness/prep/m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md) (`TEST FITNESS: PENDING`; it covers arms T1/T16/T4, equal-wall-clock protocol, first-batch controls V1d/KId/V8d/V9d/AKd, cost waves and power).

## Why this item cannot be closed here (registration blockers)

1. **The Stage 2 predicate is unresolved.** D39 item 4 runs W3 only if W2 decides `H_law`, and the record is contradictory: `NOW.md` row 6 lists the W2 read-code capacity capture as **queued**, the roadmap item `m1-w2-capacity-capture` is **held**, while `project-track.md` says "W2 read-code capacity — DONE" (that sentence appears to describe the v5.1 consumer-verifier ACCEPT, not the capture). **PM answer needed:** the capture's status, and whether `H_law` is decided.
2. **No base lineage passes both its pre-flight and its registered Q1 admission gate.** The injective pre-flight failed B3_init (127 of 3,847 < 500) and was not dispatched; the A8 lineage *reached* B3_init (1,351/3,847, clearing the 500 floor) and then hit its registered Q1 admission-collapse stop (P_succ 1,351 → 551; FrozenZero 857/3,847) — a D31 diagnostic stop with no paired DEV decision. The Stage T sheet's precondition 2 therefore stays unmet; the census (Q1–Q5) narrows the question but does not supply a lineage that survives Q1.
3. **Registration numbers are not fixed.** The non-inferiority margin, the base-lineage choice, T4 inclusion and the AKd shadow-device admission are all registration-review items (the DRAFT lists them); the item's bar is the Stage 2 bar ("beat the unigram table, then the admission-matched next-word table … on two seeds"), with the unigram top-1 baseline 1,226 and the panel's 2-SE floors (20/63/87 rows) as the non-narrowness floor. **No owner release or FIT is requested here**, and training remains HELD.

## Notebook row (staged; the page is outside this item's scope)

The item's step 6 asks for one row in `docs/research/notebook/m1-discrete-attention.md`. That page is outside this item's declared `scope:`, so the row is staged here for the same backfill path used for the m3 items: after a `notebook:` field is added to this item (umbrella-scope PR, e.g. the pattern of #2997/#3002), the row can be appended there verbatim.

| Date | Item | Change / Finding | Verification |
|---|---|---|---|
| 2026-10-22 | `m1-bvg-k-block-apply-c2` (`#2835`) | BVG-K Stage C2 (block propose/verify/apply) qualified: V0, RC, UT, JR, DC, V3, RF and C1R pass on synthetic fixture devices (3,419 updates; `Device::apply` never called). Stage T sheet exists as a machine-complete DRAFT, but registration is blocked on the W2 `H_law` predicate, a base lineage that reaches admission, and the margin/base-lineage registration items; no compute is requested and training stays HELD. | C2 receipt SHA-256 `2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`; `prep-check` on the Stage T DRAFT reports `complete: true`, `TEST FITNESS: PENDING`; the A8 dose Q1 stop (P_succ 1,351 → 551; FrozenZero 857) is in [the dose record](../../labs/a8-dose-2026-10-19/README.md), and the host census (Q1–Q5) in [the diagnosis record](../../labs/m1-a8-erosion-diagnosis/README.md). |

## Sources

- [`docs/harness/prep/m1-bvgk-stagec2-block-qualification-2026-10-18.md`](../../harness/prep/m1-bvgk-stagec2-block-qualification-2026-10-18.md) (C2 sheet and FIT).
- [`docs/harness/prep/m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md`](../../harness/prep/m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md) (Stage T DRAFT).
- [`docs/labs/injective-preflight-and-c2-qualify-2026-10-19/README.md`](../injective-preflight-and-c2-qualify-2026-10-19/README.md) (C2 PASS record).
- [`docs/labs/a8-dose-2026-10-19/README.md`](../a8-dose-2026-10-19/README.md) (the A8 dose result and its registered Q1 admission-collapse stop).
- [`docs/labs/m1-a8-erosion-diagnosis/README.md`](../m1-a8-erosion-diagnosis/README.md) (the host placement census that narrows, but does not answer, the erosion question).
- [`docs/integration/NOW.md`](../../integration/NOW.md), [`docs/integration/project-track.md`](../../integration/project-track.md) (the W2 status contradiction).