# Git Onboarding for AI-Router Research

This guide explains how every research step is encoded as a git branch and
how to navigate the project without reading dozens of markdown files manually.

---

## 1  Core idea

> **The branch name is the canonical project path.**

Each active experiment lives on a branch named `codex/RR-###-slug`.
Checking out a branch automatically prints your research context (active kill-list
stage, increment doc status, current queue) so you always know where you are.

---

## 2  First-time setup (clone → ready)

```bash
git clone <repo-url>
cd ai-router
bash setup_hooks.sh      # one-time: activates versioned git hooks
make state               # verify canonical docs are consistent
make bootstrap           # print the full startup context packet
```

`setup_hooks.sh` runs `git config core.hooksPath .githooks`. This is idempotent;
re-run it any time after switching machines.

---

## 3  Branch naming convention

```
codex/RR-###-<slug>
```

| Part | Meaning |
|---|---|
| `codex/` | all research branches live under this namespace |
| `RR-###` | the issue-registry ID that owns this branch (three-digit, zero-padded) |
| `<slug>` | short kebab-case description of the approach under test |

Examples:
```
codex/RR-061-measure-consistent-route-law
codex/RR-064-coupled-complex-phase-transport
codex/RR-069-shell-pressure-geodesic-blend
```

---

## 4  Starting a new increment

The preferred workflow uses `make new-inc`:

```bash
make new-inc RR=069 INC=0138 SLUG=shell-pressure-geodesic-blend
```

This calls `python tools/git_research.py new 069 shell-pressure-geodesic-blend --inc 0138`,
which:

1. Creates `codex/RR-069-shell-pressure-geodesic-blend` from the current HEAD.
2. Creates `docs/research/increments/INC_0138_shell_pressure_geodesic_blend.md`
   pre-filled with all required sections (kill-list stage, math object, success/
   falsification conditions) derived from the current ACTIVE_STATE.
3. Checks out the new branch (which triggers the `post-checkout` hook, printing
   the fresh context panel).

After the scaffolding is generated:
- Fill in the specific math object and hypothesis in the increment doc.
- Update `ACTIVE_STATE.md` to point at the new RR/INC.
- Run `make state` to confirm all canonical docs agree.

---

## 5  What happens when you check out a branch

The `post-checkout` hook runs `python tools/git_research.py status` and prints:

```
============================================================
  BRANCH   : codex/RR-061-measure-consistent-route-law
  ISSUE    : RR-061 — Measure-Consistent Shell Routing
  LABELS   : [active, kill-list-stage-2]
  INC DOC  : docs/research/increments/INC_0137_...md
  STAGE    : measure-consistent shell routing
  STATUS   : Queued next.

  QUEUE    : RR-061 / INC-0137
  STAGE    : measure-consistent shell routing
============================================================
  Quick commands:
    make state      # validate all canonical docs
    make bootstrap  # print full startup context
    make test       # run test suite
```

If the branch RR does not match the canonical active queue a **WARNING** line is
printed so you know immediately before working on stale or future work.

---

## 6  Commit message template

When you commit on a `codex/RR-###-*` branch the `prepare-commit-msg` hook
pre-fills the message with `[RR-###]` and a set of guided comment lines:

```
[RR-061] 
# Branch: codex/RR-061-measure-consistent-route-law
# ---
# Describe what this commit contains.
# Examples:
#   [RR-061] Add shell-pressure blend config + screen result
#   [RR-061] Fix diagnostic output for geodesic overlap
```

Delete the instructional comments and replace `# <describe>` with a concise
one-line summary. This keeps history parseable by `tools/git_research.py`.

---

## 7  Pre-push validation

Before `git push` the `pre-push` hook runs `check_research_state.py` and
**blocks the push** if:

- Canonical docs disagree on the primary RR/INC.
- The active increment doc is missing a required section.
- More than one increment doc is in `Queued next.` status.

To bypass in an emergency:

```bash
git push --no-verify
```

Never bypass without fixing the state afterwards.

---

## 8  Makefile quick reference

Run `make help` for a live list. Key targets:

| Target | What it does |
|---|---|
| `make state` | Validate all canonical research docs |
| `make bootstrap` | Print the full startup context packet |
| `make test` | Run the full test suite |
| `make branch` | Print context for the current branch |
| `make branches` | List all `codex/` branches with their kill-list stages |
| `make baseline` | Run the kmeans baseline route |
| `make pipeline` | Full sweep → parse → summarize |
| `make parse` | Parse raw result logs |
| `make summarize` | Rebuild `results/summary.csv` |
| `make new-inc RR=... INC=... SLUG=...` | Create new branch + increment doc |
| `make onboarding` | Print this onboarding doc and then `make state` |

---

## 9  Closing a branch

When an increment reaches a KEEP / KILL / REFINE decision:

1. Update the increment doc `## Status` to `Closed: KEEP.` / `Closed: KILL.` /
   `Closed: REFINE.` and fill in the decision rationale.
2. Update `ACTIVE_STATE.md` to point at the next RR/INC.
3. Update `KILL_LIST_TRACKER.md` if the stage verdict changed.
4. Run `make state` — it must pass before merging.
5. Open a PR from `codex/RR-###-slug` → `main` using the PR template.
6. After merge, the branch may be kept or deleted; the increment doc persists.

---

## 10  Source of truth ordering

When docs conflict, the priority is:

```
CORE_PROJECT_GOALS.md
  └─ PROJECT_CONTEXT.md
       └─ KILL_LIST_TRACKER.md
            └─ ACTIVE_STATE.md
                 └─ ROUTE_MATRIX.md
                      └─ DECISIONS.md
```

Most recent edit wins within the same priority level.

---

## 11  Key tools

| Script | Purpose |
|---|---|
| `tools/git_research.py` | Branch management automation (called by hooks) |
| `tools/check_research_state.py` | Canonical cross-doc consistency validator |
| `tools/context_bootstrap.py` | Prints ordered startup context packet |
| `tasks/router_proxy_eval.py` | Main experiment harness |
| `tasks/router_retrieval_eval.py` | Retrieval evaluation harness |
| `hyperbolic_router_so8.py` | Core router implementation |

---

_See also: `docs/program/BRANCHING_STRATEGY.md` for the full design rationale
and `AGENTS.md` for automated agent non-negotiables._
