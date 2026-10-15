# #1545 — fix(training): pass snap arg to export_stack in d4_map_codec_adapter test

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-30T14:08:00Z
- **Updated:** 2026-09-30T16:04:29Z
- **Closed:** 2026-09-30T14:45:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1545
- **Merged:** not merged
- **Merge commit:** ecdbe474ebe4d7df7e7b0a90dbc101663378ea53
- **Base / Head:** main / UOR-Foundation:codex/d4-export-stack-snap-arg
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# fix(training): pass snap arg to export_stack in d4_map_codec_adapter test

## What

`crates/uor-r4-training/tests/d4_map_codec_adapter.rs` calls `export_stack` with 3 arguments at 7 sites, but #1490 added a fourth parameter (`snap: Option<TransportSnap>`) to `export_stack` without updating the test. The result is 7× `E0061` at `cargo check --workspace --all-targets` — the **only** compile failure in main's fast-gate baseline (run [36723113371](https://github.com/UOR-Foundation/uor-r4/actions/runs/36723113371)).

This PR adds `None` as the snap argument at all 7 sites (lines 215, 355, 591, 781, 1010, 1046, 1164).

## Why `None` is correct

The test models are built with `StackModel::new(StackConfig{..})` and `set_served_representation`; none set a transport snap, so `model.transport_snap()` is `None` and `export_stack`'s snap-required refusal does not apply. The refusal tests (including `test_export_stack_directly_refuses_e8_matched_bit`, which asserts the E8-codec refusal message) keep their exact meaning — the snap check passes with `None` and the E8 refusal still fires.

## Scope and relationship to #1538

This is the **only** compile failure on main. #1538's branch carries the same fix for this file but also introduces an unrelated `uor-r4-workbench` compile failure (E0034/E0308/E0277) that DeepSeek must resolve there. This tiny PR greens main's baseline independently and immediately; #1538's identical lines become a no-op on rebase. It does not touch #1538's S1.4 scope.

## Verification

- Diff: 1 file, 7 insertions / 7 deletions, all `None, None`.
- Merge-rule test evidence: `fast-gate` (product_path) dispatched on this branch head (GitHub runners; main's deps — including the git-pinned `uor-foundation` and the `candle-core` path patch — resolve cleanly there per the baseline run).

References #820, #1490.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5913467632 — 2026-09-30T14:36:16Z — Casey-allard

_edited 2026-09-30T14:36:22Z_

**Claude lab: this PR is now a no-op.** #1538 merged at 14:30 UTC as `dc828747` and carries the same fix: all 7 `export_stack` sites in `crates/uor-r4-training/tests/d4_map_codec_adapter.rs` now pass `None`. `main`'s copy of that file is **byte-identical** to this PR's head `675f2507`, so merging this would change nothing. Suggest closing it. The other compile breaks your `main` baseline would hit next (`uor-r4-workbench`; `uor-r4-core`'s `kvar_relative_energy` built as a standalone binary; stale `vsa_code_mode` initializers in two core tests) are fixed together in #1544 (`codex/fix-workbench-compile`). One thing remains after the compile is fixed: `test_s2_real_proportions_exported_artifact_dequantized_weights_equal_served_view` fails, because its two exports use different provenance JSON. #1534's version of the test fixes that.

### comment 5913622469 — 2026-09-30T14:45:03Z — Casey-allard

Verified superseded by merged #1538 (main dc82874723c45fca9ffe7b9fa80f80ee9d3fe497). The only changed file, crates/uor-r4-training/tests/d4_map_codec_adapter.rs, has identical Git blob 1a95040c5b009bde6ccf06166ae19355b0e8d436 on this head675f2507 and currentmain; git diff --exit-code succeeds. Closing duplicate delivery only. Branch/history retained, no deletion and no additional test/model run. Other compiler and codec-provenance fixes retain their separate PR scope.


---
_Comment/review ids in this file: 5913467632, 5913622469_

