# Research harness: a front door to the rules the project already had

**Question.** The repo's process layer is strong on paper — the [session-goal contract](../../labs/session-goal.md),
D21's negative count, D22's TEST FITNESS gate, [CONTRIBUTING § Delivery cadence](../../../CONTRIBUTING.md#delivery-cadence-every-completed-unit-owner-9-october-2026) —
but almost nothing instruments it. A session could not say which skill it used, which agents it
dispatched, what the milestone headline was right now, what a run cost, or which past failure
applied to the file it was about to edit. This record covers the tooling built to close that gap.

**Line:** no model line — tooling. · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).
**Pre-registration:** not applicable. No model compute was run, and none may be: the harness is
not admitted to GPU or model work under D24.

## What was built

`scripts/harness/` — a standard-library-only Python CLI and importable library, plus the
registries it reads in `docs/harness/`.

| Piece | What it does |
|---|---|
| `cli.py` | 12 commands; human and `--json` output; degrades to an explicit error instead of a traceback |
| `core.py`, `registries.py`, `util.py` | path discovery, strict front-matter reader, JSON/JSONL IO, `ContextPack`, token redaction, registry dataclasses |
| `checkrules.py` | a rule engine: id, severity, check, rationale, and the mistake a rule guards |
| `briefing.py` | assembles the task brief and reports what it loaded, why, and what it left out |
| `receipts.py` | the visibility primitive: one JSON line per skill use, dispatch, heartbeat, prep, PR |
| `prep.py` | pre-registration sheets, TEST FITNESS review, frozen panels and `PASS/FAIL/AT NOISE` scoring |
| `prlint.py` | 14 gates over a PR body, including the D22 pre-registration gate and the D23 softmax interlock |
| `mistakes.py` | 13 mined lessons, each with a trigger, an evidence path and a guard where one exists |
| `status.py` | milestones, receipts, delivery state and GitHub state in one 20-second read |
| `githelp.py`, `budget.py` | pre-commit hook install and branch/merge verification; cumulative cost and a read-only pod scan |
| `docs/harness/skills/` | 9 reusable research protocols with `when`/`requires`/`outputs`/`forbidden` |
| `docs/harness/roles.json` | 9 dispatchable expert roles, each with a purpose and an allowed model route |

## What it actually found

This is the part that matters, because none of it was known before the tooling ran:

1. **Two merged branches were never cleaned up**, against the 9 October "no standing branches"
   rule: `deepseek/invariant-router` (PR #2247 merged) and `bpb-split` (PR #2081 merged). Left
   alone deliberately — they are other labs' branches and the worktrees are dirty or active.
2. **The registry had drifted from the live prompts.** M2's reopened `protected/discrete
   constructor` line is `0/3` in the prompt the lab reads but `5/3` on the GitHub card; the Kimi
   lab's `evaluation integrity` line existed in no registry at all. `briefing.lines_agree_with_prompts`
   now computes exactly this comparison, and it caught the drift (including one instance the author
   of this record introduced and then corrected: M4's `wide learned flock read`).
3. **A PR body could satisfy the D22 gate without pre-registering anything**, because the gate
   scanned the template's own unticked "pre-registration" checkbox as if it were a claim. Fixed:
   markers are read from prose only, so an unticked instruction is not evidence.
4. **The retrieval the repo already has is dormant.** `uor-knowledge` (8,492 items, FTS5, verified
   working) is a 9-day-old snapshot that is mostly other labs' worktrees; the 1,350-line
   [retired-mechanisms catalogue](../../research/retired-mechanisms-catalogue-2026-10-01.md) is
   reachable from `DECISIONS.md` on exactly one line; there is no topic index over failures. The
   harness points at these rather than building a fourth store.

## Evidence

- `python3 -m unittest discover -s tests/harness -t .` — 212 tests, all passing. Tests cover the
  front-matter reader, report provenance, receipts, the brief assembler, PR lint, the mistake
  registry, panels, budget, status, and the CLI entry points end to end.
- `python3 scripts/harness/cli.py check` — every machine-checkable rule, with the fix for each.
- `python3 scripts/harness/cli.py status` — the read above, including the two branches.
- `.github/workflows/harness.yml` — a path-filtered CI job (no Rust toolchain) so the rules run
  on the PRs that can break them without competing for the shared `required-transport` slot.

## Limitations, stated plainly

- `docs/harness/milestones.json` is a **cache** of the GitHub milestone issues and the live
  prompts. It is not authoritative, and `briefing.milestone_numbers_agree` /
  `briefing.lines_agree_with_prompts` are warnings, not syncs. Nothing here writes to GitHub.
- The receipts prove orchestration, never a result. A receipt cannot move a headline number.
- `mistakes/` holds 13 lessons with real evidence paths, but only 4 `block`-severity lessons have
  a live guard. The rest say `none: <reason>` and remain notes.
- The harness is Python tooling only. It adds no model dependency, runs no model compute, and its
  output is never model evidence.

**Next:** the owner (or a lab session) decides whether to adopt `brief` as the session entry point
in place of re-deriving context from prose; the two standing branches need their owners to clear
them; and the dormant `uor-knowledge` index needs a refresh path wired into session start before
anyone should trust it for a decision.
