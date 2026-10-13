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

## Step 2: the Rust crate reads the frozen state

`crates/uor-r4-harness` now reads and validates the state the Python harness writes, against the
same contract file. It does not write, and it is not on the labs' path — the Python harness owns
behaviour until this passes the same checks.

```
$ CARGO_TARGET_DIR=local/harness-target cargo test -p uor-r4-harness --offline
test result: ok. 18 passed; 0 failed          (+ 1 doc-test)
```

Three commands, all reading real state:

```
$ uor-harness check --repo .        # state against the frozen contract, exit 1 on drift
$ uor-harness presence --repo .     # what each lab is doing, live tier first
$ uor-harness ledger --repo .       # receipts by lab and kind
```

### `scripts/harness/parity.py` — the regression test that the two agree

Two implementations that disagree are worse than one, because the shared record then says different
things depending on which tool a lab runs. So agreement is checked, not assumed:

```
$ python3 scripts/harness/parity.py
parity: rust (uor-harness) vs python, 8 formats, 46 receipts, 2 lab(s) with presence
  agree on every format, the ledger roll-up, and the presence board
  agree on all 6 controlled cases (accept and reject)
```

It compares **structured output**, not prose, so a wording change cannot fail it and a real
disagreement cannot hide behind one. And it includes **controlled cases** with verdicts known in
advance, because agreeing on whatever state is in the working tree only proves both can read a
*valid* file. The cases: a valid row, a timestamp as a number, a missing required key, `count` as a
number, `count` as an object, and non-JSON.

### The parity check immediately found a bug in both implementations

The `count`-as-an-object case **passed in both** when the expectation was rejection. The `mixed`
kind was implemented as "accept anything", which is not what it means: `mixed` means the key carries
more than one *recorded* type — `count` is a string in heartbeat receipts and a number in team
receipts — and a value outside that set is drift. A real reader cannot handle a type that was never
observed, so both implementations were silently accepting state neither could read.

Fixed in both, by recording the observed types in the contract itself (`types: ["number", "string"]`
for `count`) rather than hard-coding them in either language. `statecheck.py` also gained `--repo`,
without which it could only ever check the checkout it lived in and a controlled comparison was
impossible.

That is the argument for building the parity check before porting behaviour: it found a
two-implementation disagreement on the very first run, in code that had tests and had passed them.

### A note on this checkout

`target/` in this checkout is a **broken symlink** to a removed archive on an external volume, so
`cargo` cannot use it. Builds here use `CARGO_TARGET_DIR=local/harness-target`, which is gitignored.
The symlink is not mine to delete and was left alone.

**Next:** port behaviour against the Python suite as the conformance oracle — claims and presence
first, since those are what the labs actually depend on — then cut CI over once the Rust path passes
the same rules. The grandfather keeps Python working throughout.

## Correction, later the same day: the Python format checker has been removed

This record describes `scripts/harness/statecheck.py` as the enforcement for the frozen contract. That
file **no longer exists**, and the record is left as written because it was true when it was written.

It was new Python added *after* D27 (`56350f55a`, 13 October) said new tooling goes in Rust, and it
duplicated a check the Rust crate already performed in `contract.rs`. Two implementations of one rule
is how they drift apart, and it was ~250 lines in the language the project had just decided against.

The check is now the Rust implementation alone, surfaced as the `harness.state_formats` rule through
`scripts/harness/stateformats.py` — a thin adapter, not a second implementation. The `rust-harness` CI
job builds the binary and then runs the rule, so it executes in CI; the `harness` job, which has no
binary, skips that one rule and says so rather than failing a check nobody broke.

The parity comparison lost its format half as a result. Comparing Rust against a Python checker that
no longer exists would be comparing Rust with itself, so that section now asserts what is still
meaningful: the state is valid, the checker accepts and rejects six known inputs correctly, the ledger
and presence boards agree across both implementations, and a Rust-written receipt is read back intact.
