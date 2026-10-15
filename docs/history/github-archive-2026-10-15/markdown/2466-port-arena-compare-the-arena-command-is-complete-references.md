# #2466 — Port arena compare; the arena command is complete (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T10:20:37Z
- **Updated:** 2026-10-14T10:36:54Z
- **Closed:** 2026-10-14T10:36:43Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2466
- **Merged:** 2026-10-14T10:36:43Z
- **Merge commit:** d140504e3f938c666525d4c384ba4d5abe171e54
- **Base / Head:** main / UOR-Foundation:deepseek/arena-compare
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21)

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

Decision: **KEEP** (tooling).

## What this delivers

**`uor-harness arena compare <id>` is Rust**, text and `--json`. **The `arena` command is complete:**
`board`, `compare` and `lineage` are all Rust, and the only live `delegate_to_python` call site left is the
general fallback, carrying **`budget show|scan`**. Unported commands **3 → 2**.

## The comparison is the report the arena exists for

It is a regression check, not a leaderboard, so nothing collapses into one number. **Direction comes from
the file's `bars` map and is never guessed** — an undeclared bar is reported as `undeclared` and cannot be
called a gain or a regression. **A KEEP must report fresh transfer as evidence, not as the word**:
`"transfer" in scope` is wrong because *"no transfer reported"* contains it.

## Verification: both implementations, fourteen case groups

| case | result |
|---|---|
| `arena compare <id>` for all five rows, text and `--json` | MATCH |
| `arena compare nope`, text and `--json` | MATCH (exit 1/1) |
| `arena compare` with no id | exit 2/2, argparse usage text (recorded divergence) |

The rows cover a **root row** (`comparable: false`, exit 0 — still `ok`), a KEEP with a transfer bar, a
panel change, an undeclared bar, and integer deltas.

| check | result |
|---|---|
| `cargo test -p uor-r4-harness --offline` | 481 passed, 0 failed, 6 ignored (was 475) |
| fmt, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | exit 0 |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |

## Two details that would have diverged quietly

**`delta = round(b - a, 6)` keeps INT-ness when both sides are ints** — Python prints `13`, and a
float-only port prints `13.0` on every integer bar in this file. **The bars compared are the UNION of both
`scores` maps, sorted**, so a one-sided bar becomes `UNAVAILABLE` naming that side rather than a delta.

## Checklist

- [x] own worktree, port with tests, record at `docs/labs/arena-compare-port-2026-10-14/README.md` plus a newest-first current-state entry, full verification, protected PR
- [x] both implementations exercised and compared byte for byte, not read back
- [x] limitations stated below

### Stated rather than hidden

* **A non-numeric `seeds`/`rows` value is treated as zero**, where Python's `or` fallback would raise on
  a string. The arena file writes integers or null.
* **The `--json` result dict's key order is load-bearing** (`reason` last on both early returns), and this
  call has **`ensure_ascii` ON**, unlike `arena board`'s.
* **`budget show|scan` still delegate**; `budget show` sits behind `collect()`, ~400 lines including the
  resource-ledger pointer-chain walk.

References #2028


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6166148742 — 2026-10-14T10:20:46Z — Casey-allard

## Verification at the exact head (local, this worktree)

```
CARGO_TARGET_DIR=$PWD/local/harness-target cargo fmt -- --check   -> clean
cargo clippy --all-targets -- -D warnings                         -> clean, 0 findings
cargo test -p uor-r4-harness --offline                            -> 481 passed, 0 failed, 6 ignored
python3 -m unittest discover -s tests/harness -t .                -> exit 0, 0 failure lines
python3 scripts/harness/parity.py                                 -> exit 0
python3 scripts/check_no_softmax.py                               -> OK 1641, baseline 1641
```

**Both implementations, fourteen case groups:**

```
arena compare base-d10                  text + json   exit 0/0  MATCH
arena compare addressed-read            text + json   exit 0/0  MATCH
arena compare starling-tree             text + json   exit 0/0  MATCH
arena compare starling-readout-distill  text + json   exit 0/0  MATCH
arena compare m2-whole-path             text + json   exit 0/0  MATCH
arena compare nope                      text + json   exit 1/1  MATCH
arena compare        (no id)                          exit 2/2  argparse usage text
```

Those five rows are the interesting ones: **`base-d10` is a root** (`comparable: false` at exit **0**, since
it is still `ok`), `addressed-read` is a **KEEP with a transfer bar**, and the `starling-*` rows carry the
**undeclared-direction** bars the `notes` field exists for.

**Two places this could have diverged quietly, and a test for each:**

* `delta = round(b - a, 6)` **keeps INT-ness when both sides are ints** — Python's `round(13, 6)` is the int
  `13` and prints `13`, so a float-only port prints `13.0` on every integer bar in `arena.json`.
* The bars compared are the **union** of both `scores` maps, sorted, so a bar only one side reports becomes
  `UNAVAILABLE` naming that side rather than a delta.

**And `arena` is now whole:** `board`, `compare` and `lineage` are all Rust. The only live
`delegate_to_python` call site left is the general fallback, carrying `budget show|scan`.

### comment 6166391784 — 2026-10-14T10:34:46Z — Casey-allard

**#2466 is verified and pushed, but `main` moved under it and the rebase conflicts in exactly one file.**

* `origin/main` is now **`40c0d6399`** (another lab's #2433 merged at 10:19); my branch is **1 commit behind**
  and the PR reads **OPEN / DIRTY**, so the queue will not take it.
* **The conflict is `docs/integration/current-state.md` only** — `UU` in the rebase, because both labs
  prepend a newest-first entry at the top of the same file. Every other path rebased cleanly
  (`arenacompare.rs` added, `lib.rs`/`main.rs` modified).
* **I aborted the rebase rather than leave it half-resolved**: the worktree is clean at `c6d236cb8`, and the
  remote branch and the PR both still point at that commit. **Nothing is lost and nothing is in a partial
  state.**

**The resolution is the one this port has used before:** rebase onto `origin/main`, keep **both** entries in
`docs/integration/current-state.md` (ours first, since ours is the newer delivery), then push and re-arm the
queue — with the `dequeuePullRequest` nudge if the entry wedges again.

**What is in it, verified locally:** `uor-harness arena compare <id>` is Rust, text and `--json`; **the whole
`arena` command is now ported** (`board`, `compare`, `lineage`), leaving the general fallback carrying only
`budget show|scan`. 14 comparison case groups against Python, all matching; 481 tests; fmt/clippy clean;
unittest and `parity.py` both exit 0.


---
_Comment/review ids in this file: 6166148742, 6166391784_

