# ORDER: one-model-end-to-end, claude-277 (P3(a): scoring passes side by side on one GPU)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `593e5b12c`. Draft PR #3664.

## Goal

Run [P3(a)'s sheet](../../../../harness/prep/one-model-end-to-end-side-by-side-2026-11-04.md) after its FIT and pre-run review. It re-scores P2b's sealed seed-1 checkpoints at P2b's pin with P2b's exact commands, on one RTX 5090:
- two passes at 192 serial against side by side, twice each;
- four at once;
- two identical passes at 2,048 side by side.

Every scored file must be byte-identical either way. Grade G and R1 to R4 on the laptop, and apply the sheet's Bar.

- **Plan step:** Casey's 4 November plan, P3 (faster scoring), part (a), arms side by side on one GPU (bound: 3 units for P3).
- **What it changes:**
  - **The job scripts.** On a positive reading, every job script on this line (P4 to P6) scores a seed's passes side by side, which cuts their wall time and pod cost.
  - **The 2,048 schedule.** R3 says how much idle time one 2,048 pass leaves under time-slicing between processes. That decides whether this line's 2,048 passes also run side by side. P3(c), memory on and off in one forward, is next whatever R3 reads, because D cannot see the room inside the kernels that P3(c) would use. The pre-run review caught this.
  - **Not the model:** no model, recall or served reading is taken.
- **Why now:** P2b's seed took 11,515 s on its pod, 2,910 s of it scoring. P5's scale probe scores many arms at 192, where one pass leaves the GPU idle about a quarter of the time (P2b's trace).
- **The pin** is `e1ad88afc5a7f43f5e7714d9ab70b8937aa6bdde`, the commit P2b scored at. That lets the run's files be set beside P2b's sealed files as a reported, cross-pod check.

## Cancel probe (run first)

1. Is the pin on `origin/main`?
2. Are P2b's seed-1 checkpoints and seal in the store?
3. Do the caps hold one more pod (at most 4 pods and $8/h across all labs)?
4. Does another open order hold these files, or does another lab run this question?
5. Are the job and the grade written, and the grade tested, before any output exists?

**Result, at `593e5b12c`: not cancelled.**
1. Yes. `git merge-base --is-ancestor e1ad88afc… origin/main` holds.
2. Yes. `uor-store ls` lists all ten checkpoint files:
   - (L) `s1/L/models/step-008192/`: 5 files, 61 MB;
   - (S) `s1/S/models/step-021845/`: 5 files, 108 MB.

   The seal `s1/SHA256SUMS.files`, fetched from the store, has SHA-256 `ca0b8aeda4e23a6f793df8b503534a48ff090724632dc384c66a0b4b27424b23`, the hash claude-276's REPORT names. The scoring inputs are the ones P2b fetched and checked.
3. Yes. `uor-pod status`: 0 pods, $0/h of $8/h; 9 verified 5090 offers, the cheapest at $0.975/h.
4. No. The only open PR on this item is this unit's draft, #3664. No other lab's open PR names scoring speed.
5. Yes. Both are written before any output exists:
   - **The job:** `local/claude/p3/job-p3a.sh`. It follows the sheet's Steps, and its scoring command is P2b's `score()` line unchanged.
   - **The grade:** `local/claude/p3/grade-p3a.sh`. It was tested on four synthetic run folders (`local/claude/p3/make-fixture.sh`):
     - equal files with B below A by 1.42× read **Positive**;
     - one changed file in `B2/2-S192` reads **Unread (instrument)** and names that pass;
     - one B on each side of A reads **Inconclusive**;
     - B within 2% of A reads **Negative**.

     Its R3 branch prints each of the sheet's three next steps.

   The launch SHA-256s of both go in the REPORT.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-277/**`
- `docs/harness/prep/one-model-end-to-end-side-by-side-2026-11-04.md`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

No code changes. The job and the grade are uncommitted lab scripts under `local/claude/p3/`.

## Done when

1. **The FIT:** a fresh non-author review of the sheet writes "TEST FITNESS: FIT", and that is posted on #3407.
2. **The pre-run review:** a fresh non-author review of the sheet, this ORDER, the job and the grade passes, and it is posted on #3407 before launch.
3. **The run:** it is run on one 5090, then sealed, put in the store and read back. The pod is down only after the read-back.
4. **The REPORT:**
   - G, I-3 and R1 to R4, tok/s first, and the Bar's word;
   - R3's next step for P3;
   - negatives first;
   - one notebook row, and a line on the item.

## Stop and ask if

Stop and decide; never stop and wait:
- **G or I-3 fails.** The schedule is not adopted. The REPORT classifies the fault as the scorer's own noise or running side by side. Nothing else is read as speed.
- **The pod or a phase dies.** Repair and rerun once. If D alone dies, the readings at 192 stand and R3 is unread.
- **The caps would be crossed.** Wait for a free slot inside them; never pass an override.
