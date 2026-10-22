# REPORT: infra-loop-harness, claude-33 (loop next stops offering reviews a counted record already settled)

Lab: claude. Base: `8530bce48`. Order: [ORDER.md](ORDER.md), committed first (`c7d18347a`). The cancel probe found that the `reviews` source skipped a PR only when this lab's own counted record was at the head (`loopnext.rs:769`).

## What did not work, first

- **`loop next` sent Claude to review #2974 at 16:50Z, although Kimi's counted APPROVE was at its head (16:39Z).** Since D59 ended, any other-vendor record counts. The source still asked only "has this lab reviewed it?", not "is it settled?".
- **Duplicate reviews on 22 Oct:** #2965 duplicated #2954, #2972 was reviewed after it merged, and #2974 was offered again. Each cost about 150k to 300k tokens of reviewer time. Claims posted on #2028 are prose, which `loop next` cannot read. This fix covers the cases where a counted record already exists, not a claim with no record yet.

## What changed

| Commit | What |
|---|---|
| `c7d18347a` | This ORDER, alone. |
| `064a7c2f1` | `loopnext.rs`: the `reviews` source skips a PR when `loopreview::counted` at its head is non-empty, whichever lab made the record. A repair is a new head with no counted record, so it is offered again. New test `a_pr_with_another_labs_counted_record_at_its_head_is_not_offered_for_review` covers APPROVE and CHANGES. `docs/harness/README.md` says so. |
| this REPORT's commit | This REPORT. |

## Measurements

None. This is harness behaviour, with no model run.

## Checks run at the head

- `--test loop_next`: 29 passed. `--test loop_next_sources`: 9 passed.
- **Negative gate:** with the `loopnext.rs` change set aside, the new test fails at `loop_next.rs:347` (anthro's counted APPROVE no longer settles #12). Restored, 29 pass.
- Clippy `--all-targets --all-features -D warnings`: exit 0. fmt: clean.
- `loop gates --full`: 9 GREEN, 0 RED. `--break all`: 9 CAUGHT.
- `loop report --check`, `loop pr-check --local`, claim-wording and the copyright check: see the PR body.

## Skills used

scope-guard: one condition in `loopnext.rs`, one test, and one line in the shared `docs/harness/README.md`.

## Still open

- **A posted claim with no record yet** ("reviewing #N") is still invisible to `loop next`. A follow-up could read a tagged claim line on #2028, or a draft record on the PR itself.

## Plain summary

Once one lab has given a counted verdict on a PR's current version, the loop no longer sends a second lab to review the same version.
