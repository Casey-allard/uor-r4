# `team dispatch` is Rust, and diffing the written line found the ledger was the wrong shape

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** the second of the six remaining items on the measured scope (#2028)

## What landed

**`uor-harness team dispatch` runs in Rust — and the `team` delegation site is GONE: 5 → 4.**

The command resolves the role, writes ONE receipt through `receipts.record`, and prints what
landed. The write half was the reason it stayed delegated: `team roster` could be ported by reading,
**a command that WRITES the shared ledger cannot be**. So `receipt::record` came with it — the
`event` mapping, the `_NEVER` key list, the empty-value rule, redaction, lab attribution, the
unattributed warning, and the `unknown-<month>` shard for a record with no lab.

## The defect the comparison found: the ledger line was COMPACT

`Receipt::to_line` documented itself as *"matching `json.dumps(record, ensure_ascii=False,
sort_keys=True)`"* and did not:

```
python: {"count": 2, "event": "harness_team_dispatched", "kind": "team", …}
rust:   {"count":2,"event":"harness_team_dispatched","kind":"team",…}
```

**Every reader parses, so nothing failed — and this module's own header says two implementations
writing one file differently "would make every ledger change unreadable".** It took the first
comparison of a WRITTEN line against Python's to see it; reading a line back is not comparing it.
Fixed with `py_dumps_sorted`, which renders Python's default separators (`", "`, `": "`), sorted
keys, and `ensure_ascii=False` for the ledger. **`--json` is the other Python default**
(`ensure_ascii=True`, insertion order) and is rendered separately by `Recorded::to_pretty`, which
reuses `registries::ensure_ascii`.

**The test that was supposed to catch this could not fail.** `a_receipt_line_is_sorted_json` split the
line on `","` — which the compact form happened to produce, and which Python's separators do not. It
now asserts the **exact line**, so separators, sort order and escaping are all pinned.

## The one divergence that remains, and it is deliberate

| | Rust | Python |
|---|---|---|
| a receipt's `ts` | `2026-10-14T08:36:06Z` | `2026-10-14T04:36:06-04:00` |

**Same instant, different rendering, and the port already decided this**: `now_rfc3339`'s own comment
says *"The Python side writes a local offset; both are valid RFC3339 and the reader parses either, so
the port does not need to know the machine's timezone."* **It is stated here because this unit is the
first to compare the written bytes, and a reader of the ledger will meet it.** Comparing the two
forms as instants is asserted in the comparison, not assumed.

## Behaviour copied rather than corrected

* **`--count 0` becomes 1**, because Python's `args.count or 1` tests truthiness.
* **A string is redacted; a list is NOT redacted element by element**, because Python tests
  `isinstance(value, str)`. A "better" port would redact inside the list and write a different ledger.
* **No lab identity warns and PRESERVES the receipt** (`"lab": ""`, shard `unknown-<month>`), rather
  than refusing: refusing loses the record instead of surfacing the gap.
* **`--scope` takes several values**, so only its first is consumed by the flag parser and the rest are
  collected by `flag_all_values` — Python's `nargs="*"`.

## Verification: both implementations on scratch repositories

Six behaviour cases, each run against a scratch checkout (its own `docs/harness`, its own empty
`ledger/`), comparing **stdout, stderr, exit code AND the written shard line**:

| case | stdout | stderr | ledger line | exit |
|---|---|---|---|---|
| full (`--milestone --task --scope a b --count 2 --state running`) | MATCH | MATCH | MATCH | 0/0 |
| minimal (role only) | MATCH | MATCH | MATCH | 0/0 |
| `--count 0` | MATCH | MATCH | MATCH | 0/0 |
| `--json` | MATCH | MATCH | MATCH | 0/0 |
| no `UOR_LAB` (unattributed) | MATCH | MATCH | MATCH | 0/0 |
| `--task '  '` (whitespace is not empty) | MATCH | MATCH | MATCH | 0/0 |

Error paths, both sides:

```
team dispatch no-such-role  -> exit 2, both: unknown role 'no-such-role'; try: uor-harness team roster
team dispatch               -> exit 2, both: unknown role None; try: uor-harness team roster
team dispatch R --count xyz -> exit 2 both; the message names the argument on both sides
team dispatch R --state bogus -> exit 2 both; ditto
```

**The two implementations agree on every case once the timestamp is compared as an instant**, which is
the one documented divergence above.

| check | result |
|---|---|
| `cargo test -p uor-r4-harness` | **438 passed, 0 failed, 6 ignored** (was 432/6: 6 new tests) |
| `cargo fmt --check`, clippy `-D warnings` | clean, 0 findings |
| `python3 -m unittest discover -s tests/harness -t .` | exit 0, 0 failure lines |
| `python3 scripts/harness/parity.py` | **exit 0** — including *Rust-written receipt read back by Python: ok* |
| `python3 scripts/check_no_softmax.py` | OK — 1641, baseline 1641 |
| `delegate_to_python` sites | **4 — was 5; the `team` site is gone** |

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** the remaining four sites — `mistakes show|capture`, `multilab sync-live|migrate-ledger`, and
`arena`/`budget`/`panel` behind the general fallback — plus the two recorded `check` defects
(`status.no_orphan_branches` reports `ok` where Python warns; the non-ASCII `--json` question).
