# REPORT: m1-bvg-k-block-apply-c2, deepseek-32 (in-scope record and the Stage T registration state)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `20f618132`. Item PR, docs-only; the item stays `ready`.

## What did not work, first

- **The Stage T sheet cannot be completed or FITTED from this item today, for three independent reasons.** (1) The Stage 2 predicate is unresolved: D39 item 4 runs W3 only if W2 decides `H_law`, and the record contradicts itself — `NOW.md` row 6 says the W2 read-code capacity capture is **queued**, the roadmap item is **held**, and `project-track.md` says "W2 read-code capacity — DONE", which appears to refer to the v5.1 consumer-verifier ACCEPT rather than the capture. (2) No base lineage passes both its pre-flight and its registered Q1 admission gate: the injective pre-flight failed B3_init (127/3,847 < 500) and was not dispatched, while the A8 lineage reached B3_init (1,351/3,847, clearing the 500 floor) and then hit its registered Q1 admission-collapse stop (P_succ 1,351 → 551; FrozenZero 857/3,847) — a D31 diagnostic stop, not a mechanism negative. (3) The margin, base-lineage choice, T4 inclusion and AKd shadow-admission are registration-review items. So a sheet filed as FITTED today would be dishonest.
- **A complete-to-shape Stage T DRAFT already exists and belongs to claude's lineage** (`docs/harness/prep/m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md`, all 8 blocks, `TEST FITNESS: PENDING`). Writing a duplicate under this item would fork the lineage, so this unit points at it instead.
- **The item's step-6 notebook page is outside its scope.** `docs/research/notebook/m1-discrete-attention.md` is not in `scope:` and the item declares no `notebook:`; the row is staged in the in-scope lab record for the same backfill path #2997/#3002 used for other items.
- **No compute, no FIT request, no owner release.** Training stays HELD; nothing here reverses that.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `docs/labs/m1-bvg-k-block-apply-c2/README.md` | docs | new 32 lines | the in-scope record: C2 PASS receipt + controls, the Stage T DRAFT pointer, the three registration blockers, the staged notebook row and its backfill note |
| `docs/loop/tasks/m1-bvg-k-block-apply-c2/deepseek-32/{ORDER,REPORT}.md` | docs | small | this order and report |

**Size:** **104 insertions / 2 deletions** at the final head (`git diff --stat origin/main...HEAD`): the lab record is 32 new lines, the task folder 25 + 45, and the generated roadmap render is +2/−2 (four changed lines).

## Measurements

- **C2 evidence:** receipt `…/bvgk-qualification.json`, SHA-256 `2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`; controls V0, RC, UT, JR, DC, V3, RF, C1R; 3,419 updates on synthetic fixture devices; `Device::apply` never called.
- **Stage T DRAFT shape:** `prep-check` reports `complete: true`, `TEST FITNESS: PENDING` for `m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md`.
- **Predicate/lineage evidence:** `NOW.md` row 6 queued; `m1-w2-capacity-capture` `status: held`; `project-track.md` W2-DONE sentence; injective B3_init 127/3,847; A8 B3_init 1,351 then the registered Q1 stop (551 / 857) in the dose record; the host census (Q1–Q5) in the diagnosis lab record.

## Checks run at the head

- `uor-harness loop report --check`, `loop pr-check --local`, `loop gates --full`, `--break all`: run before the review request.
- A fresh-context same-lab review record is **to be posted at this head** (doc-only D46 exception, like #2998); the earlier reviewer's CHANGES at `84044e8` means the ORDER's review-record clause is met only by the repair's later APPROVE.

## Skills used

- **solution-integrity**: a sheet that cannot be honestly FITTED is not written as FITTED; the blockers are recorded with citations instead of papered over.
- **holding-the-goal**: the owner direction means deliver the useful, honest unit now and put the decisions on the issue, not start a GPU run or fork a lineage.

## Still open

- **PM questions** posted on #2835: (1) W2 capture status and whether `H_law` is decided; (2) base-lineage choice once a lineage reaches admission; (3) registration items (margin from measured two-seed dispersion, T4 inclusion, AKd shadow admission).
- **Backfill:** add a `notebook:` field to this item (umbrella-scope PR) and move the staged row to `m1-discrete-attention.md`.

## Plain summary

- **The C2 half of the item is done and recorded** (receipt + controls). The Stage T half cannot honestly complete: its predicate (W2 = `H_law`), its base lineage (none passes admission) and its registration numbers are all unresolved.
- **The in-scope record says exactly that**, points at the existing Stage T DRAFT instead of forking it, stages the notebook row for its backfill, and asks the PM/owner the two questions that unblock registration. No compute, no release, item stays `ready`.