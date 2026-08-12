# ai-router — CLAUDE.md

> Read by Claude and Anthropic agents on workspace entry.
> This file mirrors the canonical project instructions.
> If this file and `.github/copilot-instructions.md` disagree,
> `.github/copilot-instructions.md` wins.

---

## Project identity

**Mathematical research project.**
Goal: prove that fixed H^4 × H^4 hyperbolic geometry can replace the dense
linear-lattice routing in transformer-style LLMs — targeting 10×–100× hardware
savings through geometry-native routing, phase transport, and sparse event compute.

Code is the experimental apparatus. Mathematics governs every decision.

---

## Mandatory startup sequence

```bash
make state      # validate canonical doc consistency — MUST PASS before any work
make branch     # print current branch + kill-list context
make bootstrap  # full context packet (use after compaction/context loss)
```

Read `docs/research/SESSION_LEDGER.md` after compaction recovery.

---

## Active gate

**Stages 2–5 are resolved. Stage 6 is the active gate.**

```
Stage 2: Measure-Consistent Shell Routing — CLOSED
Stage 3: Hopf Angular Correctness — PARTIAL-PASS
Stage 4: Phase Transport Usefulness — PARTIAL-PASS
Stage 5: Spectral / Operator Usefulness — PARTIAL-PASS (strong, finalized)
Stage 6: Sparse Event-Driven Trainability — PARTIAL ← ACTIVE GATE
  Latest closed: RR-067 / INC-0151
  Branch: main (latest state)
  Milestone: EPIC-6
```

You may not open work on Stage 7 during this session unless
`ACTIVE_STATE.md` and `KILL_LIST_TRACKER.md` both contain a written
justification for the exception and `make state` passes.

---

## The math you are working with

The routing universe is an **asymmetric H^4 × H^4 product manifold**:

- **First H^4** carries the routing geometry. The Hopf fibration maps points to
  coarse addresses: a shell law governs radial bucket assignment (shell mass must
  be H^4-measure-consistent), and an angular law governs sector assignment.
- **Second H^4** carries a discrete complex-valued state field. Phase shifts
  emerge from routing transitions — this is the cheap transport mechanism.
- **Sparse event routing** is meant to emerge on top of this geometry, not to
  replace it.

**Never merge the routing manifold, the transport field, and the retrieval
key field unless a branch explicitly proves they should be the same object.**

---

## Hard agent rules

1. **Work only on the branch named in ACTIVE_STATE.md.** Do not edit code on
   `main` or on any other `codex/*` branch without explicit authorization.

2. **State the geometry before changing it.** Before modifying any routing
   formula, shell law parameter, sector mode, or Hopf mapping, write one
   sentence naming the mathematical object being changed and what the change
   means geometrically.

3. **Never choose a config silently.** If a sector mode, baseline variant, or
   comparison partner is ambiguous, stop and ask rather than defaulting.

4. **Respect the prerequisite chain.** If you are asked to work on Stage 3 or
   later while Stage 2 is open, refuse and explain why, unless the justification
   is already written in the queue docs.

5. **Cross-stage observations stay in SESSION_LEDGER.md.** If you notice
   something relevant to a different kill-list stage, record it under
   "Cross-stage observation:" in `docs/research/SESSION_LEDGER.md`. Do not
   diverge to study it on this branch.

6. **No cleanup-as-progress.** File renames, doc restructuring, report
   reshaping, and config prettification do not appear in results commits.

7. **Every run produces three artifacts:**
   - `results/raw/<name>.log`
   - `results/parsed/<name>.json`
   - a row appended to `results/summary.csv`
   Without all three, the run is incomplete.

8. **Close increments properly.** An INC is not closed until:
   - `## Status` in the increment doc reads `Closed: KEEP.` / `KILL.` / `REFINE.`
   - `ACTIVE_STATE.md`, `KILL_LIST_TRACKER.md`, and `DECISIONS.md` are updated
   - The corresponding GitHub Issue is updated with the decision
   - `make state` passes

---

## When you are unsure what to do

Run `make state && make branch` and read the output.
Then read `router-research/CORE_PROJECT_GOALS.md`.
If still unsure, read `docs/research/KILL_LIST_TRACKER.md`.
The answer is almost always: finish the active INC before starting anything new.

---

*Full instructions: `.github/copilot-instructions.md`*
*Math context: `router-research/CORE_PROJECT_GOALS.md`*
*Current queue: `router-research/docs/research/ACTIVE_STATE.md`*
