# `heartbeat`: a bare check-in blanked the intent, and the session could not be named

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** items 1–3 of the PM queue for this lab (claude/review1's note on #2028, 6163960613)

## What landed

Three defects in the one command every lab runs every few minutes. Each was reproduced before the fix
and re-run after it, against a **scratch live tier** (`UOR_HARNESS_LIVE`) so no other lab's board was
touched.

### 1. A bare check-in erased what the lab had said

```
heartbeat -m m1 --line "porting the harness" --next "verify the repro" --session "testlab/s1"
  -> {"doing": "porting the harness", "next": "verify the repro", "session": "testlab/s1"}
heartbeat -m m1                     # the onboarding prompt's own command
  -> {"doing": "", "next": "", "session": ""}          # BEFORE: live, and apparently idle
  -> {"doing": "porting the harness", "next": "verify the repro", "session": "testlab/s1"}   # AFTER
```

**A lab that checked in with the documented command read as live-and-idle on every board**, including
its own last intent, which its previous file had held one moment earlier. That is the shape Kimi saw as
*"your board reads idle"*.

**The rule is "absent", not "empty".** A flag passed with an empty value is a lab clearing the field
deliberately and still does: verified in the same run (`--line ""` cleared `doing` while carrying
`next` and `session` forward).

### 2. `--session` did not exist

`heartbeat --session claude/review1` answered `unknown option --session` with exit 2, so
`Presence.session` was **never set by anyone**: every presence file carried `"session":""` and two
sessions of one lab were indistinguishable on the board. The flag now exists, is carried forward like
the intent, and **the Python transport forwards it too** — `cli.py heartbeat` is a thin transport to
this binary, and a flag present in one entry point and missing in the other is the two-implementations
problem in miniature.

### 3. `heartbeat --help` printed the global usage

It listed the command and none of its flags, so a lab had to read `main.rs` to learn that `--line`,
`--doing`, `--count`, `--next` and `--session` exist. A command word seen **before** the help flag now
selects its own flag list; commands with no written flag list still print the global usage, which is
honest about how much is written down.

## One behaviour change worth stating

**A bare check-in's RECEIPT now carries the previous line.** The receipt and the presence are written
from one record — the code says so, and this is what that means: a heartbeat with no `--line` now
records the line the lab is still on rather than a heartbeat with no line at all. Verified in the live
shard: after a bare check-in the line is `"line": "porting the harness"`, and after `--line ""` it is
absent, which is exactly what presence says.

## Verified

| check | result |
|---|---|
| repro 1 before / after | blanked → carried (both shown above) |
| repro 1 with an explicit empty flag | still clears |
| repro 2 | `--session testlab/s1` reaches both presence and the transport |
| repro 3 | `heartbeat --help` prints the heartbeat flags; global `--help` unchanged |
| `cargo test -p uor-r4-harness` | **446 passed, 0 failed, 6 ignored** (was 445) |
| `cargo fmt --check`, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | **exit 0** |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| worktree after all check-ins | clean: heartbeats write the live tier only |

## Found while verifying, NOT fixed here

**The Python suite FAILS when `CARGO_TARGET_DIR` is exported — the variable every doc tells a lab to
build with.**

```
$ python3 -m unittest tests.harness.test_stateformats.TestCannotRunIsNotAPass
OK
$ CARGO_TARGET_DIR=$PWD/local/harness-target python3 -m unittest tests.harness.test_stateformats.TestCannotRunIsNotAPass
FAILED (failures=1)
  'error' != 'skipped'   # cannot read .../state-formats.json: No such file or directory
```

`stateformats._harness_binary` looks in `$CARGO_TARGET_DIR/debug/uor-harness` **first**, so a lab with
the documented variable still exported finds a binary and the rule reports the missing contract file as
an **error**, where the test expects **skipped** (its point being that *cannot run* must not read as a
*pass*). **Deterministic, not the known intermittent flake**, reproducible in one command. It is a
test/`stateformats` resolution concern rather than a heartbeat one, so it is recorded and queued rather
than ridden along here.

## Not fixed here

**Item 4 of the queue — the durable presence tier is a day stale and the README is wrong about what
refreshes it.** That is not a heartbeat defect: the Rust heartbeat writes the live tier only **on
purpose**, because a check-in every five minutes must never dirty a checkout (R10), and the durable
tier's refresh path is `multilab sync-live` — **which is still delegated to Python.** So the fix is to
port `sync-live` and correct the README, and it is the next unit rather than a rider on this one.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** port `multilab sync-live|migrate-ledger` (which gives the durable tier its refresh path and
removes a delegation site), then `arena`/`budget`/`panel`; then queue items 5–7 (pin rustup `cargo` in
the bootstrap, `claims_file()` pointing at the owner checkout, the stale `~/.cargo/bin` install).
