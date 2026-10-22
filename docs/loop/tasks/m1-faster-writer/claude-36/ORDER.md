# ORDER: m1-faster-writer, claude-36 (the DRAFT pre-registration sheet for the bucketed CUDA writer)

Lab: claude. Item: [m1-faster-writer](../../../../roadmap/items/m1-faster-writer.md) (#2840). Base: `e10d10926`. Item PR (partial: `References #2840`; the item stays `ready`).

## Goal

Unit 2 of three, following the design note on #2840 (6346787358) and the host controls merged in #2983 (`e10d109`). Item step 4: draft the one-page pre-registration sheet for the CUDA writer and its finite qualification. It proposes the bar's form, and Casey registers the bar. No kernel code and no GPU run happen in this unit.

## Cancel probe (run first)

At `e10d10926`, does any sheet in `docs/harness/prep/` already cover the faster writer, or is a writer bar already registered on #2840 or #2029? If one is, the task is void. Class: instrument.

## Files in scope

- `docs/harness/prep/m1-faster-writer-DRAFT-2026-10-22.md` (new)
- This task folder

## Done when

1. **The sheet exists.** It is named `-DRAFT-` and follows the format of `docs/harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md`. Every count and line it cites matches `discrete.cu` and `tests/writer_bucket_controls.rs` at the base. It states:
   - the question;
   - the kernel change (an nvcc-built module, no NVRTC);
   - the panel: seeded and pinned tables at V = 4,096 and 16,384, and kinds 0 to 3;
   - the counters and how they are built without changing the measured kernel;
   - the proposed bar: bit-identical outputs plus per-row counts against the full scan;
   - the controls and negatives;
   - the stop rule, cost, steps, and what each outcome decides.
2. **The bar is marked as proposed.** The sheet says "proposed, not registered" until Casey registers it in this chat. His words and the time are recorded on #2840.
3. **The review requests are out.**
   - A fresh-context review of the sheet is posted, because the PR is doc-only.
   - A non-author TEST FITNESS review is requested from another lab on #2028.
4. **The REPORT is checked:** `report --check` and `pr-check --local` pass.

## Stop and ask if

- The sheet needs a table or a 16,384-piece vocabulary that does not exist at the base, and a choice between substitutes would change what the bar means.
