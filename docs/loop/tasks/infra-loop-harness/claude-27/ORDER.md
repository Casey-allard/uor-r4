# ORDER: infra-loop-harness, claude-27 (the follow-ups from the claude-25 and claude-26 reviews)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 7c. Base: `d00431587`.

## Goal

Four follow-ups, each from a review record or an owner decision on 22 Oct:

1. **Copyright.** `docs/loop/tasks/infra-loop-harness/kimi-1/REPORT.md:37` holds one eight-word run from Ron's Loop Kit. Casey's direction: "Claude, in claude-26 (Recommended)". claude-26 put the path in scope, and this unit rephrases that one sentence and then takes the path back out of the umbrella scope (#2941 review, low 1).
2. **The inbox item's done-when** (`infra-bench-inbox-rule`) still says "replace `waiting`". Casey's decision on messages to `all` is "Per-lab lines (Recommended)", and the file lives in the main checkout's `local/bench/`, not a worktree's (#2943 review). The text follows both.
3. **The bench feed's review cell** (#2942 review, low 1): a lab PR with no ORDER still shows a raw APPROVE. It reads `needs review`, or `review not counted (<lab>)` when a record exists at the head.
4. **The feed's sort** (#2942 review, low 2): a row with an unreadable ORDER sorts with `needs review`, not by its raw review.

The planted fault: **a copied sentence stays on `main`, and the bench shows an approval the merge gate cannot count.**

## Cancel probe (run first)

At `d00431587`, does `kimi-1/REPORT.md` still match the kit, and does `loopbench.rs`'s no-ORDER arm still call the raw `review_state`? Yes to both. The task stands. Class: instrument.

## Files in scope

- `docs/loop/tasks/infra-loop-harness/kimi-1/REPORT.md` (that sentence only), `docs/roadmap/items/infra-loop-harness.md` (`scope:` only) and `docs/roadmap/items/infra-bench-inbox-rule.md` (the done-when only)
- `crates/uor-r4-harness/src/loopbench.rs` and `crates/uor-r4-harness/tests/loop_bench*.rs`
- `docs/roadmap/README.md` (shared, if a render changes it)
- `docs/loop/tasks/infra-loop-harness/claude-27/**`

## Done when

- `kimi-1/REPORT.md` has 0 eight-word runs matching the kit, and only that sentence changed.
- The umbrella scope no longer lists `kimi-1/REPORT.md`.
- The item's done-when states per-lab lines and the main-checkout path.
- Tests: a no-ORDER PR with a same-vendor APPROVE at its head is not shown as APPROVE, and an unreadable row sorts with `needs review`. Each fails with its fix set aside.
- The full harness suite, `pr-check --local`, claim-wording, links and both copyright checks (LoopKit and CorvusWing) pass.

## Stop and ask if

- Any other sentence of Kimi's REPORT would have to change.
- A file outside scope must change.
