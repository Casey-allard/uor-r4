# REPORT: infra-loop-harness, deepseek-55 (strengthen the PR_LIST_FIELDS pin test)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2904. Base: `1197255b5`. `References #2904`.

## What did not work, first

- **The existing pin was hollow.** It read the whole `loopnext.rs` source and asserted `isCrossRepository` appeared; the constant's own doc comment contains that word, so deleting the field from the value still left the test green. Claude flagged this as non-blocking in #3081's approval; this unit fixes it.
- **One-line reading was wrong too.** rustfmt wraps the constant (`const PR_LIST_FIELDS: &str =` then the string on the next line), so a single-line `find` saw a line without the value and failed. The test now accumulates lines from the declaration through the terminating `;`.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-harness/tests/loop_next.rs` | the pin reads the full `PR_LIST_FIELDS` definition and its value, and asserts the gh query passes `PR_LIST_FIELDS,`; the gh-style JSON decode assertion stays |
| this folder | ORDER and REPORT |

## Measurements

- Mutation proof: with `isCrossRepository` removed from the constant's value (doc comment left intact, which the old test relied on), the test FAILS: `PR_LIST_FIELDS must request isCrossRepository, got: "const PR_LIST_FIELDS: &str =\n    \"number,headRefName,headRefOid,isDraft,mergeable,files,statusCheckRollup\";"`. Restored: green.
- Full suite: `cargo test -p uor-r4-harness --test loop_next` -> **33 passed, 0 failed**; `cargo fmt --check` clean.

## Checks run at the head

`cargo test -p uor-r4-harness --test loop_next` (33 pass), `cargo fmt --check`, `loop report --check`, `loop pr-check --local`; counted non-author review needed.

## Skills used

- `solution-integrity`: a test that cannot fail is not a test; proved the new pin by mutation before claiming it, and caught that a one-line reader was wrong once rustfmt wrapped the value.
- `scope-guard`: test file only (plus task folder); no behaviour change, no rule change.

## Still open

- None in this unit. Wider loop-tooling follow-ups remain carded on #2028 (render-review rotation, shared-checkout build guard).

## Plain summary

- The `PR_LIST_FIELDS` pin now reads the constant's real value (through the semicolon) and the call-site usage, and was mutation-tested: removing `isCrossRepository` fails it, restoring it passes; 33 tests green.