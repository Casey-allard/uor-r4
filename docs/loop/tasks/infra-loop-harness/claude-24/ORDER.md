# ORDER: infra-loop-harness, claude-24 (the three lows from the claude-23 review)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 7b. Base: `953468bb0`.

## Goal

The fresh-context review of claude-23 (#2937, merged as `44ace17dc`) approved it with three lows:

1. `parse_item` now fails on a `depends_on:` that is not an inline `[a, b]` list. `loop claim` and `loop pr-check` would refuse such an item, and `loop next` would skip it silently and drop it from the held count. claude-23's ORDER said `loop claim` must not change.
2. A dependency whose item file fails to parse is labelled `(missing)`. It is counted as unmet, but the label is wrong.
3. The test does not cover a quoted `"done"`, an empty `[]`, or several dependencies of which only some are done.

Casey's direction, in Claude's chat on 22 Oct: "Yes: build claude-24 for the three lows from the #2937 review (lenient or flagged depends_on parse, an 'unreadable' label for an unparseable dependency, wider test cases) through the delivery path."

The planted fault: **a malformed `depends_on:` makes `loop claim` refuse an item it accepted before claude-23.**

## Cancel probe (run first)

At `953468bb0`, does `parse_item` return an error for `depends_on: b-dep`? Yes (loopclaim.rs:410-412). The task stands. Class: instrument.

## Calls

- **Flagged, not lenient.** A malformed `depends_on:` no longer fails the parse. `ItemFields` records it in a new `depends_on_error`, so `loop claim` and `loop pr-check` behave as they did before claude-23. In `loop next`, such an item waits and is named with the error. It is never offered, because its dependencies cannot be checked. A lenient parse that reads it as "no dependencies" would offer blocked work, which is the fault claude-23 fixed.
- A dependency missing from `origin/main` stays `(missing)`. One whose file is there but does not parse reads `(unreadable: <why>)`.
- No other tool changes.

## Files in scope

- `crates/uor-r4-harness/src/loopclaim.rs` (the item parser and reader only) and `src/loopnext.rs` (the `any` source)
- `crates/uor-r4-harness/tests/loop_next*.rs` and `tests/loop_claim*.rs`
- `docs/harness/README.md` (the `any` bullet)
- `docs/loop/tasks/infra-loop-harness/claude-24/**`

## Done when

- `loop claim` accepts a ready, scoped item whose `depends_on:` is malformed, and `loop next` names that item as waiting and never offers it.
- An unparseable dependency reads `(unreadable: …)`, and an absent one still reads `(missing)`.
- Tests cover a quoted `"done"`, `[]`, and a mix of done and unfinished dependencies.
- Negative gates: with the fix set aside, the claim test and the label test fail.
- The full harness suite passes, as do `pr-check --local`, claim-wording, links and the copyright check.

## Stop and ask if

- Any tool other than `loop next` would change behaviour.
- Any file outside scope must change.
