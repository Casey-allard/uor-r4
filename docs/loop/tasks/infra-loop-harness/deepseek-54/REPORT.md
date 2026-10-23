# REPORT: infra-loop-harness, deepseek-54 (assign render PRs for review in loop next)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2904. Base: `2365778e9`. `References #2904`.

## What did not work, first

- **The loop never prompted a render-PR review.** The review source resolves an ORDER from each non-draft PR and records `skipped: #<n> adds no ORDER` when there is none. Render refreshes legitimately have no ORDER (the `kimi/render` sentinel path), so the one PR class that blocks every lab's `loop arrive` was never offered for review; today #3078 sat approved-but-unqueued while the views were red for two hours, then needed the PM prose merge after the nightly-attestation collision.
- **Record detection for the sentinel path cannot use the ORDER gate.** `loopreview::counted` needs a `GateInput` with an ORDER path; a render record's `order:` is the `kimi/render` sentinel. Added `loopreview::records_at_head(pr, head, bodies)` — records at the head regardless of ORDER — used only for assignment; the render gate still checks the views when a record is posted.
- **The owner lab must not be prompted for its own render refresh.** The `kimi/render` branch is the PM's step-14 lane; the assignment skips `lab == "kimi"` (other labs still see it and can review), preserving the existing behaviour where a counted record settles it for everyone.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-harness/src/loopnext.rs` | `NextPr.is_cross_repository` (from `gh pr list ... isCrossRepository`), `is_render_pr` helper, and a review branch for `OrderErr::Missing` that offers a views-only same-repo `kimi/render` PR to a non-owner lab when no record exists at its head |
| `crates/uor-r4-harness/src/loopreview.rs` | `records_at_head` helper (records at a PR head regardless of ORDER) |
| `crates/uor-r4-harness/tests/loop_next.rs` | four tests: render PR offered; recorded-at-head owes nothing; owner lab not asked; the production `gh pr list --json` field list requests `isCrossRepository` (the fixture alone cannot hide a missing production field) |
| `docs/loop/README.md` | the render-PR paragraph now states that `loop next` assigns the refresh for review |
| this folder | ORDER and REPORT |

## Measurements

- `cargo test -p uor-r4-harness --test loop_next`: **33 passed, 0 failed** (29 before this unit; 4 new render cases). The production field list is pinned by a test after review showed the fixture could pass while the real `gh` query omitted `isCrossRepository` (fixed: the field is now in `PR_LIST_FIELDS`, used by `RealNextGh`).
- Fake-gh test data: PR on `kimi/render`, four rendered views, `isCrossRepository: false`, no ORDER; `next` prints `GO review #30` with `views-only render review`; with a `kimi/render` record at `HEAD` it does not; as lab `kimi` it does not.

## Checks run at the head

`cargo test -p uor-r4-harness --test loop_next`, `loop report --check`, `loop pr-check --local`, clippy/fmt; counted non-author review needed (code PR).

## Skills used

- `solution-integrity`: separated assignment (can read records without ORDER) from the render tamper gate (still checks the views), and kept the owner lab out of its own lane instead of reusing the normal `may_review` rules blindly.
- `scope-guard`: loop tooling and loop docs only; no rule change; no GitHub write.

## Still open

- If the PM prefers the render review to rotate deterministically across non-owner labs, that is a follow-up; today every non-owner lab may see the GO and the first counted record settles it.

## Plain summary

- `loop next` now offers a views-only render refresh PR to a non-owner lab for exact-head review instead of skipping it as ORDER-less; tests cover offered/recorded/owner cases and the loop page documents it.