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
| `panels/v5.json` | the acceptance panels and their frozen targets |

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
uor-harness budget show|scan            cumulative cost, and the pod specs of running jobs
uor-harness lint check|format           lint a PR body against the contract; render a compliant one
uor-harness heartbeat --milestone m1    emit the 90-minute status card
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

**Known gap, stated rather than hidden:** there is no topic index over failures. The
catalogue and the archive INDEX are reachable only by already knowing they exist, and
`current-state.md` is chronological. `docs/harness/mistakes/` is the topic index, and
it is deliberately small: one entry per lesson that has a guard or a named trigger.

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
