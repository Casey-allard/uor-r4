# Port 1 of D30: `team` is Rust, and the front door stops saying "not ported"

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** D30; the order posted on #2028 comment 6150764050

## What moved

`crates/uor-r4-harness/src/team.rs` — new, **20 tests**. `uor-harness team claim|release|claims` now
runs in Rust and **`team` is out of the delegated set**, so the *"not ported to Rust yet"* notice no
longer prints for it. Verified by running it: no notice line.

`roster` and `dispatch` still delegate **by name**, so each remaining action says so individually —
the progress measure shrinks **action by action**, not command by command. That was the rule set in
the D30 record and this is the first place it applies.

## The rules the port had to keep

- **One arbiter per branch, and it is the committed one.** Live → committed, weakest first, the
  caller's store overwriting everything it names. An earlier Python version merged newest-wins and
  that was wrong: `claim` stamps one entry into two tiers, the copies tie, then diverge under a hand
  edit — and *newest* cannot choose between two claims that are both current.
- **Re-claiming your own branch succeeds and refreshes**; it is how a claim is renewed after a rebase.
  Only **another** lab is refused.
- **Release republishes the live mirror.** A mirror only ever added to **resurrects released claims.**
  Tested on both tiers.
- **A refused claim writes nothing.**
- **Empty branch or lab is a usage error (`EXIT_USAGE`), not a refusal** — the Python distinction, kept.
- **A missing or corrupt store reads as empty.** A lab must be able to claim on a machine whose
  advisory tier is absent; refusing there would make the advisory tier load-bearing.

## Three bugs the port found, all in the new code

1. **`%:z` is a GNU extension.** BSD/macOS `date` printed the literal `:z`, producing
   `2026-10-13T13:37:12:z` — a timestamp every reader of the committed store would have to cope with.
   Now `%z` with the colon inserted here, which works on both.
2. **`serde_json` preserves insertion order; the Python store wrote `sort_keys=True`.** An unsorted
   writer would have reordered every existing claims file on first touch, turning a one-line claim
   into a whole-file diff in review. Sorted now.
3. **`--repo` beat the subcommand word, twice.** First `args[0]` made `--repo` the action; then "the
   first word without a dash" made the repo **path** the action, because a path is a bare word too.
   The fix steps over a value-taking flag's argument — and `--repo` had to be listed explicitly,
   because it is consumed by the top-level parser and so never appeared in the subcommand value list.

**All three would have shipped as "it compiles and the happy path worked."** The fixture capture is
what caught the first two: they are only visible when the output is compared against what Python
actually produced.

## The fixtures — trap 1 from the D30 record, applied

`crates/uor-r4-harness/tests/fixtures/team/behaviour.json` — the golden behaviour **captured from the
Python harness before it was replaced**, including the refusal texts, the exit-code distinction, and
the non-obvious rules (same-lab reclaim, release-without-lab, mirror-on-release). **Captured while
Python still existed**, which was the whole point: once `team` is Rust there is no second opinion.

**This is a regression lock, not parity.** It cannot catch a bug in the interpretation of the fixture
itself, and after the port there is no implementation left to disagree with it.

## Verification

| check | result |
|---|---|
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **152 passed** (20 new) |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `uor-harness check` | 0 errors |
| Rust vs captured Python output | **identical** — sorted keys, `-04:00` offset, same refusal text |
| delegation notice for `team` | **gone** |

## Limitations, stated

- **`roster` and `dispatch` still run Python.** They read `roles.json` through `registries`, which is
  unported. Two of five actions moved, not five.
- **The Python `team` implementation is untouched and still reachable** through `cli.py`. It is not
  retired until `roster`/`dispatch` move and the Python tests that cover them port — deleting it now
  would leave those tests testing nothing.
- **Claims remain not-a-lock.** Two labs claiming one branch in different worktrees both succeed
  locally; the collision surfaces as a `claims.json` conflict at merge. The port preserves that
  accepted cost rather than quietly changing coordination semantics.
- **`now_iso` shells out to `date`.** A machine without it gets the epoch, which is a wrong timestamp
  rather than a failure. Not measured on a system without `date`.
- **I did not test concurrent claims from two processes.** The store is written whole, so a race would
  lose one claim. The Python version had the same shape and I have not changed it, but I have not
  proven it either.
