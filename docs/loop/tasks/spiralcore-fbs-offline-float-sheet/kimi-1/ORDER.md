# ORDER: spiralcore-fbs-offline-float-sheet, kimi-1

Lab: kimi. Item: [spiralcore-fbs-offline-float-sheet](../../../../roadmap/items/spiralcore-fbs-offline-float-sheet.md) (#2891). Base: `ea373090`.

## Goal

Restore the main-line D23 scanner and harness tests after #3252 added one comment-token hit in `crates/uor-r4-float-train/src/spiralcore_fbs_sheet.rs`. Change only the offending comment; preserve all model behavior and the completed item's status.

## Cancel probe (run first)

`uor-harness ratchet d23` reports exactly one file above baseline: `crates/uor-r4-float-train/src/spiralcore_fbs_sheet.rs` (0 -> 1). The only `softmax` token in that file is in the comment on line 13. This is an instrument/comment defect, not a mechanism negative.

## Files in scope

- `crates/uor-r4-float-train/src/spiralcore_fbs_sheet.rs`
- `docs/loop/tasks/spiralcore-fbs-offline-float-sheet/kimi-1/**`

## Done when

- The comment no longer trips the D23 scan; no code or model behavior changes.
- `uor-harness ratchet d23` and `cargo test -p uor-r4-harness --lib` pass, with the previous four failures resolved.
- `uor-harness loop report --check`, `uor-harness loop pr-check --local`, `uor-harness loop gates --full` and `git diff --check` pass.
- The in-unit fresh-context review approves the exact head; CI is green before queue.

## Stop and decide, never wait

Any requested code, mechanism, sheet, bar, hold or GPU change is out of scope and needs a new ORDER.
