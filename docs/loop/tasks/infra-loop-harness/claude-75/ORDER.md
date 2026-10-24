# ORDER: infra-loop-harness, claude-75 (the review gate counts the lab's own fresh review, D66)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 6. Base: `6ecb40c45`.

Phase 1, first unit, named for Claude in the #2028 note of 24 Oct (comment 6389525820) under the owner's direction recorded in #3160 (D66): "Reviewers waiting 30 minutes for other labs to see it is the entire problem. Posting and waiting for an answer is where we lose."

## Goal

D66 makes the review part of the unit. The ORDER lab's own fresh-context adversarial review counts on every PR, and the PR queues in the same session. The review gate (`loopreview::refusal`) still counts the ORDER lab's record only on a doc-only PR. `loop review --verdict` refuses to post such a record on a code PR, and `loop queue` and pr-check's review-record rule refuse it. Every lab's code PR therefore still waits on another vendor.

The change: the ORDER lab's record counts on any PR when it carries `context: fresh`, and is refused without it. Other-vendor records count as before. A record from a different lab of the same vendor is still refused. D59, while in force, is unchanged.

## Cancel probe (run first)

At `6ecb40c45`, does the gate already count a fresh same-lab record on a code PR? No: `loop_review.rs`'s `a_doc_only_pr_takes_a_fresh_same_lab_record` asserts that the fresh `claude` record on `CODE_PR` fails with "same vendor". The unit stands. Class: instrument.

## Files in scope

- `crates/uor-r4-harness/src/loopreview.rs` (`refusal` and the doc comments that state the rule)
- `crates/uor-r4-harness/tests/loop_review.rs`, `crates/uor-r4-harness/tests/loop_prcheck.rs`, `crates/uor-r4-harness/tests/loop_next.rs` (expectations that encode D46)
- `docs/harness/README.md`, `docs/loop/README.md` (the sentences that state the gate's rule)
- `docs/loop/tasks/infra-loop-harness/claude-75/**`

## Done when

- A fresh same-lab record counts on a code PR. A same-lab record without `context: fresh` is refused with "needs `context: fresh` (D66)". Another vendor's record still counts. A same-vendor record from another lab is still refused.
- Negative gate: with the `refusal` change set aside, the new code-PR assertion fails.
- `cargo test -p uor-r4-harness` passes in full, clippy with `-D warnings` and fmt are clean, and `loop pr-check --local`, `loop report --check` and the quick gates pass.
- The exact-head review is Claude's own fresh-context review (D66).

## Stop and ask if

A test outside the listed files encodes D46, or the change would alter D59's path or the CHANGES-blocks rule.
