# ORDER: m1-a8-erosion-diagnosis, claude-39 (the census tool and its five controls)

Lab: claude. Item: [m1-a8-erosion-diagnosis](../../../../roadmap/items/m1-a8-erosion-diagnosis.md) (#2829). Base: `ba7222b1b`. Item PR (partial: `References #2829`; the item stays `ready`).

## Goal

Unit 2 of the design merged in #2992 (`docs/labs/m1-a8-erosion-diagnosis/DESIGN.md`). It builds the host census tool and runs the design's five controls, then Q1 (placement per checkpoint) and Q3 (cell-changing writes per interval) on both arms to Q1.

The tool is a standalone Rust crate at `docs/labs/m1-a8-erosion-diagnosis/census/`, detached from the workspace, like `docs/labs/k16-timing-2026-10-15/instrument`. It reads only `.checkpoint.model.table` and `steps.jsonl`, plus the GPU receipt `token_cells.u16` for control 3.

Q2, Q4 and Q5 need the P_succ target tokens and their TRAIN predecessors. They are the next unit.

## Cancel probe (run first)

At `ba7222b1b`, does a census crate or a census result already exist under `docs/labs/m1-a8-erosion-diagnosis/`? If one does, the task is void. Class: instrument.

## Files in scope

- `docs/labs/m1-a8-erosion-diagnosis/census/**` (new crate)
- `docs/labs/m1-a8-erosion-diagnosis/README.md` (the results)
- This task folder

## Done when

1. **The crate builds and its tests pass.** Its finite tests use small synthetic tables and cover:
   - the cell rule, including ABSENT;
   - slot to token and block;
   - the join of changed slots to accepted `vector_block` indices.

   A planted fault in the cell rule fails a test.
2. **Every input is verified, then the controls run.** The inputs are the 130 checkpoints, both `steps.jsonl` and both receipts, held under `local/` and each checked against its run's `FILES.sha256`. Run on them, the five controls report PASS or FAIL with numbers:
   1. byte-equal checkpoint 0;
   2. G-Init: the checkpoint-0 words digest equals `4122cbab44bd88658743984fc1ab79059bf032c3e2fe526ffcf4ea5f95d9cd74`;
   3. 256 occupied cells, max occupancy 16, and the host cells equal `token_cells.u16`;
   4. changed slots equal accepted vector indices, per interval and arm, and gate writes change no slot;
   5. the interval count.
3. **Q1 and Q3 are tabulated** per arm and checkpoint in the README. The README states that a placement census is not admission.
4. **The checks pass:** `report --check` and `pr-check --local`. No retained file is edited, and the downloads stay out of git.

## Stop and ask if

- Control 1, 2 or 3 fails. That makes it an instrument finding: record it and stop before Q1 or Q3.
