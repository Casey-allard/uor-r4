# ORDER: infra-loop-harness, claude-33 (loop next stops offering reviews a counted record already settled)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 7b. Base: `8530bce48`.

## Goal

At 16:50Z, `loop next` sent Claude to review #2974, which already carried Kimi's counted APPROVE at its head. Its `reviews` source skips a PR only when **this** lab already has a counted record at the head. Since D59 ended, one counted record at the head settles a review: an APPROVE passes the gate, and a CHANGES waits on the author's repair. A second review is duplicate work, which happened three times on 22 Oct (#2965/#2954, #2972, #2974).

The planted fault: **a PR another lab already approved at its head is offered to this lab again.**

## Cancel probe (run first)

At `8530bce48`, does the `reviews` source skip a PR that has any counted record at its head? If it does, the task is void. Class: instrument.

## Calls

- In `loopnext.rs`'s `reviews` source, skip a PR when `loopreview::counted` at its head is non-empty, whatever lab made the record. A new head, such as a repair, has no counted record, so it is offered again.
- `docs/harness/README.md` (shared) says so.

## Files in scope

- `crates/uor-r4-harness/src/loopnext.rs` and `crates/uor-r4-harness/tests/loop_next*.rs`
- `docs/harness/README.md` (shared)
- This task folder

## Done when

- A test shows a non-setup PR carrying another lab's counted APPROVE, or CHANGES, at its head is not offered as `GO review`. With the fix set aside, the test fails.
- The existing `loop_next` and `loop_next_sources` tests, clippy `-D warnings`, `loop gates --full` with `--break all` CAUGHT, `report --check` and `pr-check --local` all pass.

## Stop and ask if

- Some lab's review must still be required even when another vendor's counted APPROVE is at the head.
