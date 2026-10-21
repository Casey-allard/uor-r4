# REPORT: infra-loop-pr06-review-records, claude-18 (loop PR 6: loop review with tagged review records)

Lab: claude. Base: `987f72259`. Order: [ORDER.md](ORDER.md), committed first (`bcae9a025`). The cancel probe found no `loop review` and no `uor-review v1` parser.

Three fresh-context builders each wrote one commit: the pure core, then the `loop review` command, then the switch-over of `pr-check` and `next`. The main session re-ran every builder's gates before accepting its commit, and wrote this REPORT, because a harness hook does not let subagents write REPORT files.

## What did not work, first

- **Review round 1 at `f04416f66` approved with lows, but two lenses found the same escape.**
  - **Escape from D59.** A loop setup item's ORDER could leave out `Setup row:` and be treated as an ordinary PR, so other vendors' records counted. The ORDER is read at the head, so the author controls this.
  - **The fresh-context requirement.** The gate counted any `[claude]` record on a setup PR, but D59 names "Claude's *fresh-context* reviewers". A building session could approve its own setup PR.
  - **The fixes, in `6f07c3343`:**
    - a claude record on a setup PR under D59 must carry `context: fresh`;
    - a loop item's ORDER without a valid `Setup row:` makes the gate fail, and no record counts;
    - `Setup row:` takes only a valid row token, so prose mentioning it is not a row;
    - a comment with a `uor-review v1` marker anywhere but its first line is malformed;
    - a record's `order` must equal the PR's ORDER;
    - `run_gh` strips `GIT_*`, because `loop review` posts through it.
  - **Tests added:**
    - a lab's own doc-only PR shows as an owed self-review;
    - the switch is read at the merge base, even when the PR's own diff lists and ends it;
    - the generated half of doc-only;
    - the self-review-on-a-code-PR refusal reason.
- **The D59 switch is proposed, not added.** The item says the switch lives in an owner path and that adding it is the owner's step. So this PR adds no `docs/harness/d59.json` and no `owner-paths.json` entry. Until the owner lists the path, the gate treats D59 as in force whatever the file says, so ending D59 is a two-step owner action:
  1. add `{"path": "docs/harness/d59.json", …}` to `owner-paths.json`;
  2. write the file with `"d59": "ended"` and the owner's words and time.
- **Identity is still self-declared (D55).**
  - A record's `[<lab>]` tag and its `lab:` field must match, but every lab posts from one GitHub account. A tag says who claims to have reviewed, and does not prove it.
  - The `context: fresh` line is the reviewing lab's word.
  - A `Setup row:` line is the author's declaration. The gate checks that the row exists and the item matches, which narrows the loophole but does not close it. Claude's reviewer confirming that the PR builds the row is a human-in-the-loop step, not a check.
- **Free-text records stop counting.** After this lands, `pr-check` and `next` count only `uor-review v1` records, and a comment in the old `[<lab>] Exact-head review record for …` form is ignored. Setup PRs already merged need nothing. No open PR is affected today.
- **`loop next` gained a behaviour.** A lab's own doc-only PR now shows as a review owed to that lab, as a fresh-context self-review. That matches AGENTS.md item 6, but `next` did not do it before.
- **Real GitHub writes are covered by a fake.** `loop review` posts through `gh pr comment`, and the tests use the fake. This PR's own exact-head record will be the first real post.
- **`model` and `launch` are placeholders until the supervisor exists.** They come from `--model` and `--launch`, or `UOR_LOOP_MODEL` and `UOR_LOOP_LAUNCH`; otherwise they read `unknown` and `manual`. Loop PR 10's supervisor is to write them from the launch command.
- **This PR fails its own `scope` rule,** as every loop PR does until the PM backfills `scope:` on the items.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopreview.rs` (new) | code | 981 | The `uor-review v1` record (render and parse), build-plan rows, setup-PR status, the D59 switch, the gate, and the `loop review` command behind a `ReviewGh` trait with a fake. |
| `loopprcheck.rs`, `loopnext.rs`, `loopclaim.rs`, `loopgates.rs`, `doccheck.rs`, `lib.rs` | code | +171 / −165 | `pr-check`'s `review-record` rule and `next`'s repairs and reviews now run the gate. The old free-text parser is removed. Also dispatch and doccheck flags. |
| `crates/uor-r4-harness/tests/loop_review.rs` (new); `loop_next.rs`, `loop_prcheck.rs` | tests | +961 / −43 | 25 tests in the new suite; the `next` and `pr-check` record fixtures rewritten as `uor-review v1`. |
| `docs/harness/README.md`, `docs/loop/README.md` | docs | +41 / −6 | The record, the gate and the command; the review record and step 10 marked built. |

That totals about 2,150 lines added and 214 removed, excluding this ORDER and REPORT; the review fix round added about 300 of them. (An earlier version said 1,850 and 205: the 205 was miscounted, 211 at the time.) The ORDER planned two commits; the second was split in two (the command, then the switch-over).

**Design calls.**
- **The record.** The first line is `[<lab>] uor-review v1`. It is followed by `pr`, `head`, `lab`, `model`, `launch`, `verdict`, `order`, an optional `context: fresh` and a `done-when` line (`host: …` or `gpu: sha256:…`), then free text after a blank line. A record that is untagged, has a tag that differs from `lab:`, or has a missing, duplicated or unknown field is malformed, and does not count.
- **The gate.** A record counts when its `pr` and `head` match, its lab is registered, and one of these holds:
  - for a setup PR while D59 is in force, the lab is `claude` and the record carries `context: fresh` (D59 names "Claude's fresh-context reviewers"; freshness is the reviewing lab's word, D55);
  - otherwise, its vendor differs from the ORDER lab's vendor;
  - or the PR is doc-only, the lab is the ORDER's own, and the record carries `context: fresh`.

  The record's `order` must also name the PR's own ORDER. The gate passes when a counted APPROVE exists and the last counted record is an APPROVE.
- **Setup PRs.** A PR is a setup PR when its ORDER's `Setup row:` names a row in the loop page's build-plan table, and its item is `infra-loop-harness` or that row's own item (`infra-loop-pr<NN><x>-`). If an ORDER for a loop item (`infra-loop-…`) has no valid row, the gate fails rather than treating the PR as ordinary.
- **Where the rules are read.** `pr-check` reads the table, the lab registry, the owner paths and the switch at the merge base. `loop review` and `next` read them at `origin/main`.
- **`loop review` refuses to post a record that would not count.** It runs the gate on the new record alone and posts only if the record counts. A CHANGES record can count without passing.

## Measurements

None. This is a host tool.

## Checks run at the head

The main session re-ran each builder's tests at its commit, then ran these at `6d9428587`, with this REPORT in place for the full suite.

- **Targeted suites:** `loop_review` 25, `loop_next` 22 and `loop_prcheck` 28 passed after the fix round (18, 21 and 27 before it); `loop_claim` 33 still passes after the `run_gh` change. `loop_close` 28, `loop_arrive` 17, `loop_report` 7 and `loop_gates_run_rules` 29 still pass.
- **Negative gates**, run by the builders. Each set aside only its own check, the named test FAILED, and the file was then restored byte-identical (`cmp`).
  - Planted fault, untagged: an untagged first line accepted. `planted_fault_an_untagged_record_is_refused`.
  - Planted fault, same vendor: the vendor-differs requirement dropped. `planted_fault_a_same_vendor_record_outside_d59_is_refused`.
  - Planted fault, non-Claude on a setup PR: any lab allowed to count. `planted_fault_a_non_claude_record_on_a_setup_pr_is_refused`.
  - The switch: the owner-paths listing ignored. `the_switch_ends_d59_only_when_listed_and_ended`.
  - `loop review`'s would-not-count refusal removed: the codex-on-a-setup-PR test.
  - `pr-check` passing on any well-formed record: its codex-on-a-setup-PR case.
  - `next` ignoring `may_review`: `on_a_setup_pr_only_claude_owes_the_review`;
  - fix round: the fresh-context requirement under D59 dropped: `a_claude_record_on_a_setup_pr_under_d59_needs_fresh_context`; a loop item without a row treated as ordinary: `a_loop_item_needs_a_valid_setup_row_and_others_are_ordinary`; the D59 switch read at the head in `pr-check`: `the_d59_switch_is_read_at_the_merge_base`.
- **The full suite**, `cargo test -q --locked -p uor-r4-harness --no-fail-fast`: 897 passed and 0 failed on the first run; after the fix round (at `7c8ddab46`, with this REPORT in place), 907 passed and 0 failed.
- **`loop pr-check --local --base 987f72259` on this branch:** `order`, `commit-order`, `report`, `owner-paths`, `rule-sources` and `read-path` pass; `rendered-lock`, `filemap`, `review-record` and `required-checks` are N/A; `scope` fails, as above.
- **Other gates.** `cargo fmt -p uor-r4-harness --check` is clean, and whole-crate clippy `--all-targets` is clean. Claim-wording passed, `xtask links` finds 0 broken links in the changed files, and the local copyright check finds 0 matching 8-word runs.

## Skills used

None of the private kit's skills.

## Still open

- **Owner steps to end D59:** list `docs/harness/d59.json` in `owner-paths.json`, then write the file. Until then, D59 is in force.
- **`loop queue` (PR 7c)** will refuse a PR whose gate fails, with "a later CHANGES record blocks queueing, unless it comes from a lab other than Claude on a setup PR". The gate here already ignores those records.
- **PR 12's hooks** will run `pr-check --local`, which leaves `review-record` N/A.
- **Identity remains by tags.** Binding a record to its author would need a per-lab GitHub identity, which the shared account does not give.

## Plain summary

Before this PR, a review was a free-text comment that the tools recognised by a phrase. Now `uor-harness loop review` writes a fixed-format record (`uor-review v1`): which PR, at which exact commit, by which lab, with which verdict, and how the result was checked. Both the merge check (`loop pr-check`) and the work picker (`loop next`) count only those records, through one gate.

The gate applies the project's review rule:
- an approval from a lab of a different company counts;
- for the loop's own setup PRs, while D59 is in force, only Claude's approval counts;
- a docs-only change may be approved by its own lab in a fresh context;
- a later "changes requested" undoes an approval.

The owner can end D59 later by editing one file that only the owner may change, with no code change needed.
