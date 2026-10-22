# ORDER: m1-bvg-k-block-apply-c2, deepseek-32 (in-scope record and the Stage T registration state)

Lab: deepseek. Item: [m1-bvg-k-block-apply-c2](../../../../roadmap/items/m1-bvg-k-block-apply-c2.md) (#2835). Base: `20f618132`. Item PR (docs-only; the item stays `ready`).

## Owner direction

Casey, in the DeepSeek lab chat on 2026-10-22 (observed 20:03:36Z–20:06:30Z UTC), verbatim: "please find a way to proceed without waiting on other agents if for some reason they are busy... maybe a subagent advesarial analysis or something?" This ORDER acts on that direction and on the same no-wait pattern used for the m3 close-out: deliver the in-scope record and put the PM/owner decisions on the issue, rather than waiting.

## Cancel probe (run first)

Class **instrument** (docs-only). At the base: Stage C2 is already **PASS** with a receipt (`2fff942f…`), and a machine-complete Stage T **DRAFT** exists (`docs/harness/prep/m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md`, `TEST FITNESS: PENDING`). But its registration preconditions are unmet: the W2 `H_law` predicate is contradicted in the record (`NOW.md` queued / roadmap `held` vs `project-track.md` "DONE"), no base lineage reaches its own admission gate (injective 127/3,847 < 500; A8 dose Q1 admission-collapse stop), and the margin/base-lineage/T4/AKd items are registration-review decisions. The sheet cannot be honestly FITTED from this item, and its only file is outside this item's `scope:`. The unit is the in-scope record + the decision note.

## Files in scope

`docs/labs/m1-bvg-k-block-apply-c2/**` (new lab record), and this task folder. Docs-only; no receipts file (D46 doc-only same-lab fresh-context record). No `crates/` change, no `docs/harness/prep` edit (the Stage T DRAFT lives in claude's lineage), no GPU, no owner release.

## Done when

- `docs/labs/m1-bvg-k-block-apply-c2/README.md` records the C2 PASS receipt and controls, points at the Stage T DRAFT, and states the three registration blockers; the staged notebook row for `m1-discrete-attention.md` is included with its backfill note (the page is outside this item's scope; the `notebook:` field path is #2997/#3002's pattern).
- A design/status note is posted on #2835: construction summary (K coordinates per launch; a block committed only when `l_joint < l_best`; integer hinge/table moves/adds; no softmax/matmul), the finite controls that already pass, how C2's uncovered list is covered by the sheet's first-batch controls (V1d/KId/V8d/V9d/AKd), and the two PM questions (W2 status/`H_law`; base-lineage/registration items).
- `report --check`, `pr-check --local`, `loop gates --full`/`--break all`, and a same-lab fresh-context record at the head (doc-only exception) pass.

## Stop and ask if

- Someone argues the item's done-when needs the sheet FITTED now: it cannot be, on the evidence above; that is a PM/owner decision recorded in the note, not a lab call.