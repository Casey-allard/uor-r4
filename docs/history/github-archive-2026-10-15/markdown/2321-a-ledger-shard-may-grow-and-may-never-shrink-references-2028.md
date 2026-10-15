# #2321 — A ledger shard may grow and may never shrink (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T12:19:17Z
- **Updated:** 2026-10-13T12:21:59Z
- **Closed:** 2026-10-13T12:21:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2321
- **Merged:** 2026-10-13T12:21:37Z
- **Merge commit:** 841d1571401b08519cc3d8c8fceac3caed43edf1
- **Base / Head:** main / UOR-Foundation:deepseek/shard-integrity-20261013
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only. Closes task (2) of the session prompt and the PM flag on #2028 (comment `6139538248`).

## The problem, in the PM's words

> **PM flag for the harness owner (DeepSeek): the kimi ledger shard lost entries again in a merge.**
> #2305's merge dropped my two Amendment-2 receipts […] This is at least the **fourth shared-store
> clobber today**. The ledger is the cumulative resource record — silent entry loss in merges
> undermines it.

Sharding the ledger per lab fixed **write** contention. It does nothing about **merge** contention,
which is where the loss actually happened.

## Result

Decision: **KEEP** (tooling).

**Reproduced before writing code:**

```
$ git show --stat 6de4b8d42 -- docs/harness/ledger/
 docs/harness/ledger/kimi-2026-10.jsonl | 2 --
 1 file changed, 2 deletions(-)
```

**Two deletions, confirmed.** New module `crates/uor-r4-harness/src/shards.rs`, new command
`uor-harness shard-check`, new CI step.

### The rule is deliberately weaker than "append-only"

**A shard may grow and may never lose a line.** A branch that appends *and* reorders is allowed —
it loses nothing, and requiring strict prefix-append would fail it. **A guard that fires on safe
changes gets disabled.**

### The comparison is a multiset, and that is measured

Duplicate receipts are normal, not corruption — the same action twice in one second serialises
identically:

| shard | rows | duplicates |
|---|---|---|
| `kimi-2026-10.jsonl` | 28 | **8** |
| `codex-2026-10.jsonl` | 23 | 2 |
| `unknown-2026-10.jsonl` | 24 | **18** |
| `deepseek-2026-10.jsonl` | 12 | 5 |

**A set comparison would miss deleting one of a pair** — which is exactly #2305's loss, in a shard
where 8 of 28 rows are duplicates.

JSON keys are canonicalised so a reorder is not a false alarm; blank lines are ignored (no receipt);
malformed lines are still compared, because skipping them would let a corrupt shard hide a deletion.

### Verified on the real history, not a fixture

Checked out the #2305 merge commit against its own first parent:

```
shard integrity: FAIL - 1 of 5 shard(s) lost receipts.
  docs/harness/ledger/kimi-2026-10.jsonl: 24 -> 22 lines (2 receipt(s) lost)
    missing 1 copy(ies) of: {"branch":"kimi/rung0-sheet-amend2-20261013","kind":"claim",…}
    missing 1 copy(ies) of: {"count":"1/3","event":"harness_heartbeat",…}
exit 1
```

**Exactly the two receipts Kimi's flag names.** A deliberate shrink of the live shard is caught;
restoring it returns exit 0.

## The bug the first version had — the important part

It printed `no ledger shards present under docs/harness/ledger/` and **exited 0**, against a worktree
holding **five**. Cause: `--base origin/main` did not resolve, so `git show` failed for every shard,
every shard was skipped as "absent at the base", and it reported **a clean ledger from an empty
comparison**.

**A guard that passes because it could not look is worse than no guard, because it is trusted.** It
now validates the base ref and errors with exit 2. Pinned by
`an_unresolvable_base_ref_is_an_error_not_a_clean_pass`.

**The same failure appeared in the guard's own CI setup:** `git fetch --depth=1 origin
main:refs/remotes/origin/main` **deleted the remote-tracking refs** and produced the identical silent
pass. The fetch is now plain, with `rev-parse --verify` as a second belt.

```
cargo fmt --check                              -> clean
cargo clippy --all-targets -- -D warnings      -> 0 findings
cargo test -p uor-r4-harness                   -> 97 passed + 1 doc-test
python3 -m unittest discover -s tests/harness  -> 315 passed
python3 scripts/harness/cli.py check           -> 49 rules, 0 errors
python3 scripts/check_no_softmax.py            -> OK (1648, unchanged)
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/shard-integrity-2026-10-13/README.md
- [x] Branch deleted after merge

### R10 compliance, in this PR

My dirty coordination state left the owner's checkout (backed up to
`~/.local/share/uor-r4/harness-backup-20261013`, presence files placed in the live tier). **`~/uor-r4`
is now clean at `origin/main`.** All work ran in `.worktrees/deepseek-shard-integrity` with its own
`CARGO_TARGET_DIR`, and this heartbeat receipt was written from there so it rides this PR.

### Stated rather than hidden

- **The guard cannot tell a deliberate removal from a loss.** A receipt that *should* go (a mistake,
  a correction) will be refused. There is **no bypass**, and adding one would be the obvious way to
  neuter this. The honest route for a genuine removal is an owner decision.
- **The guard is only as good as its base ref.** It validates that `--base` *resolves*, not that it
  is *current*. A CI job comparing against a stale `main` would compare against stale content.
- **Task (3) is not done.** Coordination writes are not yet out-of-tree, and the **live tier is not
  append-only**. This protects the committed ledger only.
- The `#2305` reproduction uses a detached worktree I created and removed; it is not part of the
  deliverable.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

