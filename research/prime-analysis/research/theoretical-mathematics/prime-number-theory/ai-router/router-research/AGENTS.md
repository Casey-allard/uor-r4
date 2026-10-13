# Agent Rules — Full Reference

> **Quick-read version:** see `../AGENTS.md` (repo root) or `../.github/copilot-instructions.md`.
> This file is the complete deep reference. When the quick-read and this file
> disagree, this file wins within the `router-research/` subtree.
> When this file and `CORE_PROJECT_GOALS.md` disagree, `CORE_PROJECT_GOALS.md` wins always.

---

## The mission in one sentence

Prove or falsify that fixed H^4 × H^4 hyperbolic geometry can replace the
dense linear-lattice routing in transformer-style LLMs — targeting 10×–100×
hardware savings.

**Mathematics governs every decision in this project.**
Before changing any formula, parameter, sector mode, or routing law, state
in writing which mathematical object is being changed and what the change
means geometrically.

---

## Git-first non-negotiables  ← READ THESE FIRST

- **The branch name is the canonical project path.** Do not work on `main`.
- On every fresh clone, run `bash setup_hooks.sh` before any research work.
  This activates the versioned git hooks in `.githooks/`.
- Use `make state` (not direct script calls) to validate research consistency.
- Use `make branch` or `make bootstrap` to reload context instead of manually
  reading dozens of markdown files.
- Create new branches with:
  ```
  make new-inc RR=<rr> INC=<inc> SLUG=<slug>
  ```
  Never `git checkout -b` a research branch manually — the Makefile target
  also creates the increment doc skeleton and fires the context hook.
- Commit messages on `codex/*` branches must begin with `[RR-###]`. The
  `prepare-commit-msg` hook provides the template automatically.
- `git push` is blocked by `pre-push` if canonical docs are inconsistent.
  Fix state to pass `make state` before pushing. Never use `--no-verify`
  without immediately filing a follow-up commit to repair state.
- Full onboarding guide: `docs/GIT_ONBOARDING.md`.

## Non-negotiables
- Keep diffs small and PR-shaped.
- Always keep the repo runnable.
- Start every resumed research session with `make bootstrap` (replaces
  manually reading `docs/SESSION_BOOTSTRAP.md`).
- Always reload `CORE_PROJECT_GOALS.md` before choosing the next research step.
- Treat `docs/research/ACTIVE_STATE.md` as the single live queue authority.
- Treat `docs/research/KILL_LIST_TRACKER.md` as the single kill-list status authority.
- Maintain `docs/research/SESSION_LEDGER.md` during long investigations,
  branch pivots, and post-compaction recovery.
- Never use `CURRENT_DIRECTION.md`, `HANDOFF_CURRENT.md`, or a single increment doc
  as the only source of truth.
- If `PROJECT_CONTEXT.md`, `KILL_LIST_TRACKER.md`, `ACTIVE_STATE.md`,
  `ROUTE_MATRIX.md`, and handoff/current docs disagree,
  reconcile that conflict before changing code or rewriting docs.
- After compaction or context loss, rerun in order:
  1. `make state`          — validates canonical docs
  2. `make branch`         — prints current branch context
  3. `make bootstrap`      — prints full startup context packet
  4. Read `docs/research/SESSION_LEDGER.md`
- Before queuing a new branch, be able to say:
  - which kill-list stage it advances
  - which mathematical object it tests
  - why it is not just packet/contract cleanup
- If a later translated, packet, or downstream evaluation branch is active
  while an earlier kill-list gate remains unresolved, explicitly justify that
  choice in the queue docs before continuing.
- Every run produces:
  - a log in `results/raw/`
  - a parsed JSON in `results/parsed/`
  - an appended row in `results/summary.csv`
- Add/update `docs/DECISIONS.md` when conclusions change.

## Experiment protocol
- Screen (1 seed) → confirm (2 seeds) → finalize (4 seeds).
- No massive grid sweeps unless there is a decision gate.
- A screen result is **never** a conclusion.
- A confirm result is not a conclusion until the increment doc records an
  explicit KEEP / KILL / REFINE decision with a one-sentence rationale.

## Primary mission
Prove or falsify geometry-native routing as a hardware-relevant alternative to
dense transformer-style routing.

Secondary mission:
- cut runtime
- automate experiments
- keep the human out of the terminal

Those are support goals, not replacements for the core project thesis.

---

## Mathematical objects — reference card

Before touching any code or config, identify which object you are changing:

| Object | What it is | Where it lives in code |
|---|---|---|
| **Routing manifold** | H^4 Poincaré ball with shell/sector bucketing | `hyperbolic_router_so8.py` |
| **Shell law** | maps ball radius → radial bucket; must be H^4-measure-consistent | `get_bucket()`, shell configs |
| **Hopf angular law** | maps Hopf-base projection → sector bucket | `phase4d_hopf_*` sector modes |
| **Coupled field** | second H^4; stores complex-plane values per bucket | SO(d) chart cache |
| **Phase transport** | phase shift on routing transition; geometry-induced | `phase_transport_*` configs |
| **Spectral structure** | operator eigenstructure over the route graph | `tasks/spectral_*` |
| **Sparse event gate** | threshold on route-change magnitude | `event_gate_*` in eval tasks |
| **Hardware cost surface** | flops/bandwidth curve vs dense baseline | Translation eval harness |

The three objects that **must stay conceptually separate**:
1. Routing manifold (where tokens route to)
2. Transport/state field (what moves along the route)
3. Retrieval/key field (what is indexed for lookup)

Only merge them if a branch explicitly proves they should be merged.

---

## Kill-list status (always verify against KILL_LIST_TRACKER.md)

| Stage | Description | Status |
|---|---|---|
| 1 | Hyperbolic embedding stability | PARTIAL |
| 2 | Measure-consistent shell routing | CLOSED |
| 3 | Hopf angular correctness | PARTIAL-PASS |
| 4 | Phase transport usefulness | PARTIAL-PASS |
| 5 | Spectral / operator usefulness | PARTIAL-PASS (strong, finalized) |
| **6** | **Sparse event-driven trainability** | **PARTIAL ← active gate** |
| 7 | Hardware-efficiency confirmation | PARTIAL |

Active: `RR-067 / INC-0151 closed` — next INC TBD — Milestone EPIC-6

---

## Mandatory pre-branch questions

Answer all four explicitly in `ACTIVE_STATE.md` before opening a branch:

1. Which kill-list stage does this branch advance?
2. Which mathematical object is being tested?
3. What exact result counts as **success**? As **falsification**?
4. Why is this not just packaging, cleanup, or contract work?

If you cannot answer all four, the branch should not be opened.

---

## Agent scope hard limits

| Limit | Rule |
|---|---|
| Branch scope | Work only on `ACTIVE_STATE.md`'s current branch |
| Math-first | State the mathematical object and geometric meaning before any formula change |
| No silent choices | Ask when config, baseline, or sector mode is ambiguous |
| No cross-branch edits | Never touch another `codex/*` branch's code |
| No cleanup commits | File renames / restructuring do not appear in results commits |
| Cross-stage observations | Write to SESSION_LEDGER.md — do not chase them on this branch |
| Prerequisite chain | Never promote a later stage past an open earlier gate without written justification in queue docs |

---

## Cross-pollination protocol

This project has 7 interconnected stages. A result from Stage 4 can change
what Stage 2 needs to prove. When you observe something that bears on a
different stage:

1. Do **not** diverge from the current increment.
2. Write a note in `docs/research/SESSION_LEDGER.md` under the heading:
   `## Cross-stage observation: Stage N → Stage M`
3. Include: what you observed, which branch produced it, why it's relevant,
   and what should be verified.

The human researcher reviews these during increment close.

---

## Definition of done — increment

An INC-#### is closed only when ALL of the following are true:

- [ ] Increment doc `## Status` = `Closed: KEEP.` / `Closed: KILL.` / `Closed: REFINE.`
      with a one-sentence decision rationale
- [ ] `docs/research/ACTIVE_STATE.md` points at the next RR/INC
- [ ] `docs/research/KILL_LIST_TRACKER.md` updated if stage verdict changed
- [ ] `docs/DECISIONS.md` has a new entry
- [ ] GitHub Issue for this RR updated with the decision
- [ ] `make state` passes

---

## Drift detection — stop immediately if any of these apply

- The next branch has no kill-list stage mapping in ROUTE_MATRIX.md
- The next experiment tweaks a baseline that already failed (DECISIONS.md)
- Three or more consecutive branches touched only translation/packaging
- The phrase "let's just try" appears without a written falsification condition
- An increment is being queued for Stage 3+ while Stage 2 is still open, with
  no written justification in ACTIVE_STATE.md

When drift is detected: re-read `CORE_PROJECT_GOALS.md` in full before
taking any action.

---

## Source-of-truth priority

When docs conflict, the highest-ranked document wins:

```
CORE_PROJECT_GOALS.md         ← mathematical mission — never overridden
  KILL_LIST_TRACKER.md        ← stage status
    ACTIVE_STATE.md           ← current queue
      ROUTE_MATRIX.md         ← branch-to-stage mapping
        DECISIONS.md          ← closed decisions
          SESSION_LEDGER.md   ← in-session working notes (temporary)
```

---

## Key files reference

| File | Purpose |
|---|---|
| `CORE_PROJECT_GOALS.md` | Mathematical mission, drift warnings, branch checklist |
| `docs/research/KILL_LIST_TRACKER.md` | Stage-by-stage status and blockers |
| `docs/research/ACTIVE_STATE.md` | Live queue: current RR + INC |
| `docs/research/ROUTE_MATRIX.md` | Branch-to-stage mapping |
| `docs/DECISIONS.md` | All closed decisions |
| `docs/research/SESSION_LEDGER.md` | In-session working notes |
| `docs/GIT_ONBOARDING.md` | Git workflow guide |
| `tools/git_research.py` | Branch automation brain |
| `tools/check_research_state.py` | Canonical state validator |
| `hyperbolic_router_so8.py` | Core router (routing + chart + growth) |
| `tasks/router_proxy_eval.py` | Primary experiment harness |
| `tasks/router_retrieval_eval.py` | Retrieval evaluation harness |
