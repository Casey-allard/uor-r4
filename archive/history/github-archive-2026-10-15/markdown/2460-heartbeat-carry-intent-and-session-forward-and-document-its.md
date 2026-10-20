# #2460 — heartbeat: carry intent and session forward, and document its flags (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T08:58:46Z
- **Updated:** 2026-10-14T09:05:25Z
- **Closed:** 2026-10-14T09:05:03Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2460
- **Merged:** 2026-10-14T09:05:02Z
- **Merge commit:** 2a0bb33cfd2f5735b33d969ed027226586bf30bd
- **Base / Head:** main / UOR-Foundation:deepseek/heartbeat-intent
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21)

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

Decision: **KEEP** (tooling).

## What this fixes

Three defects in `heartbeat`, the check-in every lab sends every few minutes — the first three items of
the PM queue for this lab (claude/review1's note on #2028, 6163960613). Each was reproduced before the
fix and again after it, against a **scratch live tier** (`UOR_HARNESS_LIVE`), so no other lab's board was
touched.

### 1. A bare check-in erased the intent

```
heartbeat -m m1 --line "porting the harness" --next "verify the repro" --session "testlab/s1"
  -> doing/next/session all set
heartbeat -m m1                                   # the onboarding prompt's own command
  BEFORE -> {"doing": "", "next": "", "session": ""}      live, and apparently idle
  AFTER  -> {"doing": "porting the harness", "next": "verify the repro", "session": "testlab/s1"}
```

**A lab checking in with the documented command read as live-and-idle on every board**, including its
own last intent, which its previous file held one moment earlier. **The rule is "absent", not "empty"**:
`--line ""` still clears the field (verified in the same run), so a lab can still say it has stopped.

### 2. `--session` did not exist

`heartbeat --session claude/review1` answered `unknown option --session` with exit 2, so
`Presence.session` was **never set by anyone** and two sessions of one lab were indistinguishable on the
board. The flag now exists, carries forward like the intent, **and the Python transport forwards it** —
`cli.py heartbeat` is a thin transport to this binary, and a flag present in one entry point and absent
in the other is the two-implementations problem in miniature.

### 3. `heartbeat --help` printed the global usage

It listed the command and none of its flags. A command word seen **before** the help flag now selects its
own flag list; commands with no written list still print the global usage.

## One behaviour change worth stating

**A bare check-in's RECEIPT now carries the previous line.** The receipt and the presence are written
from one record — the code says so, and this is what that means. Verified in the live shard: after a bare
check-in the receipt holds `"line": "porting the harness"`; after `--line ""` the key is absent, exactly
as presence says.

## Verification

| check | result |
|---|---|
| the three repros, before and after | fixed (transcript in the record) |
| explicit empty flag | still clears |
| `cargo test -p uor-r4-harness --offline` | 446 passed, 0 failed, 6 ignored (was 445) |
| `cargo fmt --check`, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | exit 0 |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| worktree after checking in repeatedly | clean — heartbeats write the live tier only |

## Checklist

- [x] own worktree, fix with tests, record in `docs/labs/heartbeat-interface-2026-10-14/README.md`, full verification, protected PR
- [x] each repro run before and after, on a scratch live tier rather than the shared board
- [x] the carry-forward rule is a named test, not a comment
- [x] limitations stated below

### Stated rather than hidden

* **Queue item 4 is not fixed here.** The durable presence tier is a day stale and the README is wrong
  about what refreshes it — **but that is not a heartbeat defect.** The Rust heartbeat writes the live
  tier only **on purpose** (R10: a five-minute check-in must not dirty a checkout), and the durable
  tier's refresh path is `multilab sync-live`, **which is still delegated to Python.** The fix is to port
  `sync-live` and correct the README; it is the next unit rather than a rider on this one.
* **`--milestone` now carries forward too** when it is absent, for the same reason as `doing`/`next`. The
  Python parser still requires `-m`; this binary does not, and a check-in without it no longer blanks the
  milestone.
* Queue items 5–7 are untouched: pinning rustup `cargo` in the bootstrap, `claims_file()` pointing at the
  owner checkout, and the stale `~/.cargo/bin/uor-harness` install.

References #2028


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6164717395 — 2026-10-14T08:58:55Z — Casey-allard

## Verification at the exact head `8bd1cc581` (local, this worktree)

Every repro was run against a **scratch live tier** (`UOR_HARNESS_LIVE=$PWD/local/hb/live`) and a scratch
repo, so no other lab's board was touched.

```
export CARGO_TARGET_DIR=$PWD/local/harness-target
cargo fmt -p uor-r4-harness -- --check                 -> clean
cargo clippy -p uor-r4-harness --offline --all-targets -- -D warnings -> clean, 0 findings
cargo test -p uor-r4-harness --offline                 -> 446 passed, 0 failed, 6 ignored
python3 -m unittest discover -s tests/harness -t .     -> exit 0, 0 failure lines
python3 scripts/check_no_softmax.py                    -> OK 1641, baseline 1641
python3 scripts/harness/parity.py                      -> exit 0
```

**Repro 1 — the blanking, before and after:**

```
$ uor-harness heartbeat --repo R -m m1 --line "porting the harness" --next "verify the repro" --session testlab/s1
  {"doing": "porting the harness", "next": "verify the repro", "session": "testlab/s1"}
$ uor-harness heartbeat --repo R -m m1              # bare, the onboarding command
  BEFORE  {"doing": "", "next": "", "session": ""}
  AFTER   {"doing": "porting the harness", "next": "verify the repro", "session": "testlab/s1"}
$ uor-harness heartbeat --repo R -m m1 --line ""    # explicit clear still clears
  {"doing": "", "next": "verify the repro", "session": "testlab/s1"}
```

**Repro 2 — `--session`, through both entry points:**

```
$ uor-harness heartbeat ... --session testlab/s1     -> presence session="testlab/s1"
$ python3 scripts/harness/cli.py heartbeat ... --session testlab/s2   # the transport forwards it
  -> presence session="testlab/s2", doing="via the python transport"
```

**Repro 3 — the help:**

```
$ uor-harness heartbeat --help
uor-harness heartbeat - the required check-in (at least every 5 minutes)
USAGE: / FLAGS: -m --line --doing --count --headline --next --session --lab --dry-run --no-notes
WHAT IT WRITES: ...
$ uor-harness --help     # the global usage is unchanged
```

**The one-behaviour-change, checked in the live shard** (presence and receipt are written from one
record):

```
after `--line "porting the harness"`  receipt: {"line": "porting the harness", "next": "verify the repro"}
after a BARE check-in                  receipt: {"line": "porting the harness", "next": "verify the repro"}
after `--line ""`                     receipt: no "line" key, "next" still carried
```

The worktree is clean after all of it — heartbeats write the live tier only, which is why item 4 of the
queue (the durable tier) is a `multilab sync-live` port rather than a heartbeat change, and is not in
this PR.

### comment 6164799410 — 2026-10-14T09:03:45Z — Casey-allard

## Rebase + verification at `78b3ea588`

Main moved under this branch (`#2454`, `#2459`), conflicting only in `docs/integration/current-state.md`
— both labs added a newest-first entry. Resolved by keeping **both**, this unit's above theirs, and the
whole bar was re-run on the rebased head:

```
CARGO_TARGET_DIR=$PWD/local/harness-target cargo fmt -- --check   -> clean
cargo clippy --all-targets -- -D warnings                         -> clean, 0 findings
cargo test -p uor-r4-harness --offline                            -> 446 passed, 0 failed, 6 ignored
python3 -m unittest discover -s tests/harness -t .                -> exit 0, 0 failure lines
python3 scripts/harness/parity.py                                 -> exit 0
python3 scripts/check_no_softmax.py                               -> OK 1641, baseline 1641
```

**And one finding made while verifying, now recorded rather than ridden along: the Python suite fails
when `CARGO_TARGET_DIR` is exported — the variable every doc tells a lab to build with.**

```
$ python3 -m unittest tests.harness.test_stateformats.TestCannotRunIsNotAPass
OK
$ CARGO_TARGET_DIR=$PWD/local/harness-target python3 -m unittest tests.harness.test_stateformats.TestCannotRunIsNotAPass
FAILED (failures=1)   'error' != 'skipped'
```

`stateformats._harness_binary` checks `$CARGO_TARGET_DIR/debug/uor-harness` first, so with the documented
variable still set the rule reports a missing contract file as an **error** where the test expects
**skipped**. **Deterministic, not the known intermittent flake** — one command reproduces it — and a
`stateformats`/test concern rather than a heartbeat one, so it is recorded in the lab record and queued
as its own unit.


---
_Comment/review ids in this file: 6164717395, 6164799410_

