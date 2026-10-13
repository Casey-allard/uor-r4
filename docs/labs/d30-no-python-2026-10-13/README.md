# D30 scope: no Python where Rust can replace it — what that actually costs

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** owner direction 13 October; D27, D30

## The measured size — LIVE python only

**Owner clarification, 13 October: "you only have to replace the python thats being used, if they are
retired you can leave them alone as retired and archived."** That is a real narrowing and it changes
the number, so the inventory below separates what is **invoked** from what merely exists.

**Live python is identified by being reachable from an active entry point** — CI, the harness CLI, a
shell script, or Rust — not by being present in the tree:

| area | status | lines |
|---|---|---|
| `scripts/harness/` (the CLI and its modules) | **LIVE** — the harness every lab runs | **12,684** |
| `scripts/` invoked by CI or Rust (`check_claim_wording`, `check_gate_c_regression`, `energy_per_token`, `audit_zero_matmul_*`, serving/gate scripts, corpus/fixture generators) | **LIVE** | **~6,300** |
| `tests/harness/` | **LIVE** — the harness regression suite | **4,497** |
| **live total** | | **~23,500** |
| `tools/r4-softmax-trainer/` | **retired under D23** — leave as archived | ~189 files |
| `handoff/`, `docs/evidence/`, `archive/`, `research/`, `data/` | historical record — leave | ~380 files |
| `scripts/check_no_softmax.py`, `check_no_matmul.py` | **superseded by `uor-harness ratchet`** (#2346) — retire once the Python tests that cite them port | 386 |

**`tools/r4-softmax-trainer/` is the clearest case for leaving alone.** It is a softmax trainer and D23
prohibits softmax *anywhere*, including tooling. **Porting it to Rust would move a prohibited mechanism
into the preferred language** — the opposite of the goal. It is retired, it is archived, and it stays
that way. A rule that says "no Python" applied without asking *what the Python is* would have produced
exactly that mistake.

**Anything dormant that is later revived becomes live, and is then ported.** "Leave it" is not a
judgement that the code is good — only that nothing runs it.

## Two traps that decide the sequencing

## Two traps that decide the sequencing

**1. `parity.py` (27 checks) compares Rust against Python, so it dies with Python.** It is the most
valuable verification the harness has — it caught four real bugs in the brief port, including
`(frozen: true)` vs `True` and a lost fence tag. **Once Python is gone there is nothing to compare
against.** A checked-in Rust-only implementation has no second opinion.

*The mitigation, and it has to happen while Python still exists:* capture the Python behaviour as
**fixtures** — inputs and exact outputs — and make the Rust tests assert against those. That is not
parity, and it cannot catch a change that moves both sides; it is a regression lock, which is what is
still achievable. **Every module must contribute its fixtures before that module's Python is deleted**,
which makes fixture capture part of each port rather than a cleanup step at the end.

**2. `tests/harness/` (4,497 lines) IS the harness regression suite.** Deleting Python without porting
these deletes the verification with the implementation, and the port would then be "it compiles and I
ran it once".

*The rule this sets:* **a module's tests port in the same PR that ports the module.** Not after.

## The order, and why

Coordination first, because that is what every lab touches every session and it is currently a Python
subprocess under a Rust front door:

**`team` → `multilab` → `git clean-merged` → `skill` / `mistakes` → `prep`**, then `check`'s rule set,
`lint`/`prlint`, `panel`, `budget`, `arena`, then the `scripts/` tail (`pod`, `storage`, `check_*.py`).

`delegate_to_python` is removed **only when the last command lands**. Each port drops its command from
the delegated set and its notice stops printing — **the notice is the progress measure**, so it must
shrink per port rather than disappear at the end. A port that leaves the notice printing for its own
command has not finished.

## `presence` — the owner's decision, recorded

`uor-harness presence` (Rust, both tiers) and `multilab presence` (Python, extra columns) are two
different commands with one name. **Owner decided: one command, with a deprecation notice.** So when
`multilab` ports, `presence` is the only implementation and the old spelling keeps working for a few
days while printing that it has moved.

## Where the numbers will come from

Every port posts its own line on #2028 as it merges — command, lines removed, tests ported, fixtures
captured. **"How much Python is left" is then answerable from the thread rather than from a report**,
and a claim of completion can be checked against `find . -name '*.py'`, which is the only measure that
cannot be talked around.

## What is not claimed here

**No code is ported by this record.** It is the scope, the two traps and the order. The first port
(`team`) is the next unit, and it will carry its tests and its fixtures with it.
