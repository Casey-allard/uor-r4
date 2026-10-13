# The harness state formats (frozen interface)

The harness persists its state as JSON and JSONL under `docs/harness/`. Those files are the
interface between the harness and everything else — the labs that read them, the CI gate that
reconciles them, and the Rust port now under way. They are **frozen**.

**Why frozen, and why now.** On 13 October 2026 the owner directed that the harness be ported to
Rust (D27: Rust first, including tooling). A port is only trustworthy if both implementations read
and write the *same* state, so the formats were recorded before any porting began. Without this,
the port becomes a rewrite with a new schema, and the record the labs already share would be
readable by only one of them.

## The contract

[`state-formats.json`](state-formats.json) is the machine-readable contract. It is generated from
the real files, so it cannot describe a shape that does not exist.

```
uor-harness check --repo .            # check every format (Rust)
uor-harness check --repo . --json     # machine-readable
uor-harness check --only harness.state_formats   # the same rule via the harness
```

The check lives in `crates/uor-r4-harness` (`contract.rs`). D27 puts new tooling in Rust, and an
earlier Python duplicate (`statecheck.py`) was removed — two implementations of one rule is how they
drift apart.

**A key without an `optional` mark must appear in every document of that format. A key marked
`optional` appears in some and not others.** A kind of `mixed` means the key carries more than one
JSON type across records, listed in the key's `types` array; `count` is a string in heartbeat
receipts and a number in team receipts, so `types` is `["number", "string"]` and a reader must
handle both. **A value outside `types` is drift.** An empty array matches any item shape.

That last clause was added after the parity check caught both implementations accepting an object
for `count`: treating `mixed` as "anything" is not the same as "these types", and a real reader
cannot handle a type that was never observed. See
[the parity check](#two-implementations-one-contract) below.

| format | file | holds |
|---|---|---|
| `milestones` | `docs/harness/milestones.json` | the milestone registry: lines, counts, targets, owner rules |
| `roles` | `docs/harness/roles.json` | the expert team and what each role is for |
| `arena` | `docs/harness/arena.json` | the regression arena rows, each with a declared direction |
| `claims` | `docs/harness/claims.json` | who holds which branch — the shared store |
| `presence` | `docs/harness/presence/<lab>.json` | the committed per-lab check-in |
| `panels` | `docs/harness/panels/*.json` | the acceptance panels |
| `research-data` | `docs/harness/research-data.json` | the research data manifest |
| `ledger` | `docs/harness/ledger/<lab>-<month>.jsonl` | one receipt per line, sharded per lab |

Two state directories are deliberately **not** covered, and the reason matters for the port:

- **`presence/` live tier** (`~/.local/share/uor-r4/harness/`) is intentionally outside the
  repository. It is real-time state, not the record, and a Rust port must read both tiers.
- **`docs/harness/skills/`, `mistakes/`, `prep/`** are Markdown with a front-matter block, not JSON.
  They are part of the harness's surface and the port must read them, but they are human documents
  rather than a serialization format, so they are out of scope for the format check.

## Changing a format

A format change is an interface change and follows the same rule as any other interface here: make
it deliberately and review it.

1. Change the code and the files together.
2. Regenerate: the generator lives in the record for the port,
   [`docs/labs/harness-rust-port-2026-10-13/README.md`](../../labs/harness-rust-port-2026-10-13/README.md).
3. Commit the regenerated `state-formats.json` **in the same PR**, so the diff shows exactly what
   changed about the contract.
4. Run `uor-harness check` and confirm every format passes.

**Never regenerate to silence a failure you have not understood.** An unintended shape change is
precisely the drift this file exists to catch, and the check is wired into `check`, so it runs
wherever the harness runs.

## Two implementations, one contract

The Rust port reads the same contract file. `scripts/harness/parity.py` is the regression test that
the two agree:

```
$ python3 scripts/harness/parity.py
parity: rust (uor-harness) vs python, 8 formats, 46 receipts, 2 lab(s) with presence
  agree on every format, the ledger roll-up, and the presence board
  agree on all 6 controlled cases (accept and reject)
```

It compares **structured output**, not prose, and it includes controlled cases with verdicts known
in advance — a valid row, a timestamp as a number, a missing required key, `count` as a number, and
`count` as an object. Agreement on whatever state happens to be in the working tree would only prove
both can read a *valid* file; the controlled cases prove both **reject** the same things.

Both implementations also accept `--repo`, so either can be pointed at any checkout. That is what
makes a controlled comparison possible at all, and it is how the controlled cases below are built.

The format check now has **one implementation**, so parity no longer compares it against a second —
there is nothing to compare. What parity still asserts is that the checker runs, that the state is
valid, that it accepts and rejects six known inputs correctly, and that a Rust-written receipt is
read back intact by Python.

**What parity does not prove.** Agreement about *reading* state is not agreement about behaviour.
The Python harness owns behaviour — writing claims, publishing presence, the rule engine — until
those are ported and compared the same way. This is the first such comparison, not the last.

## What the freeze guarantees, and what it does not

It guarantees that a reader written against the contract can read the state, and that the Python and
Rust implementations agree about what the state means.

It does **not** guarantee that the labs' state is correct, that a lab checked in, or that a
record is truthful. Those are different problems with different checks — presence freshness is
`multilab.presence_fresh`, claims ownership is `multilab.claims_one_owner`, and whether a record was
fabricated is not machine-checkable at all, which is why `presence/README.md` forbids test data
outright.
