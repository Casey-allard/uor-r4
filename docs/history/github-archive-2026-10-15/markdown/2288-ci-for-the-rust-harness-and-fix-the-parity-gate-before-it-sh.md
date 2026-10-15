# #2288 — CI for the Rust harness, and fix the parity gate before it ships (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T04:31:31Z
- **Updated:** 2026-10-13T04:33:55Z
- **Closed:** 2026-10-13T04:33:35Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2288
- **Merged:** 2026-10-13T04:33:35Z
- **Merge commit:** 261e5f21d512d59560b987549dd75b553a70c0a2
- **Base / Head:** main / UOR-Foundation:harness/ci-rust
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling and CI only.

## The important part: the gate was broken, and wiring it up is what revealed that

`parity.py` — the check the last two units were built around — compared violations **by count, not by
content**, and never failed when the state was actually invalid.

Demonstrated on real state. A ledger row missing its required `ts` key:

```
RUST:    ledger   2 PROBLEM(S) (47 rows)
PYTHON:  ledger   2 PROBLEM(S) (47 rows)

$ python3 scripts/harness/parity.py
  agree on every format, the ledger roll-up, and the presence board
exit=0
```

Both implementations found the fault. The gate that exists to catch disagreement **passed and exited
0**. So a state file violating the frozen contract — or a Rust implementation that disagreed with
Python about it — would have sailed through the CI job this PR adds.

That is the failure mode worth naming: **a gate that only compares two implementations passes
whenever both are wrong together.** It manufactures confidence rather than checking anything.

### Fixed, three defects

1. **Compared counts, not faults.** `{'problem_count': 2} == {'problem_count': 2}` is agreement about
   arithmetic. It now compares the normalised fault text, so *"both found the same thing wrong"* is
   what is asserted.
2. **Never failed on invalid state.** It now fails outright when state violates the contract,
   whichever implementation noticed.
3. **Compared repr style.** Python renders `'ts'`, Rust renders `"ts"` for the same fault, and the
   first content comparison flagged that as a disagreement. Quote style is normalised away —
   otherwise this becomes a check on each language's formatting, which is the kind of noise that gets
   a gate ignored.

**Verified in both directions**, because a check that only passes proves nothing:

```
valid state            -> agree on every format, ledger, presence, 6 cases    exit 0
a row missing `ts`     -> DISAGREEMENT: state violates the frozen contract     exit 1
```

## Result

Decision: **KEEP** (tooling).

`.github/workflows/harness.yml` gains `rust-harness`: pinned Rust, cached cargo, `cargo fmt --check`,
`clippy -D warnings`, `cargo test --locked`, `parity.py`, `statecheck.py`. The path filter now covers
`crates/uor-r4-harness/**`, `Cargo.toml` and `Cargo.lock` — and since this PR also edits the workflow
file, **the new job validates itself on the PR that introduces it**.

Every step run locally first, from a **cold** build directory to match CI:

```
cargo fmt -p uor-r4-harness -- --check                              clean
cargo clippy -p uor-r4-harness --locked --all-targets -D warnings   0 findings
cargo test --locked -p uor-r4-harness                               19 passed + 1 doc-test
python3 scripts/harness/parity.py --json                            ok, 8 formats, 6 cases
python3 scripts/harness/statecheck.py                               pass
```

Cold build 5.7 s; the crate depends only on `serde`/`serde_json`. `CARGO_TARGET_DIR=local/harness-target`
is set on every cargo step — not stylistic: **`target/` in the owner's checkout is a broken symlink**
to a removed external archive, so a job relying on it would fail for a reason unrelated to the code.

## Also: my stale worktree, removed with proof

You asked me to clear it. `deepseek/invariant-router` (1.3 GB) — the one I declined to remove last
unit because a branch 40 commits behind makes file-level comparison unreliable. Checked properly
this time: **no claim on it**, **clean tree**, and **both commits superseded by content** — the
`read_router` literals are present in `main`'s `cuda_stack_ops_parity.rs`, `temporal_cli.rs` and
`d4_map_codec_adapter.rs` (via #2245), and the `CellTree::route` change is in `main` (via #2247).

Archived as a restorable patch with an INDEX row and SHA256 before deletion. Removing it also cleared
a warning: `status.no_orphan_branches` went from 7 failures to 6.

**Disk: 33 GiB free**, near the 30 GB floor. Codex's `geometric-path-repair` worktree still holds 12
uncommitted files and remains untouched, so the next safe reclaim depends on Codex committing.

```
python3 -m unittest discover -s tests/harness -t .   -> 281 passed
python3 scripts/harness/cli.py check                 -> 44 rules, 0 errors
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling and CI only; no capability or headline changed
- [x] Record: docs/labs/harness-ci-2026-10-13/README.md, plus a current-state entry
- [x] Branch deleted after merge

### Stated rather than hidden

- **The CI job's proof is its first real run on this PR.** It is configured and locally simulated;
  a green job that never fails is precisely what this PR just fixed, so I am not claiming more.
- `parity.py` covers **reading**. Writing — claims, presence, receipts — is still Python-only and
  uncompared, so a Rust writer could still diverge.
- A PR touching the harness now waits on **two** jobs. That is a deliberate cost; the honest
  alternative is dropping one of them.
- Frozen state that happens to be *absent* still passes, by design — a fresh checkout has no claims
  and no presence, and treating that as drift would make the checker useless exactly when someone
  first runs it.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

