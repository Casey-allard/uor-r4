# ORDER: infra-loop-pr09-client-adapters, antigravity-5 (repair: refuse malformed CLAUDE.md marker states from 9106525 review)

Lab: antigravity. Item: [infra-loop-pr09-client-adapters](../../../../roadmap/items/infra-loop-pr09-client-adapters.md) (#2914). Setup row: 9. Base: `8ec78be1a` (`origin/main`), repairing `antigravity-4` (PR #2932) after the counted `CHANGES` record at `91065253db03d7746e2ae4105cadaf64270b13a3` (`claude`, `uor-review v1`, fresh context).

## Goal

Fix the single required item (and the LOW follow-up) from the `9106525` `CHANGES` review record:
1. **Refuse malformed `CLAUDE.md` marker states (`merge_claude_md`):** Count `<!-- uor-r4-loop-start -->` and `<!-- uor-r4-loop-end -->` markers in `~/.claude/CLAUDE.md`. Allow only `(0, 0)`, or `(1, 1)` with the start marker before the end marker. Any other marker state (a stray start or end marker, two or more blocks, or an end marker before a start marker) returns an error naming the file and the marker counts, so `loop install` refuses with exit `2` and writes nothing.
2. **LOW follow-up:** In the `(0, 0)` case, if an unmarked `CLAUDE_MD_BODY` appears inside a larger `CLAUDE.md`, replace it in place with the marked block instead of requiring it to be the entire file.
3. **Tests:** Add `repair_order_antigravity_5_regressions` in `crates/uor-r4-harness/tests/loop_guard.rs` testing all three malformed states (stray start marker, two blocks, and end marker before start marker) plus in-place migration of an embedded unmarked pointer, verifying exit `2` and zero writes on refusal.

## Cancel probe (run first)

Is `9106525` still `CHANGES` on PR `#2932`, and does `merge_claude_md` on `9106525` append a second block when given a `CLAUDE.md` with a stray `<!-- uor-r4-loop-start -->` marker? Result: yes (`loop review --check --pr 2932` exits `1`, and `loopguard.rs:3639-3658` reproduces). Class: instrument.

## Files in scope

- `crates/uor-r4-harness/src/loopguard.rs`, `crates/uor-r4-harness/tests/loop_guard.rs`, `docs/harness/onboarding/**`
- `docs/loop/README.md`, `docs/harness/README.md` (shared docs)
- `docs/loop/tasks/infra-loop-harness/antigravity-{1,2,3,4,5}/**`

## Done when

- `merge_claude_md` refuses any marker count other than `(0, 0)` or `(1, 1)` with start before end, `repair_order_antigravity_5_regressions` tests all three malformed states, and `loop gates --full`, `loop report --check` and `loop pr-check --local` pass.

## Stop and ask if

- Any file outside the scope above must change.
