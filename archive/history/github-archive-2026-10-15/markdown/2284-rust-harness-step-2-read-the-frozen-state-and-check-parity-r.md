# #2284 — Rust harness step 2: read the frozen state, and check parity rather than assume it (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T04:11:25Z
- **Updated:** 2026-10-13T04:13:29Z
- **Closed:** 2026-10-13T04:13:09Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2284
- **Merged:** 2026-10-13T04:13:09Z
- **Merge commit:** b41084ce392487c788ea70b1d45f477c5509f2a9
- **Base / Head:** main / UOR-Foundation:harness/rust-parity
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only; nothing here runs model code.

## What and why

Step 2 of the Rust port: `crates/uor-r4-harness` reads and validates the state the Python harness
writes, against the same frozen contract.

```
$ uor-harness check --repo .        # state against the contract, exit 1 on drift
$ uor-harness presence --repo .     # what each lab is doing, live tier first
$ uor-harness ledger --repo .       # receipts by lab and kind
```

**It does not write, and it is not on the labs' path.** The Python harness owns behaviour until this
passes the same checks.

## Result

Decision: **KEEP** (tooling).

### Parity is checked, not assumed

Two implementations that disagree are worse than one, because the shared record then says different
things depending on which tool a lab runs. So `scripts/harness/parity.py` makes agreement a
regression test:

```
$ python3 scripts/harness/parity.py
parity: rust (uor-harness) vs python, 8 formats, 46 receipts, 2 lab(s) with presence
  agree on every format, the ledger roll-up, and the presence board
  agree on all 6 controlled cases (accept and reject)
```

It compares **structured output**, not prose, so a wording change cannot fail it and a real
disagreement cannot hide behind one. And it includes **controlled cases** with verdicts known in
advance, because agreeing on whatever state happens to be in the working tree only proves both can
read a *valid* file: a valid row, a timestamp as a number, a missing required key, `count` as a
number, `count` as an object, and non-JSON.

### That check found a bug in both implementations on its first run

The `count`-as-an-object case **passed in both** when the expectation was rejection. The `mixed`
kind was implemented as "accept anything", which is not what it means. `mixed` means the key carries
more than one *recorded* type — `count` is a string in heartbeat receipts and a number in team
receipts — and a value outside that set is drift. A real reader cannot handle a type that was never
observed, so **both implementations were silently accepting state neither could read.**

Fixed by recording the observed types in the contract itself (`types: ["number","string"]` for
`count`) rather than hard-coding them in either language. `statecheck.py` also gained `--repo`:
without it the Python checker could only ever check the checkout it lived in, which made a
controlled comparison impossible in the first place.

That is the argument for building the parity check *before* porting behaviour — it found a
two-implementation disagreement on the first run, in code that had tests and had passed them.

### Foreign-lab presence, read identically by both

```
lab          state           via      milestone branch
codex        live (5m ago)   live     m2        -
kimi         live (23m ago)  durable  m1        kimi/d27-rust-first-2026-10-13
```

Codex committed its presence in #2279 and Kimi in #2278, so **the presence path is now proven
end-to-end by foreign labs** — not just the claim and ledger paths as I reported last round.

```
cargo fmt --check / cargo clippy          -> clean
cargo test -p uor-r4-harness              -> 18 passed + 1 doc-test
python3 -m unittest discover -s tests/harness  -> 278 passed
python3 scripts/harness/parity.py         -> agree on all formats, ledger, presence, 6 cases
python3 scripts/harness/cli.py check      -> 46 rules, 0 errors
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] `python3 scripts/check_claim_wording.py` passes
- [x] Record: docs/labs/harness-rust-port-2026-10-13/README.md, plus a current-state entry
- [x] Branch deleted after merge

### Stated rather than hidden

- **Python still owns behaviour.** Reading state is not behaviour. Claims, presence, the rule engine
  and the receipt writers are all still Python, and the labs still run Python. The grandfather keeps
  it working until the Rust path passes the same rules.
- Parity covers **reading**. It says nothing yet about writing, so a Rust writer could still diverge.
- `target/` in this checkout is a **broken symlink** to a removed external archive, so `cargo` cannot
  use it; builds use `CARGO_TARGET_DIR=local/harness-target` (gitignored). The symlink is not mine to
  delete and I left it alone. This matters for CI: the workflow will need the same treatment, and I
  have not touched CI in this PR.
- The Rust binary is not built by CI yet, so `parity.py` reports "not built" (exit 2) rather than
  failing there. Wiring it in is the next step, deliberately separate so this unit stays reviewable.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

