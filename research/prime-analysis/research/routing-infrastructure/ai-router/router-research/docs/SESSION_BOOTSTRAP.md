# Session Bootstrap

> **For agents recovering from context loss or compaction — run this first:**
>
> ```bash
> make state      # validate canonical docs — MUST PASS before any work
> make branch     # print current branch + kill-list context
> make bootstrap  # full startup context packet
> ```
>
> Then read: `docs/research/SESSION_LEDGER.md`
>
> **Active gate: Stage 2 / RR-061 / INC-0137 / GitHub Issue #1 / EPIC-2**
> Do not advance any other stage until Stage 2 is resolved.

---

Purpose: give every new session, resumed session, or post-compaction recovery a
single authoritative starting point.

This file exists because the repo has multiple layers of context:

- static theory and moonshot framing
- canonical project contract and kill ladder
- route-family queue and closed decisions
- dynamic handoff and worklog notes

If those layers are read in the wrong order, it is easy to anchor on a stale
increment and overwrite the live branch narrative.

## Primary Mission

This project is a **staged mathematical falsification program** for a
geometry-first computation hypothesis — not a router search.

The thesis: fixed H^4 × H^4 hyperbolic geometry can replace dense
linear-lattice routing in transformer-style LLMs, with a long-range target of
10×–100× hardware savings via geometry-native routing, phase transport, and
sparse event compute.

Near-term falsification chain:
- validate or kill the geometric substrate
- preserve global hyperbolic alignment
- derive measure-consistent shell and sector laws   ← CURRENTLY OPEN
- test Hopf angular correctness
- test coupled-field phase usefulness
- test spectral structure
- test sparse event-driven trainability
- hardware-efficiency confirmation

Long-term moonshot:
- geometry → operator → spectrum → phase/event dynamics
- sparse event-driven computation on a structured manifold
- eventual geometry-native hardware relevance

## Canonical Architecture Contract

The active mathematical object is asymmetric `H^4 × H^4`, not generic `R^8`.

- first factor = routing geometry
- second factor = coupled field
- routing manifold, transport/state field, and retrieval/key field must not be
  collapsed into one coordinate law prematurely
- phase should be treated as geometry-induced transport, not a free heuristic
- The Hopf fibration maps points to coarse addresses; shell law governs radial
  buckets; angular law governs sector buckets

## Falsification Ladder

Use this ordering when deciding what matters next:

1. embedding stability
2. **measure-consistent shell routing ← OPEN**
3. Hopf angular correctness
4. phase transport necessity/usefulness
5. spectral emergence measurement
6. sparse event-driven trainability
7. hardware-efficiency confirmation

Do not promote later-stage claims while earlier gates are unresolved.
If you are about to work on Stage 3+ and Stage 2 is still open, you must write
the justification in ACTIVE_STATE.md and KILL_LIST_TRACKER.md first.

## Source Of Truth Order

When files disagree, trust them in this order:

1. `CORE_PROJECT_GOALS.md`
   - compact statement of the actual mission, kill-list order, and anti-drift rules
2. `docs/PROJECT_CONTEXT.md`
   - canonical thesis, architecture contract, imported theory framing, kill ladder
3. `docs/research/KILL_LIST_TRACKER.md`
   - canonical kill-list stage status, blockers, and next branch
4. `docs/research/ACTIVE_STATE.md`
   - single live queue authority: active RR/INC, success, falsification,
     supporting evidence, deferred branches
5. `docs/routes/ROUTE_MATRIX.md`
   - active route families, live frontier, next live branch
6. `docs/DECISIONS.md`
   - closed experimental conclusions and promotions/kills
7. `docs/research/SUPPORTING_EVIDENCE.md`
   - valid downstream evidence that should not redefine the main queue
8. `docs/research/CURRENT_DIRECTION.md`
   - current interpretation and near-term queue
9. `docs/research/HANDOFF_CURRENT.md`
   - resumability packet, useful but easier to go stale
10. `docs/research/LIVE_WORKLOG.md`
   - short-term activity log, never use alone as project truth

Rule:
- never resume from `CURRENT_DIRECTION.md`, `HANDOFF_CURRENT.md`, or an increment
  doc alone
- never let a single increment doc outrank the route matrix or project context

## Mandatory Read Order

Before continuing research, read in this order:

1. `docs/SESSION_BOOTSTRAP.md`
2. `CORE_PROJECT_GOALS.md`
3. `docs/PROJECT_CONTEXT.md`
4. `docs/research/KILL_LIST_TRACKER.md`
5. `docs/research/ACTIVE_STATE.md`
6. `docs/routes/ROUTE_MATRIX.md`
7. `docs/DECISIONS.md`
8. `docs/research/SUPPORTING_EVIDENCE.md`
9. `docs/research/CURRENT_DIRECTION.md`
10. `docs/research/HANDOFF_CURRENT.md`
11. `docs/research/LIVE_WORKLOG.md`

Then load theory/control docs as needed:

- `GEOMETRIC_COMPUTATION_HYPOTHESIS.md`
- `geometric_routing_kill_tests.md`
- `phase_transport_hypothesis.md`
- `hyperbolic_router_math_review.md`
- `spectral_emergence_moonshot.md`
- `adaptive_field_computer_moonshot.md`

## Resume And Compaction Rule

After any long pause, compaction, or obvious context loss:

1. re-read the mandatory read-order list above
2. re-read `docs/research/SESSION_LEDGER.md`
3. restate the active mission, kill-list stage, and next branch from those files
4. inspect the live code/results state
5. run `python tools/check_research_state.py`
6. only then edit code or rewrite planning docs

Before selecting a next branch after recovery, answer explicitly:
- which kill-list stage does this branch advance?
- which mathematical object is under test?
- why is it not just contract or packet cleanup?

Never continue directly from memory after compaction.

## Session Trace Rule

During long investigations or branch pivots, append externalized trace entries
to `docs/research/SESSION_LEDGER.md`:

- what files were read
- what commands were run
- what conclusions were reached
- what next action is queued

Helper:

```bash
python tools/session_ledger.py --kind checkpoint --note "Investigating RR-061 divergence after INC-0062"
```

## Dynamic-Docs Safety Rule

Before editing any of these files:

- `docs/research/ACTIVE_STATE.md`
- `docs/research/KILL_LIST_TRACKER.md`
- `docs/research/CURRENT_DIRECTION.md`
- `docs/research/HANDOFF_CURRENT.md`
- `docs/research/LIVE_WORKLOG.md`
- `docs/routes/ROUTE_MATRIX.md`
- `docs/DECISIONS.md`

you must first re-read:

- `docs/SESSION_BOOTSTRAP.md`
- `CORE_PROJECT_GOALS.md`
- `docs/PROJECT_CONTEXT.md`
- `docs/research/KILL_LIST_TRACKER.md`
- `docs/research/ACTIVE_STATE.md`
- `docs/routes/ROUTE_MATRIX.md`

If `CURRENT_DIRECTION.md` and `HANDOFF_CURRENT.md` disagree with
`PROJECT_CONTEXT.md`, `KILL_LIST_TRACKER.md`, `ACTIVE_STATE.md`, or
`ROUTE_MATRIX.md`, treat them as stale until reconciled.

## Anti-Patterns

Do not:

- anchor on old increment docs first
- let a late-stage evaluation packet redefine the project goal
- treat a negative local experiment as the project thesis
- rewrite handoff/current docs before reloading root theory and route docs
- reopen killed branches without a new mechanistic reason
- mistake proxy evidence for full architectural proof

## Fast Reload Command

Use:

```bash
python tools/context_bootstrap.py --group startup --cat
```

That command prints the core startup packet in the mandatory order above.
