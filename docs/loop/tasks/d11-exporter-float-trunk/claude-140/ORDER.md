# ORDER: d11-exporter-float-trunk, claude-140 (why I-Prefix failed: same-device repeatability, source drift or cross-device drift)

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md) (#2820). Base: `12931837a`.

## Goal

**Find out why kimi-46's 2,048-step run did not reproduce claude-131's training at step 1,024.** That mismatch stopped kimi-46 at its registered I-Prefix gate, so no export-gap verdict exists. kimi-46's REPORT names this as the next unit: "distinguish same-device repeatability from cross-device/source-version drift with a separately FIT'd diagnostic". The gate must not be relaxed until this is known.

**What the two runs share and do not share.** Both ran on one RTX 5090 each, on different pods. Both are exactly equal at step 0 and differ from step 256 on.
- **Not the step count.** The d11 training loop uses a constant learning rate and `batches[step % len]` over a document-order prefix of windows. Nothing depends on the total step count. See `spiralcore_fbs_sheet.rs:461-495` and `hamilton_sheet.rs:1871-1885`.
- **Not the Q16 change.** It returns the tensor unchanged when the control is off (`hamilton_sheet.rs:600-608`).
- **Three candidate causes remain:**
  - **(a) Same-device nondeterminism.** The FBS backward kernel sums gradients with float `atomicAdd` (`spiralcore_fbs.cu:151`, "Batch sharing is summed with device atomics").
  - **(b) Source drift.** The training path changed between the two run commits, `faecaaac5` (claude-131) and `e77a1f91` (kimi-46): `spiralcore_fbs_sheet.rs` +2,988 lines, `hamilton_sheet.rs` +309.
  - **(c) Cross-device or driver drift.** The binaries carry `compute_120` PTX, which the driver compiles at load time. claude-131's pod had driver 595.80; kimi-46's run recorded none.

**Method.** One pod runs both runs' exact archived binaries, three times, with no rebuild:
- **R1 and R2:** kimi-46's `spiralcore-fbs-sheet`;
- **R3:** claude-131's.

Every evaluation point is compared with the others and with both archived receipts. The prep sheet sets out the arms, the readings and what each outcome means.

## Cancel probe (run first)

Cancel if the cause is already known, or another open order runs this diagnostic.

To check:
- the item's notebook page and #2820 after kimi-46;
- the open PRs.

Result (2026-10-27T02:50Z, `origin/main` at `12931837a`):
- #2820's last notes are kimi-46's merge (02:03Z) and this claim. No one has run a repeatability diagnostic.
- The open PRs are #3312 (antigravity, `m1-close-saved-model`) and this unit's #3315.

**Not cancelled (instrument: none needed).**

## Files in scope

- `docs/harness/prep/d11-exporter-float-trunk-prefix-repeatability-2026-10-27.md` (the prep sheet)
- `docs/research/notebook/m4-serving.md` (one row)
- this task folder

No code changes. The run uses the archived binaries as they are.

## Done when

1. The prep sheet has a fresh non-author `TEST FITNESS: FIT`, and a D67 pre-run review passes. Both are posted on #2820.
2. The three runs are complete, and the results are sealed in the store with a SHA256SUMS that is read back.
3. The outcome is classed by the sheet's decision table, with the comparison tables in the REPORT. One notebook row.
4. The pod is down, and a fresh exact-head review approves. `loop report --check`, `loop pr-check --local` and the required checks pass.

## Stop and ask if

Stop and decide; never wait.
- **An archived binary's sha256 does not match its pin,** or a binary will not start on the pod. The run stops, and the instrument is repaired.
- **R1 and R2 differ.** Same-device nondeterminism then confounds every cross-run comparison. Record it; the run completes; the outcome is (a).
