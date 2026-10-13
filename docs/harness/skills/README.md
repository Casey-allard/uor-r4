# Research skills

A **skill** here is a reusable research protocol a lab loads and follows: what to
read, what to do in order, what it must output and what is forbidden. Each one
lives at `docs/harness/skills/<id>/SKILL.md` with Agent Skills front matter that
`scripts/harness/registries.py` parses (scalars, inline lists, block lists).

Load one by id; the body is the protocol, the front matter is what the harness
matches:

```
uor-harness skill list
uor-harness skill show research-plan-cycle
uor-harness skill use research-plan-cycle -m m1
```

`skill use` appends one receipt line to `docs/harness/ledger/<YYYY-MM>.jsonl`, so
a session's actual skill use is visible after the fact. The skills encode the
contract, not a new one: [session-goal.md](../../labs/session-goal.md), D21–D24 in
[DECISIONS.md](../../integration/DECISIONS.md), `AGENTS.md` and `CONTRIBUTING.md`
stay authoritative, and a skill cannot change a criterion, a panel or a cap.

## Index

| Skill | Use it when | Outputs |
|---|---|---|
| [`research-plan-cycle`](research-plan-cycle/SKILL.md) | Starting or restarting a lab session, or after any merged PR: one D21/D22 cycle from status card to result PR to count. | Status card, one FIT pre-registration, one merged result PR with KEEP/REJECT, a dated `docs/labs` record, a current-state entry, or a pivot card at 3/3. |
| [`experiment-prereg`](experiment-prereg/SKILL.md) | Before any model compute, with no exception — the sheet and its `TEST FITNESS` review. | A nine-field pre-registration comment on the milestone issue and its `FIT`/`NOT FIT` review; a `prep` receipt. |
| [`cost-and-compute-admission`](cost-and-compute-admission/SKILL.md) | Before a run that costs compute or storage, and whenever it needs more than projected. | An admissible cost projection in the sheet, a `uor-pod` lease under lab+session, the cumulative ledger charge, a release and cleanup record. |
| [`evidence-record`](evidence-record/SKILL.md) | Every completed unit — experiment, fix, feature or negative — in the same PR that delivers it. | `docs/labs/<topic>-<YYYY-MM-DD>/README.md` plus a newest-first `docs/integration/current-state.md` entry ending in **Next:**. |
| [`panel-scoring`](panel-scoring/SKILL.md) | When a run finishes and its number must be turned into a headline movement or a KEEP. | A scored result with grader identity, per-seed values, per-category rows, failure rows, and fresh transfer for any development KEEP. |
| [`context-hydration`](context-hydration/SKILL.md) | Session start, after a compaction or handover, or when the work may not match the live plan. | The loaded-source list with a reason each, the newest `OWNER DIRECTION`, the mechanism brief, a `context` receipt. |
| [`expert-dispatch`](expert-dispatch/SKILL.md) | When an independent, bounded question is worth a separate context: review, falsification, cost check. | A dispatch receipt per teammate, one artifact at an exact path per teammate, and a collection note with accepted/rejected. |
| [`lesson-and-guard`](lesson-and-guard/SKILL.md) | When something failed twice, a line closed, or a review found a breach a document did not prevent. | `docs/harness/mistakes/<id>.md` with Trigger/Symptom/Lesson/Guard, a rule id that fires, and the citation from the record. |

The dispatchable roles that carry these skills, with their default model routes,
are in [roles.json](../roles.json) (`uor-harness team roster`).
The harness as a whole is described in [docs/harness/README.md](../README.md) and
frozen in [CONTRACT.md](../CONTRACT.md).
