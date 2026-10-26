# ORDER: d11-exporter-float-trunk, claude-134

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md). Base: `ea18d5064`.

## Goal

The item's done-when reads:

> A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. The trunk is a SpiralCore/FBS recurrence trunk with the Hamilton gate and the native vector writer head, as Stage 3 trains it. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

**This unit is snap-flip localisation, the next control named by claude-133** ([REPORT](../claude-133/REPORT.md)).
- **Where the line stands.** claude-132's residual check left four of six models indeterminate: the D11 panel (D) and the artifact (A) differ on 3.4% to 4.9% of tokens. claude-133's investigation found "other arithmetic". The Q16 control explains nothing, and no model shows a gross signature beyond the faithful models' envelope.
- **The hypothesis.** It is untested. An icosian snap whose candidates nearly tie flips between the float D11 forward and the integer artifact, and the recurrence carries the flip to later positions.
- **The control.** At each snap site and position, record which of the 120 icosian roots each side chooses:
  - the transition, one per lane;
  - the output gate on the state, one per lane;
  - the feed-forward Hamilton gate, one per 4 MLP channels;

  each per layer. Then place every disagreement relative to the rows where D and A differ, and measure D's own margin at the disagreeing site.

**The plan** (each part GPU-only, per CHARTER §3.5):
1. **The evaluator's snap trace** (`docs/labs/d11-exporter-float-trunk/instrument`).
   - `fbs_core.h` gains an optional trace pointer. When it is non-null, `fbs_recurrence` and `fbs_hamilton_gate` write each chosen index (`fbs_snap_select`) at its site.
   - The CUDA kernel and the host twin pass it through.
   - With the trace off, the computation is unchanged. A new `d11-fbs-eval trace` command writes the trace for a chosen set of windows.
   - **Requalification:** the integer-only PTX admission and G = H = R on the full DEV panel, as claude-128 qualified them. The trace itself must agree between the GPU and the host twin (G = H).
2. **The float side's snap indices.** In the D11 mode, a capture hook records the index of each snapped root, at the same sites, from the snap kernel (`icosian_snap_cuda_tensor`). Each index comes from an exact root match.
3. **A comparison mode,** `spiralcore-fbs-sheet --mode d11-snap-trace`, per model over the chosen windows:
   - per site and layer, the positions where D's and A's choices disagree;
   - whether each D-against-A token difference is preceded, in its window at or before its position, by a snap disagreement;
   - D's dot-product margin between its own choice and A's choice at each disagreeing site, which is a near-tie reading.
4. **A prep sheet, its FIT, a D67 review, and one pod run** over claude-132's six models. The windows are those with the most D-against-A token differences, and the sheet pins how many. The sheet also pins the classification:
   - **snap-flip arithmetic:** the token differences follow near-tie snap disagreements;
   - **trunk departure, localise first:** token differences with no snap disagreement before them;
   - **indeterminate:** anything else.

## Cancel probe (run first)

Cancel if a snap trace of the evaluator, or a snap-choice capture of the float D11 forward, exists or is in progress. To check:
- `git log ea18d5064..origin/main -- docs/labs/d11-exporter-float-trunk crates/uor-r4-float-train docs/loop/tasks/d11-exporter-float-trunk`;
- the open PRs;
- a search of `origin/main` for a trace in `fbs_core.h` and a snap-index capture in `hamilton_sheet.rs`.

Result (26 Oct):
- No commit after `ea18d5064` touches those paths.
- The other open PRs are #3288 and #3287 (antigravity), #3283 (kimi) and #3169 (codex). None touches the instrument, float-train, the integer crate or this item. #3287 (`m1-export-gap-and-toks`) holds only its ORDER.
- `origin/main` has no trace in `fbs_core.h`.
- The evaluator's qualified PTX hash (`013d19fd…`) is recorded only in receipts and REPORTs, not pinned in code. So the trace's new PTX is qualified on the pod, and its pins are recorded then.

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

1. **The evaluator's trace and its requalification.**
   - With the trace off, the evaluator is unchanged: G = H = R on the full DEV panel, and the integer-only PTX admission.
   - With the trace on, the trace is G = H on the chosen windows.
   - The new PTX and cubin pins are recorded. A host test pins the trace's layout on a synthetic trunk.
2. **The float capture.** Its indices are exact root matches, and its off state is unchanged. A GPU test pins both.
3. **The comparison mode,** with host tests of its arithmetic.
4. **A prep sheet, its FIT, a D67 review, and the run,** with every receipt in the store.
5. **A REPORT and a notebook row,** with `References #2820`.

## Stop and ask if

Stop and decide; never wait.
- **The trace changes the evaluator's result with it off:** that is a bug. Fix it before any run.
- **The requalification fails:** that is an instrument fault. Repair it, never read past it.
- **The control needs a host reference run:** no, that is a host fallback (§3.5).
- **The bar or the comparator** (CHARTER §3.7): the owner's. It is already asked on #2028.
