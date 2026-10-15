# #2457 — Port team dispatch; the ledger line was the wrong shape (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T08:38:15Z
- **Updated:** 2026-10-14T08:40:55Z
- **Closed:** 2026-10-14T08:40:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2457
- **Merged:** 2026-10-14T08:40:38Z
- **Merge commit:** eddc1cce1e7d0e9b94a2707a0f728de4ddbc39ea
- **Base / Head:** main / UOR-Foundation:deepseek/team-dispatch
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21)

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

Decision: **KEEP** (tooling).

## What this delivers

**`uor-harness team dispatch` is Rust, and the `team` delegation site is gone: `delegate_to_python`
sites 5 → 4.**

The command resolves the role, writes ONE receipt through `receipts.record`, and prints what landed.
**The write half is why it stayed delegated:** `team roster` could be ported by reading, a command that
WRITES the shared ledger cannot be. So `receipt::record` came with it — the `event` mapping, the
`_NEVER` key list, the empty-value rule, redaction, lab attribution, the unattributed warning, and the
`unknown-<month>` shard for a record with no lab.

## The defect the comparison found

`Receipt::to_line` documented itself as *"matching `json.dumps(record, ensure_ascii=False,
sort_keys=True)`"* and wrote COMPACT JSON:

```
python: {"count": 2, "event": "harness_team_dispatched", "kind": "team", …}
rust:   {"count":2,"event":"harness_team_dispatched","kind":"team",…}
```

Every reader parses, so nothing failed — and this module's own header says two implementations writing
one file differently "would make every ledger change unreadable". It took the first comparison of a
WRITTEN line against Python's to see it: reading a line back is not comparing it. Fixed with
`py_dumps_sorted` (Python's default separators, sorted keys, `ensure_ascii=False` for the ledger);
`--json` keeps the other Python default (`ensure_ascii=True`, insertion order).

**The test that should have caught it could not fail:** `a_receipt_line_is_sorted_json` split the line
on `","`, which the compact form happened to produce and which Python's separators do not — so after
the fix it would have passed on an empty key list. It now asserts the exact line.

## Verification: both implementations on scratch checkouts

stdout, stderr, exit code AND the written shard line, per case:

| case | stdout | stderr | ledger line | exit |
|---|---|---|---|---|
| full flags | MATCH | MATCH | MATCH | 0/0 |
| minimal (role only) | MATCH | MATCH | MATCH | 0/0 |
| `--count 0` | MATCH | MATCH | MATCH | 0/0 |
| `--json` | MATCH | MATCH | MATCH | 0/0 |
| no `UOR_LAB` (unattributed) | MATCH | MATCH | MATCH | 0/0 |
| whitespace task | MATCH | MATCH | MATCH | 0/0 |

Error paths: `team dispatch no-such-role` and `team dispatch` are byte-identical on both sides;
`--count xyz` and `--state bogus` refuse on both. Both sides agree once the timestamp is compared as
an instant.

| check | result |
|---|---|
| `cargo test -p uor-r4-harness --offline` | 438 passed, 0 failed, 6 ignored (was 432) |
| `cargo fmt --check`, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | exit 0, including *Rust-written receipt read back by Python: ok* |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `delegate_to_python` sites | **4 — was 5** |

## Checklist

- [x] own worktree, port with its tests, record in `docs/labs/team-dispatch-port-2026-10-14/README.md`, full verification, protected PR
- [x] both implementations run against scratch checkouts and the WRITTEN artifact compared, not read back
- [x] the regression is a named test (`a_recorded_line_is_byte_for_byte_pythons`), and the test that could not fail was replaced
- [x] limitations stated below
- [x] no capability claim, no headline movement, no compute

### Stated rather than hidden

* **A receipt's `ts` is `…Z` in Rust and a local offset in Python** — the same instant. This is a
  documented, deliberate divergence (`now_rfc3339`'s own comment: both are valid RFC3339 and the reader
  parses either), **and it applies to every Rust-written receipt, not only this command.** It is stated
  here because this unit is the first to compare the written bytes; the comparison asserts the two forms
  as instants rather than assuming it. **Not changed here** — it would touch heartbeat, receipt, briefing,
  presence and prep together, which is its own unit.
* **`--count xyz` and `--state bogus` refuse with exit 2 on both sides, but the message text differs:**
  argparse prints a usage block naming the argument; this port prints one line naming the argument.
* **`--repo` after the subcommand is accepted here and rejected by Python's argparse** — the divergence
  already recorded for `check` and `git switch`, in the port's favour.
* The comparison lives in the worktree (`local/compare-dispatch.py`, gitignored) and is not a CI test;
  the committed protection is the fixture-backed exact-line test plus the semantics tests.

References #2028


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6164364083 — 2026-10-14T08:38:22Z — Casey-allard

## Verification at the exact head `179e4c0bf` (local, this worktree)

```
export CARGO_TARGET_DIR=$PWD/local/harness-target
cargo fmt -p uor-r4-harness -- --check                 -> clean
cargo clippy -p uor-r4-harness --offline --all-targets -- -D warnings -> clean, 0 findings
cargo test -p uor-r4-harness --offline                 -> 438 passed, 0 failed, 6 ignored
python3 -m unittest discover -s tests/harness -t .     -> exit 0, 0 failure lines
python3 scripts/check_no_softmax.py                    -> OK 1641, baseline 1641
python3 scripts/harness/parity.py                      -> exit 0
```

**Both implementations, scratch checkouts, six cases — stdout, stderr, exit code and the WRITTEN shard line:**

```
full flags        exit 0/0  stdout MATCH  stderr MATCH  ledger MATCH  same-instant True
minimal           exit 0/0  stdout MATCH  stderr MATCH  ledger MATCH  same-instant True
--count 0         exit 0/0  stdout MATCH  stderr MATCH  ledger MATCH  same-instant True
--json            exit 0/0  stdout MATCH  stderr MATCH  ledger MATCH  same-instant True
no UOR_LAB        exit 0/0  stdout MATCH  stderr MATCH  ledger MATCH  same-instant True
whitespace task   exit 0/0  stdout MATCH  stderr MATCH  ledger MATCH  same-instant True
ALL MATCH
```

Error paths:

```
team dispatch no-such-role   rust exit 2: unknown role 'no-such-role'; try: uor-harness team roster
                             py   exit 2: unknown role 'no-such-role'; try: uor-harness team roster
team dispatch                rust exit 2 / py exit 2, both: unknown role None; try: ...
team dispatch R --count xyz  rust exit 2 / py exit 2 (message text differs, both name the argument)
team dispatch R --state bogus rust exit 2 / py exit 2 (ditto)
```

The ledger line before and after the fix (Python's, from the same scratch run):

```
before: {"count":2,"event":"harness_team_dispatched",...}
after : {"count": 2, "event": "harness_team_dispatched", ...}
```

The only remaining difference in the artifact is the timestamp rendering, which `now_rfc3339`'s own
comment already records as deliberate (`Z` here, local offset in Python, same instant); the comparison
asserts the two forms as instants rather than assuming it.



---
_Comment/review ids in this file: 6164364083_

