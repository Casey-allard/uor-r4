# ORDER: infra-loop-harness, deepseek-55 (strengthen the PR_LIST_FIELDS pin test)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (tooling). Base: `1197255b5`. `References #2904`.

## Goal

Close the follow-up card from #3081's review (deepseek on #2028): `the_production_listing_requests_the_render_sentinel_field` asserted `source.contains("isCrossRepository")`, but the `PR_LIST_FIELDS` doc comment contains that word too, so dropping the field from the constant's value would leave the test green. Pin the constant's actual definition (the declaration through its terminating semicolon, since rustfmt wraps the value) and keep the gh-style JSON decode assertion.

## Cancel probe (run first)

Class instrument/tooling (host-only; no GPU). At the base: remove `isCrossRepository` from the `PR_LIST_FIELDS` value while leaving the doc comment intact -> `cargo test -p uor-r4-harness --test loop_next the_production_listing_requests_the_render_sentinel_field` still passes (the weak pin). Task stands.

## Files in scope

- `crates/uor-r4-harness/tests/loop_next.rs`, this task folder.

## Done when

- The test reads the `const PR_LIST_FIELDS` declaration through its terminating semicolon and asserts the value names `isCrossRepository`, and that the gh query passes `PR_LIST_FIELDS,` rather than an inline string.
- Mutation evidence: dropping the field from the constant's value fails the test with a message quoting the definition; restoring it passes; the full `--test loop_next` suite passes (33) and fmt is clean.
- `loop report --check`, `loop pr-check --local` pass; counted non-author review.

## Stop and ask if

- A change beyond the test file and this folder is needed.