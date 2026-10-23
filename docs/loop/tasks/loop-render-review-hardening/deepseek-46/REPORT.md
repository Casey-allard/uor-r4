# REPORT: loop-render-review-hardening, deepseek-46

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item `loop-render-review-hardening`. Base: `0af044c7f`. Status: ready for review (PR #3048).

## What did not work, first

- **The item was unclaimable when carded** because its scope listed `loopprcheck.rs`, a shared registration file; fixed in #3045 before this claim. This unit then had to edit that shared file as an always-allowed path, with its test in `tests/loop_review.rs`.
- **The documented render workflow was refused by the gate.** `loop render` stamped HEAD, which is the merge commit after a merge-forward, while the gate requires the stamp to equal the merge base - exactly the failure Kimi hit on #3038. Implemented the default-stamp fix instead of only documenting `--at`.
- **The queue mirror could admit a fork named `kimi/render`** because `headRefName` carries no owner and `ReviewContext::of` took the sentinel on the branch name alone. Fixed by plumbing `isCrossRepository` through `Pr`, `PrGh`, `QueuePr` and requiring a known same-repository head.

## What changed so far

| File | Change |
|---|---|
| `crates/uor-r4-harness/src/looprender.rs` | `default_at(root, branch)` returns `merge-base(origin/main, HEAD)` on `kimi/render`, HEAD elsewhere; `cli` uses it and skips the working-tree roadmap render when the base is used |
| `crates/uor-r4-harness/src/loopreview.rs` | `render_sentinel(head_ref, files, cross)` requires a known same-repository head; the review path uses it |
| `crates/uor-r4-harness/src/loopprcheck.rs` | `Pr.is_cross_repository`, `PrGh::is_cross_repository` (real + fake), `ReviewContext::of` sentinel via `render_sentinel(pr.is_cross_repository)`, pr-check `--pr` fills the flag |
| `crates/uor-r4-harness/tests/loop_review.rs` | offline temp-git test for the merge-base default; sentinel tests; the end-to-end render-gate fixture (six cases through the real CLI); the queue-mirror cross-repository pin via the built binary |
| `docs/roadmap/items/loop-render-review-hardening.md` | the deepseek-42 REPORT nit is **recorded** for the report owner (that file is outside any claimable item scope, so a lab PR cannot edit it) |

The item's five follow-ups are all closed: the fixture is in, the queue-mirror fork check and the render-at-merge-base default are implemented, the roadmap README is **exempt with a recorded reason** (xtask renders the working tree; the views-only set keeps it out of the no-ORDER path; the docs job gates it advisably), and the deepseek-42 REPORT nit is **recorded for the report owner** rather than edited here, because that file sits outside every claimable item's scope. The loop-page wording is owner-only and carded on #2028.

## The fixture and its negative gates

`tests/loop_review.rs::render_gate_fixture_...` builds a temporary git repository (base commit, `kimi/render` head), a fake gh file and a test-only render stand-in and drives the same `render_gate` the CLI uses. It asserts: clean passes; a hand-edited STATE is refused by name; a fork is refused; an AGENTS edit outside the top-ten block is refused; a hand-edited block is refused; a branch-only stamp is refused. Removing the render check would let the tampered cases pass, and disabling the stamp rule would let the branch-only case pass, so both negative gates fail the suite.

## Measurements

- `cargo test -p uor-r4-harness --test loop_review`: **36 passed**. Full `cargo test -p uor-r4-harness`, clippy `-D warnings`, fmt, `roadmap check`, `report`, `pr-check` and `gates --full` results are recorded after they run. `looprender::default_at` now fetches `origin/main`, prints the chosen merge-base stamp on `kimi/render`, and returns a hard error instead of falling back to HEAD when the merge base cannot be read.
- No GPU; every test is offline and uses a temporary git repository or pure functions.

## Checks run at the head

`cargo test -p uor-r4-harness --test loop_review` **36 passed** (including the CLI-driven fixture and the queue-mirror pin); full `cargo test -p uor-r4-harness` green; clippy `-D warnings` and fmt clean; `roadmap check` PASS and README current; `loop report --check` PASS; `loop pr-check --local` PASS; `loop gates --full` 11/11 GREEN; CI green (the only recurring red is the known #2028-carded `loop_drive` lease-note flake).

## Skills used

- `solution-integrity`: fixed the workflow the gate refused instead of only documenting a workaround; refused the sentinel on unknown repository provenance rather than assuming same-repo.
- `scope-guard`: shared files edited additively; tests in the item's own test file; no owner path.

## Still open

- The loop-page wording was landed by the PM in #3050; nothing else is open for this item.

## Plain summary

- All five follow-ups are delivered: merge-base render default, queue-mirror fork check, the end-to-end fixture with every mutation pinned, the README exemption reason, and the old REPORT nit recorded for its owner. The item is `done`.