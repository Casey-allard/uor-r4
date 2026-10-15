# #2337 — uor-harness is the entry point; Python is behind it (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T15:03:33Z
- **Updated:** 2026-10-13T15:28:19Z
- **Closed:** 2026-10-13T15:28:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2337
- **Merged:** 2026-10-13T15:28:10Z
- **Merge commit:** 5c916488dd839a8f72de5b7747a71a42236d39c3
- **Base / Head:** main / UOR-Foundation:deepseek/rust-entrypoint
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only. Owner direction 13 October ("why are we using python?"); D27.

## What was wrong

**11,566 lines of Python and 8,000 of Rust — and every lab typed `python3`.** Measured: **118 invocations across 44 files**, 79 of them normative.

The port was real — presence, receipts, the D22 gate, shards, brief, notes. But **`cli.py` was still the front door.** I had even made `cli.py heartbeat` shell out to Rust and presented that as progress; it moved the *implementation* and left the *interface*, so four labs could not tell whether they were using the port at all.

**The number that mattered was never Rust lines. It was how many times a lab typed `python3` — and eleven units of work had not moved it once.**

## Result

Decision: **KEEP** (tooling).

`uor-harness` is the interface. **An unported command delegates to Python and says so on every call**, on stderr, not suppressible:

```
$ uor-harness check
uor-harness: note - "check" is not ported to Rust yet; running …/scripts/harness/cli.py check
harness check: 57 rules, 19 failed (0 error, 18 warn)
```

**Seventy-nine normative invocations rewritten** — `AGENTS.md`, `CONTRIBUTING.md`, the PR template, all of `docs/harness/**` including every skill and lesson, and `session-goal.md`. **Dated `docs/labs/<topic>-<date>/` records are exempt:** they are evidence of what was run at the time, and rewriting them would falsify the record.

### A defect caught on the way — and the reason to be careful here

The Rust `check` validates **state formats: one rule of the 57** the Python `check` covers. Exposing it as `check` would mean a lab types `uor-harness check`, sees *"state formats ok"*, and believes it validated its work **while 56 rules were never consulted.** That is the worst possible outcome for the command labs trust most.

**So `check` stays delegated, and the Rust checker is explicit: `check-formats`.** Verified — the full 57 rules reach the entry point and exit 0.

### Delegation had to be decided *before* flag parsing

Three bugs, all in that boundary:

1. Unknown flags were rejected **before** the binary knew it was delegating, so `check --only <rule>` failed with *"unknown option --only"* — delegation only worked for commands whose flags happened to be in the lists, which defeats the point.
2. Forwarding `--only` **without its value** left the value to be read as the next command word, so the binary tried to execute a command called `harness.state_formats`.
3. `--repo` was forwarded **and** added, which argparse rejects as a duplicate.

## The guard

New `check` rule **`harness.entry_point`**, **error** severity: no normative doc may invoke `python3 scripts/harness/cli.py`. A rule rather than a note, because *"remember the new command"* is exactly the instruction that decays.

`doccheck` also learns the **Rust-only commands** (`onboard`, `presence`, `registry`, `prep-render`, `prep-check`, `shard-check`, `check-formats`, `ledger-sync`, `notes`). It had been reporting `uor-harness onboard` as `unknown subcommand 'onboard'` in six places — **validating the interface against one implementation of it.**

```
cargo fmt --check                              -> clean
cargo clippy --all-targets -- -D warnings      -> 0 findings
cargo test -p uor-r4-harness                   -> 125 passed
python3 -m unittest discover -s tests/harness  -> 319 passed on 3.15 AND 3.9
python3 scripts/harness/cli.py check           -> 0 errors (58 rules)
harness.entry_point                            -> all pass
delegated `check` through the entry point      -> 57 rules, 0 errors, exit 0
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/rust-entry-point-2026-10-13/README.md
- [x] Branch deleted after merge

### Stated rather than hidden

- **Most commands are still Python.** This changes the *interface*, not the implementation: `check`, `status`, `prep new`, `skill`, `team`, `mistakes`, `arena`, `budget`, `lint` and `git` all delegate today. **The notice on each call is a rolling measure of how much port remains, and it is meant to be uncomfortable.**
- **`uor-harness` must be built and on `PATH`.** A lab without the binary has no harness now. The docs say how; **nothing verifies it**, and `onboard` is the only place that would notice.
- **The delegation costs a subprocess per call** and passes `--repo` explicitly, so a lab relying on the current directory rather than `--repo` may see a path difference.
- **`harness.entry_point` scans `discover_docs`**, so a normative doc outside that set could still call `cli.py` unnoticed.
- **The Rust `presence` shadows the Python `multilab presence`** and they are not the same command — the Rust one reads both tiers, the Python one has extra columns. `doccheck` accepts both, which is a wart rather than a decision.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

