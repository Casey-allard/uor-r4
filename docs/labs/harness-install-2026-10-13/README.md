# The harness becomes installable: rules, per-lab setup, and where decisions live

**Question.** The harness works for the labs that already run it. What stops the next lab — or a lab
starting a fresh session — from picking it up without being told anything by hand? Three things were
missing: the branch-cleanup rule was implicit, there was no per-lab install, and the project
manager's decisions had no stated home that the other labs read.

**Line:** no model line — tooling. · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

## 1. Branch removal is now an explicit rule

GitHub's `delete_branch_on_merge` is off and **cannot be turned on** — it needs an organization
admin. So removal is manual, forever, and a rule that lives only in a habit is not a rule. It now
sits in the harness section of [`AGENTS.md`](../../../AGENTS.md), which every lab reads:

```sh
git fetch origin && git checkout main && git reset --hard origin/main
python3 scripts/harness/cli.py git clean-merged          # proof it is safe to delete
python3 scripts/harness/cli.py git clean-merged --apply  # deletes proven-merged local branches
git push origin --delete <branch>                        # the remote copy is yours to remove
```

`clean-merged` proves a branch is merged **by content** — the merged PR's merge-commit tree against
the branch tree — because a squash merge leaves no ancestry link and `merge-base --is-ancestor`
would wrongly call it unmerged. A branch not proven merged is kept, and its owner decides.

The enforcement already existed and now has the rule to point at: `check` reports every orphan as
`status.no_orphan_branches` **with the exact command to fix it**, so a forgotten branch surfaces on
the next run instead of accumulating. Two are outstanding right now — `deepseek/invariant-router`
and `bpb-split`. Both belong to their owners, and the check says so rather than this record deciding
for them.

**Why minimum state matters, in the project's own terms:** a merged branch is not a research
authority. `main` is. Branches are transport.

### The rule was incomplete, and the owner's screenshot showed it

The owner looked at GitHub's branch list and found four branches. They are **two different
problems**, and only one of them is safe to delete:

| branch | state | what it needs |
|---|---|---|
| `deepseek/invariant-router` | PR **#2247 merged**, local copy left behind, checked out in a worktree | remove — the work is in `main` |
| `deepseek/handoff-doc` | PR **#2237 merged**, local copy left behind | remove |
| `claude/starling-language` | **no PR at all**, 2 commits ahead, unmerged | a decision: archive or open a PR |
| `claude/starling-confirm` | **no PR at all**, 3 commits ahead, unmerged | a decision: archive or open a PR |

The merged-PR check could not see the last two, because there is no merge to compare against. They
hold real work — `starling_bootstrap_pure` exists **only** in those branches and in no file on
`main`, and they carry changes to `crates/uor-r4-training/examples/geometric-stack.rs` (+165 lines)
and `src/geometric_stack.rs` (+77) that `main` does not have. **Deleting them would destroy evidence**,
which the owner's standing rule forbids.

So `status.no_orphan_branches` gained a second half: `githelp.remote_branches_without_pr` finds
remote branches with no PR and commits ahead of `main`, and reports each as *"it needs a decision, not
deletion"* with the archive-or-PR fix. `gh-pages` is excluded — GitHub publishes it, it is not
delivery transport, and flagging it would train people to ignore the rule.

```
warn status.no_orphan_branches: remote branch claude/starling-language has no PR and is not merged
     into main (2 commit(s) ahead, 36 behind, last updated 2026-10-12) - it needs a decision, not deletion
  fix: archive it (docs/history/branch-archive/ + an INDEX row) or open a PR; do not delete unmerged
       work
```

Both `claude/*` branches belong to the Claude lab, and a worktree named
`claude/archive-starling-branches-2026-10-13` already exists — the owner is handling them. The
harness reports; it does not delete another lab's unmerged work.

## 2. A per-lab install, in the shared space

[`BOOTSTRAP.md`](../../harness/BOOTSTRAP.md) gained the install, and a machine-local copy lives at
**`~/.local/share/uor-r4/harness/INSTALL.md`** — readable by any lab on this machine without cloning
anything. The repository copy stays authoritative and the local copy says so.

The install is deliberately small:

| | what | why |
|---|---|---|
| **required** | `export UOR_LAB=<kimi\|claude\|codex\|deepseek>` | without it work is recorded but attributed to **nobody** |
| optional | the liveness hook | Kimi's `UserPromptSubmit` hook is the only per-prompt entry point on this machine; the doc gives the shape so another runtime can copy it |
| optional | `git install-hooks` | refuses a commit on a branch another lab has claimed |

Plus the verify sequence, the Python floor (3.9, verified on 3.9.6 and 3.15.0, with the note that
`python3` differs between a login shell and an agent shell on this machine), and where the two state
tiers live.

Every command in the document was **run before it was written down**. Two were wrong on the first
pass — a `--dry-run` flag that does not exist (the real one is `--print`) and `install-hooks` which
is a `git` subcommand — which is exactly the kind of error a lab would have hit while following it.

## 3. Where the project manager's decisions live

Kimi is the project manager, so its decisions have to be findable by the other labs rather than held
in a session. The doc states four places, in order of authority: `DECISIONS.md` for a decision that
binds the project, the milestone issue the labs read each session, `current-state.md` for what
changed, and the dated `docs/labs/` record for a unit of work.

The reverse direction is file-based, because there is no socket or daemon and the retired
`lab-runner coord` must not be recreated: Kimi reads `~/.kimi-code/UOR-R4-HANDOFF.md` at session
start, this lab publishes it, backups sit beside it, and one `cp` reverts them.

It also states the convention that matters most for a PM: **a lab's claims about its own results are
not evidence.** The receipts in `docs/harness/ledger/<lab>-<month>.jsonl` and the merged PR are the
record, and when a milestone turns on a number, read the number in the artifact rather than the
summary in the message.

## 4. A parity bug found by running the check against live state

The parity check from the previous unit found a second real divergence, this time against **live
state rather than a constructed case**:

```
codex   age=11.9m  stale=False   (both agree)
kimi    age=30m25s Rust: live    Python: STALE     <- disagreement
```

Rust computed `age_minutes` as `(now - unix) / 60` and then compared *that* against the TTL, so
30m25s truncated to "30 minutes" and read live. Python keeps the fraction. **Two implementations
disagreeing about whether a lab is live** is precisely the divergence the check exists to catch.

Fixed by judging staleness in **seconds**, never on a truncated minute count, with a boundary test:
30m25s is stale, exactly 30m00s is not.

The checker also had to learn the difference between a bug and jitter. The two implementations read
the clock seconds apart, so a record sitting exactly on the TTL boundary will legitimately flap;
`parity.py` now reports that as a note and only fails when the disagreement is **away** from the
boundary. Without that distinction the check would be flaky and would get ignored, which is worse
than not having it.

## Evidence

```
cargo fmt --check / cargo clippy           -> clean, 0 findings
cargo test -p uor-r4-harness               -> 19 passed + 1 doc-test
python3 -m unittest discover -s tests/harness  -> 278 passed
python3 scripts/harness/parity.py          -> agree on all formats, ledger, presence, 6 cases
python3 scripts/harness/cli.py check       -> 0 errors
python3 scripts/check_claim_wording.py     -> passed
```

## Limitations

- **Installation is documented, not automated.** There is no installer that edits a lab's config for
  it. The identity export is still a line a lab must add, and a lab that skips it is invisible.
- **Only Kimi has a hook**, so only Kimi can be live without a manual check-in. The other labs remain
  per-session, and that is a vendor limitation rather than something the harness can fix.
- The machine-local `INSTALL.md` is **regenerated by hand** and will drift from the repository copy.
  It says the repository wins, but nothing enforces that yet.
- The two orphan branches stay outstanding until their owners remove them; this unit gives them the
  rule and the command, not a cleanup.
