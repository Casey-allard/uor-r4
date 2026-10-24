# ORDER: m1-address-as-read, kimi-36 (finish BPB status consistency repair)

Lab: kimi. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md) (#2833). Base: `bc87d5596`. Second repair for successor PR #3142.

Finished: initial pin `173d73ed2`/`40f80f8d2`; first repair `9d111c133` through `d8acb05b3`; DeepSeek's overlapping FIT prose repair integrated in merge `f7952a350`. Independent reviews at `40f80f8d` and `d8acb05b3` returned CHANGES.

## Goal

Finish the remaining status/prose reconciliation after #3140. Preserve history explicitly, correct current guidance, and avoid claiming surfaces outside this ORDER are reconciled. No registered value, artifact, arm, checkpoint, FIT verdict or hold changes.

## Cancel probe (run first)

The second review found deepseek-only FIT wording in sheet status history, stale panel/BPB prerequisite wording in README history, and an overbroad REPORT consistency claim. It also found the roadmap item's unregistered-bar sentence outside scope. #3140 repaired some occurrences; the rest still stand and need this repair.

## Files in scope

- `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md`
- `docs/labs/m1-address-as-read/README.md`
- `docs/loop/tasks/m1-address-as-read/kimi-36/**`

## Done when

- Sheet status and registration history are explicitly superseded where later updates changed prerequisites/FIT routing; current wording uses fresh-context non-author FIT, PM dispatch and owner approval in chat.
- README dated sections are explicitly historical where they mention panel/BPB prerequisites; current guidance matches the frozen panel, pinned stream and remaining checkpoint/FIT/dispatch/owner path.
- The active REPORT records the roadmap item's stale unregistered-bar sentence as a separate scoped PM follow-up and does not claim that surface is repaired here.
- All three stream hashes, panel-v2, 0.01, 19/40 and other registered values remain byte-identical in meaning; NOT_FIT and holds stand.
- Report/local PR checks, full gates and diff-check pass with retained output; another vendor reviews the new exact head on PR #3142.

## Stop and ask if

A registered value, artifact, bar, checkpoint, panel, tolerance, arm, hold or item status would change; the roadmap item must be edited under this ORDER; or any other file is needed.
