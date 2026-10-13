---
name: expert-dispatch
description: Decide whether to dispatch an expert team, pick the smallest sufficient one, give each teammate a self-contained prompt with an exact deliverable and a disjoint write scope, and record the dispatch receipt.
version: 1
harness: 1
kind: research
applies-to: [docs/harness/roles.json, docs/harness/ledger/*.jsonl, docs/labs/audit.md]
when: [dispatch an expert, subagent, teammate, second opinion, independent review, falsify this, parallel work]
requires: []
outputs: [one dispatch receipt per teammate, one returned artifact at an exact path per teammate, one collected result naming what was accepted and rejected and why]
forbidden: [dispatching a team larger than the question needs, two writers on one file, accepting prose instead of a file, dispatching a read-only specialist to edit, widening a teammate's scope after dispatch]
milestones: [m1, m2, m3, m4]
---

## When to use

Use when a question is independent, bounded and worth more than one context —
an architecture review, a source falsification, a cost check, an independent
reading of a result. Do not dispatch to look busy: a dispatch must produce an
artifact a decision can use. Roles and their default routes are in
[roles.json](../../roles.json); the roster is readable with
`uor-harness team roster`.

## Do this

1. **State the question in one sentence.** If you cannot, the dispatch is not
   ready. Pick the **smallest sufficient team**: usually one specialist per
   genuinely independent question, not a panel of generalists.
2. **Choose roles by question, not by prestige.** `roles.json` gives each role a
   `purpose` (the exact question it answers), `seat`, `when` triggers and a
   default `route`. Prefer parallel read-only specialists (reviewer, auditor,
   falsifier, scout) over serialized generalists; one writer per file.
3. **Give each teammate a self-contained prompt**: the exact deliverable and the
   path it must land at, the input identities (files, artifacts, issue, comment),
   the constraints that apply (D21/D22/D23/D24, no model compute, read-only or
   not), and the write scope it owns. A teammate that has to re-derive the task
   from conversation history will return something else.
4. **Assign disjoint write scopes before dispatch.** One owner per file; if two
   teammates must touch one file, serialize them and say which one goes second.
   Advisory overlap is not a lock — check the shared task list and the actual
   paths before starting.
5. **Record the dispatch**, so the roster's use is auditable:
   `uor-harness team dispatch <role-id> --milestone m1 --task "<one line>" --scope <paths> --count 1`
   The receipt is the answer to "did an agent actually get deployed", and the
   role's route is frozen in it.
6. **Collect and verify the artifact, not the summary.** Open the file the
   teammate claimed to write. Check its identity (path, size, hash where it
   matters) and that it answers the question asked. A claim of success is not a
   result.
7. **Record the dispatch receipt and the disposition**: which artifacts were
   accepted, which were rejected and why, and what changed in the plan as a
   result. A dispatch that changed nothing is a cost with no decision — say so
   and do not repeat it.
8. **Escalate routes deliberately.** DeepSeek routes are the default specialist
   (owner direction, 25 September 2026); Kimi/Moonshot is reserved for an explicit
   owner request. Use a configured route from `roles.json`; never invent a
   provider/model pair, and never print an API key.

## Output

- One dispatch receipt per teammate in `docs/harness/ledger/<YYYY-MM>.jsonl`.
- One artifact per teammate at an exact path, inside its declared write scope.
- One collection note: question, role and route, artifact path, accepted or
  rejected with the reason, and the plan change it caused.

## Do not

- **A teammate that returns prose instead of a file is a failed dispatch.** Re-dispatch
  it with the path restated, or do the work yourself; never merge prose as if it
  were the artifact.
- Do not dispatch two writers to one file, and do not let a read-only role edit.
- Do not widen a teammate's scope mid-flight; stop it, restate the task, dispatch
  again. Scope is a constant, not a hint.
- Do not dispatch the whole roster to "get coverage"; unused roles are cost.
- Do not accept an expert's claim about the repo without opening the source it
  cites — a citation is a pointer, not evidence.
- Do not use a route that is not in `roles.json`, and do not dispatch a paid
  provider outside the owner's caps.
