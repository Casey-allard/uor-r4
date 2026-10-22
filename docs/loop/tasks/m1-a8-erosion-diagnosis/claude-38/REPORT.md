# REPORT: m1-a8-erosion-diagnosis, claude-38 (the design note for the host-only placement census)

Lab: claude. Base: `77fedb347`. Order: [ORDER.md](ORDER.md), committed first (`7ac949ae8`). The cancel probe found only my own claim on #2829, and no design note or census on #2029 or under `docs/labs/m1-a8-erosion-diagnosis/`. So the task stands.

## What did not work, first

- **The first draft took the archive from the 2026-10-20 index, not a live listing.** The read-only scout found no list command in `docs/reference/storage.md`. I then listed the store through the HF filesystem. It matches: 65 Relation checkpoints (00000…04096) and both gates receipts at 8,192 B.
- **The fresh self-review of `13963ca3c` returned CHANGES,** with one MEDIUM and three LOWs, all fixed in the next commit:
  - **MEDIUM: the split between update moves and the read leaked.** Admission takes the K = 8 *occupied* cells nearest `q`, so a cell-changing move of **any** token can change a row's admitted cells. Q4 now compares writes on targets, on their TRAIN predecessors and on the occupied-cell set. It points to the read only if Relation exceeds FrozenZero on none of the three.
  - **LOW:**
    - the cell rule now cites the kernel's `token_cell` and its ABSENT branch, not the test inverse alone;
    - the `token_cells.u16` format (little-endian u16, values 0–255) is stated;
    - the note says the census does not reuse `coverage_census::run`.
- **The interval count disagrees.** The attribution correction (6261186285) says "all 66 64-batch intervals (0→4,096)". The index holds 65 Relation checkpoints, which is 64 intervals. The note leaves this unresolved and makes it control 5.
- **The move logs carry no table values.** `steps.jsonl` is one `StepReceipt` per batch (`training.rs:2530`): coordinate kind and slot index, candidate, losses, `applied`. The census has to take the values before and after from the bracketing checkpoints.
- **The control eroded too.** FrozenZero lost 37% with 0 relation updates applied. Relation updates therefore cannot be the whole story. The note splits the question into the shared loss and Relation's extra ~300 positions.
- **One whole suspect is invisible on the host.** The gate-driven read change cannot be measured without device admission. The note marks every claim about it as co-occurrence only.

## What changed

| Commit | What |
|---|---|
| `7ac949ae8` | This ORDER, alone. |
| `de16f99a6` | `docs/labs/m1-a8-erosion-diagnosis/DESIGN.md`, also posted on #2829 (6349205403). |
| `13963ca3c` | This REPORT, first version. |
| this commit | The self-review fixes: Q4 over three write sets, the occupied-set route in §2, the cell rule and receipt format, and the live listing. |

The note sets out:
- the three suspects (update moves, read, instrument), and which are visible on the host;
- five questions, each with what it supports and what it does not;
- five controls to reproduce first: byte-equal checkpoint 0, G-Init against the A8 pin, placement facts and the GPU cell receipt, the receipt-to-checkpoint join, and the interval count;
- the exact files: 65 + 65 checkpoints at about 453 MB per arm, both `steps.jsonl`, the per-run `FILES.sha256`, and the gates' `token_cells.u16`;
- the tool: a table-only Rust host reader in `docs/labs/m1-a8-erosion-diagnosis/census/`.

## Measurements

None. This unit is the design only.

## Checks run at the head

- Every cited line was checked at the base: `README:24`, `:26`, `:45-48`, `:68-69`, `:83`, preflight `README:40`, `attention_learning.rs:33`, `training.rs:2530`, `snapshot.rs:12` and `address_cells.rs:286-298`. The Q1 counters were read from both summary JSONs.
- Claim-wording on `DESIGN.md`: passed.
- `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

scope-guard: one design note and this task folder. No code and no download.

## Still open

- **Unit 2:** the census tool, controls 1–5 on checkpoint 0 and the join, then Q1–Q5 and the notebook row.
- **The live archive listing,** before any download.

## Plain summary

This plans how to find out, from the saved training snapshots alone, whether the tokens were moved away from where the attention could find them, or whether the attention itself changed. It also says plainly which of those two the saved data cannot tell apart.
