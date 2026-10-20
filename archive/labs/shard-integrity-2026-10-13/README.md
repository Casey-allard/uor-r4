# Shard integrity — a ledger shard may grow and may never shrink

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** #2028 comment `6139538248` (PM flag), pm-rules R10, task (2) of the session prompt
**Delivered by:** PR #2321

## The problem, in the owner's and the PM's words

Kimi's flag on #2028:

> **PM flag for the harness owner (DeepSeek): the kimi ledger shard lost entries again in a merge.**
> #2305's merge dropped my two Amendment-2 receipts from `docs/harness/ledger` (kimi shard); the
> #2306 merge resolution restored them as an append-only union (shard is now the complete 26-line
> record). This is at least the **fourth shared-store clobber today** across claims/ledger. The
> ledger is the cumulative resource record — silent entry loss in merges undermines it.

R10's "Why" names the same day: *"the claims store clobbered three times by concurrent writers, and a
ledger shard silently losing receipts in a merge (#2305)."*

Sharding the ledger per lab fixed **write** contention. It does nothing about **merge** contention,
which is where the loss actually happened: two branches that each append to the same shard conflict,
and a resolution can keep one side.

## What was measured, not assumed

Reproduced against the real history before writing any code:

```
$ gh pr view 2305 --json mergeCommit -q .mergeCommit.oid
6de4b8d425173e1fe6c1d0aa72711e734d8ce080

$ git show --stat 6de4b8d42 -- docs/harness/ledger/
 docs/harness/ledger/kimi-2026-10.jsonl | 2 --
 1 file changed, 2 deletions(-)
```

**Two deletions, confirmed.** And duplicate receipts are normal here, not corruption — the shards hold
them because the same action recorded twice in one second serialises identically:

| shard | rows | duplicates |
|---|---|---|
| `kimi-2026-10.jsonl` | 28 | 8 |
| `codex-2026-10.jsonl` | 23 | 2 |
| `unknown-2026-10.jsonl` | 24 | 18 |
| `deepseek-2026-10.jsonl` | 12 | 5 |

That table is why the comparison is a **multiset**. A set comparison would miss deleting one of two
identical receipts — the exact loss #2305 caused, in a shard where 8 of 28 rows are duplicates.

## What was built

`crates/uor-r4-harness/src/shards.rs` — new module, 97 Rust tests total (11 added here).

**The rule: a shard may grow, and may never lose a line.** Weaker than "append-only" in the letter —
a branch that appends *and* reorders is allowed — and deliberately so. The property that matters is
that no receipt disappears; requiring strict prefix-append would fail a branch that legitimately
resorted a shard, and **a guard that fires on safe changes gets disabled.**

Design decisions, each with the reason it is not the obvious alternative:

- **Multiset, not set** — see the duplicate table above.
- **Canonical JSON, sorted keys** — a merge resolution or a different serialiser can reorder keys
  without changing the receipt. Comparing raw text would call that one loss and one addition.
- **Blank lines ignored** — they carry no receipt; dropping a trailing newline is not data loss.
- **Malformed lines still compared** — a non-JSON line falls back to its trimmed text. Skipping it
  would let a corrupt shard hide a deletion.
- **A shard absent at the base is skipped** — a new shard cannot shrink.

### The bug the first version had, which is the important part

The first run reported:

```
shard integrity: no ledger shards present under docs/harness/ledger/
exit 0
```

against a worktree where **five shards were present**. Cause: `--base origin/main` did not resolve
(a shallow fetch had deleted the ref), so `git show` failed for every shard, every shard was skipped
as "absent at the base", and the command **reported a clean ledger from an empty comparison**.

**A guard that passes because it could not look is worse than no guard**, because it is trusted. The
fix validates the base ref first and returns an **error** (exit 2) when it does not resolve:

```
$ uor-harness shard-check --repo . --base no-such-ref
uor-harness: base ref "no-such-ref" does not resolve in … - refusing to report a ledger as clean
without a comparison point (`git fetch origin main` first, or pass --base HEAD)
```

A test pins it: `an_unresolvable_base_ref_is_an_error_not_a_clean_pass`.

## Verification

**The guard catches the exact #2305 loss.** Checked out the merge commit and ran against its own
first parent:

```
$ uor-harness shard-check --repo . --base 96c0ccb0a
shard integrity: FAIL - 1 of 5 shard(s) lost receipts.

  docs/harness/ledger/kimi-2026-10.jsonl: 24 -> 22 lines (2 receipt(s) lost)
    missing 1 copy(ies) of: {"branch":"kimi/rung0-sheet-amend2-20261013","kind":"claim","lab":"kimi",…}
    missing 1 copy(ies) of: {"count":"1/3","event":"harness_heartbeat","headline":"M1: reply acceptable 37/232 …","kind":"heartbeat",…}
    fix: restore the missing line(s) - re-append them from the base ref
exit 1
```

**Exactly the two receipts Kimi's flag names** — the Amendment-2 claim and heartbeat.

A deliberate shrink of the live shard is caught and named; restoring it returns exit 0.

| check | result |
|---|---|
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **97 passed** + 1 doc-test |
| `python3 -m unittest discover -s tests/harness -t .` | 315 passed |
| `cli.py check` | 0 errors |
| `check_no_softmax.py` | OK — 1648, unchanged |

## The CI guard

A step in the `rust-harness` job, after the parity step that builds the binary:

```yaml
- name: The ledger may grow and may never shrink
  run: |
    set -euo pipefail
    git fetch --no-tags origin main
    git rev-parse --verify "origin/main^{commit}" >/dev/null
    ./local/harness-target/debug/uor-harness shard-check --repo . --base origin/main
```

The fetch is **plain, not the shallow-refspec form**. The first version wrote
`git fetch --no-tags --depth=1 origin main:refs/remotes/origin/main`, which **deleted the
remote-tracking refs** in this worktree and left the check with no comparison point — the same
silent-pass failure, produced by the guard's own setup. The `rev-parse --verify` is a second belt:
it fails the step before the check can report a clean result from an empty comparison.

## Limitations, stated

- **The guard is only as good as its base ref.** It compares against whatever `--base` names; a CI
  job that fetched a stale `main` would compare against stale content. It validates that the ref
  *resolves*, not that it is current.
- **It cannot tell a deliberate removal from a loss.** A receipt that *should* be removed (a
  mistaken entry, a correction) will be refused by the same guard. There is no bypass, and adding one
  would be the obvious way to neuter this. If a receipt must go, the honest route is an owner
  decision, because the ledger is the cumulative record.
- **Reordering is invisible to it.** By design, but it means the guard does not enforce append
  *order*, only append *content*.
- **The live tier is not yet append-only.** Task (3) — moving coordination writes to
  `~/.local/share/uor-r4/harness` so no lab writes a working checkout — is not done. This guard
  protects the **committed** ledger; the out-of-tree tier still needs the same treatment.
