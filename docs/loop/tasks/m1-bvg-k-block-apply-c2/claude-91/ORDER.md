# ORDER: m1-bvg-k-block-apply-c2, claude-91

Lab: claude. Item: [m1-bvg-k-block-apply-c2](../../../../roadmap/items/m1-bvg-k-block-apply-c2.md) (#2835). Base: `af761681a`.

## Goal

The item's done-when reads:

> Already met: Stage C2 finite controls PASS (block apply exact) and are recorded, with the receipt above.
>
> Still to do: a one-page Stage T pre-registration sheet is posted on the issue, complete enough for a non-author TEST FITNESS: FIT review. Nothing in it runs until its FIT and the lab's own pre-run review (D67). Its bar is the Stage 2 bar: "beat the unigram table, then the admission-matched next-word table (the version W2 produces, not the 6,300 full-panel table), on two seeds." The estimate to carry is about $12 for two seeds.

Write that sheet, if its preconditions hold. Otherwise record why not, and route the item so `loop next` stops offering it before they can hold.

## Cancel probe (run first)

At `af761681a`, a FIT-ready Stage T sheet cannot be written. The binding reason is the first below; the second is supporting.

1. **No base lineage reaches its own admission gate. This is the binding reason.**
   - The draft `docs/harness/prep/m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md`, "Preconditions" item 2, requires a base that "passes its pre-flight and Q1 gates". Registration must fix that base, and a non-inferiority margin "from a measured two-seed dispersion of the chosen base".
   - On 19 Oct none did. The injective pre-flight admitted 127 of 3,847 P_succ rows at checkpoint 0, below 500, and `rows_v1` stopped at Q1 on admission collapse.
   - A sheet written today would leave its base, margin and cost blank, so no non-author FIT could pass it. That fails the C2 done-when ("complete enough for a non-author TEST FITNESS: FIT review").
   - The nearest candidate is A8's address-structured cells. That is the draft's "address-structured injective cell assignment" family: 1,351 of 3,847 admitted at the start, but eroded by Q1 in the first dose.
     - Its qualification (#2830) is finite controls only and produces no lineage.
     - Its full-dose rerun, `m1-a8-full-dose-rerun` (#2831, after #2830), is what would produce a base that passes Q1.
2. **W2 does not yet warrant a learning-law change. This is the supporting reason.**
   - `docs/roadmap/items/m1-bvg-k-stage-t-training.md` (#2836) says Stage T "is only warranted if W2 blames the learning law (H_law)".
   - W2 ran on 25 Oct (#2845, #3177, `docs/labs/m1-w2-capacity-capture/README.md`): **H_mixed**, row 3 of the registered sheet `docs/harness/prep/m1-read-code-capacity-2026-10-17.md`. That row's next step is "Code-utilization/capacity diagnostic; do not change the learning law yet".
   - Writing a sheet runs no compute and changes no law. But the diagnostic, `m1-w2-code-utilization` (#3179; sheet PR #3181), is what supplies the evidence for that call.

Class: **mechanism-independent sequencing.** The cancel closes this configuration, "write the Stage T sheet now". It closes no mechanism: BVG-K C2 stays qualified, and Stage T stays on the M1 path (D79).

## Files in scope

- `docs/loop/tasks/m1-bvg-k-block-apply-c2/claude-91/**` (this record)

## Done when

- The cancel is recorded with its evidence.
- The routing (both items depending on `m1-a8-full-dose-rerun` and `m1-w2-code-utilization`, so `loop next` stops offering the sheet) is done in a separate `infra-loop-harness` unit, claude-92. Item files are outside this item's scope, and the guard refuses them here.
- The decision is posted as `[claude] owner log:` notes on #2835 and #2836 (D69).
- A fresh-context review, which also serves as the decision review (D68), approves at the exact head.
- The PR says `References #2835` and `References #2836`. Neither item closes.

## Stop and ask if

Never stop and wait. If the decision review finds the W2 sheet or D79 says Stage T must be registered now regardless of the band, write the sheet instead, under this ORDER.
