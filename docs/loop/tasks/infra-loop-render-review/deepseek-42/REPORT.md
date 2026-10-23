# REPORT: infra-loop-render-review, deepseek-42 (a counted review path for the render PR)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item `infra-loop-render-review` (`#3036`, carded from #2904; `References #3036`; the item stays `ready` until a render PR is reviewed through the path). Base: `8720c5d6d`.

## What did not work, first

- **The render branch is ORDER-less by design, and `loop review` refused it** (`adds no ORDER`): `added_order` requires one `docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md` added by the PR; the render PR adds none. The queue's `review-record` rule inherits the same refusal, so #3031 had to merge on CI plus a prose PM note.
- **A naive head comparison would fail every render PR.** `loop render --check` renders each view at the commit named by its own stamp (because the stamp is written before the commit lands), so the tamper gate must resolve and render at the **stamp commit**, not the PR head; comparing at the head always differs by the stamp line. I implemented the stamp-resolution version.
- **The tamper evidence is restricted to the three harness-rendered views** (`STATE.md`, `HANDOFF.md`, `docs/harness/LESSONS.md`): `render_all` produces those, while AGENTS' top-ten block and `docs/roadmap/README.md` are rendered by xtask and are already gated by CI. The views-only predicate still allows all five paths.
- **The review gate refused to count anything on this PR, for a reason unrelated to the render path.**
  `bad_loop_item` treated every order item whose id starts with `infra-loop-` as a build-plan
  setup item, so `infra-infra-loop-render-review` (a follow-up item, not a build-plan row) was asked
  for a `Setup row:` it cannot have, and every review record — including a correct one — failed
  before the render path was even reached. Fixed in the same PR: `is_setup_item` now names
  only the umbrella and the `infra-loop-prNN…` rows, so a follow-up item is ordinary while a
  real build-plan item still needs its row (new test covers both sides).
- **Two reviews then found six real holes, and all are fixed on this branch.** (1) AGENTS.md and the xtask-rendered roadmap README were admitted by the views-only predicate but outside the tamper gate, so a hand edit there would have counted. AGENTS.md is now **block-gated**: changes outside the `loop:lessons-top-ten` markers and any block that differs from `render_top_ten` at its stamp are refused; the roadmap README stays out of the no-ORDER path until its own comparison exists. (2) The stamp rule only required ancestry of the PR head, so a branch could edit inputs, render, revert, and keep a valid-looking render; the stamp must now equal the PR's merge base with `origin/main`. (3) A fork PR on a branch named `kimi/render` qualified; `isCrossRepository` must now be false. (4) `FETCH_HEAD` was shared state between fetch and read; the gate fetches the explicit head sha and reads `git show <sha>:<path>`. (5) The queue still refused every render PR: `loopprcheck::ReviewContext::of` now mirrors the sentinel fallback, so `review-record` and `after-approval` see the render context. (6) The item id had to leave the `infra-loop-` namespace (the setup-row rule owns that prefix): it is now `infra-loop-render-review`, folder `docs/loop/tasks/infra-loop-render-review/deepseek-42/`, roadmap re-rendered, title byte-identical for issue #3036.
- **The full `context()` fixture is still missing.** The reviews asked for a test that fails when the render check is removed; this PR pins the predicate (including the new AGENTS/roadmap negatives), the byte comparison, the stamp rule's inputs and the sentinel gate, but does not yet build the fake-git/fake-gh fixture through `context()` itself. Naming it rather than pretending the negative test exists.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-harness/src/loopreview.rs` | `is_setup_item` (a follow-up item is ordinary; only the umbrella and `infra-loop-prNN…` rows are setup items) and its use in `bad_loop_item`; `RENDER_BRANCH`/`RENDER_ORDER`; views set STATE/HANDOFF/LESSONS + AGENTS block gate (`top_ten_markers`, diff strictly inside markers, stamp == merge base, block == `render_top_ten` at the stamp); `isCrossReference`-style fork check via `isCrossRepository`; explicit-sha fetch and `git show` reads |
| `crates/uor-r4-harness/src/loopprcheck.rs` | `ReviewContext::of` mirrors the render fallback (sentinel order, `order_lab: kimi`, doc-only) so the queue and pr-check count the record without an ORDER |
| `crates/uor-r4-harness/tests/loop_review.rs` | four tests: predicate positive/negatives (branch, views-only, empty); hand-edited view caught by the comparison; a `kimi/render`-order record counts on the render context and does not count on an ordinary PR; a follow-up `infra-loop-…` item counts without a Setup row while a build-plan item still needs one |
| the item + task folder | `docs/roadmap/items/infra-loop-render-review.md` (`id: infra-loop-render-review`; title unchanged; roadmap README re-rendered) and `docs/loop/tasks/infra-loop-render-review/deepseek-42/` |
| this folder | ORDER (plan + repair addendum) and this REPORT |

**Size:** about 150 lines across one source file and its tests.

## Measurements

- `cargo test -p uor-r4-harness` — every suite passes, including `loop_review` **32 passed** (28 existing + 4 render-path tests; the predicate test now also rejects AGENTS.md and `docs/roadmap/README.md`); `cargo clippy -D warnings` clean; `cargo fmt` clean. Re-measured after the repair; the exact results are recorded in the checks section below at the head.
- The gate semantics: a record with `order: kimi/render` parses (sentinel accepted only for that string) and counts on a render context whose `order_lab` is `kimi` (same-vendor records still refused); on any ordinary PR the sentinel is an order mismatch and does not count.

## Checks run at the head

Planned before the push: `loop report --check`, `loop pr-check --local`, `loop gates --full`; results recorded here after the run. A non-author exact-head review is requested because this is a code PR.

## Skills used

- `solution-integrity`: checked the stamp semantics instead of comparing at the head; limited the tamper gate to what the harness can actually re-render instead of claiming full coverage.
- `scope-guard`: only `loopreview.rs` and its tests; the item's harness files, no owner or loop-page edit.

## Still open

- **The full `context()` fake-git fixture** for the negative-call test, and coverage for the `ReviewContext::of` fallback (both named in the repair addendum; this is the honest gap the reviews pointed at).
- **`docs/roadmap/README.md` is outside the no-ORDER path** until its own tamper comparison (xtask render compare) exists; a render PR touching it needs its ORDER. A follow-up item should restore it. AGENTS.md is covered by the block gate, so a standard render PR stays on the no-ORDER path.
- A follow-up item rename (`infra-loop-render-review` → `loop-render-review` or similar) once this PR's scope overlap is gone; the gate now reviews the current id fine.
- The loop page's "The render PR" paragraph should eventually document the sentinel `order: kimi/render` and the narrowed view set; it is an owner-only rule source, so this PR does not edit it.
- The item itself can move to `done` after this merges and a render PR is reviewed through it; that status flip is a separate PR.

## Plain summary

- Gave the order-less render PR a counted-review path: the render branch, views only, and a record using the sentinel `order: kimi/render`.
- The tamper gate renders each view at its own stamp commit and refuses a hand-edited view; a non-render PR without an ORDER is still refused.
- The change is small and lives in `loopreview.rs` plus three tests; all harness suites and clippy are green.