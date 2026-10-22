# ORDER: infra-loop-harness, claude-35 (a counted CHANGES blocks until a new head or the same lab approves)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 6. Base: `fb6df367b`.

## Goal

The review gate passes when the **last** counted record at the head is an APPROVE. A CHANGES from one lab is therefore cleared by any other lab's later APPROVE at the same head, so a PR can merge with open findings.

On 22 Oct 2026 at 17:51Z, Casey was asked: "When one lab posts CHANGES on a PR's current head and another lab later posts APPROVE on that same head, should the APPROVE be allowed to override the CHANGES?" Casey answered "No: CHANGES blocks (Recommended)": "Any counted CHANGES at a head blocks the merge until the author pushes a new head (the repair), or the same lab that posted CHANGES re-reviews and approves."

The planted fault: **another lab's later APPROVE clears a counted CHANGES at the same head.**

## Cancel probe (run first)

At `fb6df367b`, do the records CHANGES (lab A) and then APPROVE (lab B) at one head pass `loopreview::gate`? If they fail, the task is void. Class: instrument.

## Calls

- `loopreview` gets `standing_changes`, the labs whose last counted record at the head is CHANGES. `gate` passes when a counted APPROVE exists and no CHANGES stands. A new head already drops every record, because a record counts only at the current head.
- `loop next`'s `repairs` source sends the author to repair when any CHANGES stands, not only when the last record is one. Otherwise the gate blocks the PR while `loop next` sees nothing to do.
- `loop queue` reads the same gate through `pr-check`, so it needs no change of its own.
- A `loop-review-changes-blocks` gate in `docs/harness/gates.json`, whose break restores the last-record rule.
- `docs/harness/README.md` states the rule.

## Files in scope

- `crates/uor-r4-harness/src/loopreview.rs`, `loopnext.rs`, `loopprcheck.rs` (doc comment only)
- `crates/uor-r4-harness/tests/loop_review.rs`, `loop_prcheck.rs`, `loop_next*.rs`
- `docs/harness/gates.json` and `docs/harness/README.md` (shared)
- This task folder

## Done when

1. **The gate holds.** A test shows that A's CHANGES followed by B's APPROVE at one head fails the gate. A's CHANGES followed by A's own APPROVE passes it. With the fix set aside, the test fails.
2. **`loop next` agrees.** A `loop next` test shows that the author of a PR whose only stale CHANGES is followed by another lab's APPROVE gets `GO repair`.
3. **The full checks pass:** the existing `loop_review`, `loop_prcheck`, `loop_queue` and `loop_next*` tests, clippy `-D warnings`, `loop gates --full` with `--break all` CAUGHT, `report --check` and `pr-check --local`.

## Stop and ask if

- A lab that posted CHANGES can no longer be reached, so its CHANGES could block a PR for good. The owner's answer names a new head as the way out. A different exit is a new owner call.
