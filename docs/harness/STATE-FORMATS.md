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
cannot handle a type that was never observed. See [One implementation](#one-implementation) below.

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
   [`archive/labs/harness-rust-port-2026-10-13/README.md`](../../archive/labs/harness-rust-port-2026-10-13/README.md).
3. Commit the regenerated `state-formats.json` **in the same PR**, so the diff shows exactly what
   changed about the contract.
4. Run `uor-harness check` and confirm every format passes.

**Never regenerate to silence a failure you have not understood.** An unintended shape change is
precisely the drift this file exists to catch, and the check is wired into `check`, so it runs
wherever the harness runs.

## One implementation

The Rust harness (`crates/uor-r4-harness`) is the only reader and writer of these formats. Until
20 October 2026 a Python mirror read the same contract, and `scripts/harness/parity.py` checked that
the two agreed on every format, the ledger roll-up, the presence board and six controlled accept and
reject cases. The owner retired the mirror (rule: Rust only, D27, CHARTER §3.11); both it and the
parity script are kept byte-identical under `archive/scripts/harness/`.

What the comparison guarded is now asserted in the crate: the contract tests in
`crates/uor-r4-harness/src/contract.rs` run every format against the real state and assert the
reject cases with verdicts known in advance (a missing required key, a kind change, a type a mixed
field never observed), because agreement on whatever state happens to be in the working tree only
proves a checker can read a *valid* file. `uor-harness check` runs the format check as
`harness.state_formats` wherever the harness runs.

## What the freeze guarantees, and what it does not

It guarantees that a reader written against the contract can read the state, and that a change to
what the state means shows up as a reviewed diff of `state-formats.json` rather than as drift.

It does **not** guarantee that the labs' state is correct, that a lab checked in, or that a
record is truthful. Those are different problems with different checks — presence freshness is
`multilab.presence_fresh`, claims ownership is `multilab.claims_one_owner`, and whether a record was
fabricated is not machine-checkable at all, which is why `presence/README.md` forbids test data
outright.
