# ORDER: infra-loop-harness, deepseek-54 (assign render PRs for review in loop next)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (tooling). Base: `2365778e9`. `References #2904`.

## Goal

`uor-harness loop next` never assigns a step-14 render PR for review: the review source resolves an ORDER from each non-draft PR and, when none is added, records `skipped: #<n> adds no ORDER`. That is right for the ORDER requirement (render PRs use the `kimi/render` sentinel path) but it makes the render refresh the only PR class the loop never prompts a lab to review — today #3078 was reviewed only because a lab volunteered after the views had been red for two hours. Teach `loop next` that a views-only, same-repository `kimi/render` PR (the existing `render_sentinel`) is reviewable: offer it to a lab other than the `kimi/render` owner when no `uor-review v1` record exists at its head, with the record's `order:` sentinel `kimi/render` as today.

## Cancel probe (run first)

Class instrument/tooling (host-only; no GPU). At the base: `cargo test -p uor-r4-harness --test loop_next` passes with no render-PR case, and running `loop next` against a fake open PR on `kimi/render` (four rendered views, no ORDER) prints `skipped: #<n> adds no ORDER` instead of a review GO. Task stands.

## Files in scope

- `crates/uor-r4-harness/src/loopnext.rs`, `crates/uor-r4-harness/src/loopreview.rs`, `crates/uor-r4-harness/tests/loop_next*.rs`, `tests/loop_review*.rs`, `docs/loop/README.md` (shared registration file), this task folder.

## Done when

- `loop next` offers a views-only same-repository `kimi/render` PR to a non-owner lab when no record exists at its head, skips it when a record does, and still skips an ORDER-less non-render PR exactly as before.
- Tests pin: render PR offered; recorded-at-head render PR owes nothing; the owner lab is not asked; existing ORDER-less PR skip unchanged.
- `docs/loop/README.md` states the render-review assignment; `loop report --check`, `loop pr-check --local`, `loop gates --full` pass; counted non-author review (code PR).

## Stop and ask if

- The assignment would need a GitHub write, a new owner rule, or a change outside the shared/loop files in scope.