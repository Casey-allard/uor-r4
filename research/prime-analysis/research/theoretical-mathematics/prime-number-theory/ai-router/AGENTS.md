# ai-router — Agent Instructions (all agents)

> This file is read by OpenAI Codex, Claude, and all CLI agents on repo entry.
> Read it completely. It is short by design — depth is in the linked docs.

---

## Mission

Prove or falsify: **fixed H^4 × H^4 hyperbolic geometry can replace dense
linear-lattice routing in transformer-style LLMs.**

Target: 10×–100× hardware savings via geometry-native routing + phase transport
+ sparse event compute.

**This is mathematics first. Code is the test apparatus.**

---

## STOP — do this before any action

```bash
make state      # must pass before touching anything
make branch     # prints your kill-list context
```

If `make state` fails → fix the inconsistency → then continue.

---

## Kill-list — STRICT PREREQUISITE ORDER

```
Stage 1  Hyperbolic embedding stability          [PARTIAL]
Stage 2  Measure-consistent shell routing        [CLOSED]
Stage 3  Hopf angular correctness                [PARTIAL-PASS]
Stage 4  Phase transport usefulness              [PARTIAL-PASS]
Stage 5  Spectral / operator usefulness          [PARTIAL-PASS strong — finalized]
Stage 6  Sparse event-driven trainability        [PARTIAL]  ← ACTIVE GATE
Stage 7  Hardware-efficiency confirmation        [PARTIAL — blocked by Stage 6]
```

Active gate: **RR-067 / INC-0151 closed** — next INC TBD (Stage 6 transition)

**You may not promote Stage 7 while Stage 6 is unresolved unless you:**
1. Write the justification in ACTIVE_STATE.md
2. Write the justification in KILL_LIST_TRACKER.md
3. Pass `make state`

---

## Mathematical architecture (memorize this)

```
Asymmetric H^4 × H^4 product manifold
├── First H^4 = routing geometry
│   ├── Hopf fibration → coarse route address (base R^3)
│   │   ├── shell law    → radial bucket   (STAGE 2 — currently OPEN)
│   │   └── angular law  → sector bucket   (STAGE 3)
│   └── phase transport → geometry-induced motion (STAGE 4)
├── Second H^4 = coupled complex-value field
│   └── phase shift on routing transitions
└── Sparse event routing on top of geometry (STAGE 6)
    └── Hardware cost reduction (STAGE 7)
```

Three objects — keep separate unless a branch proves they should merge:
1. **Routing manifold** (where tokens go)
2. **Transport/state field** (what moves)
3. **Retrieval/key field** (what is looked up)

---

## Mandatory questions before queuing any branch

Answer ALL FOUR or do not queue the branch:

1. Which kill-list stage does this advance?
2. Which mathematical object is being tested?
3. What exact result counts as success? As falsification?
4. Why is this not just packaging or cleanup?

---

## Experiment protocol

```
screen (1 seed) → confirm (2 seeds) → finalize (4 seeds)
```

Screen is never a conclusion. No grid sweeps without a decision gate.

---

## Agent scope rules (hard limits)

| Rule | Detail |
|---|---|
| One branch per session | Only touch `ACTIVE_STATE.md`'s current branch |
| Math description first | Before any formula/param change, state the geometry it changes |
| No silent config choices | Ask when baseline or sector mode is ambiguous |
| No cross-branch edits | Don't modify another RR branch's code |
| No cleanup commits | Renaming/restructuring is not experiment progress |
| Cross-stage observations | Record in SESSION_LEDGER.md, don't chase them |

---

## Source-of-truth priority (highest wins on conflict)

```
CORE_PROJECT_GOALS.md > KILL_LIST_TRACKER.md > ACTIVE_STATE.md
  > ROUTE_MATRIX.md > DECISIONS.md > SESSION_LEDGER.md
```

---

## Recovery after compaction / context loss

```bash
make state         # 1. validate docs
make branch        # 2. print branch context
make bootstrap     # 3. full startup packet
```

Then read: `docs/research/SESSION_LEDGER.md`

---

## GitHub project tracking (issues = stories, milestones = epics)

| Milestone | Kill-list stage | Status |
|---|---|---|
| EPIC-1 | Hyperbolic embedding stability | partial |
| EPIC-2 | Measure-consistent shell routing | closed |
| EPIC-3 | Hopf angular correctness | partial-pass |
| EPIC-4 | Phase transport usefulness | partial-pass |
| EPIC-5 | Spectral / operator usefulness | partial-pass (strong) |
| **EPIC-6** | **Sparse event-driven trainability** | **ACTIVE** |
| EPIC-7 | Hardware-efficiency confirmation | blocked |

When closing an increment: update the corresponding GitHub Issue with the
KEEP / KILL / REFINE decision before pushing.

---

*Detailed instructions: `.github/copilot-instructions.md`*
*Kill-list: `router-research/docs/research/KILL_LIST_TRACKER.md`*
*Git workflow: `router-research/docs/GIT_ONBOARDING.md`*
