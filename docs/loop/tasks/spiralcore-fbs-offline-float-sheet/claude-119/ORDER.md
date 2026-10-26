# ORDER: spiralcore-fbs-offline-float-sheet, claude-119

Lab: claude. Item: [spiralcore-fbs-offline-float-sheet](../../../../roadmap/items/spiralcore-fbs-offline-float-sheet.md). Base: `6b8974a62`.

## Goal

The item's done-when reads:

> Descent and tokens per second are reported from an offline-float sheet with a matched control.

This unit builds and runs antigravity-81's registered sheet, [`spiralcore-fbs-offline-float-sheet-2026-10-25.md`](../../../../harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md), merged with its FIT in #3233. It continues [claude-118](../claude-118/ORDER.md), which restored `crates/uor-r4-float-train/**` to the item's scope under a D68 decision (#3251, owner log #2891 comment `6405361056`). The four arms are `Fbs_1Atom_Active`, `Fbs_2Atom_Mix`, `Fbs_IdentityTransport_Control` and `Fbs_FrozenQueries_Null`. No arm, bar or pin changes.

## Cancel probe (run first)

Class: instrument. Run at `6b8974a62`; all must hold:

- **The scope is live.** `origin/main`'s item scope includes `crates/uor-r4-float-train/**`. **Holds.**
- **The runner is not on main.** There is no FBS sheet binary, and no 2-atom or identity-transport arm. **Holds.**
- **Nobody else builds it.** No other branch or claim exists for this item, and no open PR touches `crates/uor-r4-float-train`. **Holds.**

## Files in scope

The item's scope: `crates/uor-r4-float-train/**`, `docs/mechanisms/spiralcore-fbs.md`, `docs/harness/prep/spiralcore-fbs-offline-float-sheet*`, `docs/labs/spiralcore-fbs-offline-float-sheet/**` and `docs/loop/tasks/spiralcore-fbs-offline-float-sheet/**`. The item's notebook, `docs/research/notebook/float-training.md`, is also allowed.

**The D68 conditions, word for word from the owner log:**
- The three pinned files stay untouched: `spiralcore_fbs.cu`, `spiralcore_fbs.rs` and `cuda_stack.cu`.
- The new arm kernels go in a new `.cu` file, built as its own `build.rs` module and PTX.
- The arm hooks go in new or unpinned files.
- The run receipt records the new kernel source and PTX sha256. The pre-run review re-hashes the §5 pins on the pod.
- No new dependencies.
- §3 holds: no softmax or squashing, no matmul, nvcc ahead of time with no NVRTC. The 1/√2 combine is a constant factor.
- The in-crate Q12 oracle is checked against the existing integer SpiralCore/FBS form on the I-Export-Exactness fixtures.

## Done when

Built and run in stages, each committed with its own checks:

1. **The arms** (`src/spiralcore_fbs_arms.cu`, `src/spiralcore_fbs_arms_prelude.cu`, `src/spiralcore_fbs_arms.rs`, and a new `build.rs` module `spiralcore_fbs_arms_f32.ptx`).
   - Identity transport, `y = x_σ(o)`.
   - The 2-atom mix, `y = (A_1 x_σ1 + A_2 x_σ2)/√2`, from queries of shape `[outputs, 2·stages, 8]`.
   - Frozen queries: the standard projection with `dq = 0`, and the queries excluded from both optimizer updates, so they keep their seed-`s` values.
   - A thread-local arm hook, read in `StackModel::linear` (unpinned), and `StackModel::widen_fbs_queries_two_path`.
   - Finite CUDA tests (`#[ignore]`d, run on the pod) pin each arm against a reference or an exact invariant, and fail when the arm is set aside.
2. **The runner**, `src/spiralcore_fbs_sheet.rs` with the bin `spiralcore-fbs-sheet --mode synthetic-probe|throughput-probe|full`.
   - The model is the sheet's own: `hamilton_sheet::build_hamilton_model`, with the same steps, seeds and data.
   - It reuses the Hamilton DEV loader, windows, `evaluate_dev_panel` (float and the D11-compatible export mode), the export gap and the paired discordance.
   - It records `tokens/s` (decode, context, training), I-1 to I-4, the four gates, and the kernel and pin hashes.
3. **The type check:** stub-nvcc `--features cuda` and clippy; the host tests pass.
4. **The pod,** after a D67 pre-run review: one RTX 5090, running probe (a), probe (b), then `full` on builds A and B, with `fresh_v2` only if its gates pass. The store gets `results/claude/spiralcore-fbs-offline-float-sheet-2026-10-25/` with `SHA256SUMS`.

Then:
- one row in `float-training.md`;
- the REPORT, with negatives first;
- a fresh claims check, and the exact-head review;
- `Closes #2891` when the item's done-when is met.

## Stop and ask if

Stop and decide, never stop and wait.

- **An instrument check (I-1 to I-4) fails.** Then follow the sheet's stop rule: classify it as an instrument fault, repair, re-run. It is never a negative.
- **A registered arm cannot be built as the sheet defines it.** Then write a D68 decision card, and never substitute an arm silently.
- **The build alone fills one PR.** Then land stages 1 to 3 as References #2891, and run in the next unit.
