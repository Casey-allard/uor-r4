# REPORT: infra-loop-harness, claude-24 (the three lows from the claude-23 review)

Lab: claude. Base: `953468bb0`. Order: [ORDER.md](ORDER.md), committed first (`ca259f995`). The cancel probe confirmed that at the base `parse_item` returned an error for `depends_on: b-dep` (loopclaim.rs:410-412).

## What did not work, first

- **claude-23 changed `loop claim` after all, though only for an input no item has today.** Its strict `depends_on` parse made `parse_item` fail on a line that is not an inline list. `loop claim` and `loop pr-check` would then refuse the item, and `loop next` would skip it silently and leave it out of the held count. Its ORDER said `loop claim` must not change. No item was affected, because `roadmap check` blocks that form and all 87 items use the inline one, but the guarantee was weaker than claimed.
- **A lenient parse was rejected.** Reading a malformed line as "no dependencies" would let `loop next` offer an item whose dependencies it cannot check. That is the fault claude-23 fixed.
- **The first negative-gate run did not cover the label.** The malformed-list assertion fails first and stops the test, so the `(unreadable: …)` assertion was never reached. A second gate set aside only the label line.

## What changed

| Commit | What |
|---|---|
| `2961efa43` | `ItemFields` gains `depends_on_error`. A `depends_on:` that is not an inline list is recorded there rather than failing `parse_item`, so `loop claim` and `loop pr-check` behave as they did before claude-23. In `loop next`'s `any` source, such an item waits and is named with the error (`x waits: depends_on … is not a [a, b] list`). A new `read_item_opt` tells an absent dependency (`(missing)`) from one that does not parse (`(unreadable: <why>)`). There are three new tests: `a_malformed_depends_on_does_not_stop_a_claim` (loop_claim), `a_malformed_or_unreadable_dependency_waits_and_says_why` and `only_unfinished_dependencies_are_named_and_quoted_done_or_empty_lists_pass` (loop_next). |
| this REPORT's commit | The harness README's `any` bullet names the three reasons an item waits, and says that `loop claim` and `loop pr-check` do not check dependencies. |

## Measurements

None. This is tooling, with no model run.

## Checks run at the head

- fmt is clean, and whole-crate clippy with `-D warnings` is clean.
- `loop_next` 27 and `loop_claim` 34 passed.
- Negative gates. With the `src/` change set aside (82 patch lines against the base), `a_malformed_depends_on_does_not_stop_a_claim` fails (loop_claim.rs:219) and `a_malformed_or_unreadable_dependency_waits_and_says_why` fails (loop_next.rs:433). With only the label line reverted to `(missing)`, the second test fails at its label assertion (loop_next.rs:437). After each restore the tree is clean and the tests pass. The third test is coverage for the review's L3, so it pins no fix.
- Live, on the real repository: this branch's binary and the shared `main` binary both GO `infra-loop-pr03-render`. All 87 items carry an inline `depends_on:`, so nothing changes for today's items.
- Full harness suite at `2961efa43`, plus the README: 1,004 passed, 0 failed. That is claude-23's 1,001 and the three new tests.
- Claim-wording finds nothing in the changed files. `xtask links` reports 930 broken links repo-wide, the same as `main`'s CI, and none in the changed files. The copyright check finds 0 of 1,119 8-word runs.

## Skills used

solution-integrity: flagged, not lenient. A parse failure that only `loop next` acts on keeps one parser and does not hide what cannot be checked.

## Still open

- `loop claim` still does not refuse an item with unfinished dependencies. That is a separate call for Casey or Kimi.
- An item whose `scope:` does not parse is still skipped silently by `loop next` (loopnext.rs, the `let Ok(item) = … else { continue }`). `roadmap check` blocks that form.

## Plain summary

A badly written dependency line no longer stops a lab from claiming an item. The "what next" command still holds that item back and says why. When a dependency's own file is broken, it now says "unreadable" rather than "missing".
