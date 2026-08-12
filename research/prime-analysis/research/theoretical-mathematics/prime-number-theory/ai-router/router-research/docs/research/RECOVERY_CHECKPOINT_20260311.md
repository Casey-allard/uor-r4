# Recovery Checkpoint (2026-03-11)

Purpose: preserve the active investigation state before more context loss.

## Superseded Note

This checkpoint captured a conservative recovery read before the corrected
reruns of `INC-0062`, `INC-0063`, and `INC-0064` were completed.

Parts of the old conclusion are no longer canonical:
- the project should no longer be described as having gone off-course at
  `INC-0062`
- the old inert negative for `INC-0063` was falsified by a data-path bug
- `INC-0064` is no longer merely queued; the corrected screen is
  mechanism-positive

Use `docs/research/CURRENT_DIRECTION.md` and
`docs/research/HANDOFF_CURRENT.md` as the updated resume surface.

## Original Recovery Finding (archival)

The project appears to have gone off-course starting around `INC-0062`.

The key issue is not that `INC-0062` failed.
The issue is that later work treated `INC-0062` as if it had closed the
Hopf-angular correction gate and therefore justified moving on to phase-first
branches.

The repo evidence read during this recovery says that interpretation was too
strong.

## Files Re-read During Recovery

Primary source-of-truth files:
- `docs/SESSION_BOOTSTRAP.md`
- `docs/PROJECT_CONTEXT.md`
- `docs/routes/ROUTE_MATRIX.md`
- `docs/DECISIONS.md`
- `docs/research/CURRENT_DIRECTION.md`
- `docs/research/HANDOFF_CURRENT.md`
- `docs/research/LIVE_WORKLOG.md`

Relevant branch-control files:
- `docs/research/MECHANISM_FIRST_PLAN.md`
- `docs/research/increments/INC_0060_h4_hopf_measure_diagnostics.md`
- `docs/research/increments/INC_0061_measure_consistent_route_law.md`
- `docs/research/increments/INC_0062_hopf_base_angular_law.md`
- `docs/research/LEARNED_KNOWLEDGE.md`
- `docs/research/PROGRESS_TRACE.md`
- `docs/program/PROJECT_BOARD.md`
- `docs/program/ISSUE_REGISTRY.md`

## Original Durable Reading (archival)

1. `RR-061` still looks like the correct active gate.
   - `docs/program/PROJECT_BOARD.md` still lists:
     - `RR-061` Derive a measure-consistent `H^4` / Hopf route law
   - `RR-064` is listed as next, not now.

2. `INC-0060` and `INC-0061` did not justify moving on from measure / angular
   correction.
   - `INC-0060` says geometry routing is real, but measure mismatch is still a
     root issue.
   - `INC-0061` says shell-only equal-mass correction failed twice and that the
     next correction must target the Hopf base directly.

3. `INC-0062` created a useful control, not a closed angular-law victory.
   - `phase4d_hopf_base` is healthy and fast.
   - It is useful as a no-fiber-phase coarse-address control.
   - But `INC-0062` explicitly says it did not validate the stronger
     angular-correction hypothesis outright.

4. Moving directly to strict phase-necessity work after `INC-0062` was
   therefore probably premature.
   - Kill ladder order in the canonical docs is:
     1. embedding
     2. measure-consistent shell routing
     3. Hopf angular correctness
     4. phase transport necessity/usefulness
   - If `INC-0062` did not close Hopf angular correctness, phase-first
     follow-ons were off-sequence.

## Updated Working Conclusion

Do not use this checkpoint alone to block `INC-0065`.

The corrected reruns changed the safer resume point to:
- keep `RR-061` open as a mathematical guardrail
- treat corrected `INC-0062` / `INC-0063` / `INC-0064` as canonical
- resume from `INC-0065`, not from the stale inert-phase narrative

## Original Pre-Rerun Next Step (archival)

Before the corrected reruns were completed, the safest conservative next step
looked like a return to `RR-061` with stronger angular-law-sensitive
diagnostics.

That is no longer the active resume instruction. Keep it only as context for
why the recovery paused the stale phase queue in the first place.

## Live Code State At Checkpoint

Confirmed:
- `phase4d_hopf_base` exists in the router and proxy harness
- `phase4d_hopf_transport` exists
- `phase4d_hopf_product_phase` does not exist in the live tracked code
- `phase4d_hopf_chi` already exists and has unit coverage

## Resume Rule

If context is lost again:
1. read `docs/SESSION_BOOTSTRAP.md`
2. read `docs/PROJECT_CONTEXT.md`
3. read `docs/routes/ROUTE_MATRIX.md`
4. read `docs/DECISIONS.md`
5. read `docs/research/CURRENT_DIRECTION.md`
6. read `docs/research/HANDOFF_CURRENT.md`
7. read this file only as archival recovery context
8. continue from `INC-0065`, not from the stale inert-phase narrative
