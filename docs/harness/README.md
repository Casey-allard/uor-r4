# The research harness

The repo has strong *policy* — the [session-goal contract](../labs/session-goal.md),
D21's negative count, D22's TEST FITNESS gate, the [delivery cadence](../../CONTRIBUTING.md#delivery-cadence-every-completed-unit-owner-9-october-2026) —
but almost nothing that **instruments** it. A session cannot tell from the repo which
skill it used, which agents it dispatched, what the milestone headline is right now,
what it cost, or which past failure applies to the file it is about to edit.

`scripts/harness/cli.py` is that instrument. It is a zero-dependency Python CLI
(and importable library) that turns the repo's existing authorities into one
queryable surface and emits **receipts** so orchestration is visible after the fact.

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
uor-harness heartbeat --milestone m1    check in (live tier; 5-minute window)
uor-harness git install-hooks|status    the pre-commit guard, and this branch's delivery state
uor-harness check [--staged]            every machine-checkable rule in one pass
```

## Where it sits against what already exists

The repo did not lack for machinery; it lacked a front door. This harness is the front
door, not a fourth store. Use what is already here, through it:

| Already built | What it is | How the harness uses it |
|---|---|---|
| `~/.local/bin/uor-knowledge search` | SQLite FTS5 over 8,492 project items with provenance | the place to look up a term's history; **the index is a snapshot, so refresh before acting** (`scripts/sync_project_knowledge.py --open-issues --ingest`) |
| `docs/research/retired-mechanisms-catalogue-2026-10-01.md` | 1,350 lines of retired mechanisms with `CLEAN / TOO TIGHT / CONFOUNDED / UNDER-POWERED / POLICY-KILLED / INSTRUMENT-DEFECT` verdicts | the deepest negative-results record in the repo; a `mistakes` entry cites it as evidence rather than restating it |
| `docs/history/branch-archive/INDEX.md` | 193 rows of preserved branch patches with restore bases | where retained-but-inactive source lives; the harness does not duplicate it |
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

## Getting started (any lab, no install)

The harness is standard-library Python with **no dependencies and no setup step**. From a
checkout of this repository:

```sh
export UOR_LAB=<your lab>          # kimi | claude | codex | deepseek
uor-harness status
```

That is the whole bootstrap. Two details are worth knowing because they are the only things
that ever go wrong:

**Interpreter discovery, not Python version, is the real risk.** On this machine `python3`
resolves to **3.9.6 in a login shell** (`/etc/zprofile` puts `/usr/bin` first) and **3.15.0 in a
non-login shell** (an agent tool's PATH puts `/opt/homebrew/bin` first). Both work — the
declared floor is 3.9 and every interpreter present is above it — but if you want one
reproducible answer, pin it:

```sh
/usr/bin/uor-harness status        # system python, always 3.9+
uv run --python 3.12 scripts/harness/cli.py status    # or a uv-managed 3.12
```

An interpreter below 3.9 now gets an explicit message naming the interpreter and the fix,
instead of a raw traceback.

**`--repo` is a global option and must come before the subcommand.** `cli.py check --repo X`
fails; `cli.py --repo X check` works. From a directory that is not a checkout, the tool fails
loudly rather than guessing.

Verified: all 269 tests pass on both 3.9.6 and 3.15.0.

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
export UOR_LAB=kimi        # or: cli.py --lab kimi ...
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

**What this cannot do, stated plainly:**

- It is not real-time. Presence is as fresh as the last check-in.
- It cannot see a lab that never runs the harness. Absence is not evidence of idleness.
- It cannot cross vendor CLIs. There is no shared runtime, no push channel, and each vendor's
  session state is private by design.
- A lab that ignores a claim can still work the branch. The harness reports; it does not police.

## Rule engine

`scripts/harness/checkrules.py` holds the rules; each rule has an id, a severity and a
message that names the exact fix. `--staged` mode is used by the pre-commit hook. A rule
added for a mistake cites that mistake's id, so a lesson becomes a guard rather than a note.

## Before you install the hook

`uor-harness git install-hooks` resolves the hooks directory with
`git rev-parse --git-path hooks`, which in a linked worktree points at the **shared**
`.git/hooks` of the whole repository — including the owner's checkout and every other lab's
worktree. Installing there changes commit behaviour for labs that never asked for it, so:

- the installer refuses to overwrite a `pre-commit` hook it did not write, and
- it is **not installed by default**. Run `check` and `lint` yourself, or install the hook
  deliberately after telling the other active labs.

The hook only ever blocks on `error` severity; `warn` and `info` print and pass.
