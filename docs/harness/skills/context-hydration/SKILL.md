---
name: context-hydration
description: Start a session with the right context and nothing stale — refresh main, read the newest OWNER DIRECTION, load the mechanism brief, run the harness brief, and record what was loaded and why.
version: 1
harness: 1
kind: research
applies-to: [docs/integration/current-state.md, docs/mechanisms/*.md, docs/labs/session-goal.md, docs/harness/milestones.json]
when: [start a session, hydrate context, what should I read, stale context, brief, owner direction]
requires: []
outputs: [a loaded-context list with a reason per source, the milestone headline and its newest OWNER DIRECTION, the mechanism brief for the run, a context receipt in docs/harness/ledger]
forbidden: [repeating a broad historical audit for a routine task, treating a dated next as the live plan, loading a whole audit into context, reading or printing secret paths]
milestones: [m1, m2, m3, m4]
---

## When to use

Use at the start of a session, after a compaction or a handover, and whenever the
work does not obviously match the live plan. The readiness test is one line: the
newest `OWNER DIRECTION` on your milestone issue has a recorded response, and
every source you are working from was refreshed in this session.

## Do this

1. **Refresh first, read second.**
   `git fetch origin && git log --oneline -1 origin/main`
   Read the milestone issue only after the fetch, so the newest direction cannot
   be missed:
   `gh issue view 2029 --repo UOR-Foundation/uor-r4 --comments`
   (`OWNER DIRECTION` comments are appended over time; treat the newest one as
   authoritative over older cards and over this file.)
2. **Name the milestone and the line.** From the issue, copy the headline
   number and its target, and the line and count. Cross-check against
   [milestones.json](../../milestones.json) — if the file disagrees with the
   issue, the issue is authoritative and the file is a bug to report, not to edit
   from a skill.
3. **Load the mechanism brief for the run you will make**:
   each file in [docs/mechanisms/](../../../mechanisms/) states the idea, what has
   been tried, and **what a fair test needs**. Read that page before the
   pre-registration, and cite it in the sheet.
4. **Run the harness brief** for the milestone and your files:
   `uor-harness brief --milestone m1 --files <paths you will touch> --query "<your task in one line>"`
   The pack prints every source with why it was selected and what was left out.
   It is the starting context; it performs no network access and no model compute.
5. **Read only what the task needs.** Minimum set: the milestone issue's newest
   `OWNER DIRECTION`, the top entry of
   [current-state.md](../../../integration/current-state.md), the mechanism brief,
   the specific source files and the nearest relevant record — not the whole
   history. Do not repeat a broad historical audit for a routine task.
6. **Resolve the dated "next" rule.** A **dated "next" inside an old record does
   not override the live plan.** Many `docs/labs/*/README.md` files end in a
   "Next:" from their own day; if it conflicts with the newest direction on the
   milestone issue or the top of `current-state.md`, the live plan wins and the
   old record keeps its text as evidence.
7. **Check the interlock you are about to work under.** For model work:
   `python3 scripts/check_no_softmax.py` is the D23 ratchet, and D24 forbids CPU
   model compute and non-decisive GPU work. A mechanism brief written before
   D23/D24 may still describe a now-excluded configuration.
8. **Record what was loaded and why.** The context you loaded, the sources with
   their paths, the ones you dropped and the reason — normally the harness brief
   already lists these; keep that list in the session's first status card so the
   next session can audit the starting point. A `context` receipt line in
   `docs/harness/ledger/<YYYY-MM>.jsonl` is the machine-readable form.
9. **Stop hydrating when the readiness test passes.** Load more only when a
   specific decision needs it: extra context that cannot change the next step
   costs tokens and invites working from a stale plan.

## Output

- The milestone headline, its target, and the line and count, copied verbatim.
- The newest `OWNER DIRECTION` and what it changes for this session.
- The list of loaded sources with a reason each, and the dropped ones with the
  reason they were dropped.
- The mechanism brief that the next run will be pre-registered against.
- A context receipt in `docs/harness/ledger/<YYYY-MM>.jsonl`.

## Do not

- Do not start from a cached context, an old chat summary or a quoted headline;
  refresh `origin/main` and the issue first.
- Do not treat this file as authority over the milestone issue, `AGENTS.md`,
  `DECISIONS.md` or the newest owner comment.
- Do not obey a dated "next:" from an older record over the live plan.
- Do not load a full historical audit, an entire issue history or unrelated
  mechanism briefs "just in case".
- Do not read or print secret paths (Runpod config, credentials); the harness
  redacts token-shaped text and refuses secret paths — keep it that way.
