# REPORT: infra-loop-pr05-policy-check, claude-16 (loop PR 5: loop pr-check and the policy job)

Lab: claude. Base: `24130f6df`. Order: [ORDER.md](ORDER.md), committed first (`a2efaffc8`), with the owner's `.github` approval quoted and timed. The cancel probe found no `loop pr-check` and no `gate.yml`.

Four fresh-context builders wrote commits 1a, 1b, 2 and 3, one commit each, and two more wrote the review fixes R1 and R2. The main session wrote commit 4, the CI wiring, and its fixes, because a mistake there could stop the five required checks from being published. It also re-ran every builder's gates before accepting the commit, and wrote this REPORT: a harness hook does not let subagents write REPORT files.

## What did not work, first

- **Review round 1 at `aa652450b` asked for changes.** It used four fresh-context lenses: two on the rule code, one on the tests and docs, one on CI. CI and the second rule-code lens asked for changes; the other two approved with lows. The fixes:
  - **The policy job shared a cargo cache with the tests job (CI, high).** The tests job compiles the PR's code, build scripts included, so the PR could plant compiled dependencies that the "base" policy build would link. `55c31591d` drops the policy job's cache. The same commit makes a failed `merge-base` fail the tests job instead of reading as "no ORDER".
  - **Commit-order holes (R1, `2ae6d3294`):**
    - an "evil merge" before the ORDER commit carried work the rule never saw; a merge before the ORDER commit now fails;
    - the ORDER's `Item:` or `Lab:` could be re-pointed after the work; they must now match the commit that added the ORDER;
    - a renamed ORDER did not count as added;
    - `diff.ignoreSubmodules` could hide gitlinks;
    - `\s` and quote patterns crossed lines, and headings inside code fences counted.
  - **GitHub and content-rule holes (R2, `f98555de9`):**
    - statuses were read one page at a time, so a spoof could be pushed off page one; both endpoints now paginate;
    - only the newest check run per name is judged, and its summary must end with the policy record;
    - the verdict is the first `**Verdict:` line, not any substring;
    - `rule-sources` now catches `## _Rules_`, indented and setext headings, and `.markdown` or `.mdx` files, and treats rule text renamed or copied out of a listed file as added;
    - `read-path` judges growth without the PR's own ORDER, and refuses symlinked files;
    - a run with the test fake set says `FAKE GitHub`.
- **The policy job's step ended at the first non-zero exit.** GitHub runs `run:` steps as `bash -e`, and `set -uo pipefail` does not clear `-e`. On the first two pushes, the base harness's exit 2 ended the step before its verdict was written. The step's own comment claimed the opposite, and neither the local dry run nor the CI review caught it. `d11c6271e` adds `set +e`.

  The step was then checked under `bash -e` with stub harnesses:
  - the bootstrap answer gives `unavailable` and exit 0;
  - another exit 2 gives `error` and exit 1;
  - a failed rule gives `fail` and exit 0.

  The next real run (39633345204, at `d11c6271e`) passed the policy job and recorded `policy: unavailable (run 39633345204)` in all five required checks.
- **This PR cannot be judged by its own policy (bootstrap).** The policy job builds the base commit, which has no `pr-check`. `06f6ef62d` records exactly that answer as `unavailable`; `required-checks` accepts only `pass`. The pass path first runs on the next loop PR.
- **This PR fails its own `scope` rule.** No loop item has a `scope:` line yet, including this one, so `scope` reports "item infra-loop-pr05-policy-check has no scope: at the base". This is the rule working, not a defect. Until the PM backfills scopes in an item PR, every loop PR fails `scope`, and its CI verdict reads `policy: fail`.
- **Two rules cannot run in CI.** When the policy job runs, the review record and the five required checks do not exist yet. So `--ci` marks `review-record` and `required-checks` N/A, and they are enforced only at queue time, by `loop pr-check --pr <n>`.
- **No local workflow linter.** `actionlint` is not installed. Both files parse as YAML, and the job wiring was read back from the parse. The real proof is this PR's own runs, described above.
- **Not fixed: who wrote a review record or an owner approval.** Every lab posts from the same GitHub account (loop page, P-3 and P-4), so a review record's `[<lab>]` label cannot be bound to its author. An ORDER's `## Owner approval` section is likewise written by the PR's author. Both are self-reports that a reviewer reads.
- **The first full suite run failed.** No builder had added `pr-check`'s new flags to doccheck's flag list. The builders' real-repo runs passed only because no doc cited those flags until commit 4. That commit's loop page cites `--ci`, and the documented-commands rule caught it. Commit `0a88f9d92` fixes this.
- **The `--json` flag never reached `pr-check` at first.** The top-level parser consumes `--json`, so `cmd_loop` now passes it back to `pr-check` only. Commit 1a found and fixed this.
- **The source check is narrow.** `required-checks` accepts a check run from the `github-actions` app whose summary carries `policy: pass (run <id>)`. It does not confirm that the run id belongs to `ci.yml` on that head. Any workflow with `checks: write` could post such a run, but adding one means changing `.github/**`, which `owner-paths` refuses without the owner's approval.
- **`rule-sources` catches rule-shaped files and headings only.** It flags a new `.md` file named like a rule book, and a new heading with the word "rule" or "rules". A rule written as plain prose elsewhere is not detected.
- **Not built here.** The loop page defers `docs/loop/baseline.json` to PR 5, but this item's done-when does not include it, so it is not built. The scope section's test-removal and `deps:` rules are not in this item either. Both are flagged for the PM.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopprcheck.rs` (new) | code | 1,750 | `loop pr-check`: eleven rules, size by class, text and `--json` output, and a read-only `PrGh` trait with a test fake. |
| `crates/uor-r4-harness/tests/loop_prcheck.rs` (new) | tests | 1,278 | 27 tests over fixture repositories: at least one failing fixture PR per rule, plus green fixtures. |
| `main.rs`, `lib.rs`, `loopgates.rs`, `loopclaim.rs`, `doccheck.rs` | code | 24 | Dispatch, `--json` pass-through, the item's `notebook:` key, and doccheck's loop group and flags. |
| `.github/workflows/gate.yml` (new) | code | 162 | The `policy` job builds from the base code with no cache, and records `unavailable` for a base older than `pr-check`. The `tests` job runs `loop gates --full`. |
| `.github/workflows/ci.yml` | code | 52 | The `policy` and `gate-tests` callers. `required-transport` now `needs: [policy]` with `always()`, and records the verdict in its summaries. |
| `docs/harness/README.md`, `docs/loop/README.md` | docs | 34 | Usage, rules, CI, and the build-plan row. |

The total is about 3,300 lines, excluding this ORDER and REPORT. The ORDER estimated four builder commits and the CI wiring. That is what was built, and review added two fix rounds and three CI fixes on top.

**The rules.** Lists, items and budgets are read at the merge base; files the PR adds are read at the head.
1. `order`: exactly one ORDER, its lab registered, its `Lab:` line matching the folder, an item that exists, and `Setup row:` for setup items.
2. `scope`: every changed path is inside the item's globs (`*` never crosses `/`), or is always allowed.
3. `commit-order`: the ORDER commit comes first, and it is alone in its folder.
4. `report`: seven sections, in order, each non-empty.
5. `owner-paths`: an owner-path change needs a `## Owner approval` section that names the path, quotes the answer and gives the time.
6. `rule-sources`: no new rule-named file and no new rule heading outside the listed files.
7. `rendered-lock`: only `kimi/render` changes the rendered views.
8. `read-path`: the budgets. Growing past a budget fails; already being over budget warns.
9. `filemap`: a row for each directory at depth 1 and 2.
10. `review-record`: a counted exact-head APPROVE (D59, D46).
11. `required-checks`: `policy: pass` from GitHub Actions, and no commit status of the same name.

Commit 1b's `.github` refusal is part of `owner-paths`.

## Measurements

None. This is a host tool and CI wiring.

## Checks run at the head

The main session re-ran each builder's gates before accepting its commit. It ran these checks first at `0a88f9d92`, and again after the review fixes (`f98555de9`), with this REPORT in place.

- `cargo test -q --locked -p uor-r4-harness --test loop_prcheck` gave 22 passed, then 27 after the fixes.
- **Negative gates**, run by the builders. Each one set aside only its own rule's lines, then restored the file byte-identical, checked with `cmp`. In every case the named test FAILED:
  - `scope`: `an_out_of_scope_change_fails_scope`;
  - `owner-paths`: `a_github_change_without_owner_approval_fails` (the item's named refusal);
  - `commit-order`: `commit_order_needs_the_order_commit_first_and_alone`;
  - `rule-sources`, `rendered-lock` and `filemap`: their refusal tests;
  - `review-record`: the codex-on-a-setup-PR test (D59);
  - `required-checks`: the commit-status test;
  - fix round R1: the evil-merge test and the re-pointed-ORDER test;
  - fix round R2: the newest-run test (`required_checks_judge_the_newest_run_per_name`), and the heading-forms test (`rule_sources_refuses_rules_outside_the_list`, with the old heading regex restored).

  `order`, `report` and `read-path` have failing fixture PRs, but no set-aside gate.
- **The full suite**, `cargo test -q --locked -p uor-r4-harness --no-fail-fast`. The first run gave 794 passed and 1 failed. The real-repo documented-commands rule rejected the loop page's `uor-harness loop pr-check --ci`, because the new flags were not in doccheck's list. Commit `0a88f9d92` registers them. The re-run gave 795 passed and 0 failed. After the review fixes, at `f98555de9` with this REPORT in place, it gave 800 passed and 0 failed.
- **`loop pr-check --local --base 24130f6df`** on this branch:
  - `order`, `commit-order`, `rule-sources` and `read-path` (28,306 B) pass;
  - `owner-paths` passes ("owner approval recorded for .github/**");
  - `rendered-lock`, `filemap`, `review-record` and `required-checks` are N/A;
  - `scope` fails, as reported above;
  - `report` passes once this file is committed.
- Both workflow files parse as YAML. `required-transport` reads back `needs: [policy]` with `always()`.
- `cargo fmt -p uor-r4-harness --check` is clean, and clippy reports no warnings in the touched files. Claim-wording passed. `xtask links` found 0 broken links in the changed files. The local copyright check found 0 matching 8-word runs.

## Skills used

None of the private kit's skills.

## Still open

- **Scopes.** Backfilling `scope:` on the loop items (and the rest) is the PM's item PR. Until then `scope` fails every loop PR.
- **The pass path.** On this PR the policy job can only record `unavailable`. The first `policy: pass` or `fail` verdict comes from the next loop PR's CI (PR 7b), and that PR's REPORT should quote it.
- **Folded in with PR 8.** `rendered-lock` and `filemap` stay N/A until PR 8 commits the views and FILEMAP. The roadmap README and `baseline.json` join the lock then.
- **Hardening.** `required-checks` could confirm the run id against the `ci.yml` run for the head sha. Required acknowledgements now wait for the base build, about a minute without a cache.
- **Owner call, if wanted.** Review records and owner approvals are self-reports (see above). Binding them would need a per-lab identity, which a single shared account does not give.
- **`loop queue`** (PR 7c) will call `pr-check --pr <n>`, and PR 12's hook will call `pr-check --local`.

## Plain summary

`uor-harness loop pr-check` judges a loop PR against the loop's rules: the PR must start with an ORDER, stay inside its item's scope, carry a complete REPORT, get the owner's recorded approval before touching owner files, and not add rules, edit rendered files or blow the reading budgets. It reads the rules from the base branch, so a PR cannot relax them. With `--pr <n>`, it also checks GitHub: is there an approving review record at the exact head commit, and did the five required checks really come from the policy job?

On every pull request, CI runs the same check using the base branch's code. It writes the result into the five required checks without turning them red, as the owner chose. The real lock is applied at queue time.
