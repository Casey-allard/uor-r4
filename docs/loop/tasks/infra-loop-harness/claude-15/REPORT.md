# REPORT: infra-loop-pr07a-claim, claude-15 (loop PR 7a: loop claim, suborder and ask)

Lab: claude. Base: `32092fca6`. Order: [ORDER.md](ORDER.md), committed (`cdeca0e45`) before the work. The cancel probe found no `claim`, `suborder` or `ask` under `loop`.

Fresh-context builders wrote the code: three for the first head, then one per fix round. The main session re-ran the gates and wrote this REPORT, because a harness hook does not let subagents write REPORT files.

## What did not work, first

- **The first head was not safe enough. Two fresh-context reviews of `f94fea8af` asked for changes.** The fixes came in four commits:
  - **`3fd8372af`: escape and stale inputs.**
    - Suborder containment was a plain string prefix, so a child could escape its parent: `crates/a/**` contained `crates/a/../b/x.rs`, and `docs/*.md` contained `docs/secret.rs`. Containment is now only an exact match, or a parent `<dir>/**` with the child under `<dir>/`.
    - One validator, used by both claim and suborder, now refuses any scope glob with a leading `/`, a `..` or `.` segment, `./`, `//`, `[`, `]`, `{`, `}` or `\`.
    - The checks read the item from a possibly stale working tree. They now fetch first and read items from `origin/main`.
    - An open order whose item had no `scope:` line refused every claim by every lab. It is now warned about and skipped for overlap. It still counts as "claimed".
  - **`145e726f2`: the remote side.**
    - Nothing checked for an existing remote branch, so a stale one could be fast-forwarded and later deleted. A claim now refuses when `origin/<lab>/<topic>` exists, and pushes with `--force-with-lease=refs/heads/<b>:`, which succeeds only when the branch is absent.
    - A failure after the push used to leave the remote branch and PR behind without saying so. A full rollback now closes our PR, deletes our pushed branch with a lease to our own commit, and removes our worktree and branch. Anything left behind is named, and the claim exits 2.
    - Push and delete are now plain `git` calls, no longer part of the `Gh` trait.
  - **`916c4d7e6`: reading other orders.**
    - ORDERs were read at the head branch name, which fails for fork PRs and for PRs whose branch was deleted, and that failure blocked every claim. Reads now go through `refs/pull/<n>/head`, URL-encoded. An unreadable PR ORDER warns and counts as an order of unknown item.
    - Any lower-numbered open PR naming the same item is now a race rival, not only drafts.
    - Worktrees are created under, and scanned from, the main checkout's `.worktrees`.
  - **`e7fd08521`: tests and docs.**
    - New tests for `--lab-topic`, bad ids, overlap in the reverse direction, and the ask refusals.
    - Cards are written with `create_new`, so they are never overwritten.
    - Suborder reads the parent item from `origin/main`.
    - The README now gives the containment rule and the folder conventions.
- **A wrong statement corrected.** Commit 3 of the first head added the loop flags to `main.rs`'s `SUBCOMMAND_FLAGS` and `SUBCOMMAND_VALUE_FLAGS`, saying `loop claim --lab-topic` had been refused without them. The fix-round negative gate disproved that.
  - With `--lab-topic` removed from the table, `loop claim --lab-topic t9` still worked. `loop` arguments pass straight through to the loop parser.
  - The table entries are therefore not load-bearing. They are kept, because they are harmless: an audit of every other command found no bare-switch use of those flag names. No test pins them.
- **The real GitHub paths are untested.** All tests use the fake: `gh pr list`, `gh api …/contents`, `gh pr create --draft` and `gh pr close`. The real `git push` and leased delete are tested against a bare local remote.
- **A squash-merged branch is never an ancestor of `main`.** So an order also counts as merged when its ORDER file has identical bytes on `origin/main`.
- **Round 2 found a test that could not fail.** Two fresh-context reviews of `8f1298986` approved with lows.
  - The bad-id test asserted only exit 2 and that nothing was created. With the id check removed, both of its cases still exit 2: one because the item does not exist on `origin/main`, the other because git refuses the branch name. The test now also asserts the id refusal's own message.
  - A negative-gate line above credited containment with the `..` escape. That line is corrected.
- **Loop PR 3 merged first, so this branch merges `main`.** The conflicts were both-sides additions to the loop dispatch and doccheck's lists. A separate fresh-context check approved the resolution.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopclaim.rs` (new) | code | 1,080 | `loop claim <item> [--lab-topic <topic>]`: refusals, then writes, then race settlement. GitHub reads and PR create and close sit behind the `Gh` trait. |
| `crates/uor-r4-harness/src/loopsub.rs` (new) | code | 331 | `loop suborder --parent <ORDER> --scope <globs>` and `loop ask --scope\|--blocked`. |
| `main.rs`, `lib.rs`, `loopgates.rs`, `doccheck.rs` | code | 34 | Dispatch beside `gates`, flag tables, and the doccheck `GROUPS` and `FLAGS`. |
| `crates/uor-r4-harness/tests/loop_claim.rs` (new) | tests | 939 | 33 tests over fixture repositories with a bare remote and the fake `gh`. `GIT_*` is stripped. |
| `docs/harness/README.md`, `docs/loop/README.md` | docs | 24 | Usage, exit codes, containment, folders, and the test-only fake. |

That totals about 2,400 lines, excluding this ORDER and REPORT.

**Design calls.**
- **What counts as claimed.** An item is claimed when an open order names it. Open orders are:
  - the ORDERs in this lab's worktrees under the main checkout's `.worktrees/<lab>-*`, not yet merged;
  - the ORDERs in open PRs, read through `refs/pull/<n>/head`.

  An order counts as merged when its head is an ancestor of `origin/main`, or when its ORDER bytes are on `origin/main`.
- **Refusals.** Each exits 2. They run after `git fetch origin main`, against items read from `origin/main`:
  - the fetch fails;
  - the item is not `ready`;
  - the item is already claimed;
  - the item has no `scope:` line;
  - a scope glob is invalid;
  - the item's scope overlaps an open order, judged by conservative literal prefix, with notebook pages excepted;
  - the lab is at its `open_pr_limit`;
  - the target worktree or branch exists locally;
  - the branch exists on origin.
- **Writes, in this order.**
  1. Create the worktree under the main checkout's `.worktrees/<lab>-<topic>` (`--no-track`, from `origin/main`).
  2. Commit the skeleton ORDER at `docs/loop/tasks/<item-id>/<lab>-<n>/ORDER.md`.
  3. Push only if the branch is absent.
  4. Open the draft PR.
  5. Re-list the open PRs.

  Any failure along the way runs the full rollback, removing only what this claim created, and exits 2.
- **Lost race.** If any lower-numbered open PR has an ORDER naming the item, the claim runs the same rollback. It exits 1 when everything was removed, and 2 when anything is left that a person must clear.
- **Suborder.**
  - A child glob must be contained in a parent glob: equal to it, or under `<dir>/` when the parent is `<dir>/**`.
  - Writes `local/<lab>/<topic>/orders/<n>.md` and prints the subagent's block.
- **Ask.**
  - Exactly one of `--scope` (with `--file --reason --lines --own-code`) or `--blocked` (with `--limit --measured --command --state`), plus `--order`.
  - Writes a card with `create_new` to `local/<lab>/<topic>/asks/<n>.md` and posts nothing.
- **Test-only fake.** `UOR_LOOP_GH_FAKE=<json>`, with `prs`, `orderTexts`, `nextPrNumber`, `afterCreate` and `createFails`. Write calls are logged to `<json>.calls.jsonl`.

## Measurements

None. This is a host-only tool.

## Checks run at the head

The main session re-ran these at `e7fd08521`. After round 2 it re-ran the test runs, and the two new negative gates below, on the merge with `main` plus the test fix.

- `cargo test -q --locked -p uor-r4-harness --test loop_claim`: 33 passed.
- **Negative gates, run by the builders.** Each set aside only its own lines, and the copy was restored byte-identical afterwards. In each case the named test FAILED:
  - the overlap check (the planted fault): `an_overlapping_claim_is_refused`;
  - race settlement: the lost-race test;
  - suborder containment, as originally written: both suborder-outside tests;
  - the old string-prefix containment: the `docs/secret.rs` case of `suborder_containment_cannot_be_escaped`. The `..` case in that test never reaches containment: the glob validator refuses it first, so the validator, not containment, pins it. (Round 2 corrected an earlier version of this line, which credited containment with both. The main session re-ran this gate. With the old containment back, `docs/secret.rs` was accepted with exit 0, and the test FAILED.)
  - the id check (round 2): `a_bad_item_or_topic_id_exits_2_and_creates_nothing`, now that it asserts the refusal's message. Before that, it passed with the check set aside.
  - the rollback's leased delete: the create-fails rollback test;
  - the `ls-remote` refusal: the existing-remote-branch test;
  - the draft-only rival filter: the ready-for-review rival test.
- **The `--lab-topic` table entry's negative gate did not fail**; see above.
- **Lease behaviour, checked against a scratch bare remote:**
  - an empty lease creates the branch only when it is absent, and is refused when the branch exists;
  - a delete leased to our commit deletes, and is refused if the branch has moved.
- `cargo test -q --locked -p uor-r4-harness --no-fail-fast`: 740 passed, 0 failed at `e7fd08521`. On the merge with `main` (`8d216ff8f`), which adds loop PR 3's 31 tests, it gave 771 passed and 0 failed. After the test fix, `--test loop_claim` again gave 33 passed.
- `cargo fmt --check` is clean, and clippy reports no warnings in the touched files.
- The PR body lists claim-wording, `xtask links` and the local copyright check.

## Skills used

None of the private kit's skills.

## Still open

- **The real `gh` paths are checked by reading the code only.** One open question: `gh pr list --json files` may cap each PR's files at 100.
- **No item has a `scope:` line yet**, so `loop claim` refuses every item today until the PM backfills scopes.
- **The older `uor-harness team claim` registry (`docs/harness/claims.json`) is not read.** Retiring it belongs to PR 8.
- **`docs/research/notebook/README.md` (the index) passes PR 2's notebook check.** Unrelated here, but noted.
- **Round 2's low findings, left for a follow-up:**
  - One open order with a malformed scope still refuses every claim. This happens when its item's frontmatter does not parse, or when one of its globs fails the validator. An unscoped order, by contrast, is warned about and skipped. Fail-closed is defensible here, but it is inconsistent with the unscoped case. No item has a scope yet, so it cannot happen today.
  - A lower-numbered rival whose ORDER cannot be read (say, after a transient `gh api` error) is not counted. Both claims could then win.
  - The main checkout is taken as the parent of `--git-common-dir`. That is wrong for a bare repository with worktrees, a layout this repository does not use.
  - The git calls drop `GIT_DIR`, `GIT_WORK_TREE` and `GIT_INDEX_FILE`, but inherit the other `GIT_*` variables. This copies `loopgates.rs`.
  - A push that fails after `origin` accepted the ref leaves the branch unnamed in the rollback. A later claim with the same topic is still refused.
  - No test makes suborder's parent item differ between the working tree and `origin/main`.

## Plain summary

`uor-harness loop claim` lets a lab take a roadmap item only when it is ready, unclaimed, and scoped away from everyone else's open work, and only while the lab is under its PR limit. Before checking, it fetches and reads `main`. It creates the worktree, commits the ORDER, pushes only a new branch, and opens a draft PR. If anything fails, or another lab got there first, it undoes exactly what it created and says what, if anything, is left.

`loop suborder` gives a subagent a narrower scope that it cannot escape. `loop ask` writes the two stop-and-ask cards.
