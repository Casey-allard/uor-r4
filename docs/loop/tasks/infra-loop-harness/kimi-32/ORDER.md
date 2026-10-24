# ORDER: infra-loop-harness, kimi-32

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (PM premise record). Base: `127632bc3`. `References #2904`.

## Goal

Stop `loop next` repeatedly offering a finite D11 probe that has already answered its question, without claiming the full premise is closed. Record the existing M1 closing-bar hold through a PM decision row.

## Cancel probe (run first)

At `127632bc3`, `cargo test -p uor-r4-integer --lib discrete_d11` passes 3/3 on fresh main, independently confirmed by two labs. It adds no new information and does not meet the two-seed M1 closing bar. The premise remains unproven, so marking it `closed` would be false, while leaving it `open` repeats the same finished probe. The task stands. Class: PM record/instrument.

## Files in scope

- `PLAN.md`
- `docs/integration/DECISIONS.md`

## Done when

- The D11 premise row is `held`, not `closed`, with its next evidence named as the registered M1 closing bar. D60 explains the evidence, rejected alternatives (`closed` or repeated no-new-info probes), and low cost to reopen when that bar is ready. No roadmap item status, bar, owner hold or other PLAN content changes.
- `loop next` no longer offers the completed finite command; roadmap check/render/sync and REPORT, quick/full gates and local PR checks pass. An other-vendor exact-head review is counted before merge.

## Stop and ask if

The premise's scientific status, bar or owner release would change, or a file outside this list is needed.

Size estimate: about 25 lines.

## Owner approval

Owner: Casey Allard (project owner). This ORDER changes `docs/integration/DECISIONS.md`, an owner path, for D60 only.

**Recorded use of the owner's standing direction (deepseek, 2026-10-24T00:34Z).** The owner's own words, quoted: "yo just assume I say yes to this kind of stuff" — scoped by the same instruction to **routine bar/classification cards with a recommendation, with each use recorded**; held training, GPU, spend, charter changes and deletions still need the owner's own words.

D60 is a routine classification: the finite D11 probe is answered (3/3, independently confirmed twice) while the registered two-seed M1 closing bar has not run, so the premise becomes `held` — it changes no bar, no hold, no spend and no charter text, and reopening is cheap when the closing bar is ready.

The approval was recorded at **2026-10-24T00:34Z** on #2028 ([comment](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6377575257)) and on PR #3105 ([comment](https://github.com/UOR-Foundation/uor-r4/pull/3105#issuecomment-6377575521)). If the owner objects on reading this, the recording is void and this PR is to be reverted rather than merged.

— folded into this ORDER by deepseek on 2026-10-24T02:33Z under Casey's direct instruction in the deepseek chat: "you have permission to do whatever you need so that you do not sit around doing nothing / resume the loop".