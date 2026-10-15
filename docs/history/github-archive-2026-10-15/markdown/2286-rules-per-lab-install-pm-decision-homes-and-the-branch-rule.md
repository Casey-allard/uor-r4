# #2286 — Rules, per-lab install, PM decision homes, and the branch rule completed (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T04:21:07Z
- **Updated:** 2026-10-13T04:24:21Z
- **Closed:** 2026-10-13T04:22:48Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2286
- **Merged:** 2026-10-13T04:22:48Z
- **Merge commit:** be742611ad5c71f0aecaefe928f97d1ad56c2ecb
- **Base / Head:** main / UOR-Foundation:harness/install-and-branch-rule
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only.

## Your three directions, and what they found

### 1. Branch removal is now a rule

GitHub's `delete_branch_on_merge` is off and cannot be turned on without an org admin, so removal is
manual forever — and a habit is not a rule. It now sits in `AGENTS.md`, which every lab reads, with
the exact sequence. `check` already enforced it as `status.no_orphan_branches` with the fix.

**But the rule was incomplete, and your screenshot showed it.** Four branches, two different problems:

| branch | state | what it needs |
|---|---|---|
| `deepseek/invariant-router` | PR **#2247 merged**, local copy left | remove |
| `deepseek/handoff-doc` | PR **#2237 merged**, local copy left | remove |
| `claude/starling-language` | **no PR at all**, 2 ahead, unmerged | a decision |
| `claude/starling-confirm` | **no PR at all**, 3 ahead, unmerged | a decision |

The merged-PR check cannot see the last two — there is no merge to compare against. They hold real
work: `starling_bootstrap_pure` exists **only** in those branches and nowhere on `main`, plus +165
lines in `examples/geometric-stack.rs` and +77 in `src/geometric_stack.rs`. **Deleting them would
destroy evidence.**

So the rule gained a second half — `githelp.remote_branches_without_pr` finds remote branches with no
PR and commits ahead of `main`, and reports each as *"it needs a decision, not deletion"*:

```
warn status.no_orphan_branches: remote branch claude/starling-language has no PR and is not merged
     into main (2 commit(s) ahead, 36 behind, last updated 2026-10-12) - it needs a decision, not deletion
  fix: archive it (docs/history/branch-archive/ + an INDEX row) or open a PR; do not delete unmerged work
```

`gh-pages` is excluded: GitHub publishes it, it is not delivery transport, and flagging it would train
people to ignore the rule. Both `claude/*` branches belong to the Claude lab and a worktree named
`claude/archive-starling-branches-2026-10-13` already exists for them — the harness reports, it does
not delete another lab's unmerged work.

### 2. Install instructions in the shared space

[`BOOTSTRAP.md`](docs/harness/BOOTSTRAP.md) gained the per-lab install, and a machine-local copy
lives at **`~/.local/share/uor-r4/harness/INSTALL.md`** — readable by any lab on this machine without
cloning anything. The repository copy stays authoritative and the local copy says so.

One required line (`export UOR_LAB`), the optional liveness hook whose shape another runtime can
copy, the optional pre-commit branch check, a verify sequence, the 3.9 floor with the note that
`python3` differs between a login shell and an agent shell, and where the two state tiers live.

**Every command was run before it was written down**, which caught two errors a lab would have hit
following it: a `--dry-run` flag that does not exist (the real one is `--print`), and `install-hooks`,
which is a `git` subcommand.

### 3. Where the project manager's decisions live

Four places, in order of authority, now stated in the install doc: `DECISIONS.md` for a decision that
binds the project, the milestone issue the labs read each session, `current-state.md`, and the dated
`docs/labs/` record. The reverse direction is file-based — Kimi reads
`~/.kimi-code/UOR-R4-HANDOFF.md` at session start, this lab publishes it, backups sit beside it, one
`cp` reverts — because there is no socket and the retired `lab-runner coord` must not come back.

It also states the convention that matters most for a PM: **a lab's claims about its own results are
not evidence.** The receipts and the merged PR are the record; read the number in the artifact.

## Result

Decision: **KEEP** (tooling). A second parity bug was also caught, against live state this time: Rust
truncated `age_minutes` before comparing, so a record aged 30m25s read **live** while Python read
**STALE** — two implementations disagreeing about whether a lab is live. Judged in seconds now, with a
boundary test. `parity.py` also learned to separate that class of bug from clock jitter: it notes a
disagreement within 60s of the TTL boundary and only fails when the two disagree away from it, because
a flaky check gets ignored.

```
cargo fmt --check / cargo clippy          -> clean, 0 findings
cargo test -p uor-r4-harness              -> 19 passed + 1 doc-test
python3 -m unittest discover -s tests/harness  -> 281 passed on 3.9.6 AND 3.15.0
python3 scripts/harness/parity.py         -> agree on all formats, ledger, presence, 6 cases
python3 scripts/harness/cli.py check      -> 0 errors
python3 scripts/check_claim_wording.py    -> passed
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling and rules only; no capability or headline changed
- [x] Record: docs/labs/harness-install-2026-10-13/README.md, plus a current-state entry
- [x] Branch deleted after merge — per the rule this PR adds

### Stated rather than hidden

- **The install is documented, not automated.** No installer edits a lab's config. The identity
  export is still a line a lab must add, and a lab that skips it stays invisible.
- **Only Kimi has a per-prompt hook**, so only Kimi can be live without a manual check-in. The other
  labs remain per-session; that is a vendor limitation, not something the harness can fix.
- The machine-local `INSTALL.md` is **regenerated by hand** and will drift from the repository copy.
  It says the repository wins, but nothing enforces that yet.
- **I did not delete any of the four branches.** Two are another lab's unmerged work, and the other
  two are local copies I confirmed are safe — but they are not mine to remove, and `clean-merged`
  correctly declines to touch them. My own branches are removed as always.
- The Rust crate and `parity.py` are still **outside CI**; wiring them in is the next unit.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

