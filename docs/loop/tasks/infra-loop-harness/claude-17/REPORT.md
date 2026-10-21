# REPORT: infra-loop-pr07b-session, claude-17 (loop PR 7b: loop next, arrive, report and close)

Lab: claude. Base: `8c2c76e42`. Order: [ORDER.md](ORDER.md), committed first (`4be9a7687`). The cancel probe found no `next`, `arrive`, `report` or `close` under `loop`.

Five fresh-context builders wrote one commit each: `report`, `arrive`, `next`, `close --session` and `close --verify`. Four more wrote the review fixes. The main session re-ran every builder's gates before accepting its commit. It updated the loop page and wrote this REPORT, because a harness hook does not let subagents write REPORT files.

## What did not work, first

- **Review round 1 at `0974f2ef7` asked for changes.** Four fresh-context lenses reviewed it. `close` safety and `next`/`report` asked for changes; `arrive` with the shared code, and the tests and docs, approved with lows.
  - **PICKUP was lost on delivery.** Two lenses found this independently. `close --session` wrote PICKUP inside the task worktree, and `close --verify` deletes that worktree. `/local/` is gitignored, so the cleanliness check never saw the file. `arrive` read PICKUP from the current worktree, so even a surviving copy was invisible to the next session. Fixed in `6a5432ee4` and `702b97154`:
    - PICKUP now lives under the main checkout's `local/<lab>/`, the lab's own area on the loop page;
    - `--verify` keeps a worktree that holds ignored files other than `target/`, the rendered views and the gate ledger.
  - **`next` judged a PR by the first ORDER-shaped path it touched, not the ORDER it adds.** A PR editing another lab's ORDER was judged as that lab's. A PR deleting an ORDER made `next` exit 2 for every lab. `b6e3c0b2a` uses only the ORDER the PR adds, and skips an unreadable PR with a note.
  - **The other `close` fixes (`6a5432ee4`):**
    - the content proof also accepts an equal patch, so a PR whose file main also changed is not refused for ever;
    - renames list both paths;
    - the local branch delete is one atomic `update-ref -d`, and a branch some worktree still has checked out is kept;
    - `--session --pr` checks that the PR is open and on this branch;
    - a FAKE banner prints when the test fake is active;
    - SSH auth variables survive the env stripping.
  - **The other `arrive` fixes (`702b97154`):**
    - the read path counts the rendered STATE and HANDOFF while none is committed;
    - a hook must be executable;
    - standing references skip fences and subheadings;
    - `--json` is passed back only for the action word.
  - **The other `next` and `report` fixes (`5c64948d5`):**
    - more than 100 files and the labels are read across pages;
    - fill markers inside code fences are ignored;
    - vendors come from `origin/main`;
    - `--receipts` keeps only rows recorded on a clean tree at HEAD;
    - two ordering tests: review before an item, and STOP before a repair.
- **Review round 2 at `97479baa0` approved with lows, and its medium came from round 1's own fix.**
  - The `next`/`arrive`/`report` lens found that skipping an unreadable PR, which is right for reviews, made the lab's own repairs fail open. A transient `gh` error on its own PR could hide a pending REQUEST CHANGES.
  - The `close` lens found that `git patch-id --stable` ignores whitespace, so a squash differing only in whitespace passed the content proof. It reproduced this in a scratch repository.
  - The final round, `af5d12319`:
    - own-PR read errors make `next` exit 2;
    - an own PR whose ORDER is missing or ambiguous is a repair;
    - `patch-id --verbatim`;
    - the branch's config section is removed after the atomic delete;
    - tests for a closed `--pr` and for the gate-ledger exception.
  - A deleted ORDER path at a PR head is recognised from `gh`'s 404 or "Not Found" error text, which is string matching. A delta check then found the match also read the command line `run_gh` echoes into its error, which holds the PR number and path (PR #4040, or a `claude-404` folder, would turn a 5xx into "absent"). `path_absent` now reads only `gh`'s own message after `" failed: "`; with the old match its unit test FAILED.
- **The first real policy verdict is `fail`, as expected.** CI built `uor-harness` from main (which now has `pr-check`) and judged this PR: `policy: fail (run 39644023923)`, recorded in all five required checks, which stay green. In the job log only `scope` fails, line for line the same as the local run. Loop PR 5's REPORT asked for this run to be quoted. `loop pr-check --pr 2925` would refuse to queue it, but `loop queue` (PR 7c) is not built. Until the supervisor exists, the loop page says the CHARTER §5.5 delivery path applies as written.

- **Step 13 is not followed literally, and the owner should decide.** The loop page says `close --verify` should "run `uor-hygiene --apply`". Hygiene's `--apply` removes every clean, merged worktree with its branch, other labs' included. That crosses the lab boundary: never touch another session's files, and never delete another lab's work. So `close --verify` prints hygiene's plan (`--dry-run`) and does not apply it. The loop page's rule text is unchanged; its command cell says so. Applying it, or limiting hygiene to the lab's own worktrees, is the owner's call. This is reversible.
- **Most of `loop next`'s sources do not exist yet.**
  - Owner directions and premise probes need PLAN (loop PR 8). GPU collections need the watchers (loop PR 10). No lab has a lane, and no item has `lane:` or `rank:`.
  - `next` names each missing source and the PR that builds it, and never guesses.
  - No item has a `scope:` either, so `any` finds nothing. Run against real GitHub, `next` answered "STOP held work: 26 item(s) held, none ready and claimable".
- **Step 11 is only half built.** `next` finds the lab's PRs that need repair (a conflict, a failed check, or a counted REQUEST CHANGES at the head). Writing the repair ORDER `<lab>-<n+1>` is still done by hand.
- **The first full suite run failed once (850 passed, 1 failed).** Loop PR 4's `watch_and_other_actions_are_refused` used `loop next` as its example of an action that is not built yet, and this PR builds it. `5cc3c63e5` switches the example to `loop drive` (loop PR 10). The specified behaviour changed; the test was not weakened.
- **`report --check` refused this REPORT.** The REPORT names the `<!-- fill:` marker in prose, inside backticks, and the check matched the marker anywhere on a line. A template prompt is a line of its own, so `95014fbe3` counts only a line that starts with the marker. A new assertion in the fill-line test reproduces this REPORT's case. With the old match restored, it FAILED.
- **The ORDER template had a dead link.** Its item link is relative to a task folder, so from `docs/loop/templates/` it resolved nowhere and `xtask links` caught it. `4282b9ff0` puts it in a code span as an example.
- **The binary's own commit is unknown.** No commit is baked into `uor-harness` (its build script explains why). So `arrive` checks the checkout against `origin/main` and the installed binary against the newest local build, not the binary against `origin/main`.
- **`arrive` found a real mismatch on its first run.** The shared git `pre-commit` hook differs from the one the current binary installs. It reports this as a WARN, and `uor-harness git install-hooks` fixes it. Client hooks (Claude Code settings) are N/A until loop PR 9.
- **GitHub reads and writes are tested only through fakes.** These are: the PR body and the ready state (`close --session`); the merge state (`close --verify`); and the PR list, comments, files and labels (`next`). The real `gh` paths are checked by reading the code.
- **`hygiene` cannot be cited in docs.** The harness's documented-commands rule does not list the `hygiene` subcommand, so the README names the `uor-hygiene` wrapper instead. This is outside scope and noted for the PM.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopreport.rs` (new) | code | 340 | `loop report --check`, `--new` and `--receipts`. |
| `crates/uor-r4-harness/src/looparrive.rs` (new) | code | 593 | `loop arrive`. In order: fetch, checkout, binary, hooks, client hooks, views, PICKUP, read path. |
| `crates/uor-r4-harness/src/loopnext.rs` (new) | code | 689 | `loop next`. The sources in the loop page's order, the stops, and a read-only `NextGh` trait with a fake. |
| `crates/uor-r4-harness/src/loopclose.rs` (new) | code | 1,209 | `loop close --session` and `--verify`, with a `CloseGh` trait whose fake logs every write. |
| `loopprcheck.rs`, `loopclaim.rs`, `loopgates.rs`, `labs.rs`, `doccheck.rs`, `main.rs`, `lib.rs` | code | 100 | Shared helpers: `report_file_problems`, `parse_review_record`, `size_line`, `find_order`, `main_checkout`. Also dispatch, doccheck's loop group and flags, and `--json` pass-through. |
| `crates/uor-r4-harness/tests/loop_{report,arrive,next,close}.rs` (new), and one line of `loop_gates_run_rules.rs` | tests | 2,062 | 72 tests over fixture repositories with bare remotes and fakes. |
| `docs/loop/templates/{ORDER,REPORT,PICKUP}.md` (new) | docs | 76 | Templates in our own words. Template prompts are `<!-- fill: -->` lines, which `report --check` refuses. |
| `docs/harness/README.md`, `docs/loop/README.md` | docs | 93 | Usage, and the steps marked built. |

That totals about 5,160 lines, excluding this ORDER and REPORT. The review fix rounds added 1,071 of them and removed 130. (Earlier versions of this line said 3,770, an undercount even before review, and then that the fix rounds added "about 1,260", which was only the difference from that undercount.) The ORDER set out five commands across five builder commits, which is what was built.

**Design calls.**
- **Exit codes.** `next` exits 0 for GO, 1 for STOP and 2 when it cannot run. The others exit 0 when ok, 1 when a check fails or the command refuses, and 2 when they cannot run.
- **`report --check`** reuses `pr-check`'s section checker. It also refuses a leftover `<!-- fill:` line, so a bare template never passes. `--receipts` copies this head's results from `local/<lab>/gates.jsonl` into the task folder, as raw lines, latest per gate.
- **`arrive`** renders the views from `origin/main` into `local/<lab>/views/`. A committed view that disagrees with its inputs FAILs: the repository wins. A PICKUP of 8 KB or more FAILs, and so does a standing reference that cites no §, D-row or L-id.
- **`close --session`** refuses, before any write, when:
  - the REPORT is incomplete;
  - the PR head is not local HEAD;
  - the PICKUP would be 8 KB or more.

  It then sets the body, with the size line first and `References #<issue>` from `docs/roadmap/issues.json`. It marks a draft ready and writes PICKUP under the main checkout's `local/<lab>/`.
- **`close --verify`** deletes nothing until four proofs pass. Then each deletion has its own guard, and the deletions run independently of one another. Another lab's PR is verified but never cleaned up. The proofs:
  1. GitHub reports MERGED.
  2. The `(#N)` squash commit is on `origin/main`.
  3. Main's copy of every changed file equals the PR head, or the PR's patch equals the merge's patch for those files.
  4. `loop gates --full` passes in a throwaway worktree of `origin/main`.

  The deletion guards:
  - the worktree must be clean, under `.worktrees/<lab>-*`, and hold no ignored files beyond `target/`, the rendered views and the gate ledger; it is never force-removed, and a change to the old `claims.json` alone is reverted first;
  - the local branch is deleted atomically, and only at the PR head and while no worktree has it checked out;
  - the remote delete only succeeds if the branch is still at the PR head.

## Measurements

None. This is a host tool.

## Checks run at the head

The main session re-ran each builder's tests at its commit. It ran the checks below first before review, and again after the final fix round (`af5d12319`), with this REPORT in place for the full suite.

- **Targeted suites, after the fixes:** `loop_report` 7, `loop_arrive` 17, `loop_next` 20 and `loop_close` 28 passed. `loop_prcheck` (27) and `loop_gates_run_rules` (29) still pass. Before review the four new suites gave 6, 12, 12 and 19.
- **Negative gates for the fix rounds**, run the same way:
  - PICKUP written to the old worktree path: the survives-verify test FAILED;
  - the ignored-files guard dropped: the kept-notes test FAILED;
  - the patch-id alternative dropped: the shared-file test FAILED;
  - `arrive` reading PICKUP from the worktree: the main-checkout test FAILED;
  - the executable-bit guard dropped: its test FAILED;
  - the `--json` fix set aside: `json_goes_only_to_the_action_word` FAILED;
  - first-match ORDER selection restored: the other-lab-ORDER test FAILED;
  - an unreadable ORDER made fatal: the deleted-ORDER test FAILED;
  - the receipts tree match dropped: the dirty-row test FAILED;
  - the paginated-files fallback dropped: the 100-files test FAILED;
  - the final round: the own-PR skip restored: `an_own_pr_whose_comments_cannot_be_read_cannot_run` FAILED; `--stable` restored: `a_squash_differing_only_in_whitespace_deletes_nothing` FAILED.
- **Negative gates.** These were run by the builders. Each set aside only its own check, ran the test (which FAILED), then restored the file, confirmed byte-identical with `cmp`. The checks and their tests:
  - **The planted fault:** `report --check` ignoring the section problems; `a_report_missing_a_section_is_refused`.
  - **`arrive`'s PICKUP citation check:** the uncited-reference test.
  - **`next`:** the STOP-file check, and repairs ranking ahead of reviews; `a_stop_file_stops_on_the_kill_switch` and `repairs_come_before_reviews`.
  - **`close --session`:** the REPORT check before writes, and the push-first head check.
  - **`close --verify`:** the MERGED proof, the content proof, the own-lab check, and the remote lease (swapped for a plain delete).
- **The full suite**, `cargo test -q --locked -p uor-r4-harness --no-fail-fast`: the first run gave 850 passed and 1 failed (see above); at `41bb400ed`, with this REPORT in place, 851 passed and 0 failed; after the fix rounds (at `8f5b1d11e`), 870 passed and 1 failed: `receipt_command_contract`'s `the_short_milestone_form_is_recorded_like_the_long_one` compares two receipt lines whose seconds-resolution `ts` straddled a second (17:59:41 against 17:59:42). This PR does not touch that command or its test; the target then passed three times in a row. It is a flaky test that already existed, listed under "Still open". After the final round (at `097f576f1`, with this REPORT in place): 877 passed and 0 failed; at `42edc4089`, after the `path_absent` fix, 878 passed and 0 failed.
- **`loop pr-check --local --base 8c2c76e42`** on this branch: `order`, `commit-order`, `report`, `owner-paths`, `rule-sources` and `read-path` (28,257 B) pass, `rendered-lock`, `filemap`, `review-record` and `required-checks` are N/A, and `scope` fails because the item has no `scope:` at the base, as for every loop PR until scopes are backfilled.
- **Real-world runs** in this worktree:
  - `loop arrive` exited 0, with a WARN for checkout and hooks;
  - `loop next` ended in STOP held work (exit 1), reading GitHub without writing.
- `cargo fmt -p uor-r4-harness --check` is clean, and whole-crate clippy `--all-targets` is clean. Claim-wording passed; `xtask links` finds 0 broken links in the changed files (after `4282b9ff0`); the local copyright check finds 0 matching 8-word runs, the templates included.

## Skills used

None of the private kit's skills.

## Still open

- **Owner call: hygiene in step 13.** Should it be applied, or limited to the lab's own worktrees? See the first bullet above.
- **The repair ORDER.** Writing `<lab>-<n+1>` for step 11 is not built.
- **Missing sources for `next`.** PLAN (PR 8) will add owner directions and probes. Watchers (PR 10) will add GPU collections. Lanes need a `lane:` key and a PM ranking.
- **Real `gh` paths.** `close --session` ran once against real GitHub, on this PR: it set the body, marked the PR ready and wrote the PICKUP. The other real paths are covered only by reading the code. `next` judges checks by FAILURE and TIMED_OUT only, so an ERROR status does not trigger a repair.
- **PICKUP.** The standing references in a written PICKUP are left empty for the lab to fill.
- **Review lows not fixed:**
  - a failed hygiene dry-run does not change `--verify`'s exit code;
  - the test fakes are honoured in release builds, but now print a FAKE banner;
  - `next` runs `git fetch` of `origin/main`, the only write to the checkout it makes.
- **A flaky test outside this PR.** `receipt_command_contract`'s short-and-long-form test compares whole receipt lines, including a seconds-resolution timestamp, so it fails when the two runs straddle a second. It needs the `ts` field masked before comparing; that is outside this item.

## Plain summary

These commands carry a lab through one session:
- `loop next` says whether to work and on what. It stops on the kill switch, prefers repairs to the lab's own PRs, then reviews owed, then a ready item it can claim. It names the sources the project has not built yet.
- `loop arrive` refreshes the checkout and checks that the tools, hooks, rendered notes and reading budget are sound.
- `loop report --check` refuses a REPORT with a missing or unfilled section.
- `loop close --session` turns the REPORT into the PR description, marks the PR ready and leaves a note for the next session.
- After merge, `loop close --verify` proves the merge landed exactly as reviewed. Only then does it delete the lab's own branch and worktree, and it deletes nothing it cannot prove belongs to that lab.
