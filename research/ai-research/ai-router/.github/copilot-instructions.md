# GitHub Copilot — Project Instructions for ai-router

> These instructions are loaded automatically by GitHub Copilot in every session
> on this repository. Read them fully before any action.

---

## What this project is

This is a **mathematical research project** to prove or falsify the thesis:

> Fixed Riemannian geometry (H^4 × H^4 hyperbolic product manifold) can replace
> the dense linear-lattice routing used in transformer-style LLMs — reducing
> hardware cost by 10×–100× through geometry-native routing, phase transport,
> and sparse event compute.

This is theoretical physics/mathematics first. Code is the experimental
apparatus, not the product.

---

## The kill-list (strict prerequisite chain)

These 7 stages are ordered. A later stage **cannot be promoted over an earlier
open gate** without explicit written justification in the queue docs.

| # | Stage | Status |
|---|---|---|
| 1 | Hyperbolic embedding stability | PARTIAL |
| 2 | Measure-consistent shell routing | CLOSED |
| 3 | Hopf angular correctness | PARTIAL-PASS |
| 4 | Phase transport usefulness | PARTIAL-PASS |
| 5 | Spectral / operator usefulness | PARTIAL-PASS (strong, finalized) |
| 6 | **Sparse event-driven trainability** | **PARTIAL ← active gate** |
| 7 | Hardware-efficiency confirmation | PARTIAL |

**Canonical source of truth:** `router-research/docs/research/KILL_LIST_TRACKER.md`

---

## Current active work

- **Primary RR:** RR-067
- **Latest closed INC:** INC-0151 (Stage 5 finalized: KEEP)
- **Branch:** `main` (latest state); experiment branches use `codex/*`
- **Milestone:** EPIC-6 · Sparse Event-Driven Trainability

Stages 2–5 are resolved. The active gate is Stage 6.
Do not start Stage 7 work without written justification in the queue docs.

---

## Before doing ANYTHING in a session

Run these commands in order and read the output before touching code or docs:

```bash
cd /path/to/ai-router
make state      # validates all canonical docs are consistent
make branch     # prints current branch / kill-list context
make bootstrap  # full startup context packet if needed
```

If `make state` fails, **fix the inconsistency first** — do not proceed until
it passes.

---

## The mathematical architecture

```
H^4 (routing geometry)
  └─ Hopf fibration → coarse route address (base R^3)
       └─ shell law → radial bucket (shell_pmax, shell mass)
            └─ angular law → sector bucket
  └─ phase transport → geometry-induced cheap motion

H^4 (coupled field, asymmetric product H^4 × H^4)
  └─ complex-plane value storage
  └─ phase shift on routing transitions

sparse events → emerge on top of geometry
hardware savings → follow from sparse events + routing compression
```

**The three objects must stay conceptually separate:**
1. Routing manifold (where tokens route to)
2. Transport/state field (what moves along the route)
3. Retrieval/key field (what is indexed for lookup)

Merge them only if a branch explicitly proves they should merge.

---

## Research protocol

Every experiment follows:
```
screen (1 seed) → confirm (2 seeds) → finalize (4 seeds)
```

A screen result is never a conclusion. A confirm result is not a conclusion
until the increment doc records an explicit KEEP / KILL / REFINE decision.

No grid sweeps without a decision gate that requires them.

---

## Decision authority (highest wins when docs conflict)

```
CORE_PROJECT_GOALS.md                    ← mathematical mission, never override
  └─ KILL_LIST_TRACKER.md               ← stage status, single source of truth
       └─ ACTIVE_STATE.md               ← current queue (RR + INC)
            └─ ROUTE_MATRIX.md          ← branch-to-stage mapping
                 └─ DECISIONS.md        ← permanent record of closed decisions
                      └─ SESSION_LEDGER.md  ← in-session notes (temporary)
```

---

## Agent guardrails

- **Scope:** Work only on the active branch and its increment doc. Do not modify
  code on `main` or on a different `codex/*` branch.
- **Math first:** Before changing a routing formula, sector assignment, or shell
  law, describe the mathematical object being changed and what the change means
  geometrically.
- **No silent assumptions:** If a config parameter, sector mode, or comparison
  baseline is ambiguous, ask — don't pick one silently.
- **No cross-contamination:** Do not use results from one RR branch to justify
  changes on a different RR branch without updating DECISIONS.md.
- **No cleanup-as-progress:** Renaming files, restructuring configs, or tidying
  docs does not count as experiment progress and should not appear in a results
  commit.
- **Cross-pollination rule:** If you observe a result that could be relevant to
  another kill-list stage, record it in SESSION_LEDGER.md under a
  "Cross-stage observation" heading — do not diverge to fix it on this branch.

---

## Branch workflow (summary — full guide in docs/GIT_ONBOARDING.md)

```bash
# Start new work:
make new-inc RR=<rr> INC=<inc> SLUG=<slug>

# Validate before pushing:
make state

# See all branches + their kill-list stages:
make branches
```

Commit messages must start with `[RR-###]`. The hook provides the template.

---

## Definition of done for an increment

An increment (INC-####) is closed only when **all** of these are true:

1. The increment doc has `## Status` set to `Closed: KEEP.` / `Closed: KILL.` /
   `Closed: REFINE.` with a one-sentence decision rationale.
2. `ACTIVE_STATE.md` points at the next RR/INC.
3. `KILL_LIST_TRACKER.md` stage status is updated if the verdict changed.
4. `DECISIONS.md` has a new entry.
5. `make state` passes.
6. The GitHub Issue for this RR is updated with the final decision.

---

## What is NOT progress

The following are **support work only** — they do not advance a kill-list stage
and must not displace active gate work:

- Packet manifests / carry-forward contracts
- Comparison inheritance cleanup
- Route-id selection cleanup
- Report reshaping / doc prettification
- Adding new eval harness features not required by the active INC

---

## Drift detection

**Stop immediately and re-read CORE_PROJECT_GOALS.md if:**
- The next branch has no kill-list stage mapping
- The next experiment is a tweak of a baseline that already failed
- Three or more consecutive branches have touched only translation/packaging
- The phrase "let's just try" appears without a falsification condition
- An increment is queued for Stage 7 while Stage 6 is still open, with no
  written justification

---

*For full project context: `router-research/CORE_PROJECT_GOALS.md`*
*For kill-list status: `router-research/docs/research/KILL_LIST_TRACKER.md`*
*For git workflow: `router-research/docs/GIT_ONBOARDING.md`*
*For GitHub issues/milestones: https://github.com/Casey-allard/ai-router/milestones*
