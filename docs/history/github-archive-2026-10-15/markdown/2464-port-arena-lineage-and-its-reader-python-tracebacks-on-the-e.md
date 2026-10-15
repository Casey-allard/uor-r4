# #2464 — Port arena lineage and its reader; Python tracebacks on the empty path (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T09:42:13Z
- **Updated:** 2026-10-14T10:04:48Z
- **Closed:** 2026-10-14T10:04:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2464
- **Merged:** 2026-10-14T10:04:32Z
- **Merge commit:** bd927bac963a278a703941c64546c0221e0aa642
- **Base / Head:** main / UOR-Foundation:deepseek/arena-lineage
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21)

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

Decision: **KEEP** (tooling).

## What this delivers

**`uor-harness arena lineage <id>` is Rust**, and it brought `arena::load` with it — the reader that
`board` and `compare` will reuse, so the next unit is rendering rather than reading.

**Delegation: live call sites 1 → 2; unported commands 5 → 4.** The general fallback now carries
`budget` x2, and `arena board|compare` have their own named line. The site count rises while the
unported count falls, and both are stated.

## Python crashes on this path

`arena lineage <unknown>` calls `emit(f"arena: no lineage for {args.id!r}", file=sys.stderr)` at
`arena.py:465`, but `emit(text, out=None)` has **no `file` parameter** — so the command exits 1 with a
`TypeError` traceback. This port prints **the message the Python code intended** and exits **1**, the
same code the crash produces. Reproducing a traceback is not a fidelity target.

## Verification: both implementations, eight case groups

| case | result |
|---|---|
| `arena lineage <id>` for all five rows in `docs/harness/arena.json` | MATCH |
| a root row and a row with a parent chain | MATCH |
| `--json arena lineage <id>` (Python ignores the flag, so this does too) | MATCH |
| `arena board` (still delegated) | stdout + exit MATCH; stderr carries the delegation notice |
| `arena lineage <unknown>` | exit 1/1; stderr differs because Python tracebacks |
| `arena lineage` with no id | exit 2/2, argparse usage text (recorded divergence) |

| check | result |
|---|---|
| `cargo test -p uor-r4-harness --offline` | 472 passed, 0 failed, 6 ignored (was 468) |
| fmt, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | exit 0 |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |

## Checklist

- [x] own worktree, port with tests, record at `docs/labs/arena-lineage-port-2026-10-14/README.md` plus a newest-first current-state entry, full verification, protected PR
- [x] both implementations exercised and compared byte for byte, not read back
- [x] limitations stated below

### Stated rather than hidden

* **`load` treats a missing or unreadable `docs/harness/arena.json` as an EMPTY arena**, exactly as
  Python does: the arena starts empty, and a lab with a broken file gets the same answer as one that has
  measured nothing.
* **Delegation resolves `scripts/harness/cli.py` relative to `--repo`**, so `arena board` cannot run
  against a directory that has `docs/harness` but no `scripts/`. Pre-existing `delegate_to_python`
  behaviour, not something this unit changes; it is why the `board` comparison runs in the real checkout.
* **`lineage` stops on a repeated id**, because the arena file is hand-written JSON and a parent loop
  must not hang the command.

References #2028


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6165471686 — 2026-10-14T09:42:20Z — Casey-allard

## Verification at the exact head (local, this worktree)

```
CARGO_TARGET_DIR=$PWD/local/harness-target cargo fmt -- --check   -> clean
cargo clippy --all-targets -- -D warnings                         -> clean, 0 findings
cargo test -p uor-r4-harness --offline                            -> 472 passed, 0 failed, 6 ignored
python3 -m unittest discover -s tests/harness -t .                -> exit 0, 0 failure lines
python3 scripts/harness/parity.py                                 -> exit 0
python3 scripts/check_no_softmax.py                               -> OK 1641, baseline 1641
```

**Both implementations, eight case groups:**

```
arena lineage base-d10                  exit 0/0  out ok  err ok
arena lineage addressed-read            exit 0/0  out ok  err ok
arena lineage starling-tree             exit 0/0  out ok  err ok
arena lineage starling-readout-distill  exit 0/0  out ok  err ok
arena lineage m2-whole-path             exit 0/0  out ok  err ok
--json arena lineage addressed-read     exit 0/0  out ok  err ok
arena lineage nope                      exit 1/1  out ok  err DIFF (Python tracebacks)
arena board (delegated)                 exit 0/0  out ok  err: delegation notice, by design
```

That is **all five rows** in `docs/harness/arena.json`, plus a root, a parented chain, and the `--json`
echo Python ignores. `arena board` still matches on stdout and exit code; its stderr carries this
binary's `note - "arena" is not ported to Rust yet` line, which is how every delegated command has
reported itself since the port began.

**The `nope` case is why the record exists:** Python raises `TypeError: emit() got an unexpected keyword
argument 'file'` at `arena.py:465` (`emit(text, out=None)` has no `file` parameter) and exits 1 with a
traceback. This port prints **the message the code intended** and exits **1** — the same code. A
traceback is not a behaviour to preserve.

### comment 6165713059 — 2026-10-14T09:56:06Z — Casey-allard

**PR #2464 is verified and QUEUED, but the merge queue is not draining.** Reporting the exact state rather
than pretending it merged:

* `gh pr merge 2464 --auto` reports **already queued to merge**, the PR is **OPEN / CLEAN**, and **every**
  check passes (`Rust harness and Python parity`, `fmt / clippy / tests / no_std / κ`, `harness rules and
  tests`, `cargo audit`, `fuzz smoke`, `wasm-pack`, `Gate C trend alarm`, `required acknowledgements`).
* It sat CLEAN for ~12 minutes across two poll cycles. The only other open PR is **#2433 (another lab)**,
  also not moving, so this looks like a queue-wide hold rather than anything about this PR.
* **The branch and worktree are deliberately KEPT** (`deepseek/arena-lineage`,
  `.worktrees/deepseek-arena-lineage`): the work is verified and must not be lost to a stuck queue. The
  next round completes the merge, verifies the hunk on fresh `origin/main`, and does the cleanup.

**What is in it:** `uor-harness arena lineage <id>` is Rust, plus `arena::load` (shared with the
`board`/`compare` units to come). **Live delegation call sites 1 → 2, unported commands 5 → 4.** It also
documents that **Python tracebacks on `arena lineage <unknown>`**: `arena.py:465` calls
`emit(..., file=sys.stderr)` but `emit(text, out=None)` has no `file` parameter. This port prints the
message the code intended and exits 1 — the same code the crash produces.

**Verified:** 472 Rust tests, fmt/clippy clean, unittest exit 0, `parity.py` exit 0, interlock 1641 =
baseline; eight comparison case groups against Python, all matching except the two documented
divergences (the traceback, and argparse usage text).


---
_Comment/review ids in this file: 6165471686, 6165713059_

