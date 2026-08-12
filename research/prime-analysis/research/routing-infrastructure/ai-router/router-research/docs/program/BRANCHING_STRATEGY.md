# Branching Strategy

## Purpose
Use git branches to separate hypothesis families, systems rescues, and packaging work so evidence stays reviewable. **The branch name is the canonical project path** — checking out a branch prints your full research context automatically via git hooks.

## Automation infrastructure

All branching automation lives at the repo root. After cloning:

```bash
bash setup_hooks.sh     # one-time: activates .githooks/ versioned hooks
make help               # see all available targets
make state              # validate canonical doc consistency
make bootstrap          # print full startup context
```

### Versioned git hooks (`.githooks/`)

| Hook | Trigger | What it does |
|---|---|---|
| `post-checkout` | any `git checkout` switching branches | prints the research context panel for the new branch |
| `prepare-commit-msg` | any `git commit` | pre-fills `[RR-###]` prefix + guided artifact template |
| `pre-push` | any `git push` | blocks push if canonical docs are inconsistent |

Hooks are tracked in `.githooks/` (not `.git/hooks/`) so they survive clones
and are version-controlled. `setup_hooks.sh` sets `core.hooksPath = .githooks`.

### `tools/git_research.py`

The automation brain. Called by hooks; also exposes a direct CLI:

```bash
python tools/git_research.py status    # current branch context panel
python tools/git_research.py list      # all codex/ branches + kill-list stages
python tools/git_research.py validate  # full state check
python tools/git_research.py new <rr> <slug> [--inc <inc>]
```

### `Makefile` (repo root)

Single entry point wrapping all the above. Prefer `make <target>` over direct
script calls in documentation and instructions.

---

## Default Pattern
- `main`
  - current reproducible merge-worthy frontier
  - receives merges only from `codex/*` branches that have passed `make state`
- `codex/RR-###-short-slug`
  - one active issue / increment per branch
  - created via `make new-inc RR=### INC=#### SLUG=slug`

## Branch Types

| Type | Example | Rules |
|---|---|---|
| Research increment | `codex/RR-061-measure-consistent-route-law` | one kill-list stage per branch |
| Deep math | `codex/RR-050-dynamic-h4-state` | may diverge harder; no unrelated systems work |
| Systems packaging | `codex/RR-053-index-reuse-packaging` | only after a research result justifies operationalization |

## Rules
1. One hypothesis family per branch.
2. One merge-worthy decision per branch when possible.
3. Do not mix geometry-law changes with systems-harness changes unless the increment explicitly requires both.
4. Every branch must map back to one `RR-###` issue in `docs/program/ISSUE_REGISTRY.md`.
5. Create branches using `make new-inc` — it scaffolds the increment doc and fires the context hook.
6. Before merge, the following must be updated and pass `make state`:
   - increment doc `## Status` → `Closed: KEEP/KILL/REFINE.`
   - `docs/research/ACTIVE_STATE.md` → points at the next RR/INC
   - `docs/research/KILL_LIST_TRACKER.md` → stage verdict if changed
   - `docs/program/ISSUE_REGISTRY.md` and `docs/program/PROJECT_BOARD.md`
   - `docs/DECISIONS.md`

## Merge Rule
Merge only when the branch has:
- config path (JSON in `configs/`)
- at least one analysis artifact (log + parsed JSON + summary row)
- an explicit keep/kill/refine decision recorded in the increment doc

## Why This Helps
- Keeps translated retrieval work from contaminating deep geometry branches.
- Makes agent fleets parallelizable — each branch is a self-contained unit.
- Makes post-compaction recovery trivial: `git checkout <branch>` instantly
  surfaces the context panel without reading any docs manually.
- Canonical push validation (`pre-push` hook) prevents shipping inconsistent state.

