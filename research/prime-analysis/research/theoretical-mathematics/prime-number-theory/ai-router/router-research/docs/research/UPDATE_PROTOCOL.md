# Research Update Protocol

## Purpose
Preserve enough state after every research increment that a later session can resume without reconstructing intent from raw logs.

## Bootstrap Rule
Before continuing a research branch or editing any current-direction docs, re-read:
- `docs/SESSION_BOOTSTRAP.md`
- `CORE_PROJECT_GOALS.md`
- `docs/research/SESSION_LEDGER.md`
- `docs/PROJECT_CONTEXT.md`
- `docs/research/KILL_LIST_TRACKER.md`
- `docs/research/ACTIVE_STATE.md`
- `docs/routes/ROUTE_MATRIX.md`
- `docs/DECISIONS.md`

If those files disagree with `CURRENT_DIRECTION.md` or `HANDOFF_CURRENT.md`,
resolve the conflict before continuing. Do not let a stale handoff packet define
the project direction by itself.

Run:
- `python tools/check_research_state.py`

before editing queue or handoff docs, and again after the reconciliation pass.

## Required Updates After Every Closed Increment Or Control
1. Update the active increment or control doc with:
   - hypothesis
   - kill-list stage
   - mathematical object under test
   - why this branch advances the core project goal
   - config path
   - analysis path
   - gate note path
   - screen/confirm/finalize outcome
   - explicit keep/kill/refine decision
2. Update:
   - `docs/research/ACTIVE_STATE.md`
   - `docs/research/KILL_LIST_TRACKER.md`
   - `docs/research/CURRENT_DIRECTION.md`
   - `docs/research/LIVE_WORKLOG.md`
   - `docs/research/PROGRESS_TRACE.md`
   - `docs/program/ISSUE_REGISTRY.md`
   - `docs/program/PROJECT_BOARD.md`
   - `docs/DECISIONS.md`
   - `docs/routes/ROUTE_MATRIX.md`
   - `docs/reports/REAL_TASK_COMPARISON.md`
   - `results/INDEX.md`
3. If the result changes the open questions or research queue, update:
   - `docs/research/INCREMENTS.md`
   - `docs/research/OPEN_QUESTIONS.md`
   - `docs/research/FLEET_ASSIGNMENTS.md`
   - `docs/research/SUPPORTING_EVIDENCE.md` if the branch becomes supporting or
     is removed from the live queue
4. If a mechanism family name changes, update:
   - `docs/research/PHI_PHI_PHI_FAMILY.md`
   - `docs/research/PHI_PI_LOGPHI_PLAN.md`

## Next-Branch Gate
Before queuing the next increment, record explicitly:
- which kill-list stage it advances
- what would falsify or promote the branch
- why the work is not only packet, contract, or reporting cleanup

If the answer is primarily “inheritance cleanup,” stop after one pass and
return to a branch that tests the underlying mechanism.

If the active branch is later in the kill list than an earlier `open` or
`partial` stage, record the justification explicitly in `ACTIVE_STATE.md`.

## Consistency Rule
When the live route queue changes, update these in the same pass:
- `docs/research/ACTIVE_STATE.md`
- `docs/research/KILL_LIST_TRACKER.md`
- `docs/routes/ROUTE_MATRIX.md`
- `docs/research/CURRENT_DIRECTION.md`
- `docs/research/HANDOFF_CURRENT.md`
- `docs/research/LIVE_WORKLOG.md`
- `docs/DECISIONS.md`

If that full reconciliation is not done yet, explicitly mark the stale file in
its latest update section instead of silently leaving conflicting guidance on disk.

## Minimum Resume Packet
If the session may be cut, make sure these files are current:
- `docs/SESSION_BOOTSTRAP.md`
- `docs/research/KILL_LIST_TRACKER.md`
- `docs/research/ACTIVE_STATE.md`
- `docs/research/CURRENT_DIRECTION.md`
- `docs/research/LIVE_WORKLOG.md`
- `docs/research/PROGRESS_TRACE.md`
- latest file in `docs/research/increments/` or `docs/research/controls/`
- latest gate note in `docs/governance/gates/`

## Long-Run Rule
- Before starting any sweep or confirm that may outlive the current context window:
  - append the exact next action to `docs/research/PROGRESS_TRACE.md`
  - append a checkpoint entry to `docs/research/SESSION_LEDGER.md`
- After the run lands:
  - append the exact artifact paths and promotion/kill decision to `docs/research/PROGRESS_TRACE.md`
  - append a result entry to `docs/research/SESSION_LEDGER.md`

## Branch Naming Rule
- Keep exact artifact labels in configs, logs, parsed JSON, and gate notes.
- Use mechanism-level names in research docs when a family becomes coherent.
- Always map the mechanism-level name back to the exact artifact label.

## Recommendation Rule
- Do not promote a route from screen to confirm without writing down why that promotion happened.
- Do not claim a runtime win unless it survives the relevant control batch.
- When a branch improves the routed family but still loses to `R0`, record both truths explicitly.
