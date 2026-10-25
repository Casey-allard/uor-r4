# The loop: how five labs work on UOR-R4

This page describes the loop that every lab follows. Most of its tooling is built; the steps table and the build plan say which parts are still to build.
CHARTER.md §5.10 holds the rule; this page explains it. Where they disagree, the charter wins.

**Status.** Most `loop …` commands on this page are built: each step names the loop PR that built it, and a step marked "to build" is still done by hand. The commands are subcommands of `uor-harness`; the rest come from the PRs in [the build plan](#build-plan).
Markers used below: **(exists)** means the tool or file is in `main` today; **(to build)** means a PR in the build plan adds it.

## Credit

The loop is adapted from **the Loop Kit by Ron Brazil**. His kit keeps one AI agent on course with a written order for each task, changes that can be undone, a few short notes that carry memory between sessions, and checks anyone can rerun.
UOR-R4 keeps those ideas and adapts them for five labs from five vendors working at once. The kit's own text is private to the owner and is not in this repository. Everything here is written in our own words.

## Kit-to-loop map

Each part of the kit, what it is for, and what takes its place here. "Private skill" means the kit's file is installed only in each client's local skills folder, as D44 sets out.

| Kit part | What it is for | In the UOR loop | Where it stands |
|---|---|---|---|
| start-here (skill) | A first-run interview that sets up the notes and the plan | Done once, by the owner on 20 Oct 2026; the answers are D43–D58 | Done. Not installed, so it cannot start a second setup |
| Safe copy of the original | Keeping the starting point recoverable | GitHub `main`, the protected branches, the immutable `archive/` and the HF store | Dropped (D44) |
| scope-guard (skill) | Giving each task an edge and making it reversible | The ORDER's scope, stop-and-ask, one worktree per task, and the in-unit fresh-context review (§5.5, D66) | The ORDER exists now. `loop scope` and the policy check come in PR 5 |
| A line budget per task | Catching oversized changes | Size is reported in the REPORT, never capped | Dropped as a cap (D48) |
| session-close (skill) | Ending a session so the next one loses nothing | The REPORT, the lab's PICKUP, and the rendered STATE and HANDOFF | By hand now. `loop close` comes in PR 7b |
| holding-the-goal (skill) | Questions to ask before saying no, and before proposing a fix | The opener in every launch prompt, and a stop-and-ask card that names the limiter | Private skill now. `loop ask` is built in loop PR 7a |
| solution-integrity (skill) | Questions to ask before, during and after a change | The opener; the REPORT names the parts used | Private skill now |
| working-discipline (skill) | Habits for arriving, reading, measuring and reporting | The arrival check against the repository, and the REPORT's order (negatives first) | By hand now. `loop arrive` comes in PR 7b |
| gates (skill) | A single command for all the checks, a log of past runs, a planted fault for each check, and a live view of a long run | `loop gates`, with its ledger and its planted faults, and `loop board` | PR 4 and PR 11 |
| House rules file | The short rules every session reads first | `AGENTS.md`, with CHARTER §5.10 | Exists |
| PLAN | The route from today to done | `PLAN.md`, PM-written | Built, loop PR 8 |
| STATE | What is true now | `STATE.md`, rendered by the harness | PR 3 |
| HANDOFF | The next few tasks | `HANDOFF.md`, rendered by the harness | PR 3 |
| DECISIONS | The reasons behind each choice | The append-only table in `docs/integration/DECISIONS.md`, from D43 on | Exists |
| LESSONS | Lessons the project paid for | `LESSONS.md`, rendered from the mistakes store; L-001 is records drift | PR 3 and PR 8 |
| PICKUP | The note that resumes the next session | `local/<lab>/PICKUP.md`, not committed (D47) | By hand now. `loop close` comes in PR 7b |
| FILEMAP | What each folder holds | `docs/start/FILEMAP.md` | Built, loop PR 8 |
| ORDER and REPORT | One task's order, and its account | `docs/loop/tasks/<item>/<lab>-<n>/`, committed with the PR | Exists |
| Archive | Finished work, moved unchanged, leaving a pointer behind | `archive/`, byte-identical and never edited | Exists |

## What the loop is

Each lab runs in a loop. A small supervisor program starts a fresh session. The session does one unit of work, then ends. The supervisor then starts the next session.
A unit of work is one of: a build, a repair, a review, or collecting a GPU run.
Every unit has a written order (ORDER). The order names the item, the files the lab may touch, when the work is done, a cancel probe, and when to stop.
A change enters `main` only when its gates pass and its exact head carries a counted review. Since 24 Oct 2026 the review is part of the unit: the ORDER lab's own fresh-context adversarial review counts, and other labs audit after the merge (D66). The owner ended D59, which had routed setup PRs to Claude's reviewers alone, on 22 Oct 2026.
Shared status pages are rendered by the harness from committed inputs. Nobody edits them by hand, so they cannot quietly drift. Drifting records cost this project more than any other failure.

## The steps for one lab

Steps that need no judgement run in the supervisor, without a model.

| # | Step | What happens | Command | Who |
|---|---|---|---|---|
| 1 | Next | Decide go, wait or stop, and which kind of session. Order of preference: owner direction, repairs to the lab's own PRs, reviews, GPU collections, premise probes, the top of the lab's lane, then `any`. Only the kill switch stops. Every other no-work case is WAIT, naming what is pending and when to look again; `loop next --wait` and `loop drive` sleep and ask again. | `loop next` (built, loop PR 7b; WAIT and `--wait`, claude-31) | supervisor |
| 2 | Arrive | Fetch; check the binary, hooks, notes and read path; render the views locally. If the repository disagrees with a note, the repository wins. | `loop arrive` (built, loop PR 7b) | session |
| 3 | Opener | Read the opener and the thinking skills the task needs. Name the parts used in REPORT. | — | session |
| 4 | Claim | Refused unless the item is ready and unclaimed, its scope overlaps no open order (notebook pages excepted), and the lab is under its open-PR limit. Create a worktree `.worktrees/<lab>-<topic>` from fresh `origin/main`, commit the ORDER, open a draft PR, then list again: of two racing drafts, the higher PR number closes before any work. Post the §5.3 intent note. | `loop claim` (built in loop PR 7a) | session |
| 5 | Probe | Run the cancel probe first. Commit its result, classed `instrument` or `mechanism`, before any commit outside the task folder. If more than three files are expected, commit a plan first too. A cancel closes this order or configuration only, never a mechanism (§4.1, §4.2). | gate (to build) | session |
| 6 | Work | Work in scope with WIP commits. GPU work needs a prep sheet, a non-author FIT, the lab's own fresh pre-run review (D67) and `scripts/pod/uor-pod`. | `prep` (exists) | session, subagents |
| 7 | Gates | Quick gates during work, the full set before review. | `loop gates` (built, loop PR 4) | session |
| 8 | Report | Write REPORT, a notebook row for any result, and receipts, all in the task folder. | `loop report --check` (built, loop PR 7b) | session |
| 9 | Ready | Build the PR body from REPORT, size first; mark the PR ready; write PICKUP; end the session. | `loop close --session` (built, loop PR 7b) | session |
| 10 | Review | Inside the unit, before the PR goes ready (D66): the ORDER lab's own fresh-context adversarial reviewer (a lab of another vendor also counts; for a loop setup PR while D59 is in force, Claude) reads only the item chain, the ORDER and the diff, never the plan. Other labs audit after the merge. A `host` done-when is re-run at the head. A `gpu` done-when is checked by the sealed artifact's checksum and run receipt, never re-run on the laptop (§3.5). | `loop review` (built, loop PR 6) | session (D66; D59: Claude for setup PRs) |
| 11 | Repair | Requested changes, a red gate or a queue ejection give the author lab a new order `<lab>-<n+1>` in the same task folder, added to the same PR in a commit that touches only its own folder. It lists finished steps in the past tense with their commits. The highest `n` is the active ORDER every tool judges; the earlier ones are history, and a review record for one of them no longer counts (claude-22). | `loop next` finds repairs (loop PR 7b); the repair ORDER is still written by hand | session |
| 12 | Queue | Only when the policy gate is green at the head, the approval the policy gate accepts is at the head (Claude's for a setup PR under D59; otherwise the ORDER lab's own fresh-context record or another vendor's, D66), no later "changes requested" that the gate counts exists, and no owner or PM comment came after the approval. | `loop queue` (built, loop PR 7c) | supervisor |
| 13 | Verify | Find the `(#N)` squash commit on `main`; re-run a `host` done-when on fresh `main`; delete the branch and worktree only when GitHub reports MERGED; run `uor-hygiene --apply`. A failure becomes a repair. | `loop close --verify` (built, loop PR 7b; it prints the hygiene plan and does not apply it, see claude-17 REPORT) | supervisor |
| 14 | Render | Refresh the render PR. | `loop render --pr` (to build) | Kimi; a cover lab after 2 hours red |
| 15 | Restart | Start the next fresh session, after `loop next` says go: one fresh headless session per unit, its pid/group recorded, the soft 2-hour check-in and a watcher for a leased GPU run. | `loop drive` (built, loop PR 10) | supervisor |

`loop drive` now runs this cycle for one or more sessions on the laptop, and it calls `loop next` for every stop including the kill switch.

`loop next`'s sources beyond the kill switch, repairs and reviews: `owner-direction` reads the first `open` row of PLAN's `## Owner directions` table and `premise-probes` read PLAN's premise table (both find their Status/Probe/Direction columns by header name; a malformed open row is skipped and named, not hidden), and `lane` reads the lab's owner-set lane plus the PM's item `lane:`/`rank:` ranking; all name what is missing when the data is absent. `any` offers the first claimable ready item ranked by effective milestone (the smallest milestone among the item and every item that depends on it, so an infra item an M1 item needs ranks as M1), then by how many not-done items it unblocks, then by id. GPU collections read the lease watcher's record under `local/<lab>/gpu-collections/`: a finished record that no session has collected is offered as work, and `loop drive --collect <id> --artifact <path>` records where the artifacts went. Each renewal updates a liveness timestamp, so a `running` record is treated as stale only after six hours **without a renewal**; a killed watcher writes no note, and a record without a heartbeat falls back to its start time. Until it is wired into every client's headless launcher, a lab does steps 1–15 itself and the delivery path in CHARTER §5.5 applies as written.

## The notes

Notes carry memory between sessions. Most are rendered, so nobody types them.

| Note | Path | Who writes it | Rule |
|---|---|---|---|
| STATE | `STATE.md` (replaces `STATUS.md`) | the renderer only | Carries a stamp naming the `main` commit of its inputs. `render --check` re-renders at the stamp and needs an exact match, so a hand edit fails. Lines that drop out are appended verbatim to `archive/state/<YYYY-MM>.md`. |
| HANDOFF | `HANDOFF.md` | the renderer only | Open items only, each with its item id and title, its check for done, the probe that would cancel it, an estimate and where it stands; a blocked table with owners; a not-doing list. "Unknown" must name the item that will find out. |
| PLAN | `PLAN.md` (folds in `ROADMAP.md`) | the PM, by PR | Goal; premises with their probes and cancel lines; the M1–M8 route written backwards; approved sequences; not-doing. A route or premise change needs a decision row. |
| DECISIONS | `docs/integration/DECISIONS.md` | owner or PM, by PR | Append-only. New rows add a "rejected" column and a "cost to reverse" column. Charter changes are also logged in `docs/rules/changes.md`. |
| LESSONS | `docs/harness/LESSONS.md` | rendered from `docs/harness/mistakes/*.md` | Each lesson keeps a permanent `L-NNN` key. L-001 is "records drifted". The PM picks a top ten that is rendered into AGENTS.md. |
| FILEMAP | `docs/start/FILEMAP.md` | the PR that adds a folder | One row per directory, two levels deep. A folder without a row fails, and so does a row whose path is gone. |
| ORDER, REPORT, receipts | `docs/loop/tasks/<item>/<lab>-<n>/` | the lab's top session | Committed in the task's PR and never changed after merge. At milestone close the PM moves folders byte-identical to `archive/loop/`. |
| PICKUP | `local/<lab>/PICKUP.md` (gitignored) | `loop close` | What the next session needs. A standing-rule line may only cite a charter section, a decision row or a lesson id, so no lab keeps a private rulebook. |

**What is rendered.** STATE, HANDOFF, LESSONS, the top-ten block in AGENTS.md, `docs/roadmap/README.md` and `docs/loop/baseline.json`. Loop PR 3 builds `uor-harness loop render`: STATE, HANDOFF and LESSONS are rendered and checked, the AGENTS block is rendered between the markers PR 8 added, the roadmap view goes through xtask, and `baseline.json` is deferred to PR 5. The renderer reads committed files only: roadmap items, milestone bars and holds, the newest notebook row per line, mechanism status lines, the lab registry, the mistakes files and files at one commit. It reads no clock and no GitHub, so a render is repeatable.

**The render PR.** One PR, on branch `kimi/render`, changes the rendered views. A lab PR that touches a rendered view fails. `loop next` offers a views-only render PR for another lab's exact-head review even though it carries no ORDER (the render sentinel path); only the `kimi/render` owner lab is not asked. The refresh merges `origin/main` forward, renders, commits and pushes, never by force. The stamp the gate checks is the PR's merge base with `origin/main`, which after the merge-forward is main's tip, not the merge commit — so render at it explicitly: `uor-harness loop render --at "$(git merge-base origin/main HEAD)"`, then commit and push. A freshness check goes red when views lag their inputs by more than 2 hours.

**Rule sources.** An owner-only list names the only files allowed to state a rule: CHARTER.md, AGENTS.md, this page, `pm-rules.md`, DECISIONS, the mistakes files and `docs/rules/changes.md`. Other rule-like files become generated files or short pointers.

**Read-path budget.** AGENTS.md stays within about 80 lines. STATE and HANDOFF stay under 24 KB each, PICKUP under 8 KB and an ORDER under 4 KB. The whole read path fails above 75 KB (76,800 bytes) since loop PR 11 raised the warning to an error, and above 125 KB the command names the hard limit.

**Today.** `docs/harness/claims.json` is still edited the old way. `STATUS.md` and `docs/integration/NOW.md` are retired with pointers (loop PR 8): STATE.md, HANDOFF.md and PLAN.md carry their functions.

## Scope, without a line cap

- **Declaring scope.** Each roadmap item gains a `scope:` list of globs (a root-wide `**` is refused), a size estimate, a lane, a rank, a notebook line, a done-when class (`host` or `gpu`) and a "Cancel if" section. The ORDER copies the item's scope and may only narrow it.
- **Checked at the base.** The gate reads the item as it is at the PR's base. Every order glob must fall inside the item's globs. Widening scope takes a separate item PR, reviewed by Kimi, or by the owner when Kimi and the cover lab are both away.
- **Always allowed:** the lab's own task folder, its item's status line, new rows on its notebook page, new mistake files and FILEMAP rows.
- **Detection.** A hook refuses an out-of-scope write; `loop scope` runs in the quick gates; the policy gate checks every changed path, both sides of a rename. The fix restores the file from the base, or removes it if new.
- **Stop and ask.** `loop ask --scope` names the file, the reason, roughly how many lines, and whether the lab's own code could change instead. `loop ask --blocked` names the limit, how it was measured, the command that checks it, and whether the work is refused or just not reached yet. The order pauses and the supervisor moves on.
- **Size is reported, never capped.** In the owner's words: size is reported, never capped. The gate prints size first: files and lines by class (code, tests, docs, generated). REPORT states the same numbers. The guard warns past twice the estimate, and REPORT explains any overrun. Diffs over about 600 lines are split across fresh reviewers. Nothing is rejected unread.
- **Tests.** Removing a test, ignoring it, or changing an expected value is refused unless the item names that test with a decision row or an answered card.
- **Dependencies.** A new `Cargo.lock` entry needs a `deps:` line naming a decision row or answered card. Whitespace-only file changes are flagged.
- **REPORT sections:** what did not work; what changed, with size and the ratio to the estimate; measurements with conditions and command; checks run at the head; skills used; what is still open; and a plain summary with every term explained.

## Gates

A gate is a check anyone can rerun. Each gate names what it proves, and a planted fault that must turn it red.

**The runner (built, loop PR 4).** `loop gates [--quick|--full|--keep-going|--status|--watch|--break G|all]` reads `docs/harness/gates.json`, which lists four gates today: `cargo fmt --check` for the claim-wording and harness crates, claim-wording, and the progress-window tests. **The progress window (built, loop PR 11):** `--watch` shows gate N of M, the gate running now, reds as they land and a finish estimate from the ledger. It opens only for a full run in a terminal, never in CI or a headless run, closes after the run, and gives up after 5 minutes without change.

**Gates that exist today:** `cargo fmt`; clippy and library tests for changed crates; `uor-harness check`; `lint check`; the d23, d25 and d32 ratchets; claim-wording; `cargo xtask links`, `archive-verify --base`, `roadmap check` and `docs-catalog --check`; `shard-check`.
**Gates to build:** `render --check`, `scope`, `order --check`, `report --check`, `budget`, rule sources, milestone bars against CHARTER §4.8, and the local private-text check.

**Run rules.**
- A gate that cannot run is REFUSED, never a pass.
- After a red, later gates read UNKNOWN unless `--keep-going` is given.
- The true exit code is kept, with a log in `local/<lab>/<topic>/`.
- The files each gate reads are hashed at start and end. A change during the run voids it.
- No research compute, pod, store or GitHub write happens inside a gate (§3.5, §4.5).

**Ledger.** `local/<lab>/gates.jsonl` records gate, verdict, seconds, commit and the working-tree hash. An uncommitted edit makes a result STALE. The ledger is not committed; CI's result at the head is the authority.

**Breaks.** `--break G` plants the gate's declared fault in a scratch worktree. The gate must go red with its declared failure text. Red for another reason, such as a build failure, is REFUSED. Every gate must declare a break: appended text, or one exact replacement (`replace`, whose `from` must occur exactly once), so the planted fault can be a removed branch rather than an added case. When a PR changes a gate, the old breaks also run against the new gate, so a gate and its break cannot be loosened together. A break that stays green is a gate defect: add the case, never relax the check. A bug that escapes every gate becomes a new gate whose break is that bug.

**CI (built, loop PR 5).** One workflow, `gate.yml`, which `ci.yml` calls on every pull request, with no path filters:
- `tests` runs `loop gates --full` with a read-only token. It takes the lab from the PR's ORDER; a PR with no ORDER is outside the loop and is skipped with a notice.
- `policy` runs `uor-harness loop pr-check --ci`. It checks order, scope, commit order, REPORT, owner paths, rule sources, the rendered-file lock, read path and FILEMAP. Since D76 (owner, 25 Oct 2026), six of these are advisory and only print `WARN`: scope, commit order, rule sources, rendered-file lock, read path and FILEMAP. Order, banned presence, REPORT, owner paths, the review record and the required checks still fail a PR. Banned presence also scans added files, which it skipped before D76. On PR events it builds the base branch's code and reads the PR's commits only as data, never running the PR's code. The review record and the required checks are checked at queue time, because they do not exist yet when the job runs.
- **The policy gate is advisory on GitHub.** There is no org admin, so there is no GitHub App and no ruleset change. `required-transport` runs after the policy job, always. It records `policy: <verdict> (run <id>)` in each of the five required checks' summaries and leaves them green (owner, 21 Oct 2026: "Record only"). The policy is enforced locally: `loop pr-check --pr <n>` refuses a PR whose required checks lack `policy: pass` from GitHub Actions, or carry a commit status of the same name. A supervisor queues a PR only when that check is green and the review it accepts is at the head: Claude's for a setup PR under D59; otherwise the ORDER lab's own fresh-context record or another vendor's (D66).

## The five labs

| Lab | Vendor | Role |
|---|---|---|
| Kimi | Moonshot | PM, and builder or reviewer on other labs' items |
| Claude | Anthropic | builder and reviewer; leads the loop's architecture |
| Codex | OpenAI | builder and reviewer |
| DeepSeek | DeepSeek | builder and reviewer |
| Antigravity | Google | builder and reviewer |

**Registry (to build).** `docs/harness/labs.json`, owner-only, lists each lab's vendor, client, allowed model ids, lane and open-PR limit (3). A `UOR_LAB` value outside it is refused.

**Roles.** The owner is Casey, and the last fallback for item and scope PRs. The PM is Kimi, with a cover lab the owner names (§5.2). The PM ranks lanes through item PRs, reviews scope and item PRs, owns the render refresh, records owner directions verbatim the same day, triages cards, spot-checks reviews, and asks about stale drafts (24 hours without a push), never deleting them.

**Identity is by tags.** Every claim, PR, review and comment carries the lab's tag. Labs leave work tagged by another lab alone unless it is stale or needs help. There are no signing keys, no ssh-agent use and no list of signers. Tags show who did what; they do not prove it, because all labs share one GitHub account and one macOS user.

**The review record (built, loop PR 6).** A PR comment headed `uor-review v1` with: `pr`, `head` (40 hex), `lab` (the tag), `model` (written by the supervisor from the launch command), launch id, `verdict` (APPROVE or CHANGES), `order`, and the done-when result at the head (`host`) or the artifact checksum (`gpu`). The gate needs an APPROVE at the current head, from the ORDER lab itself with `context: fresh` (D66: the review is part of the unit) or from a lab of another vendor, with no later CHANGES. Until the other labs run the loop, a loop setup PR needs its APPROVE from Claude's fresh-context reviewers, whichever lab built it (D59). The gate checks the `claude` tag (D55). A PR counts as a setup PR only if its ORDER names its build-plan row in a `Setup row:` line, under the loop item or that row's own item. Claude's reviewer confirms that the PR builds that row. A record from any other lab on a setup PR does not count. Since PR 6, only `uor-review v1` records count; records posted in free text before it need nothing.

**Owner releases.** Since D67 (24 Oct 2026) a GPU or training run goes ahead on the lab's own pre-run review, and only D41's donor pass still needs the owner at dispatch. Where the owner does release, he approves it in the lab's own chat. The lab records the owner's words verbatim, with the time. There is no typed phrase.

**Outsiders.** A PR from a fork, or one without a lab tag, is an outsider PR. A maintainer lab adopts it with a tagged record that names an order the lab writes for it. GPU steps follow the §5.3 outsider rule.

**Concurrency.**

| Failure | Guard |
|---|---|
| Two labs take one item | Draft-PR claim and a re-list; the higher number closes before work. |
| Parallel edits to one file | Overlap refusal at claim. |
| A shared file in every PR (one PR was ejected from the queue for this) | Lab PRs change inputs only; one render PR changes the views. |
| Two PRs append to one notebook page | Allowed; on ejection the supervisor merges forward with both rows. |
| Writes in another lab's worktree | A hook refuses writes outside the lab's own worktrees and `local/<lab>/`. |
| A healthy GPU run cancelled | Leased runs are exempt from the check-in; a supervisor stops only its own pids. |
| A lab lifts GPU caps | `uor-pod` ignores cap overrides while `UOR_LAB` is set; labs stage such commands for Casey. |

## The 2-hour check-in and the stops

**The 2-hour check-in is soft.** At 2 hours on a task (cumulative across sessions), the supervisor (`loop drive`, loop PR 10) checks progress:
- **Real progress continues**, with a note on the owning issue. Real progress means new commits, gates turning green, changes inside scope, and no repeated failure.
- **A spiraled or stalled session closes.** Stalled means no commits and no tool calls; spiraled means the same failure again and again. On close the supervisor commits and pushes the WIP, writes a REPORT stub and a PICKUP from git state, and posts a card to the PM.
- **No lab ends a session on WAIT without a timer.** It holds `uor-harness loop next --wait --max <its client's command limit>`, or arms its client's own timer, and continues on GO. The PM still prompts any lab that went silent.
- Leased GPU runs are exempt. A watcher outlives the session, renews the lease, notes it on the issue and never cancels a healthy run.

**Stops and waits.** Only the kill switch stops a lab: the file `local/loop/STOP`, or the label `loop:pause` on issue #2028, which works from a phone. Every other case that leaves a lab without work is `WAIT`, printed with what is pending and when to look again (600 s), with exit 3:
- its own PRs awaiting review, once `open_pr_limit` refuses claims (the PRs are named);
- held work;
- items waiting on dependencies;
- an empty queue.

`loop drive` sleeps `--poll` seconds and asks `loop next` again on WAIT. A lab running its own session holds `loop next --wait` instead. A pending owner decision or decision card parks only its own item: the lab takes other claimable work meanwhile, and never acts on its own reading of the question.

**Reroutes.** Three sessions without movement turn the item into a `decision` with a pivot note (§4.4). Full GPU caps skip GPU items. A REFUSED gate becomes a repair or a card.

## Shared registration files

A few files are where every loop PR registers its piece: dispatch, documentation, gates and the rendered roadmap page. If item scopes listed them, `loop claim` would let only one lab hold a harness item at a time. So these nine are shared. No item scope lists them, `pr-check`'s `scope` rule always allows them, and `loop claim` refuses an item whose scope covers one:

- `crates/uor-r4-harness/src/lib.rs`, `main.rs`, `loopgates.rs` (the dispatcher), `doccheck.rs` and `loopprcheck.rs`;
- `docs/harness/README.md`, `docs/loop/README.md`, `docs/harness/gates.json` and `docs/roadmap/README.md`.

Casey decided this in Claude's chat. On 2026-10-21 at 23:43Z he answered "Exempt them (Recommended)" for the first six. On 2026-10-22 at 00:05Z he answered "Add all three (Recommended)" for `loopprcheck.rs`, `gates.json` and the roadmap page. Keep edits there small and additive, and merge `origin/main` into your branch whenever it moves. A test for a change to a shared module goes in the PR's own test file.

## Subagents

A subagent gets a narrower order. `loop suborder --parent <order> --scope <globs>` (built in loop PR 7a) refuses a scope outside the parent's and writes `local/<lab>/<topic>/orders/<n>.md`. It prints the subagent's block: the opener, the order, the narrowed scope, the stop list, and this clause: never claim, open, review or merge a PR; never post as the lab or push; never decide (§5.2); report to the parent. The top session lists every suborder and writes REPORT itself.

A marker file in the worktree names the active ORDER, so the parent's scope holds at any depth where hooks fire inside subagents.
Per client: Claude Code fires hooks inside subagents, and a SubagentStart hook injects the rules and the ORDER. Antigravity and Kimi Code also fire hooks inside subagents (P-1, 21 Oct). Antigravity names a subagent by its own `conversationId`; Kimi Code's subagent tool calls carry no agent id, so a guard places them between SubagentStart and SubagentStop. The DeepSeek harness has no hook runner, and Codex is unverified. Until loop PR 9 adds their guards, these clients' subagents follow the order as an instruction, and the policy gate checks the result.

## Install and onboarding (outline)

- **Who installs.** `loop install --all` (built, loop PR 9) is plan-only by default: it prints the full list of files it would write and writes nothing. Writing requires an explicit `--apply` (`loop install --all --apply`), which `loop guard` refuses from any lab session (alongside `git install-hooks`), so only the owner runs it from Terminal. With `--apply` it backs up each config it edits to `local/loop/install-backups/`, refuses to replace a differing file without `--replace`, and writes only to gitignored `local/` inside the repository.
- **The kit's skills** (`gates`, `holding-the-goal`, `scope-guard`, `session-close`, `solution-integrity`, `working-discipline`; `start-here` excluded) are installed into each client's own user-level skills folder (`~/.claude/skills`, `~/.codex/skills`, `~/.agents/skills`, `~/.gemini/config/skills`, `~/.dsh/skills`) from the owner's private copy. No text from the kit enters this repository.
- **One hook entry point.** `loop guard --client C --lab L --event E` (built, loop PR 9) replaces today's presence hooks. It exits at once outside `~/uor-r4` and its worktrees, so it never touches other projects. It catches accidental and habitual rule breaks and is not an OS sandbox; a shell deny-list can never be complete (for example, `echo 'pkill x' | sh`), so the clean-tree policy gate (`loop pr-check`) checks the resulting branch diff before merge.
- **Per client.** Claude Code reads `~/.claude/CLAUDE.md` importing AGENTS.md and user-level hooks in `~/.claude/settings.json`. Codex reads the AGENTS.md chain, `~/.codex/hooks.json` and `~/.codex/rules/uor-r4.rules` (which forbids force push, push to `main`, `gh pr merge`, `--no-verify` and `pkill`); Codex keeps full access. Antigravity reads AGENTS.md and a user-level plugin at `~/.gemini/config/plugins/uor-r4-loop/{plugin.json,hooks.json}`. Kimi's private start and handoff files (`~/.kimi-code/UOR-R4-START.md` and `~/.kimi-code/UOR-R4-HANDOFF.md`) become pointers of at most 1 KB, and its own second loop in `~/.kimi-code/config.toml` is retired. DeepSeek runs in its own harness with `~/.dsh/hooks.json` and a patch overlay (`~/.dsh/profiles/headless/cordis.patch.yml` and `local/deepseek/headless-overlay.yml`) on every launch.
- **Onboarding check.** `loop onboard --client C` (built, loop PR 9) prints PASS, FAIL, REFUSED or N/A for: rules reachable; skills found; hooks on the current binary; an in-process canary write to a denied path refused; `UOR_LAB` set; binary current. It ends with the lab's lane and first PICKUP. The owner pastes one short prompt into each client to run it.
- **Copyright check.** `loop copyright` (built, loop PR 9) compares repository text with the owner's private kit copy. It runs on the laptop only. Nothing about the kit, not even hashes, goes to CI.

## Charter changes

PR A carries the charter changes: the new CHARTER §5.10 "The loop", the §5.5 review rule as it then stood (code and model PRs needed an exact-head review from another vendor's lab; D66 replaced it on 24 Oct 2026), and the decision rows. Later steps, each with owner approval: PLAN and HANDOFF replace NOW.md; labs cannot lift GPU caps; rendered views, notebooks and task folders become the running record, with the old files moved to the archive byte-identical; LESSONS rendered from the mistakes files; duplicate PM rules removed; milestone bars reconciled with §4.8.

## Build plan

All Rust. Small PRs, through today's delivery path. From PR A on, every PR carries an ORDER and REPORT in `docs/loop/tasks/` and a counted exact-head review (from another vendor until 24 Oct 2026; the ORDER lab's own fresh-context review since D66).

**Teams.** Claude leads the architecture, with Kimi as PM reviewing. Codex and DeepSeek build PRs 2–7 and 10–12. Antigravity builds PR 9. During the setup, every review came from a different vendor than the author (D46, replaced on 24 Oct by D66). While Claude runs alone, Claude builds every setup PR, and each other client's adapter is added when that lab joins (D52, amended 21 Oct).

**Review of the setup itself (D59, ended 22 Oct 2026).** From 21 Oct every setup PR, whichever lab built it, got its exact-head review from Claude's fresh-context reviewers alone (charter §5.5; owner, 21 Oct, amended the same day). With no timeout, one blocked reviewer stalled every setup PR, so the owner ended it on 22 Oct ("End D59 now"). Setup PRs now take any other-vendor fresh-context review, like every PR. The switch is `docs/harness/d59.json`, listed in `docs/harness/owner-paths.json`. `loop review`, `loop next` and the bench read both at `origin/main`; `pr-check`'s review-record rule reads them at the merge base.

**PR 0, probes.** Scratch tests; results go to PLAN's premise table.

| Premise | Probe | If it fails | Result (21 Oct 2026; details in [claude-2's REPORT](tasks/infra-loop-harness/claude-2/REPORT.md)) |
|---|---|---|---|
| P-1 | Hooks fire headless and inside subagents, and which compaction events exist, per client. | That client's subagents get read-only tools; the policy gate checks the result. | **Claude Code: passes on one run, except compaction.** After the owner's CLI sign-in, one headless run fired these events: SessionStart; PreToolUse before each of the 3 tool calls, including the subagent's own tool call, which names the subagent's `agent_id`; SubagentStop; and Stop. A guard can therefore tell which subagent is calling. The compaction hook is configured but did not trigger in a run this short, so it is untested. Evidence: [#2028 comment](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6306306665). **Kimi Code 2.0.2: passes on three headless runs, except compaction.** SessionStart, TurnStarted, PreToolUse, PostToolUse, SubagentStart, SubagentStop and Stop fired. Inside a subagent, its own tool call fired PreToolUse and PostToolUse, and its turn fired TurnStarted marked `origin_name` "subagent". A subagent's tool calls carry the shared `session_id`, not an agent id, so a guard attributes them by their place between SubagentStart and SubagentStop. The plan-file Write fired PostToolUse only; SessionEnd did not fire on exit; headless sessions start in plan mode, and ExitPlanMode approves itself. PreCompact and PostCompact exist but did not trigger, so compaction is untested. Evidence: [#2028 comment](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6327710559). **Antigravity 1.4.3: passes, headless and inside subagents; no compaction event.** All six `hooks.json` events (SessionStart, PreInvocation, PreToolUse, PostToolUse, PostInvocation, Stop) fired in the parent and in the subagent. A subagent's PreToolUse carries its own `conversationId`, so a guard can record the root id at the first SessionStart. The hook schema has no compaction event. Evidence: [#2028 comment](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6327600976). **DeepSeek harness: fails, with no hook runner.** No hook is wired in any profile, and none fired, headless or in a subagent, although the headless JSON event stream shows tool calls and subagents. The catalog lists hook bridges (`@deepseek-ai/dsh-hooks-claude-code` and `-codex`), but none is installed. There is no compaction event. Evidence: [#2028 comment](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6327536813). **Owed by codex.** |
| P-2 | A supervisor can run each client headless: login persists, no prompts, model pinned, stoppable by pid. | That client stays interactive-only. | **Claude Code: passes, with limits.** The first run failed because the CLI sign-in had expired; after the owner signed in on 21 Oct, runs over about 2 minutes needed no new sign-in and showed no prompts. Only the pinned model (`claude-haiku-5-5`) ran, although the CLI logged `unrecognized_model` for that id. SIGTERM to the session pid ended it about 2 s later, together with a background shell task it had started in a process group of its own. Not tested: a foreground tool's group, and signalling a whole group. After the stop, Claude Code ran a short second session that wrote a summary, which a supervisor must not count as a lab session. The first tool call came about 20 s after launch, on one run. Evidence: [#2028 comment](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6306306665). **Kimi Code: passes.** A detached `kimi -p` run pinned with `-m kimi-code/kimi-for-coding` needed no sign-in and showed no prompts, and the pinned model ran. The SessionStart payload reports the config default instead, so a supervisor records the model from its own launch command. SIGTERM ended the run about 4 s later with its two child processes, and no second session started. Evidence: [#2028 comment](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6327710559). **Antigravity: passes.** `agy -p` with `--dangerously-skip-permissions` kept its login and showed no prompts, and `--model` pinned the model, as every hook payload shows. SIGTERM to the `agy` pid ended it about 0.45 s later, together with its MCP children and a tool's own process group; nothing was left running, and no second session started. The first tool call came about 26 s after launch. Evidence: [#2028 comment](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6327600976). **DeepSeek harness: passes with an overlay.** Stock `dsh headless` stops before the model for lack of a credential. The harness's per-launch overlay, which re-enables the saved sign-in and pins `deepseek-account` and `deepseek-flash`, ran twice with no prompts; it relies on the desktop app's saved sign-in. SIGTERM to the pid left nothing running, and no second session started. The launcher starts no child processes, so stopping children was not exercised. A supervisor must pass the overlay on every launch. Evidence: [#2028 comment](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6327536813). **Owed by codex.** |
| P-3 | Tags survive the squash merge and appear on every claim, PR, review and comment. | Tags move into the PR body and review record only. | **Passes on one sample.** The squash commit of #2895 on `main` (`fedde5eae`) keeps the `[claude]` title tag. Harness claims record the lab. Both review comments on #2895 carry their `[kimi]` and `[claude]` tags. These tags are text a lab writes about itself, from one shared account, so they mark ownership; they do not prove who wrote something. |
| P-4 | The policy result can feed a check that is already required. | The policy gate stays advisory; `loop queue` is the lock. | **Passes, tamper-evident only.** The five required contexts are check runs published by the `required-transport` job in `.github/workflows/ci.yml`, and the ruleset accepts them from any app, so a policy job can feed them with no ruleset change. Two routes get around it: (1) `ci.yml` runs from the PR's own code, so a PR that edits `.github/**` could skip the policy; (2) because nothing pins the source, a commit status with the same name, posted through the API by any token with write access, also satisfies a required check, and every lab posts as the same account. Pinning the source needs an org admin, which the project does not have (D54). So the guard is the policy job's own record plus `loop queue`, which will refuse (PR 5) a PR whose required checks did not come from the policy job, or that changes `.github/**` without the owner's approval. |
| P-6 (optional) | A render PR opened by Actions can be checked and queued. | Supervisors keep opening it. | Not run. |

P-5 is dropped: Codex keeps full access.

| PR | What it builds | The fault that must turn it red |
|---|---|---|
| A | CHARTER §5.10, the §5.5 review rule, decision rows, this page, the loop roadmap item | — (charter text; checked by roadmap and claim-wording gates) |
| 1 | Items for PRs 2–12; the kit mapping on this page; `labs.json`, `owner-paths.json`, `rule-sources.json`; five-lab hint; `version` reports its build commit | a sixth lab name is refused |
| 2 | Item keys, "Cancel if", done-when class, validator | an item scoped `**` is refused |
| 3 | `loop render` for every view, `--check` at the stamp, determinism tests | a hand-edited STATE fails |
| 4 | `loop gates`: runner, `gates.json`, ledger with working-tree hashes, REFUSED and UNKNOWN, breaks | `--break all` turns each gate red for its own reason |
| 5 | `loop pr-check`; `gate.yml` with the policy job advisory (built) | one fixture PR per rule fails |
| 6 | `loop review` with tagged review records (built) | an untagged review record, a same-lab record without `context: fresh` (D66), or a setup-PR record from a lab other than Claude is refused |
| 7a | `claim`, `suborder`, `ask` | an overlapping claim is refused |
| 7b | `next`, `arrive`, `report`, `close`; templates in our words; per-task receipts (built) | a REPORT missing a section is refused |
| 7c | `queue`, `board` (built) | a later CHANGES record blocks queueing, unless it comes from a lab other than Claude on a setup PR (D59) |
| 8 | Migration of the notes; AGENTS.md under the 80-line gate; DECISIONS columns; lesson ids; FILEMAP; `pm-rules.md`; bars reconciled | `xtask links` fails a planted broken link |
| 9 | `guard`, `install`, `onboard`; the local copyright check; committed onboarding receipts (built) | each client's canary write is refused |
| 10 | `drive`, watchers, kill switch, render refresh, compaction hooks, the soft check-in | a fake stalled client past 2 hours is closed and carded, while a fake progressing one continues with a note |
| 11 | `--watch`; warnings raised to errors | the window gives up after 5 minutes without change |
| 12 | Git hooks: binary current; `pr-check --local` before push; `Lab:` and `Order:` trailers | a commit without trailers is refused |

After PR 12, labs switch on one at a time: Claude, Codex, Kimi, Antigravity, DeepSeek.

## Risks

| Risk | Guard |
|---|---|
| Records drift again (lesson L-001) through render lag | Freshness check on every push and hourly; `arrive` renders locally; refresh first. |
| A stale bar or headline | Bars reconciled with §4.8 before the first render and gated after; every M1 row declares whether it is the headline. |
| Private rulebooks drift (memories, client skills, PICKUP) | Rule-sources list; standing rules are pointers only. |
| Tooling crowds out research | Research continues in parallel; the build is a finite list with a not-doing list; pivot after three sessions without movement. |
| The policy gate is advisory on GitHub | `loop queue` refuses locally; the policy result feeds required checks where it can. |
| Tags can be forged under one account and one user | Supervisors write the model and tag; the PM spot-checks; tags record, they do not prove. |
| Hooks fail open or are unverified | A canary every session, shown on the board; the policy gate; P-1 first. |
| Headless clients stall or lose login | P-2; the 2-hour check-in closes a stalled session. |
| Rubber-stamp reviews | The record carries a re-run result or checksum; the PM spot-checks. |
| Research evaluation creeps onto the laptop | Done-when classes; `gpu` items are checked by checksum only. |
| Queue ejections | Lab PRs change inputs only; receipts live in task folders; notebook rows merge forward. |
| The kit's text leaks | Local copyright check; installer refusals. |
| GPU overspend or a killed healthy run | `uor-pod` refuses lab overrides; leased runs exempt; own pids only. |
| Outsiders blocked | Outsider PRs and a maintainer lab's adoption record. |
