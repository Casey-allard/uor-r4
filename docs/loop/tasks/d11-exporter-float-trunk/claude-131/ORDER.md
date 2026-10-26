# ORDER: d11-exporter-float-trunk, claude-131

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md). Base: `ef8fb33a0`.

## Goal

The item's done-when reads:

> A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. The trunk is a SpiralCore/FBS recurrence trunk with the Hamilton gate and the native vector writer head, as Stage 3 trains it. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

**This unit builds the configuration claude-130's decision review chose (D2): dual-forward training.**
- **What claude-130 found.** D11-mode training brought the artifact to within 25 and 12 rows of the model's own D11 and export-mode forwards. But the plain-float forward drifted away, and it is the sheet's comparator (CHARTER §3.7). The net gaps were +278 and −332 against F_I 69 and 59: outside, in opposite directions. The loss half held.
- **What changes.** Each training step's loss is the plain-float `MarginHinge` plus the D11-mode `MarginHinge`, weighted 1:1 and pinned before the run. Each loss's backward runs under its own gate mode (the icosian gate's custom backward reads the hook state), and the gradients are summed tensor by tensor. So both forwards are trained at once: the comparator is no longer left to drift, and the artifact's own forward still gets a signal.
- **Kept as claude-130 had them:** the bridges (the gates' and the transition's S³ tangent bridge, the Q12 query straight through), Stage 4b's data, steps, seeds and arm (`Fbs_1Atom_Active`), the exporter, claude-128's GPU evaluator, claude-129's scorer and Stage 4b's bar.
- **claude-130's low review findings, fixed here:**
  - the export receipt now carries each seed's training trajectory (`run`);
  - the Q12 bridge now passes no gradient where the i16 clamp saturates, as its forward does;
  - the Q12 bridge's tests stay on the GPU, by the GPU-only rule, and now cover the clamp.

## Cancel probe (run first)

Cancel if dual-forward training of the FBS trunk exists or is in progress. To check:
- `git log ef8fb33a0..origin/main -- crates/uor-r4-float-train docs/loop/tasks/d11-exporter-float-trunk docs/harness/prep/`;
- the open PRs;
- a search of `origin/main` for a dual training forward in `crates/uor-r4-float-train`.

Result (26 Oct):
- No commit after `ef8fb33a0` touches those paths.
- The open PRs are #3281 (this one), #3280 (antigravity), #3273 (kimi) and #3169 (codex). None of the other three touches float-train, the integer or training crates, or this item.
- `origin/main` has no `TrainForward` and no dual training forward.

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

1. **The dual forward.**
   - `run_fbs_arm_with` takes a `TrainForward` (`plain`, `d11` or `dual`); `plain` and `d11` train exactly as before.
   - In `dual`, each step's loss is the plain loss plus the D11-mode loss, 1:1, with each backward under its own mode and the gradients summed.
   - `spiralcore-fbs-sheet --train-forward dual` drives the probe and the export; the default stays `d11`.
   - A host test pins the names and the sheets; GPU tests pin the gradient sum and the Q12 clamp.
2. **A prep sheet, its FIT and a D67 pre-run review.** The sheet holds Stage 4b's bar, scored through the evaluator, with F this candidate's own plain-float forward, plus a throughput probe. It also carries the decision review's two conditions:
   - **A guarded PASS.** A PASS where either seed's artifact top-1 falls below claude-129's plain-trained artifacts (1,907 on seed 1, 2,294 on seed 2) is flagged, and does not close #2820 without a decision review.
   - **The seed spread, read out.** Each seed's plain-float and export-mode top-1, admitted rate and the two-seed loss band are reported, with a stated reading.
3. **The run.** Two seeds trained, exported and evaluated (G = H = R at every DEV-window position), and `G-D11-ExportGap` read.
4. **A REPORT and a notebook row.**
   - `Closes #2820` if the gap is inside the band on both seeds and the guard does not flag it.
   - Otherwise `References #2820`, with what the sheet's FAIL or flagged row decides.

## Stop and ask if

Stop and decide; never wait.
- **Weighting other than 1:1 looks better** after any reading: no. The weight is pinned before the run, and a change is a new configuration with its own sheet.
- **Dual training diverges, does not learn, or projects past about an hour per seed** on the throughput probe. Classify it as instrument or mechanism first; a mechanism result is a negative of this configuration only.
- **A step needs a non-geometric surrogate** (a softmax, a sigmoid or a smooth relaxation): no. Only the S³ tangent bridge and straight-through rounding are allowed.
- **The comparator itself** (CHARTER §3.7) looks wrong: that is the owner's. One `[claude, for Casey]` note on #2028, and the item parks.
