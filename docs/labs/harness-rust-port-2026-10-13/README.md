# The harness moves to Rust: freezing the state formats first

**Question.** D27 (owner, 13 October 2026, on the Kimi session) makes Rust the default for all new
project code *including tooling*, and grandfathers the existing `scripts/harness/` Python. Nearly
everything the harness gained on 12 October — presence, committed claims, sharded receipts, the
touch primitive — is **new Python in that directory**, so it contradicts D27 as written. The owner
resolved the contradiction directly: **port the harness to Rust**, and **freeze the state formats as
the interface first**.

**Line:** no model line — tooling. · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).
No model compute was run.

## Why freeze first

A port is only worth anything if both implementations read and write the same state. Without a
frozen contract, "port" quietly becomes "rewrite with a new schema", and the record the labs
already share becomes readable by only one of them. So the formats were recorded *before* any
porting began.

[`docs/harness/state-formats.json`](../../harness/state-formats.json) is the machine-readable
contract, **generated from the real files** so it cannot describe a shape that does not exist. Eight
formats, 43 ledger rows, 8 panels:

| format | holds |
|---|---|
| `milestones`, `roles`, `arena`, `research-data` | the registries: lines and counts, the expert team, the regression arena, the data manifest |
| `claims` | who holds which branch — the shared store |
| `presence` | the committed per-lab check-in |
| `panels` | the acceptance panels |
| `ledger` | one receipt per line, sharded per lab |

[`docs/harness/STATE-FORMATS.md`](../../harness/STATE-FORMATS.md) is the readable half, and
`scripts/harness/statecheck.py` is the enforcement: wired into `check` as rule
`harness.state_formats`, so it runs wherever the harness runs, including CI.

```
$ python3 scripts/harness/cli.py check --only harness.state_formats
harness check: 1 rules, all pass
```

## Three real defects found while freezing

Writing the contract against the actual files found three things a schema written by hand would
have missed, and each one would have produced a port that disagreed with Python:

**1. Required versus optional.** Deriving from the first file declared every observed key required,
and 766 ledger rows "failed" immediately. The truth is that a receipt's shape depends on its kind —
`skill`, `heartbeat`, `claim` and `team` receipts share only `kind`, `lab`, `schema` and `ts`. The
contract now marks a key optional when any document omits it, and only those four are always
required.

**2. Absence must not invent a type.** A key present in some rows was inferring `mixed` from the
rows that lacked it. Its type now comes only from the documents that actually carry it.

**3. `count` is genuinely polymorphic.** It is a **string** in heartbeat receipts (`"1/3"`) and a
**number** in team receipts. That is not a bug to normalize away — it is real, the contract says
`mixed` explicitly, and a reader must handle both. Finding it now is the point: a Rust port with
`count: String` would fail at runtime on real data.

Two further shapes are documented rather than "fixed": an empty array matches any item shape
(M5–M8 panels legitimately have no bars yet), and `presence/`'s live tier lives outside the
repository by design.

## What the freeze guarantees, and what it does not

It guarantees a reader written against the contract can read the state, and that the two
implementations agree about what the state *means*.

It does **not** guarantee the state is truthful. A fabricated presence record matches the format
perfectly — that was the whole problem with the two records removed in #2275, and it is why
`presence/README.md` forbids test data outright rather than relying on a schema check.

## Port plan, and what is next

The Python harness is 10,387 lines across 16 modules, with ~3,900 lines of tests defining its
behaviour. That is not one sitting, so:

1. **Freeze the formats** — this unit.
2. **A Rust crate reading the frozen state**, with round-trip tests proving it reads what Python
   wrote. `tools/lab-runner` exists in the workspace but is the retired `coord` mechanism and is
   explicitly not to be recreated, so this is a fresh crate.
3. **Port behaviour** against the Python test suite, which becomes the conformance oracle.
4. **Cut CI and the labs over** once the Rust binary matches Python on the rules the labs depend on.
   The grandfather means Python keeps working until then, so nothing freezes in the meantime.

Stated plainly: the Python harness is still what the labs run today, and it stays that way until the
Rust path passes the same checks.

**Next:** the Rust crate. Its first test is that it reads every file in `state-formats.json` and
agrees with the contract — the same contract, checked by two languages.
