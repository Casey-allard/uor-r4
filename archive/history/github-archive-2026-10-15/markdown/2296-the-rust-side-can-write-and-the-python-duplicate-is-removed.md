# #2296 — The Rust side can write, and the Python duplicate is removed (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T05:08:10Z
- **Updated:** 2026-10-13T05:18:14Z
- **Closed:** 2026-10-13T05:17:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2296
- **Merged:** 2026-10-13T05:17:52Z
- **Merge commit:** c1ba70de204c189b92d3b84ee3b54137f4e3f6c6
- **Base / Head:** main / UOR-Foundation:harness/rust-writer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only. Rust replaces Python.

## You were right, and I had it backwards

D27: *"Rust is the default for all new project code — model, training, tooling, data preparation and
scripts... Python is used only where Rust genuinely cannot do the job... **no new Python is added
where Rust works**."*

I added **three new Python files after D27 landed**: `statecheck.py` (248 lines), `parity.py` (410),
`doccheck.py` (374). This fixes the clearest case and closes the untested half of the port.

## Result

Decision: **KEEP** (tooling).

### 1. The Rust side can now write

Every check until now compared the two implementations on state **Python wrote**. That proves Rust can
read the shared record and says **nothing** about whether it can add to it. A writer that is subtly
wrong is worse than no writer: it corrupts the record the labs coordinate through.

`uor-harness receipt` appends to this lab's shard, matching the Python contract exactly — sharded by
lab and month, one JSON object per line with keys sorted, append-only, and the four keys the contract
requires always present (`kind`, `lab`, `schema`, `ts`).

**Verified end to end on the real ledger:** Rust wrote a receipt, created a new shard
(`deepseek-2026-10.jsonl`), and **Python read it back with every field intact**. The test then removed
its own receipt, so the interop proof left no trace in the record. That exchange is now a permanent
check in `parity.py`, running in a temp directory so CI cannot pollute real state.

```
$ python3 scripts/harness/parity.py
  state valid in all 8 formats (one implementation; nothing to compare)
  ledger roll-up and presence board agree across both implementations
  the format checker accepts and rejects all 6 controlled cases
  Rust-written receipt read back by Python: ok
```

### 2. The Python format checker is deleted

`statecheck.py` was a **second implementation** of a check the Rust crate already performed in
`contract.rs`. Two implementations of one rule is how they drift apart, and it was ~250 lines of new
Python in the language the project had just decided against.

The check is now the Rust `uor-harness check`, surfaced as the `harness.state_formats` rule through a
thin adapter (`scripts/harness/stateformats.py`) — an adapter, not a second implementation.

**The CI wrinkle, handled honestly:** the `harness` job has no Rust binary, so the rule reports
`skipped` there rather than failing a check nobody broke. The `rust-harness` job builds the binary and
runs `check --only harness.state_formats`, so the rule **does** execute in CI.

Parity lost its format half as a result. Comparing Rust against a Python checker that no longer exists
would be comparing Rust with itself, so that section now asserts what is still meaningful — state
valid, six controlled cases accept/reject correctly, ledger and presence agree across
implementations, and the write interop above.

**The dated records describing `statecheck.py` are left as written**, with a dated correction
appended. Rewriting them would falsify the record — the same prescriptive-versus-descriptive line the
doc checker draws.

### Two bugs found building this, both in my own new code

- `--repo` incremented the argument index **twice**, skipping the command entirely, so
  `--repo DIR receipt ...` ran nothing and printed usage.
- `--lab` is a global flag the subcommand also reads; the top-level loop consumed it without
  forwarding, so **every** receipt invocation was rejected as an unknown option.

```
cargo fmt / clippy -D warnings        -> clean, 0 findings
cargo test -p uor-r4-harness          -> 37 passed + 1 doc-test
python3 -m unittest                   -> 295 passed on 3.15 AND 3.9
python3 scripts/harness/cli.py check  -> 46 rules, 0 errors
```

## Also: the PM's two proposed checks, implemented

`docs/harness/pm-rules.md` (Kimi, merged as #2295) asked the harness owner to consider two machine
checks. Both are now in `multilab`, and between them they found a real problem on the first run:

| rule | flags |
|---|---|
| `multilab.claim_not_merged` | a claim whose branch already has a merged PR |
| `multilab.claim_stale` | a claim held >24h with no live heartbeat |

**They caught three live claims immediately:** Kimi holds claims on `kimi/d27-rust-first-2026-10-13`,
`kimi/pm-rules-2026-10-13` and `kimi/rung0-sheet-bars-2026-10-13`, whose PRs **#2278, #2295 and
#2290 are all merged** and whose branches are deleted — verified against GitHub, not inferred.

### And the merged-claim check had a false positive worth recording

The first version treated "not on origin" as merged. **But a branch that was never pushed looks
identical**, so a lab holding a claim on unpushed work would have been told to release it — a check
that destroys work rather than protecting it.

It is now evidence-based: a merged PR, or no commits ahead of `origin/main`. When neither can be
established it returns `None` and the rule **stays quiet**, because guessing here is worse than
saying nothing. Tests pin both the real case (a merged branch is flagged) and the never-pushed case
(it is not).

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/harness-rust-port-2026-10-13/README.md (dated correction appended)
- [x] Records corrected with dated notes, not rewritten
- [x] Branch deleted after merge

### Stated rather than hidden

- **Two of the three are still Python.** `parity.py` and `doccheck.py` remain and still need porting.
  `doccheck` is the more egregious: checking an argument parser for named commands is a pure-Rust job
  with no Python need at all. I have not done it in this PR because a half-ported tool is worse than
  a working one, and I would rather say so than pretend the audit is complete.
- **`parity.py` is inherently two-language** — it must invoke Python to read the Python-written state.
  Its comparison logic can move to Rust; the invocation cannot, until Python itself is gone.
- The `harness` CI job now **skips** one rule. That is a real coverage gap in that job, mitigated by
  `rust-harness` running the rule, and it is visible rather than silent by design.
- Removing `statecheck.py` means a Python-only environment can no longer check the formats at all. The
  rule says `skipped` with the build command rather than pretending to pass.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

