# The research harness

The repo has strong *policy* — the [session-goal contract](../labs/session-goal.md),
D21's negative count, D22's TEST FITNESS gate, the [delivery cadence](../../CONTRIBUTING.md#delivery-cadence-every-completed-unit-owner-9-october-2026) —
but almost nothing that **instruments** it. A session cannot tell from the repo which
skill it used, which agents it dispatched, what the milestone headline is right now,
what it cost, or which past failure applies to the file it is about to edit.

`uor-harness` is that instrument. It is one Rust binary (the crate
[`crates/uor-r4-harness`](../../crates/uor-r4-harness)) that turns the repo's existing authorities
into one queryable surface and emits **receipts** so orchestration is visible after the fact. It
is the only implementation: the Python mirror it was ported from was retired on 20 October 2026
(owner rule: Rust only, D27, CHARTER §3.11) and is kept byte-identical under
`archive/scripts/harness/`.

## What it is not

- Not a new authority. `AGENTS.md`, `DECISIONS.md`, the milestone issues, `docs/mechanisms/`
  and the dated `docs/labs/` records stay authoritative. The harness reads them.
- Not a model. It contains no model code and runs no model compute. It cannot move a
  headline number, and by [D24](../integration/DECISIONS.md) no GPU work may run on it.
- Not a replacement for the human record. Every harness output is a *view* that cites the
  file or issue it came from.

## The registry (`docs/harness/`)

| File | Role |
|---|---|
| `milestones.json` | machine view of the live milestone contract (m1–m4): issue, lab, headline, panel target |
| `skills/<id>/SKILL.md` | one research skill, [Agent Skills](https://code.claude.com/docs/en/skills) front matter + protocol |
| `roles.json` | the expert team: dispatchable roles, their default model route and their seat |
| `mistakes/<id>.md` | one durable lesson: trigger, symptom, lesson, and a machine-checkable guard |
| `panels/*.json` | the acceptance panels and their frozen targets |
| `arena.json` | the lineage of evaluated configurations and every bar's direction |
| `presence/<lab>.json` | where each lab is and when it last checked in (committed) |
| `claims.json` | one owner per branch, visible to every lab (committed) |
| `ledger/<lab>-<month>.jsonl` | one receipt shard per lab, so concurrent PRs never conflict |
| `pm-rules.md` | the PM operating rules every lab follows through the harness (R1–R10) |

`docs/mechanisms/` (the idea-level briefs) stays where it is and is cited by skills.

## Visibility: the two receipts

`uor-harness skill use` and `uor-harness team dispatch` append one JSON line each to
`docs/harness/ledger/<YYYY-MM>.jsonl`. That file is the answer to "was a skill actually
used, and did an agent actually get deployed". Local scratch state (heartbeats, budgets)
goes to the gitignored `.harness/`.

## Commands

```
uor-harness status                      what is true now: milestones, rules, today's receipts
uor-harness brief --milestone m1        assemble the context pack for a task (the thing to paste)
uor-harness skill list|show|use         the registry, and a receipt for using one
uor-harness team roster|dispatch        dispatchable roles, and a receipt for dispatching one
uor-harness mistakes list|find          durable lessons, selected for the files you are about to touch
uor-harness prep new|fit                the pre-registration sheet, and its TEST FITNESS review
uor-harness panel show|score            the frozen acceptance panels
uor-harness arena board|compare|lineage  the lineage of evaluated configurations, compared row by row
uor-harness multilab presence|claims|sync-live   what each lab published, who holds which branch
uor-harness budget show|scan            cumulative cost, and the pod specs of running jobs
uor-harness lint check|format           lint a PR body against the contract; render a compliant one
uor-harness ratchet d23|d25             the D23 (norm-exp/squashing/activation/rotary) and D25 (matmul/tensor-core) ratchets
uor-harness heartbeat --milestone m1    check in (live tier; 5-minute window)
uor-harness git install-hooks|status    the three managed git hooks (owner-only), and this branch's delivery state
uor-harness git check-message <file>    the commit-msg hook: Lab: and Order: trailers on a lab branch
uor-harness git push-check              the pre-push hook: pr-check --local for each pushed lab branch
uor-harness check [--staged]            every machine-checkable rule in one pass
```

### `loop render` (loop PR 3)

```sh
UOR_LAB=claude uor-harness loop render                       # render the views in place from HEAD
UOR_LAB=claude uor-harness loop render --check               # compare the views on disk with a fresh render
UOR_LAB=claude uor-harness loop render --out /tmp/views      # write the views under a scratch directory
UOR_LAB=claude uor-harness loop render --at abc1234          # render from an older commit
```

It reads committed files only, at one commit, and prints that commit in a stamp on the first line of each view. Views: `STATE.md`, `HANDOFF.md` (open items by status, one blocked table for held and decision items, the not-doing list), `docs/harness/LESSONS.md`, the top-ten block in `AGENTS.md` and `docs/roadmap/README.md`. `--check` re-renders at the stamped commit and compares, so a view that lags newer inputs is not flagged here; exit 1 means a view differs, 2 means it could not run.

- **AGENTS block.** Render writes `AGENTS.md` only when it holds both `<!-- loop:lessons-top-ten:start -->` and `<!-- loop:lessons-top-ten:end -->`, and replaces only the bytes between them. Without both markers it leaves the file alone and prints `AGENTS.md: no markers; top-ten block not rendered`. In place, the block is spliced into the working-tree `AGENTS.md` between its own markers and every other byte stays as it is on disk; if the commit has the markers but the file on disk has them missing or malformed (one only, a duplicate, or the end first), render refuses with exit 2 and writes nothing. `--check` compares only the bytes between the markers, passes with that note when neither the commit nor the file has them, and exits 1 when the commit has them and the file's are missing or malformed; malformed markers in the commit exit 2. Loop PR 8 adds the markers.
- **Roadmap view.** `docs/roadmap/README.md` comes from `cargo run -q --locked -p xtask -- roadmap render [--check]`, which this command runs. Any xtask or cargo failure, a signal or a missing cargo prints `docs/roadmap/README.md: REFUSED` and exits 2; only exit 1 under `--check` is a difference (exit 1). The child runs without `GIT_*`, `CARGO_TARGET_DIR`, `RUSTFLAGS` and `CARGO_BUILD_*`. With `--out` the step is skipped with a note, because xtask renders inside the repository; with `--at`, or a `--check` whose stamp names a commit other than HEAD, it is skipped because xtask renders the working tree.
- **Before loop PR 8.** Until loop PR 8 commits the views, run `loop render --out <dir>` on the real repository: an in-place `--check` there exits 2 because the views are not committed yet, and an in-place render would write untracked view files.

### `loop migrate --check` (loop PR 8)

Verifies the notes-migration invariants: retired notes carry pointers to byte-identical archive copies, AGENTS.md carries the lessons markers, every lesson is keyed `id: L-NNN`, the after-charter DECISIONS table keeps its two columns, FILEMAP covers every tracked directory two levels deep, the read-path budget holds, and every link in the migrated views resolves. Read-only: exit 0 pass, 1 with each failure named, 2 on a usage error.

### `loop gates` (loop PR 4)

```sh
UOR_LAB=claude uor-harness loop gates --quick      # run the quick gates in docs/harness/gates.json
UOR_LAB=claude uor-harness loop gates --watch      # a full run in a terminal also shows a progress window
UOR_LAB=claude uor-harness loop gates --keep-going # run every gate even after a red
UOR_LAB=claude uor-harness loop gates --status     # latest result per gate; STALE after an uncommitted edit
UOR_LAB=claude uor-harness loop gates --break all  # plant each declared fault in a scratch worktree
```

Verdicts: GREEN, RED, REFUSED (could not run, or its reads match no file), UNKNOWN (not run after an earlier non-green result), VOID (a read file changed during the run). A break declares appended text or one literal replacement (`replace`, whose `from` must occur exactly once), so a fault can remove behaviour rather than add a case. A break run is CAUGHT (green at HEAD, then red with its declared text), REFUSED (already not green before the break, or red for another reason) or DEFECT (stayed green); `--break` refuses an uncommitted `docs/harness/gates.json`, since breaks run against the committed tree. Logs and exit codes go under `local/<lab>/gates/<run-id>/`; the ledger is `local/<lab>/gates.jsonl`. A leading `cargo` in a gate command resolves to `$CARGO`, then `~/.cargo/bin/cargo`, then `PATH`.

**The progress window (built, loop PR 11).** `--watch` writes a status file after every gate under `local/<lab>/gates/<run-id>/status.json` and draws it on stderr: `gate N of M`, the gate running now, each red as it lands, and a finish estimate from the latest seconds per gate in the ledger ("no estimate yet" on a first run). It opens only for a full run in a terminal, never in CI (`CI` set) and never in a headless or piped run, and it closes after the run. It gives up after five minutes without a change to the status file and says so; a single gate that runs longer than five minutes therefore closes the window while the run continues, unchanged. Nothing in the run depends on it: if it is off, crashes or gives up, the gates and the exit code are the same.

### `loop drive` (loop PR 10)

The supervisor. It asks `loop next` first and starts a session **only on `Go`**: the kill switch
(`local/loop/STOP`, or the `loop:pause` label on #2028), held work, an empty queue and the other
stops are the loop page's, and `drive` calls them rather than re-implementing them. The headless
client command follows a `--` separator and carries exactly one `{prompt}` element, which the
supervisor replaces with the session prompt (the caller's `--prompt`, or a default built from the
`loop next` decision). The client runs in its own process group; the pid, pgid, client, model,
prompt SHA-256 and start time are written under `local/loop/drive/<lab>/last-session.json`, and only
that recorded pid/group is ever signalled, so another lab's worktree and a client's after-stop
summary process are never touched. `--model` must appear in the client command: the drive refuses a
model the launch argv does not carry, so the recorded model is the one actually launched. `--order <ORDER.md>` names the active ORDER for the scoped WIP
commit a stalled close makes (staging only its task folder, with the `Order:` trailer); `--exempt-lease`
marks a session whose lease covers it so the check-in never closes it; the command and the signals are
unix-only, and the flag has no meaning on other targets.

```sh
UOR_LAB=claude uor-harness loop drive --client claude --model <id> --prompt-file <path> --max-sessions 2
UOR_LAB=claude uor-harness loop drive --watch --pid <pid> --interval 60
UOR_LAB=claude uor-harness loop drive --watch --lease --interval 300
```

**Supervisor stops card the PM.** `STOP` (the loop next asked the supervisor to stop), an explicit multi-session `--max-sessions` limit being reached, and an error exit each post a card asking the PM to prompt the lab; a clean session exit that the supervisor restarts, and the default single-session unit, are not stops and post no card.

**The 2-hour check-in is soft (D53).** A session under two hours continues. Past it, real progress -
new commits, new gate runs or a changed working tree - continues **with a note** posted on the work
issue (the PR the decision names, or #2028 for an item or command target) through `gh`, with a copy
kept under `local/loop/drive/<lab>/note-<time>.md`; a stalled or
spiraled session is closed with a REPORT stub and a PICKUP written from git state and a card to the
PM (in `local/loop/drive/<lab>/close-<time>/`, and posted on #2028 when `gh` succeeds). `--watch` is
the session-bound watcher: while its recorded pid is alive it runs the renew command given after
`--`, notes it, and ends when the run ends; it never cancels a healthy run. `--watch --lease` is the
watcher for a leased GPU run and **outlives the lab session** when started detached: it renews while
the renew command succeeds, notes each renewal, and stops when that command reports the lease ended. The `loop-drive` gate proves these with a fake stalled client
past two hours (closed and carded) and a fake progressing one (continues with a note); its planted
fault removes the stalled-session close.
### `loop claim`, `loop suborder` and `loop ask` (loop PR 7a)

```sh
UOR_LAB=claude uor-harness loop claim <item-id> [--lab-topic <topic>]
UOR_LAB=claude uor-harness loop suborder --parent <ORDER> --scope <glob>[,<glob>...]
UOR_LAB=claude uor-harness loop ask --scope --order <ORDER> --file <path> --reason <text> --lines <n> --own-code <yes|no>
UOR_LAB=claude uor-harness loop ask --blocked --order <ORDER> --limit <text> --measured <text> --command <text> --state <refused|not-reached>
```

`loop claim` refuses unless the item is ready and unclaimed, has a `scope:` line that covers none of the nine shared files (see the `scope` rule below), overlaps no open order's scope (notebook pages and shared files excepted) and the lab is under its `open_pr_limit`. It then creates `.worktrees/<lab>-<topic>` from fresh `origin/main`, commits a skeleton ORDER at `docs/loop/tasks/<item-id>/<lab>-<n>/ORDER.md` (the folder is the item id), opens a draft PR and lists the PRs again. It refuses if the branch already exists on `origin` or `origin` cannot be checked, and pushes only while the branch is absent there. Any failure after the worktree exists, and a lost race, runs one rollback that attempts every step: close our PR, delete our branch from `origin` only while it is still at the commit we pushed, remove our worktree and branch by exact path and name. Exit codes: 0 claimed; 1 lost the race to a lower-numbered open PR for the same item and the rollback was clean; 2 refused, a write failed, or a rollback left something, which is named so a person can remove it.
For `loop claim`, `<topic>` is `--lab-topic`, which defaults to the item id; for `loop suborder` and `loop ask` it is the parent ORDER's folder name.

`loop suborder` refuses, with exit 2, any glob not inside the parent ORDER's scope (its item's `scope:`): a child glob is contained when it equals the parent glob, or the parent is `<dir>/**` and the child lies under `<dir>/`. The parent item is read from `origin/main`. It writes `local/<lab>/<topic>/orders/<n>.md` (created with `create_new`, so a concurrent card takes the next number), where `<topic>` is the parent ORDER's folder name, and prints the subagent's block with the stop rules. `loop ask` writes a stop-and-ask card to `local/<lab>/<topic>/asks/<n>.md` and posts nothing; a missing field exits 2.

Tests only: `UOR_LOOP_GH_FAKE=<file>` replaces `gh` entirely. The file holds either the array `gh pr list --json number,headRefName,isDraft,files` prints, or an object `{"prs": [...], "nextPrNumber": n, "afterCreate": [...]}`; `afterCreate` PRs join the list once the draft PR exists, and each PR may carry `"orderTexts": {path: text}` for the ORDER files it changes. `"createFails": true` makes `create_draft_pr` fail without recording anything. Each GitHub write the claim makes (`create_draft_pr`, `close_pr`) is appended as one JSON line to `<file>.calls.jsonl`. Never set it outside tests.

### `loop pr-check` (loop PR 5)

```sh
UOR_LAB=claude uor-harness loop pr-check [--base <rev>] [--head <rev>] [--head-ref <branch>] [--pr <n> | --local | --ci] [--json]
```

Checks a loop PR: the diff from `git merge-base <base> <head>` to `<head>` (`--base` defaults to `origin/main`, `--head` to `HEAD`), read with `git`, never the working tree; only `review-record` and `required-checks` read GitHub, read-only through `gh api` and only with `--pr <n>`. `--local` makes those two rules N/A ("local run"), `--ci` makes them N/A because the policy job runs before the review record and the checks exist, and with none of the three they fail ("needs --pr <n>, or --local"); give at most one. **Registries and items are read at the merge base**, so a PR cannot loosen its own check; the files it adds are read at the head. It prints a size line by class (tests, code, docs, generated, other), one `PASS|FAIL|N/A <rule> <summary>` line per rule with indented details, then `verdict: PASS|FAIL`; `--json` prints one object instead. Exit codes: 0 every rule passes; 1 a rule failed, including a rule whose input could not be read; 2 the check could not run (bad rev, git failure, usage). There are no warnings left: `read-path`'s warning became an error in loop PR 11.

- `order`: the PR adds one `docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md`, or a repair chain: several, all of one lab in one folder, whose highest `n` (numeric, so `-10` beats `-9`) is the **active ORDER** every tool judges, while the rest are history (loop step 11). ORDERs of two labs or two folders, or two with the same `n`, fail. The active ORDER's lab is in `docs/harness/labs.json`, its `Lab:` line names the same lab, its `Item:` names an item in `docs/roadmap/items/`, and an `infra-loop-pr*` item's ORDER carries a `Setup row:` line.
- `scope`: every changed path (both sides of a rename) matches a glob in the item's `scope:` (`*` and `?` never cross `/`; a `**` segment matches zero or more segments), or is always allowed: the folder of every ORDER in the chain, one of the nine shared files (`crates/uor-r4-harness/src/{lib,main,loopgates,doccheck,loopprcheck}.rs`, `docs/harness/README.md`, `docs/loop/README.md`, `docs/harness/gates.json`, `docs/roadmap/README.md`; `loopclaim::SHARED_FILES`, set by Casey on 21 and 22 Oct), a `status:`-only edit of the item file, rows added to the item's `notebook:` page (every base line kept, in order, so a row may land inside the table before trailing prose), a new file under `docs/harness/mistakes/`, or rows added to `docs/start/FILEMAP.md`. An item with no `scope:` fails.
- `commit-order`: among the PR's non-merge commits (`git rev-list --reverse --topo-order <merge-base>..<head>`; merges are skipped), the first to add the ORDER (status `A`; for a repair chain, the lowest `n`) comes before any commit that changes a path, and changes nothing outside the ORDER's folder. Each later ORDER of a chain is added in a commit that touches only its own folder. A merge commit before the ORDER commit fails, and the ORDER's `Item:` and `Lab:` at the head must equal those in the commit that added it.
- `report`: `<active ORDER folder>/REPORT.md` exists at the head with `## ` headings starting (any case) "What did not work", "What changed", "Measurements", "Checks run at the head", "Skills used", "Still open", "Plain summary", in that order, each followed by a non-blank line before the next `## `.
- `owner-paths`: a changed path (either side of a rename) that matches an entry of `docs/harness/owner-paths.json` at the merge base (`<prefix>/**` covers everything below the prefix; any other entry is exact) needs a `## Owner approval` section in the ORDER that names each matched entry (as written, or its prefix), holds a double-quoted answer and an ISO UTC time (`2026-10-21T15:25:16Z`). With no owner path changed it passes.
- `rule-sources`: outside the files in `docs/harness/rule-sources.json` at the merge base (and `archive/`, `docs/loop/tasks/`, `docs/roadmap/items/`), the PR adds no markdown file (`.md`, `.markdown` or `.mdx`, any case) whose name contains `rule` (any case) and no markdown heading containing the word `rule` or `rules` (`(?i)(^|[^a-z0-9])rules?([^a-z0-9]|$)`, so `_Rules_` counts). A heading is an ATX line (`^ {0,3}#{1,6}` then a space, a tab or the line end) or a setext heading (a non-blank line over a line of only `=` or only `-`, up to 3 leading spaces); it is added when the base file lacks it. A file renamed or copied out of a listed or exempt file is compared with an empty base, so moved rule text counts as added.
- `rendered-lock`: `STATE.md`, `HANDOFF.md`, `docs/harness/LESSONS.md` and AGENTS.md between the `loop:lessons-top-ten` markers change only when the head ref is `kimi/render` (or `origin/kimi/render`); N/A while none exists. `--head-ref <branch>` names the head ref; it defaults to `--head` when that is a branch, else the current branch.
- `read-path`: AGENTS.md at most 80 lines; `STATE.md`, `HANDOFF.md` under 24,576 B each; the ORDER under 4,096 B; AGENTS + CHARTER + STATE + HANDOFF + ORDER fail above 76,800 B and name the hard limit above 128,000 B. Every overrun fails, whether or not it grew: the former warnings (over budget without growth, and a whole path above the 76,800 B line) became errors in loop PR 11. The whole path's growth compares AGENTS + CHARTER + STATE + HANDOFF alone, since the base never has the ORDER. Any of AGENTS.md, CHARTER.md, STATE.md, HANDOFF.md or the ORDER that is a symlink at the head (mode `120000`) fails.
- `filemap`: when `docs/start/FILEMAP.md` exists at the head, it has a row (first cell a backticked path) for every depth-1 and depth-2 directory (not dot-names, nothing under `archive/`), and every row names a directory at the head.
- `review-record` (`--pr <n>`): runs the review gate of [`loop review`](#loop-review-loop-pr-6) on the PR's comments (`gh api repos/{owner}/{repo}/issues/<n>/comments --paginate`); only `uor-review v1` records count, and free-text records no longer do. The gate's context comes from the merge base: the build-plan rows of `docs/loop/README.md` (missing fails the rule), the vendors of `docs/harness/labs.json`, and D59 from `docs/harness/owner-paths.json` and the switch. The ORDER lab, item and `Setup row:` come from the PR's ORDER at the head; the PR is doc-only when every changed path is `docs` or `generated`. It passes when the gate passes; the details are the gate's, naming every record and why it counted or not. The `[<lab>]` tag is self-declared: every lab posts from the same GitHub account (loop page, P-3/P-4), so the comment author cannot bind it. A `gh` error fails the rule.
- `required-checks` (`--pr <n>`): at the head sha, each of `fmt / clippy / tests / no_std / κ`, `cargo audit`, `fuzz smoke (nightly libfuzzer)`, `wasm-pack build (dashboard facade, deploy.yml parity)` and `Gate C trend alarm` has its newest check run (the highest id, from any app) from the `github-actions` app with a summary ending `policy: pass (run <digits>)` (written by `required-transport` in `.github/workflows/ci.yml`), and no commit status uses one of those names, since any token can post a status (loop page, P-4). Check runs and statuses are read in full (`gh api repos/{owner}/{repo}/commits/<sha>/check-runs?per_page=100 --paginate`, and `.../statuses?per_page=100 --paginate`). A `gh` error fails the rule.

Tests only: `UOR_PRCHECK_GH_FAKE=<file>` replaces `gh` for these two rules with `{"comments": [{"author", "body"}], "checkRuns": [{"name", "app", "summary"}], "statuses": [{"context", "state"}]}`, answered for every PR and sha; a check run without `id` takes its list position. While it is set, the text output prints `FAKE GitHub: UOR_PRCHECK_GH_FAKE=<path> (tests only)` before the rules and `--json` carries `"fake_github": true`. Never set it outside tests.

**In CI.** `ci.yml` calls `.github/workflows/gate.yml` on every pull request. Its `policy` job builds `uor-harness` from the base commit and runs `loop pr-check --ci` against the PR's head, which it reads as data. `required-transport` then writes `policy: <pass|fail|error> (run <id>)` into the five required checks' summaries and leaves them green. That record is what `required-checks` reads back. Its `tests` job runs `loop gates --full` on the PR's code with a read-only token. The [loop page](../loop/README.md#gates) gives the reasons.

### `loop review` (loop PR 6)

The pure core is built in `crates/uor-r4-harness/src/loopreview.rs`; `pr-check`'s `review-record` rule and `loop next` count records through its gate. A review record is a PR comment in this form:

```text
[<lab>] uor-review v1
pr: <number>
head: <40 lowercase hex>
lab: <lab>
model: <model id or "unknown">
launch: <launch id or "manual">
verdict: APPROVE | CHANGES
order: docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md
context: fresh            (optional)
done-when: host: <what was re-run and its result>   |   done-when: gpu: sha256:<64 hex>
```

Free text (the findings) may follow after a blank line. A comment that does not contain `uor-review v1` is not a record; one that contains it only below its first line is refused (a record must start the comment). One whose first line contains it but is wrong is refused and named in the gate's details: an untagged first line, a tag that differs from `lab:`, a missing, duplicated or unknown field, or a bad `pr`, `head`, `verdict`, `order`, `context` or `done-when`.

The gate counts a record only when its `pr` and `head` are the PR's current ones, its `order` is the PR's active ORDER (the highest of a repair chain; see `order` above), and its lab is registered. On a setup PR while D59 is in force, only `claude` records with `context: fresh` count (D59: Claude's fresh-context reviewers). Otherwise the record's lab must be of another vendor than the ORDER lab's (D46), or the PR is doc-only and the record comes from the ORDER lab itself with `context: fresh`. It passes when a record counts and no counted CHANGES stands: each lab's last counted record at the head must be an APPROVE. A counted CHANGES stands until the author pushes a new head (where no record counts yet) or the same lab posts a later APPROVE; another lab's APPROVE does not clear it (owner, 22 Oct 2026 17:51Z: "No: CHANGES blocks"). A setup PR is one whose ORDER `Setup row:` names a row of the [loop page](../loop/README.md#build-plan) build-plan table and whose item is `infra-loop-harness` or that row's own item (`6` → `infra-loop-pr06-…`). The row is the token right after `Setup row:` (`[0-9]+[a-z]?` or `A`, then `.`, a space or the end of the line); other text after the label names no row. An `infra-loop-…` item whose ORDER names no valid row, a row not in the table, or a row that is not its own fails the gate with no record counted, and no review is owed on it until the ORDER is fixed. D59 stays in force unless `docs/harness/owner-paths.json` at the merge base lists `docs/harness/d59.json` exactly and that file reads `"d59": "ended"`. The claude-18 ORDER proposes the file; the owner adds it and lists it.

```sh
UOR_LAB=claude uor-harness loop review --pr <n> --verdict APPROVE --host "<what you re-ran>: ok" --dry-run
UOR_LAB=claude uor-harness loop review --pr <n> --verdict CHANGES --gpu <64 hex> --body-file findings.md
UOR_LAB=claude uor-harness loop review --pr <n> --verdict APPROVE --host "<check>" --context fresh
UOR_LAB=claude uor-harness loop review --check --pr <n>
```

`loop review` needs a lab (`UOR_LAB` or `--lab`; none exits 2). It reads the PR's head and changed files and its active ORDER (as `loop next` resolves it: the `docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md` files readable at the head and absent from `origin/main`, one, or a repair chain of one lab in one folder whose highest `n` is active; none, or ORDERs of two labs or two folders, exit 2). After `git fetch origin main` it reads the build-plan rows, the lab registry, `owner-paths.json` and the D59 switch from `origin/main`. A PR is doc-only when every changed file's size class is `docs` or `generated`.

- With `--verdict`, it writes one record at the PR's head: `model` from `--model`, else `UOR_LOOP_MODEL`, else `unknown`; `launch` from `--launch`, else `UOR_LOOP_LAUNCH`, else `manual`; `done-when` from exactly one of `--host` and `--gpu`. It refuses (exit 1, nothing posted) a record the gate would not count, and prints why: a non-claude record, or a claude record without `--context fresh`, on a setup PR under D59, a loop item whose ORDER names no valid setup row, a same-vendor record on a code PR, `context: fresh` on a code PR. `--body-file` text follows the record after a blank line. `--dry-run` prints the comment; otherwise it is posted with `gh pr comment` and the command prints `review: posted <verdict> on PR #<n> at <head>`.
- With `--check`, it runs the gate over the PR's comments and prints each detail line: exit 0 when it passes, 1 when it does not.

Exit 2 also covers a usage error and a git or `gh` failure, a failed post included.

### `loop report` (loop PR 7b)

```sh
UOR_LAB=claude uor-harness loop report --check      # check <task folder>/REPORT.md in the working tree
UOR_LAB=claude uor-harness loop report --new        # write REPORT.md from the template; never overwrites
UOR_LAB=claude uor-harness loop report --receipts   # copy HEAD's gate results into <task folder>/receipts.jsonl
UOR_LAB=claude uor-harness loop report --check --order docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md
```

The task folder is that of the active ORDER among the `docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md` files this branch adds against `git merge-base origin/main HEAD` (added, or the new side of a rename): one, or the highest `n` of a repair chain of one lab in one folder; none, or ORDERs of two labs or two folders, exit 2 and name them, and `--order <path>` names it instead. Exit codes: 0 ok; 1 a check failed; 2 cannot run (usage, no ORDER found, git failure, an existing REPORT for `--new`).

- `--check` reads `REPORT.md` from the working tree, so uncommitted text counts. It prints `PASS report` or `FAIL report` with one indented line per problem: the problems `loop pr-check`'s `report` rule finds (the same function), each `<!-- fill:` template line still present (naming its section), or a missing file.
- `--new` copies `docs/loop/templates/REPORT.md` into the task folder without its first line, filling in `{id}` (the ORDER's `Item:`), `{task}` (`<lab>-<n>`) and `{lab}`. It opens the file with `create_new`, so an existing REPORT is never touched. The fresh REPORT fails `--check` until every fill line is replaced.
- `--receipts` needs a lab (`UOR_LAB` or `--lab`; none exits 2). From `local/<lab>/gates.jsonl` it keeps the lines whose `commit` is the full `HEAD` sha and whose `tree` is the hash of a clean working tree at `HEAD` (a row recorded on a dirty tree is not a receipt for `HEAD`), takes the latest per `gate`, and writes them sorted by gate name, each byte-for-byte as recorded, to `<task folder>/receipts.jsonl` through a temporary file and a rename. No such result exits 1: run `loop gates` on a clean tree first.

Templates in `docs/loop/templates/`: `REPORT.md` (the seven sections, one fill line each), `ORDER.md` (the `Lab:`, `Item:`, `Setup row:` and `Base:` lines, then Goal, Cancel probe, Files in scope, Done when, Stop and ask if) and `PICKUP.md` (branch, head, task folder, open items and next step for the next session).

### `loop arrive` (loop PR 7b)

```sh
UOR_LAB=claude uor-harness loop arrive          # loop step 2: one line per check, then arrive: OK|FAIL
UOR_LAB=claude uor-harness --json loop arrive   # {"lab","checks":[{"check","verdict","summary","details"}],"ok"}
```

Needs a lab (`UOR_LAB` or `--lab`). Exit codes: 0 every check PASS or N/A; 1 a check FAILs; 2 cannot run (usage, no lab, not a git repository). **The checks that used to WARN FAIL since loop PR 11** — the loop relies on them, so a session may not start on a stale or unguarded checkout. The checks, in order:

- `fetch`: `git fetch origin main`; a failure FAILs with git's message, and the later checks read the `origin/main` already present.
- `checkout`: HEAD against `origin/main`; behind or unplaceable FAILs.
- `binary`: the running binary against the newest build in this checkout, by age (no commit is baked into the binary; `build.rs` says why); older FAILs with the install command.
- `hooks`: the git `pre-commit` hook must be the exact text `uor-harness git install-hooks` writes and executable; otherwise FAILs (a matching hook without an execute bit FAILs "not executable"). `client-hooks` is N/A until loop PR 9.
- `views`: renders STATE, HANDOFF and LESSONS from `origin/main` into `local/<lab>/views/`; a view committed at `origin/main` whose text below the stamp line differs from its render FAILs, because the repository's inputs win (run `loop render`).
- `pickup`: `local/<lab>/PICKUP.md` under the main checkout, where `loop close --session` writes it (so a linked worktree reads the same file); absent is N/A, an unlocatable main checkout FAILs; 8,192 bytes or more FAILs, and so does each non-blank line under `## Standing references` that cites no `§<n>`, `D<n>` or `L-<nnn>` (fenced code blocks and `#` subheading lines are skipped; only a `## ` heading outside a fence ends the section).
- `read-path`: AGENTS.md, CHARTER.md, STATE.md and HANDOFF.md at `origin/main` (a view not committed there is measured as its render in `local/<lab>/views/`) plus the PICKUP the `pickup` check read; AGENTS.md over 80 lines, STATE.md or HANDOFF.md at 24,576 bytes or more, a total above 76,800 bytes, or a total above 128,000 bytes all FAIL.

### `loop next` (loop PR 7b)

```sh
UOR_LAB=claude uor-harness loop next          # loop step 1: one line per source, then GO <kind> <target> — <why> | STOP <reason>
UOR_LAB=claude uor-harness --json loop next   # {"lab","sources":[{"source","status","detail"}],"decision":{"go","kind","target","reason"}}
```

Needs a lab (`UOR_LAB` or `--lab`). Exit codes: 0 GO, 1 STOP, 2 cannot run (usage, no lab, a git or `gh` error). It reads GitHub only; the `any` source runs `loop claim`'s checks, which fetch `origin main`. The sources, in order; the first that yields work decides:

- `kill-switch`: STOP when `local/loop/STOP` exists in the main checkout or issue #2028 carries the label `loop:pause`.
- `owner-direction`: reads PLAN.md's `## Owner directions` table and offers the first row **whose Status column header is found by name** and reads `open` (a note cell saying "open" is not a status); with no PLAN.md or no table it names what is missing instead of guessing.
- `repairs`: the lab's own open PRs (head `<lab>/...`) that conflict, have a check at the head with conclusion FAILURE or TIMED_OUT, or on which a counted `uor-review v1` CHANGES stands at the head (the review gate's rule: no later APPROVE from the same lab); GO `repair #<n>`, lowest number first.
- `reviews`: open non-draft PRs on which the review gate would count a `uor-review v1` record from this lab (taken to carry `context: fresh`: on a setup PR under D59 `claude` only; else a lab of another vendor than the ORDER lab, D46, or the ORDER lab itself on a doc-only PR) and that have no counted record at the head from any lab (one counted APPROVE passes the gate, and one counted CHANGES waits on the author's new head); GO `review #<n>`, lowest number first. The gate's context (build-plan rows, vendors, D59) comes from `origin/main`, as in `loop review`. A PR without an ORDER is skipped.
- `gpu-collections`: not built — a GPU run needs a collection record the watcher (loop PR 10) does not write yet; that record is a separate unit.
- `premise-probes`: reads `PLAN.md`'s `## Premises` table, finds the Premise/Probe/Status columns by header name, and offers the first open row whose Probe cell carries the command in backticks; malformed open rows are skipped and named, and with no `PLAN.md`/table it says so instead of guessing.
- `lane`: reads the lab's optional `lane` in `docs/harness/labs.json` (owner-only) and the item `lane:`/`rank:` keys (the PM's ranking), and offers the lowest-`rank` claimable item in that lane; with either absent it names what is missing.
- `any`: the first `status: ready` item at `origin/main` with a `scope:` and every `depends_on:` item `status: done`, in id order, that `loop claim` would accept; GO `item <id>`. An item waits while a dependency is missing, unreadable or not done, or while its `depends_on:` is not an inline `[a, b]` list. A waiting item is never offered, and when nothing is claimable the `any` line names it and why: for example `x waits on y (decision)`, `x waits on y (unreadable: …)`, or `x waits: depends_on … is not a [a, b] list`. `loop claim` and `loop pr-check` do not check dependencies, and a malformed `depends_on:` does not stop them.

Then STOP `held work` when some item is `status: held`, else STOP `empty queue`.

### `loop close` (loop PR 7b)

```sh
UOR_LAB=claude uor-harness loop close --session              # loop step 9: PR body from REPORT, mark ready, write PICKUP
UOR_LAB=claude uor-harness loop close --session --pr <n>     # name the PR instead of finding it by branch
UOR_LAB=claude uor-harness loop close --session --dry-run    # print the body and the PICKUP; write nothing
UOR_LAB=claude uor-harness loop close --verify --pr <n>      # loop step 13: prove the merge, then delete this lab's worktree and branches
UOR_LAB=claude uor-harness loop close --verify --pr <n> --no-gates   # the same without the done-when gates
```

Needs a lab (`UOR_LAB` or `--lab`). Give exactly one of `--session` or `--verify`. `--session` exit codes: 0 done; 1 refused (a check failed, nothing written); 2 cannot run (usage, no lab, no ORDER, no PR or several, a git or `gh` failure). `--session`, in order:

- The task folder is `loop report`'s: the one ORDER this branch adds. Its `REPORT.md` must pass `loop report --check` (the same function), else exit 1 naming the problems.
- The PR is `--pr <n>` (exit 1 unless `gh` reports it `OPEN` with the current branch as its head branch), else the one open PR whose head branch is the current branch. Its head must equal local `HEAD`, else exit 1 `push first: the PR head is <x>, HEAD is <y>`.
- The body: the `size: …` line `loop pr-check` prints (the same function) for `git merge-base origin/main HEAD` to `HEAD`, a blank line, the REPORT without its `# ` title line, then `References #<issue>` when `docs/roadmap/issues.json` maps the ORDER's `Item:` to an issue, then the Claude Code footer when the lab is `claude`.
- The PICKUP is `docs/loop/templates/PICKUP.md` filled with the branch, the head sha, the task folder, the REPORT's "Still open" section verbatim under `## Open items` and "Await the exact-head review of PR #<n>" under `## Next step`; `## Standing references` is left empty. 8,192 bytes or more exits 1 before any write.
- Writes, in order: the PR body (`gh pr edit <n> --body-file`), `gh pr ready <n>` only for a draft, then `local/<lab>/PICKUP.md` under the main checkout (the parent of `git rev-parse --git-common-dir`; overwritten), never inside the task worktree, which `--verify` removes. A failed body write exits 2 with nothing else written; a failed `ready` exits 2 saying the body was already set, and writes no PICKUP.

`--verify --pr <n> [--no-gates]` (loop step 13) prints one `PASS|FAIL|SKIP|N/A <step> <detail>` line per step. Exit codes: 0 verified and cleaned up; 1 refused or not verified, or something kept; 2 cannot run. Every proof must pass before anything is deleted, and nothing is deleted when a proof fails:

- `state`: `gh pr view <n> --json state,mergeCommit,headRefName,headRefOid` reports `MERGED`.
- `squash`: after `git fetch origin main`, the merge commit is an ancestor of `origin/main` and its subject ends with `(#<n>)`.
- `content`: after `git fetch origin refs/pull/<n>/head`, for the files the PR changed (`git diff --name-only --no-renames` from the merge base of the head and the merge commit's first parent), either each is identical between the merge commit and the PR head, or the PR's patch (merge base to head) and the merge's patch (first parent to merge commit) over those files have the same `git patch-id --stable` (main may have changed another part of a shared file before the squash).
- `done-when` (`SKIP` with `--no-gates`): `loop gates --full` in a detached worktree of `origin/main` at `<main checkout>/local/<lab>/verify-<n>`, removed with `git worktree remove --force` afterwards whatever the result. Red is a FAIL: a failure becomes a repair.

Then, only for this lab's own branch (`<lab>/...`; another lab's PR prints `SKIP cleanup` and exits 0), three independent steps, each with its own guard, every one attempted (a kept worktree does not stop the remote delete, which is still leased to the PR head):

- `worktree`: the worktree on the branch, under `.worktrees/<lab>-*`, removed with `git worktree remove` (never `--force`) only when clean. A change to `docs/harness/claims.json` alone is reverted first; any other change keeps it. Because removal deletes ignored files too, any ignored file other than `target/`, `local/<lab>/views/` and `local/<lab>/gates.jsonl` (from `git ls-files --others --ignored --exclude-standard --directory`) keeps it with `FAIL worktree holds ignored files: <first 10 paths>`. None is `N/A`.
- `local-branch`: kept while a worktree has it checked out; otherwise one atomic `git update-ref -d refs/heads/<b> <head>`, which fails, keeping it, when the branch has moved off the PR head.
- `remote-branch`: `git push origin --force-with-lease=refs/heads/<b>:<head> :refs/heads/<b>`, so a branch that moved is kept; already gone is `N/A`.
- `hygiene`: prints the dry-run plan of `uor-hygiene` (this binary's `hygiene` action, run as a subprocess in the main checkout) and never passes `--apply`, because `--apply` removes every clean merged worktree, other labs' included, which the lab boundary forbids (a decision against the loop page's literal step 13).

Tests replace `gh` with `UOR_LOOP_CLOSE_GH_FAKE=<json>`, and every run with it set prints `FAKE GitHub: UOR_LOOP_CLOSE_GH_FAKE=<path> (tests only)` first (`prs`, each with an optional `state` defaulting to `OPEN`, optional `failSetBody`/`failReady`, `merged` keyed by PR number for `--verify`); never set it outside tests. Tests also set `UOR_LOOP_CLOSE_NO_HYGIENE=1`, which prints `SKIP hygiene` instead of running hygiene.

### `loop queue` (loop PR 7c)

```sh
uor-harness loop queue --pr <n> --dry-run   # loop step 12: every check, and the merge it would run; writes nothing
uor-harness loop queue --pr <n>             # the same, then `gh pr merge <n> --match-head-commit <head>`
uor-harness --json loop queue --pr <n>      # {"pr","head","dry_run","verdict","fake_github","checks":[{"check","verdict","summary","details"}]}
```

The policy gate is advisory on GitHub (D54), so this command is the lock: it queues a PR only when every check below passes. It prints one `PASS|FAIL|N/A <check> <summary>` line per check, and exits 0 queued (or, with `--dry-run`, would queue), 1 refused, 2 cannot run (usage, a `gh` or `git` failure). A failed merge prints `FAIL merge` with `gh`'s error and `verdict: MERGE-FAILED` (JSON `"merge-failed"`), and exits 2. In order:

- `pr-state`: `gh pr view` reports the PR `OPEN`, not a draft, and based on `main`. A missing `baseRefName` cannot be read, so the run exits 2.
- `head`: after `git fetch origin main` and `git fetch origin pull/<n>/head`, the fetched commit equals the head GitHub reports; otherwise "the PR moved; re-run".
- Every `loop pr-check` rule, through the same function (`loopprcheck::evaluate`), in `--pr` mode against `origin/main` at that head, with the PR's branch as the head ref. Any FAIL refuses. This covers the review gate (`review-record`: the last counted `uor-review v1` record at the head is an APPROVE, so a later counted CHANGES refuses, while a CHANGES that does not count, such as another lab's on a setup PR under D59, does not); the policy job's record on every required check, with no commit status using a required name (`required-checks`); and an owner approval in the ORDER for `.github/**` and every other owner path (`owner-paths`).
- `after-approval`: the comments are read again after the rules ran, so a record posted meanwhile is seen. The last counted record must be an APPROVE, and every comment after it is judged: a well-formed `uor-review v1` record is left to the gate, a `[bot]` author is ignored, and a body starting with the tag of a registered lab whose role is not `pm` is ignored. Any other comment refuses, because every lab posts from one GitHub account (D55), so an untagged comment may be the owner's, and a `[kimi]` one is the PM's. A malformed `uor-review v1` record also refuses, because it may be a hold, for example "the APPROVE above missed X". The labs and their roles are read from `origin/main`, never from the PR. Only the PR's issue comments are read: a hold left as a review body or as an inline review comment is not seen.
- `merge`: the only GitHub write. It is skipped by `--dry-run`.

Tests replace `gh` with `UOR_LOOP_QUEUE_GH_FAKE=<json>` (`pr` with `headRefOid`, `headRefName`, `baseRefName`, `state`, `isDraft`, every one required; `comments` with `author` and `body`; `checkRuns`; `statuses`; `failMerge`); merge calls are logged to `<file>.calls.jsonl`, and every run with it set prints `FAKE GitHub: …` first. Never set it outside tests.

### `loop board` (loop PR 7c)

```sh
uor-harness loop board          # every registered lab: vendor, client, role, presence, claims held, open PRs
uor-harness --json loop board   # the same as JSON, one entry per lab, plus the untagged PRs
uor-harness loop board --bench local/bench [--every 60]   # write the bench's STATE, HANDOFF, PLAN, DECISIONS and .gates.jsonl
```

Read-only. For each lab in `docs/harness/labs.json`, sorted by name, it shows vendor, client and role (`-` when none); presence from the live tier, falling back to the committed copy, with the tier named (`no check-in` when the lab has none); the number of claims held by the lab, live and committed (`multilab::claims` merges `docs/harness/claims.json` with the live tier, so stale committed claims count too); and its open PRs, those whose title starts with `[<lab>]` exactly, each with number, draft or ready and a 7-character head, from `gh pr list --state open --limit 200 --json number,title,headRefOid,isDraft`. Open PRs with no registered lab tag are listed under `untagged`. It makes no GitHub write: its GitHub trait has no write method. No hook canary yet: that is loop PR 9. Exit 0 rendered, 2 cannot run. Tests replace `gh` with `UOR_LOOP_BOARD_GH_FAKE=<json>`, holding the array `gh pr list --json` prints.

`--bench DIR [--every N]` (`loopbench.rs`, claude-25) writes five files into DIR in the CorvusWing bench's shape (`.gates.jsonl` and the decisions, plan, handoff and state pages), each as a temp file then a rename. Every repo file is read from `origin/main` after `git fetch -q origin main`: roadmap items become Tasks rows whose Status word sets the lane (`done`, `in progress: <lab>-<n>, PR #N, <review>` for an item an open lab PR's ORDER names, `held`, `decision pending (owner)`, `ready, waits on <id>`, `ready`); each open lab PR that adds an ORDER becomes a Next row (CHANGES first, then needs review, APPROVE, draft) with the ORDER's first Done when and Stop and ask if bullets; `docs/integration/DECISIONS.md` rows map column by column; committed `receipts.jsonl` and each `local/<lab>/gates.jsonl` become one gate line per run. DIR must sit outside the repository or under `local/` (else exit 2); `INBOX.md` is never written. Without `--every` it writes once (exit 0, 1 on a failed refresh); `--every N` (N >= 30) loops and keeps going after an error. GitHub reads only; tests replace `gh` with `UOR_LOOP_BENCH_GH_FAKE=<json>`.

### `loop guard`, `loop install`, `loop onboard` and `loop copyright` (loop PR 9)

```sh
uor-harness loop guard --client <C> --lab <L> --event <E> [--input <path>]
uor-harness loop install --all [--home <dir>] [--zip <path>] [--replace] [--apply] [--dry-run]
uor-harness loop onboard --client <C> [--home <dir>] [--write-receipt]
uor-harness loop onboard --check-receipts
uor-harness loop copyright [--zip <path>]
```

- `loop guard` is the single hook entry point for `antigravity`, `claude`, `codex`, `deepseek` and `kimi`. It checks whether the payload's directory (`cwd` or `workspacePaths`) and the process working directory lie inside the UOR-R4 repository or its worktrees **first**, exiting `0` at once outside the repository even if `--client` is unknown or stdin JSON is malformed. Inside the repository it locks `local/<lab>/guard/.lock` and tracks subagent bindings (`agent_id` bound to `local/<lab>/<topic>/orders/<n>.md` at `SubagentStart` in Claude Code; root vs child `conversationId` in Antigravity, failing closed when `SessionStart` has not run and replacing stale root ids on a fresh root `SessionStart`; `SubagentStart`/`SubagentStop` depth per `session_id` in Kimi Code). On `PreToolUse` it checks file writes and shell file-modifying commands (`>`, `>>`, `tee`, `cp`, `mv`, `rm`, `install`, `ln`, `touch`, `truncate`, `chmod`, `chown`, `sed -i`, `perl -i`, `ruby -i`, `dd of=...`, `git rm/mv/restore/checkout`) against owner-only paths (`docs/harness/owner-paths.json`), the shared primary checkout (R10), cross-lab worktrees and `local/` folders, rendered views, `archive/`, active ORDER scope (from the worktree's claimed `<lab>/<topic>` branch or `local/<lab>/guard/active-order` only, never a closed historical task) and bound subagent suborders. It tokenizes shell commands (stripping path prefixes from `argv[0]` and skipping `git`/`gh` global options) to refuse owner release and cap-override commands (D56), `loop install --apply`, `git install-hooks`, hook bypasses (`--no-verify`, `git commit -n`/`-nm`, `-c core.hooksPath=...`), `git stash`, `git push --force`/`-f`/`+refspec` (`--force-with-lease` is allowed), pushes to `main` or `refs/heads/main` (including refspecs such as `feat:main` and `git push -u origin main`), `gh pr merge` without `--match-head-commit <40-hex>`, `gh api .../merge`, `pkill`/`killall`, credential reads (`~/.netrc`, `~/.aws/credentials`, `~/.runpod/config.toml`, `.env*`, `auth.json`, `credentials.json`, `id_rsa`, `id_ed25519` via any command) and environment dumps (`printenv`, bare `env`, `export -p`). Allowed calls exit `0` (printing `{"decision":"allow"}` for Antigravity `PreToolUse`); refused calls exit `2` with `REFUSED: <reason>` on stderr.
- `loop install` is **plan-only by default**: without `--apply` it prints the full list of files it would write and writes nothing. Writing requires an explicit `--apply` (which `loop guard` refuses from any lab session, and which `loop install` itself refuses when targeting `$HOME` while `UOR_LAB` is set, so only Casey runs it from Terminal). With `--apply`, it installs the six allowed Loop Kit skills (`gates`, `holding-the-goal`, `scope-guard`, `session-close`, `solution-integrity`, `working-discipline`; skipping `start-here` and quoting unquoted YAML colons in `description:`) and each client's user-level hook and pointer files:
  - `claude`: `~/.claude/settings.json`, `~/.claude/CLAUDE.md`, `~/.claude/skills/<6 skills>/SKILL.md`
  - `codex`: `~/.codex/hooks.json`, `~/.codex/rules/uor-r4.rules`, `~/.codex/skills/<6 skills>/SKILL.md`
  - `kimi`: `~/.kimi-code/config.toml` (retiring the second loop hook), `~/.kimi-code/UOR-R4-START.md` (`<= 1` KB), `~/.kimi-code/UOR-R4-HANDOFF.md` (`<= 1` KB), `~/.agents/skills/<6 skills>/SKILL.md`
  - `antigravity`: `~/.gemini/config/plugins/uor-r4-loop/plugin.json`, `~/.gemini/config/plugins/uor-r4-loop/hooks.json`, `~/.gemini/config/skills/<6 skills>/SKILL.md`
  - `deepseek`: `~/.dsh/hooks.json`, `~/.dsh/profiles/headless/cordis.patch.yml`, `~/.dsh/skills/<6 skills>/SKILL.md`, and `local/deepseek/headless-overlay.yml`
  Inside the repository it writes only under gitignored `local/` (`local/deepseek/headless-overlay.yml` and `local/loop/install-backups/`). Differing existing files are refused unless `--replace` is given, which backs them up to `local/loop/install-backups/` first.
- `loop onboard --client <C>` prints `PASS`, `FAIL`, `REFUSED` or `N/A` for `rules`, `skills`, `hooks`, `canary` (evaluated in-process on `loop guard`), `identity` and `binary`, followed by `lane:` and `pickup:`. `--write-receipt` writes `docs/harness/onboarding/<client>.json`, and `--check-receipts` verifies all five committed receipts and re-runs each client's in-process canary write refusal check.
- `loop copyright` compares tracked repository markdown and text files against the owner's private Loop Kit archive (`--zip <path>` or `~/Downloads/LoopKit*.zip`) using 8-word windows in memory. It runs on the laptop only and never stores kit text or hashes in the repository.

## Where it sits against what already exists

The repo did not lack for machinery; it lacked a front door. This harness is the front
door, not a fourth store. Use what is already here, through it:

| Already built | What it is | How the harness uses it |
|---|---|---|
| `~/.local/bin/uor-knowledge search` | SQLite FTS5 over 8,492 project items with provenance | the place to look up a term's history; **the index is a snapshot, so refresh before acting** (`scripts/sync_project_knowledge.py --open-issues --ingest`) |
| `docs/research/retired-mechanisms-catalogue-2026-10-01.md` | 1,350 lines of retired mechanisms with `CLEAN / TOO TIGHT / CONFOUNDED / UNDER-POWERED / POLICY-KILLED / INSTRUMENT-DEFECT` verdicts | the deepest negative-results record in the repo; a `mistakes` entry cites it as evidence rather than restating it |
| `archive/history/branch-archive/INDEX.md` | 193 rows of preserved branch patches with restore bases | where retained-but-inactive source lives; the harness does not duplicate it |
| `docs/mechanisms/*.md` | one brief per novel mechanism incl. "what has been tried, honestly" | loaded into every brief for that milestone |
| `scripts/research-lab-context.sh` | read-only session-start receipt (HEAD, SHA-256 of authority files, disk) | run alongside `status`; it is the machine receipt, `status` is the project view |
| `~/.local/share/uor-r4/ledger/` + `compute/ledger.jsonl` | 357 closeout receipts and the append-only compute ledger | the authoritative cumulative cost; `budget show` reads it rather than inventing a total |
| `scripts/pod/uor-pod` | the only sanctioned GPU path | `budget scan` reads its status; it is never wrapped or bypassed |

**The regression arena** ([ARENA.md](ARENA.md)) is the other half of the panels. A panel says
what a number has to reach; the arena records what each evaluated configuration actually scored,
against which parent, at what scope — and reports the comparison **row by row**, never as an
average. It holds no transformer or dense row by design (D23/D25, and the owner's 3 October
direction), so it cannot answer "is this better than a transformer" and does not try. Its job is
to catch a capability that was already won being silently lost.

Every bar declares its direction. That is not fastidiousness: the first version compared raw
numbers, and reported the starling readout's move from 36 to 69 candidates per query as a *gain*
— a 92% cost increase, congratulated. An instrument that praises a regression will be believed,
so direction is data in [`arena.json`](arena.json) and an undeclared bar is reported as neither.

**Known gap, stated rather than hidden:** there is no topic index over failures. The
catalogue and the archive INDEX are reachable only by already knowing they exist, and
`current-state.md` is chronological. `docs/harness/mistakes/` is the topic index, and
it is deliberately small: one entry per lesson that has a guard or a named trigger.

## Getting started (any lab)

The harness is one Rust binary with no runtime dependencies. Build it once from a checkout of this
repository and put it on `PATH`:

```sh
CARGO_TARGET_DIR=local/harness-target ~/.cargo/bin/cargo build --release -p uor-r4-harness
export PATH="$PWD/local/harness-target/release:$PATH"
export UOR_LAB=<your lab>          # kimi | claude | codex | deepseek | antigravity
uor-harness onboard
uor-harness status
```

Two details are worth knowing because they are the only things that ever go wrong:

**Use rustup's `cargo`.** On the owner's laptop `/opt/homebrew/bin/cargo` is a Node CLI that
shadows the Rust toolchain whenever Homebrew precedes `~/.cargo/bin` on `PATH`; `uor-harness
onboard` reports which `cargo` it found.

**A stale binary rejects new flags.** `unknown option <flag>` usually means the binary predates the
source; `uor-harness version` says which commit it was built from and whether `origin/main` is
ahead. `--repo DIR` works before or after the subcommand; from a directory that is not a checkout,
the tool fails loudly rather than guessing.

**Joining for the first time?** [BOOTSTRAP.md](BOOTSTRAP.md) is a ten-minute, no-compute check that proves the tool runs on your runtime and that your claims and presence are visible to the other labs.

## Working in a multi-lab project

Three labs work this repository, plus this one. Each runs its own copy of the CLI, and **nothing here is a
socket or a daemon** — GitHub is the bus and `main` is the truth. What the harness adds is the
part GitHub alone does not give: a committed record of who holds what, and what each lab is
doing between merges.

| question | command | where it reads from |
|---|---|---|
| what is each lab doing? | `multilab presence` | the **live** tier first, committed `docs/harness/presence/<lab>.json` as the fallback |
| who holds this branch? | `multilab claims` | the live mirror, then `origin/main`'s committed `docs/harness/claims.json`, then the caller's own checkout |
| what actually ran? | `status` (section 2) | `docs/harness/ledger/<lab>-<month>.jsonl`, **committed shards only** — a receipt written to the live tier appears here once `ledger-sync` has carried it in |

### Two tiers: live and durable

All these labs run on one machine, so cross-lab state has two homes with different jobs:

| tier | path | purpose | in git? |
|---|---|---|---|
| **live** | `~/.local/share/uor-r4/harness/` (override with `$UOR_HARNESS_LIVE`) | what peers see *now* — a check-in is visible in the second it is written | no |
| **durable** | `docs/harness/presence/<lab>.json`, `docs/harness/claims.json` | the record — survives restarts, reaches `git`, visible to any future reader | yes |

**A heartbeat writes the LIVE tier, from one record with its receipt**, so the two cannot disagree
about what a lab is doing. **It does not write `docs/harness/presence/`**: a check-in runs every few
minutes, and dirtying every lab's checkout on that cadence is exactly what R10 forbids. **The
committed record is refreshed deliberately, by `multilab sync-live`**, which copies each live row —
**timestamp, intent and session included**, not restamped — into the committed tier. So a committed
presence file is as old as the last person who chose to publish it, and **that is the design, not a
stall**: this section used to say every heartbeat wrote both, and it was wrong. The reader prefers
**live** because it is fresher by construction, falls back to **durable** when the shared directory
is absent, and always reports which one answered:

```
$ uor-harness multilab presence
2 lab(s) known (stale after 5m) · live tier: /Users/casey.allard/.local/share/uor-r4/harness
lab          state                via      milestone branch       doing
kimi         live (0m ago)        live     m1        main         evaluation integrity 1/3
```

`multilab sync-live` copies the live tier into the committed record — deliberately explicit
rather than automatic, because the live tier is written on every check-in (cheap, uncommitted)
while the record is refreshed when a lab wants the record to show it.

**A reader with no access to this laptop's live tier sees the committed record**, which is what
makes `sync-live` worth running before a handover, a review, or a remote session: without it, a
GitHub reader sees labs that stopped publishing at the last merge.

### Where a write lands (R10)

The owner checkout (`~/uor-r4`) stays on clean `origin/main`; no lab edits, builds or runs mutating
tools in it — that is R10. **The harness enforces the rule on itself**, because it used to break it:
every write resolved its target from the working directory, so a lab whose shell sat in the owner
checkout dirtied it with a claim, a receipt or a presence row, and another lab had to hand-move the
entries out before the checkout could be reset.

| command | writes | where |
|---|---|---|
| `heartbeat` | presence + one receipt | **live tier only** |
| `receipt`, `skill use`, `prep fit`, `team dispatch` | one receipt | **live tier only** |
| `ledger-sync` | the committed shard | the caller's own checkout |
| `team claim` / `team release` | `docs/harness/claims.json` | the caller's own checkout |
| `multilab sync-live` | the committed `presence/<lab>.json` | the caller's own checkout |
| `prep new`, `mistakes capture` | a record file | the caller's own checkout |

So a coordination write never needs a checkout: the check-in, the receipts and the live claims mirror
land in `~/.local/share/uor-r4/harness/`, and `ledger-sync` carries the receipts into the committed
shard from the lab's own worktree before the PR.

**`status` reads the committed shards, and only those.** That is the record, and it is why
`ledger-sync` exists: a receipt is written to the live tier and reaches the `status` roll-up once it has
been carried into the committed shard and merged. Readers of the live tier (`multilab presence`, the
unread-note report, `multilab claims`' mirror) see it immediately.

**A committed write refuses the shared checkout.** When the resolved repository is the primary
worktree of a repository that has linked worktrees — the shared checkout — the write stops with the
rule and the fix:

```
uor-harness: refusing to write the claims store into the shared checkout /Users/casey.allard/uor-r4
(R10): the owner checkout stays on clean origin/main, and no lab edits or runs mutating tools in it.
Run this from your own worktree (.worktrees/<lab>-<task>) or pass --repo <that worktree>; a
coordination write that needs no checkout at all goes to the live tier.
```

The identification is the *shape* of the clone, not a hard-coded path: a single-checkout clone, a
scratch directory and an unreadable git state are never refused, because refusing there would break
a lab that simply cloned the repository. Reads are never refused — reading claims, presence and the
ledger from the shared checkout is exactly what R10 permits.

**Presence is published by `heartbeat`**, deliberately: the session contract already requires a
card every 90 minutes, so a lab that follows the contract becomes visible for free and one that
never checks in is *visibly absent* rather than silently unknown. A file older than five minutes
reads `STALE`, and STALE is not "idle" — it means the lab may be working without checking in,
which is what the contract forbids.

**Identity.** Set `UOR_LAB` once per session, or pass `--lab`:

```
export UOR_LAB=kimi        # or: uor-harness --lab kimi ...
```

Without it, receipts land in `ledger/unknown-<month>.jsonl` and are attributed to nobody. They
are never dropped, but they cannot answer "which lab did this", which is the whole point.

**The ledger is sharded per lab** so two labs never write the same file. One shared ledger
conflicts on every concurrent PR; that was resolved by hand twice in a single session, which is
why it is now one file per lab per month. `multilab migrate-ledger` moves any older unsharded
receipts across without losing a line.

**Why claims moved into the committed tree.** They used to live in a gitignored
`.harness/claims.json`, which is right for one checkout and useless for four: a claim nobody else
can read cannot prevent the collision it exists to prevent. **The shared tier is read through git**
— `git show origin/main:docs/harness/claims.json` — and not from the shared checkout's working copy:
that checkout is frozen by R10 and falls behind, and reading it reported a released claim as held
long after main had dropped it. The committed store still cannot
*lock* anything — a filesystem cannot stop another vendor's CLI — so what enforces it is
`status.branch_claimed` on every `check`, the pre-commit hook, and the one-checkout policy in
[docs/labs/working-in-one-checkout.md](../labs/working-in-one-checkout.md).

**A claim dies with its branch, and the post-merge step buries it.** `uor-harness git clean-merged`
proves which local branches are already in `origin/main` **by content** (a squash merge leaves no
ancestry link, so ancestry would call merged work unmerged), and `--apply` deletes them — **it now also
releases the claims on those branches**, in the caller's own committed store and in the live mirror,
because a mirror that is only ever added to resurrects released claims. An unmerged branch keeps its
claim, and an `Unknown` state keeps it too: "cannot prove it" is not "dead". Before this, a merged
branch's claim sat in `claims.json` until its owner noticed, and the next lab to want that branch name
collided with a claim on finished work — a conflict in the file, not in the work.

**Its reach is exactly "the branch still exists locally."** A claim whose branch was already deleted
cannot be proved merged — there is no tree to compare, and after a squash merge no ancestry either — so
`clean-merged` **reports** it (`claims whose branch is gone`) with the one command that clears it,
`uor-harness team release --branch <branch> --lab <lab>`, and never releases it on a guess. That residue
is also what `status.branch_claimed` flags on every `check`; this report says which branches they are
without going to the tracker.

**Three branches are never deleted (owner, 20 Oct 2026).** `main`, `gh-pages` (the live documentation
site and Studio) and `gh-pages-preview` are listed in `docs/harness/branches.json`. Every branch delete in
the harness (`clean-merged`, the merged-branch delete, the remote delete) refuses a protected name, and the
built-in three stay protected even if the file is damaged. `status.protected_branches_present` fails
`check` when either site branch, or `main`, is missing from `origin`. Every other branch is `<lab>/<topic>`,
one per open PR, deleted after its merge. `scripts/storage/uor-hygiene` skips protected branches too.

**Writes are atomic, and a release lands for everyone when the PR does.** The committed store and the
live mirror are both written by a temporary file plus a rename, so a peer reading while you publish sees
the old file or the new one and never a half-written one — every claims write is a read-modify-write over
a file several labs share. A release is immediate in the mirror *for the caller*; for everyone else it
takes effect when the delivery PR lands, because `origin/main` still names the branch until then. That is
why a release rides a PR rather than sitting in a mirror nobody reviews.

**What this cannot do, stated plainly:**

- It is not real-time. Presence is as fresh as the last check-in.
- It cannot see a lab that never runs the harness. Absence is not evidence of idleness.
- It cannot cross vendor CLIs. There is no shared runtime, no push channel, and each vendor's
  session state is private by design.
- A lab that ignores a claim can still work the branch. The harness reports; it does not police.

## Rule engine

`crates/uor-r4-harness` holds the rules; each rule has an id, a severity and a message that names
the exact fix. `--staged` mode is used by the pre-commit hook. `uor-harness status --refresh` and the
`status.*` rules read GitHub through `gh` (cached for 15 minutes in the live tier); without `gh` they
say what they could not check rather than reporting a pass.
A rule added for a mistake cites that mistake's id, so a lesson becomes a guard rather than a note.
`prlint.d23_interlock` runs both `uor-harness ratchet d23` (`softmax`, `softplus`, `sigmoid`, `tanh`,
`gelu`, `silu`, `rope`, `ropei`, `rotary`) and `uor-harness ratchet d25` (`.matmul(`, `MatMul`, `cublas`,
`gemm`, `wmma`, `mma_sync`) in process across `crates/` and `docs/labs/*/instrument/`.

## Before you install the hooks

`uor-harness git install-hooks` manages three hooks: `pre-commit` (runs `uor-harness check --staged`),
`commit-msg` (calls `uor-harness git check-message <file>`), and `pre-push` (calls
`uor-harness git push-check`). It resolves the hooks directory with `git rev-parse --git-path hooks`,
which in a linked worktree points at the **shared** `.git/hooks` of the whole repository — including
the owner's checkout and every other lab's worktree. Installing there changes commit behaviour for
every worktree, so the command is **owner-only**: it refuses when `UOR_LAB` is set, and loop PR 9's
guard repeats the refusal at the client.

By default it **prints the plan and writes nothing**; `--install` writes. Every refusal is checked
before anything is written, all three hooks are staged under temporary names and renamed into place
(so a failure during the run restores what was replaced), and the installer refuses to overwrite a
hook it did not write, a symlink, or an unreadable file. Every hook it replaces is copied first to
`local/loop/install-backups/<unix-seconds>-<pid>/<hook>`, which also covers the pre-v2 `pre-commit`.
**Git LFS's stock `pre-push` body is the one foreign hook it replaces by design**, because the
managed hook chains to LFS itself; the stock body is backed up like any other.

| Hook | Runs | Blocks |
|---|---|---|
| `pre-commit` | `uor-harness check --staged` | `error` severity only; `warn` and `info` print and pass |
| `commit-msg` | `uor-harness git check-message <file>` | on a `<registered-lab>/...` branch with `UOR_LAB` set: a message whose final paragraph (after git's comment block is stripped) has no `Lab:` or `Order:` trailer, or whose `Lab:` does not equal `UOR_LAB`. Git-generated merge, revert, fixup! and squash! subjects pass, and the owner's commits and non-loop work are never checked |
| `pre-push` | Git LFS's own `pre-push` first, on the same stdin (exiting 2 when `git-lfs` is missing), then `uor-harness git push-check` | a pushed `<lab>/...` branch whose `loop pr-check --local` fails; a new branch is checked, a delete is skipped, and a `pr-check` that cannot run is allowed |

Every hook lets the commit or push through when `uor-harness` is not on `PATH`, and when the harness
itself cannot produce a result (any exit other than 0 or 1). The trailers are tags, not proof (D55):
they record the lab and the ORDER, and no lab may bypass a hook with `--no-verify`. A provably older
binary is printed as a warning by `check-message`; it does not block the commit. The lab-session
detection is `UOR_LAB` only, the charter's convention: `env -u UOR_LAB` is outside what a hook can
govern. `loop arrive` checks only the `pre-commit` hook today and reports a missing or different one
as **WARN**, not FAIL: installing the hooks is the owner's act, and a check that is always red
teaches labs to ignore FAIL. Extending arrive to all three needs `looparrive.rs` in pr12's scope.
