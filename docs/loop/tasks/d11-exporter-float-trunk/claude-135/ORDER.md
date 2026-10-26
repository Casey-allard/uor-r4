# ORDER: d11-exporter-float-trunk, claude-135

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md). Base: `8ac2ad9e3`.

## Goal

The item's done-when reads:

> A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. The trunk is a SpiralCore/FBS recurrence trunk with the Hamilton gate and the native vector writer head, as Stage 3 trains it. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

**This unit is the forced-snap control.** It is the next step set by the decision review recorded after [claude-134](../claude-134/REPORT.md) (#2820 owner log, comment 6424294556).
- **Where the line stands.** claude-134 left all six of claude-132's models Indeterminate: every window flips early, so placement cannot say whether the snaps cause the 2–5% residual. Every first flip is a near-tie.
- **The control is an intervention.** Run the D11 forward (D) with every icosian snap forced to the artifact's (A's) traced choice, over the full DEV panel. Count the rows where forced-D and A still differ. At every forced site, record D's own choice and margin as its would-flip: with the states held together, each would-flip is a flip on its own, not a carried one.

**The plan** (each part GPU-only, per CHARTER §3.5):
1. **The float side's snap override** (`hamilton_sheet.rs`). `with_snap_override` applies a given root per snap call and block inside `icosian_snap_cuda_tensor`, where an integer zero block (255) applies root 0. `with_snap_capture` still records D's own choice and margin there. With the override off, nothing changes.
2. **A comparison mode,** `spiralcore-fbs-sheet --mode d11-forced-snaps`, per model over all 428 DEV windows (14,287 rows).
   - Its readings:
     - forced-D against A;
     - the would-flips: count, rate per site, D's margins, exact ties;
     - unforced D against A, for the reference.
   - Its instrument gates:
     - D forced to its own captured choices reproduces D exactly;
     - a forced wrong root changes the tokens;
     - A's trace is G = H on the full panel, its tokens equal the stored, qualified run, and `I-Layout` holds.
3. **A prep sheet** with numeric bars, an intermediate outcome, and the faithful models as the reference; then its FIT, a D67 review, and one pod run over the six models. The evaluator is unchanged, and its trace kernel is qualified at PTX `5a947f09…`.

## Cancel probe (run first)

Cancel if a snap override of the float D11 forward, or a forced-snap comparison, exists or is in progress. To check:
- `git log 10a6b2c2e..origin/main` on float-train, the instrument and this item's tasks;
- the open PRs;
- a search of `origin/main` for a snap override in `hamilton_sheet.rs`.

Result (26 Oct, `origin/main` at `8ac2ad9e3`):
- No commit after `10a6b2c2e` touches float-train, the instrument or this item's tasks.
- The other open PRs are #3295 (kimi) and #3169 (codex), and neither touches a file in this scope.
- `origin/main` has no snap override in `hamilton_sheet.rs`.

**Not cancelled (instrument: none needed).**

## Files in scope

- `crates/uor-r4-float-train/**`
- `docs/harness/prep/d11-exporter-float-trunk*`
- `docs/labs/d11-exporter-float-trunk/**`
- `docs/loop/tasks/d11-exporter-float-trunk/**`
- `docs/research/notebook/m4-serving.md`

## Done when

1. **The snap override,** with tests:
   - off, it changes nothing;
   - forcing D's own choices reproduces D bit for bit;
   - a forced wrong root changes the output.

   A GPU test pins all three. Host tests pin the override's layout and chunk order.
2. **The comparison mode,** with host tests of its arithmetic and its outcome rule.
3. **A prep sheet, its FIT, a D67 review, and the run,** with every receipt in the store.
4. **A REPORT and a notebook row,** with `References #2820`.

## Stop and ask if

Stop and decide; never wait.
- **The override changes D with it off,** or self-forcing does not reproduce D: that is a bug. Fix it before any run.
- **The evaluator would need to change:** requalify it as claude-134 did, never read past it.
- **A host reference run:** no, that is a host fallback (§3.5).
- **The bar or the comparator** (CHARTER §3.7): the owner's, already asked on #2028. Nothing waits on it.
