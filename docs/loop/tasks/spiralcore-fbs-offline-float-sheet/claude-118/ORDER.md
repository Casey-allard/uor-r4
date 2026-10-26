# ORDER: spiralcore-fbs-offline-float-sheet, claude-118

Lab: claude. Item: [spiralcore-fbs-offline-float-sheet](../../../../roadmap/items/spiralcore-fbs-offline-float-sheet.md). Base: `383ffa813`.

## Goal

The item's done-when reads:

> Descent and tokens per second are reported from an offline-float sheet with a matched control.

The sheet is antigravity-81's registered [`spiralcore-fbs-offline-float-sheet-2026-10-25.md`](../../../../harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md), merged in #3233 with its FIT. Its four arms are `Fbs_1Atom_Active`, `Fbs_2Atom_Mix`, `Fbs_IdentityTransport_Control` and `Fbs_FrozenQueries_Null`. Its checks are I-1 to I-4, and its gates are G-Fbs-Sheet-Fit, G-Fbs-Capacity, G-Fbs-ExportGap and G-Fbs-Transfer.

This unit builds the sheet's runner in `crates/uor-r4-float-train` and runs it on one RTX 5090. It changes no arm, bar or pin.

## Cancel probe (run first)

Class: instrument. Run at `383ffa813`; all must hold:

- **The runner does not exist.** No `Fbs_2Atom_Mix`, identity-transport or frozen-query FBS arm and no FBS sheet binary are on main. `secant_descent.rs` has `FbsZeroCreditNull` as a query-credit hook, which the frozen-query arm can reuse. **Holds.**
- **Nobody else is building it.** No branch or worktree for this item exists apart from `claude/spiralcore-fbs-float`. The antigravity-81 REPORT lists the runner as still open. Antigravity's drive stalled ("spiraled") and holds no claim. **Holds.**
- **The scope can be restored.** #3232 narrowed the item to docs while #3231 held the crate. #3231 merged at 18:05Z on 25 Oct, and no open PR touches `crates/uor-r4-float-train`. A fresh D68 decision review approved restoring the crate glob only, not `crates/uor-r4-integer`, with conditions (#2891, owner log 6405361056). **Holds.**
- **The pins still match.** The sheet's three §5 sha256s match origin/main (the decision review checked them). **Holds.**

## Files in scope

- `crates/uor-r4-float-train/**`, under the D68 conditions below
- `docs/mechanisms/spiralcore-fbs.md`
- `docs/harness/prep/spiralcore-fbs-offline-float-sheet*`
- `docs/labs/spiralcore-fbs-offline-float-sheet/**`
- `docs/loop/tasks/spiralcore-fbs-offline-float-sheet/**`
- the item file's `scope:` line and the rendered roadmap, for the D68 scope change only

**D68 conditions:**
- The pinned `spiralcore_fbs.cu`, `spiralcore_fbs.rs` and `cuda_stack.cu` stay byte-identical.
- The new arm kernels go in a new `.cu` file, built as its own `build.rs` module and PTX.
- The arm hooks live in new or unpinned files.
- No new dependency, and no change to `crates/uor-r4-integer`.
- The run receipt records the new kernel source and PTX sha256.

## Done when

**This unit lands only the scope decision.** The loop guard reads an item's scope from `origin/main`, so that "a branch cannot widen its own scope" (`crates/uor-r4-harness/src/loopguard.rs`). A crate file could not be written until the restored scope merged, and the first kernel write was refused. Under this ORDER's own stop rule ("grows past one PR"), the work splits:

- **claude-118 (this PR):** the item's `scope:` line gains `crates/uor-r4-float-train/**`, under the D68 decision above. The owner log is posted on #2891, the REPORT is written, and the PR is reviewed, merged and verified as **References #2891**.
- **The next claim on this item** builds and runs the sheet in the stages below, under its own ORDER. That ORDER carries this one's design and D68 conditions.

The stages for that next unit, each committed with its own checks:

1. **The arms.**
   - The identity-transport and 2-atom kernels go in a new `src/spiralcore_fbs_arms.cu`, with a thread-local arm hook in the unpinned CUDA launcher.
   - The frozen-query arm reuses the zero-credit hook.
   - The 2-atom arm adds a second, independently seeded query parameter per FBS projection, trained by `StackAdamW`.
   - Finite CUDA tests (`#[ignore]`d, run on the pod) pin each kernel against a host reference. They fail when the arm is set aside.
2. **The runner**, `src/spiralcore_fbs_sheet.rs` with the bin `spiralcore-fbs-sheet --mode synthetic-probe|throughput-probe|full`.
   - The model shape, steps, seeds and data are exactly the sheet's.
   - It reuses the Hamilton runner's DEV loader, windows, evaluator and paired discordance.
   - It records `tokens/s` (decode, context, training), I-1 to I-4, the gates, the kernel and pin hashes, and receipts.
3. **The D11 export.** The trained `(atom: u16, state: u8)` table and the Q12 `i32` writer oracle feed `G-Fbs-ExportGap`. On the I-4 fixtures, the oracle is checked against the integer SpiralCore/FBS form.
4. **The type check.** A stub-nvcc check with `--features cuda`, and clippy.
5. **The pod**, after a D67 pre-run review: one RTX 5090 through `uor-pod`, running probe (a), probe (b), then `full` on builds A and B, with `fresh_v2` only if its gates pass. Results go to the store, `results/claude/spiralcore-fbs-offline-float-sheet-2026-10-25/`, with `SHA256SUMS`.

Then:
- one row in `docs/research/notebook/float-training.md`;
- the REPORT, with negatives first;
- a fresh claims check, and the exact-head review;
- `Closes #2891` if the sheet's done-when is met.

## Stop and ask if

Stop and decide, never stop and wait.

- **An instrument check (I-1 to I-4) fails.** Then follow the sheet's stop rule: classify it as an instrument fault, repair, re-run. It is never a negative.
- **A registered arm cannot be built as the sheet defines it.** Then write a decision card under D68, and never substitute an arm silently.
- **The work grows past one PR.** Then land stages 1 to 4 as References #2891, and run in the next unit.
