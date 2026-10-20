# #2282 — Freeze the harness state formats before the Rust port (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T03:59:42Z
- **Updated:** 2026-10-13T04:01:44Z
- **Closed:** 2026-10-13T04:01:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2282
- **Merged:** 2026-10-13T04:01:22Z
- **Merge commit:** 9a20ca9538f801c193004a8e5a69ff40aa140e90
- **Base / Head:** main / UOR-Foundation:harness/freeze-state-formats
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only: this touches the harness, which runs no model code.

## What and why

D27 makes Rust the default for all new code **including tooling** and grandfathers the existing
`scripts/harness/` Python — but nearly everything the harness gained on 12 October (presence,
committed claims, sharded receipts, the touch primitive) is **new Python in that directory**, so it
contradicted D27 as written. The owner resolved it directly: **port the harness to Rust**, and
**freeze the state formats as the interface first**.

Freezing first is the whole point. A port without a frozen contract is a rewrite with a new schema,
and the record four labs already share becomes readable by only one implementation.

[`state-formats.json`](docs/harness/state-formats.json) is the machine-readable contract,
**generated from the real files** so it cannot describe a shape that does not exist: eight formats,
43 ledger rows, 8 panels. [`STATE-FORMATS.md`](docs/harness/STATE-FORMATS.md) is the readable half.
`scripts/harness/statecheck.py` enforces it, wired into `check` as `harness.state_formats` so it runs
wherever the harness runs, including CI:

```
$ python3 scripts/harness/cli.py check --only harness.state_formats
harness check: 1 rules, all pass
```

## Result

Decision: **KEEP** (tooling).

**Three defects found while deriving the contract**, each of which would have produced a port that
disagreed with Python:

1. **Required versus optional.** Deriving from the first file declared every observed key required,
   and 766 ledger rows failed at once. A receipt's shape depends on its kind — `skill`, `heartbeat`,
   `claim` and `team` receipts share only `kind`/`lab`/`schema`/`ts`. The contract now marks a key
   optional when any document omits it.
2. **Absence inventing a type.** A key present in only some rows inferred `mixed` from the rows that
   lacked it. Its type now comes only from documents that carry it.
3. **`count` is genuinely polymorphic**: a **string** in heartbeat receipts and a **number** in team
   receipts. Not something to normalize away — real, now recorded as `mixed`, and a Rust
   `count: String` would have failed on live data.

Also documented rather than "fixed": an empty array matches any item shape (M5–M8 panels have no
bars yet), and `presence/`'s live tier lives outside the repository by design.

```
python3 -m unittest discover -s tests/harness -t .   -> 278 tests, OK
python3 scripts/harness/cli.py check                 -> 46 rules, 0 errors
python3 scripts/harness/statecheck.py                -> pass
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] `python3 scripts/check_claim_wording.py` passes
- [x] Record: docs/labs/harness-rust-port-2026-10-13/README.md
- [x] Newest-first entry in docs/integration/current-state.md, with **Next:**
- [x] Branch deleted after merge

### Stated rather than hidden

- **The Python harness is still what the labs run.** This freezes the interface and starts the
  port; it does not cut anything over. The grandfather keeps Python working throughout.
- The freeze does **not** make the state truthful. A fabricated presence record matches the format
  perfectly — that was the whole problem with the records removed in #2275. Schema validity and
  record truthfulness are different properties with different checks.
- `presence/`'s live tier and the Markdown surfaces (`skills/`, `mistakes/`, `prep/`) are
  deliberately outside `statecheck.py`; the port must still read them, and the record says so.
- Porting 10,387 lines of Python with ~3,900 lines of tests is multi-session. The Python suite
  becomes the conformance oracle — the plan is in the record, not claimed as done here.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

