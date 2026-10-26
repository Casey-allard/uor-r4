# ORDER: d11-exporter-float-trunk, claude-130

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md). Base: `6bce5d5ed`.

## Goal

The item's done-when reads:

> A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. The trunk is a SpiralCore/FBS recurrence trunk with the Hamilton gate and the native vector writer head, as Stage 3 trains it. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

**This unit builds the configuration claude-129's FAIL row names: train the trunk in the D11 mode before export.**
- claude-129 measured the exported artifact's own gap. The top-1 half missed on both seeds (net 81 and 349 against F_I 60 and 70), while the loss half held.
- The next configuration trains on the artifact's computation, so that rounding into the artifact at export changes little.

**The D11-mode forward** is `hamilton_sheet::with_d11_mode`:
- both Hamilton gates and the recurrence transition snapped to the 120 icosians;
- the scored query at Q12.

**What it needs in order to train.** Its snaps and roundings either pass no gradient, or pass one only on the gate path:
- **The gates:** `Hamilton_IcosianHard` already trains through its snap with the straight-through S³ tangent-space bridge (`raw_hamilton_gate_bwd`).
- **The transition:** its snap (`icosian_snap_quaternions`) builds a new tensor, with no gradient.
- **The Q12 query:** `round()` passes none.

**The plan:**
1. **Bridges, for training only.** The transition gets the same geometric straight-through bridge as the gates: the forward is the snapped root, and the backward runs through the unit normalization, the S³ tangent projection. The Q12 query gets a straight-through rounding.

   Both are active only under a training flag, so the evaluation forward stays bit-identical to claude-125's D11 mode, which the GPU evaluator matches. The exported tables are rounded at export, as before.
2. **A D11-mode training arm** in the FBS sheet: `Fbs_1Atom_Active` trained with the D11-mode forward and the bridges, on Stage 4b's data, steps and seeds.
3. **The gap.** Export each seed. Score the artifact's own gap with claude-128's evaluator and claude-129's scorer, against the arm's own float panel and Stage 4b's noise band, under Stage 4b's bar.

## Cancel probe (run first)

Cancel if a D11-mode training of the FBS trunk exists or is in progress. To check:
- `git log 6bce5d5ed..origin/main -- crates/uor-r4-float-train docs/loop/tasks/d11-exporter-float-trunk`;
- the open PRs;
- a search of `crates/uor-r4-float-train` for a transition-snap backward.

Result:
- No commit after `6bce5d5ed` touches those paths.
- No open PR other than #3279 (this one) touches float-train or this item.
- `icosian_snap_quaternions` has no backward.

**Not cancelled (instrument: none needed).**

## Files in scope

- `crates/uor-r4-training/**`
- `crates/uor-r4-integer/**`
- `crates/uor-r4-float-train/**`
- `docs/harness/prep/d11-exporter-float-trunk*`
- `docs/labs/d11-exporter-float-trunk/**`
- `docs/loop/tasks/d11-exporter-float-trunk/**`
- `docs/research/notebook/m4-serving.md`

## Done when

1. **The bridges.**
   - The training flag routes the transition snap and the Q12 query through straight-through bridges.
   - With the flag off, the D11 forward is unchanged. claude-125's and claude-126's D11 tests pass as they are.
   - Host tests pin both: the bridge passes a gradient, and with the flag off the forward equals the snap.
2. **The arm, a prep sheet, its FIT and a D67 pre-run review.** The sheet holds Stage 4b's bar, scored through the evaluator, plus a throughput probe.
3. **The run.** Two seeds trained, exported, evaluated (G = H = R at every DEV-window position), and `G-D11-ExportGap` read.
4. **A REPORT and a notebook row.**
   - `Closes #2820` if the gap is inside the band on both seeds.
   - Otherwise `References #2820`, with what the sheet's FAIL row decides.

## Stop and ask if

Stop and decide; never wait.
- **A bridge needs a non-geometric surrogate** (a softmax, a sigmoid or a smooth relaxation): no. Only the S³ tangent bridge and straight-through rounding are allowed.
- **Training in the D11 mode diverges or does not learn** on the throughput probe. Classify it as instrument or mechanism first; it is a negative of this configuration only.
