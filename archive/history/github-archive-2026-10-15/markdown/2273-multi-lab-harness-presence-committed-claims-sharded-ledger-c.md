# #2273 — Multi-lab harness: presence, committed claims, sharded ledger, CI reconciliation (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T03:22:46Z
- **Updated:** 2026-10-13T03:25:29Z
- **Closed:** 2026-10-13T03:25:08Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2273
- **Merged:** 2026-10-13T03:25:08Z
- **Merge commit:** 60c82f05a1fbcb3491b9b1cc7ba227428c919bf1
- **Base / Head:** main / UOR-Foundation:harness/multilab
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only: this pull request touches the harness, which runs no model code and cannot move a
headline number. Every figure below measures the harness itself, taken on this machine. The
milestone reference is in the Line declaration above.

## What and why

The harness was written for one lab; four now use it. Three things break at that scale, and
reconnaissance (two read-only subagents) confirmed all three:

**1. Claims were invisible.** They lived in a gitignored `.harness/claims.json`, per worktree — so
lab B could not see that lab A held a branch. A claim nobody can read cannot prevent the collision
it exists to prevent. Claims now live in the committed `docs/harness/claims.json`; a claim
untouched for half a day stops blocking, and only its holder can release it.

**2. Nothing said what a lab was doing.** A receipt only exists after its PR merges.
`heartbeat` now also publishes `docs/harness/presence/<lab>.json` — deliberately, because the
session contract already requires a 90-minute card, so a lab that follows the contract becomes
visible for free and one that never checks in is *visibly absent* rather than silently unknown.
Stale after 30 minutes; an unparseable timestamp reads STALE, never live.

**3. One ledger, every lab appending.** That conflicts on every concurrent PR. I resolved the
union by hand twice in one session, which is why it is now one file per lab per month
(`ledger/<lab>-<month>.jsonl`). `multilab migrate-ledger` moved the existing file across without
losing a line — an unreadable line is quarantined in its own shard, never dropped.

## Result

Decision: **KEEP** (tooling).

**Enforcement, not a dashboard.** `.github/workflows/harness.yml` now reconciles the milestone
registry against the **live** issues using the workflow token, and fails the PR when they
disagree. That is what makes `main` converge no matter which lab opened the pull request. It
reads GitHub and writes nothing.

```
$ python3 scripts/harness/cli.py check --only status.registry_vs_github
harness check: 6 rules, all pass          # live today, so the gate is real, not aspirational
```

**Bootstrap, verified.** No install step, no dependencies:

```sh
export UOR_LAB=<kimi|claude|codex|deepseek>
python3 scripts/harness/cli.py status
```

The real risk turned out to be **interpreter discovery, not version**: `python3` is **3.9.6 in a
login shell** and **3.15.0 in an agent shell**, because `/etc/zprofile` reorders PATH. Both work —
the declared floor is 3.9, all 14 modules load on it, and an older interpreter now gets an explicit
message naming itself and the fix instead of a raw traceback.

**A real performance defect, fixed.** `mistakes.stale_check` walked the entire tree with `rglob`
and ran three `fnmatch` calls per (path, pattern) pair for every lesson — with `research/` at
11,171 files that was **46.5 seconds on Python 3.9**, dominating the whole `check`. It now uses
`git ls-files` (the index that already exists) and resolves literal paths before attempting any
glob:

```
mistakes.stale_check on Python 3.9:   46.5s -> 0.43s
full suite on Python 3.9:             205s  -> 20s
```

```
$ python3 -m unittest discover -s tests/harness -t .     -> 262 tests, OK
$ /usr/bin/python3 -m unittest discover -s tests/harness -t . -> 262 tests, OK  (3.9.6)
```

## Checklist

- [x] Checks run at this exact head (above; suite passes on both interpreters)
- [x] No model compute; the harness is tooling and cannot move a headline
- [x] `python3 scripts/check_claim_wording.py` passes
- [x] Research record: docs/labs/multilab-harness-2026-10-12/README.md
- [x] New entry at the top of docs/integration/current-state.md, with **Next:**
- [x] Bootstrap documented in docs/harness/README.md and verified from a foreign cwd
- [x] Branch deleted after merge, per the owner's direction (auto-delete is off for this repo)

### Stated rather than hidden

- **"The mesh" has no referent in this repository.** A tracked search finds only geometric meshes,
  `THREE.Mesh`, and two unrelated service-mesh catalog entries. There is no coordination mesh, and
  the retired `lab-runner coord` is explicitly not to be recreated (`docs/labs/protocol.md`).
- **This is not real-time and cannot be.** Threads, sessions and context are per-vendor and private.
  GitHub is the bus, `main` is the truth, and a lab that never runs the harness is simply absent.
- **Claims report; they do not lock.** A filesystem cannot stop another vendor's CLI.
- 11 pre-existing receipts are attributed to nobody (`unknown-<month>.jsonl`). They are preserved,
  not deleted, but nothing can recover which lab wrote them.
- Two orphan branches remain (`bpb-split`, `deepseek/invariant-router`); `bpb-split` holds 375
  lines not in `main` and is another lab's work.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

