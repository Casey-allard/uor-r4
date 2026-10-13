# CI learns about the Rust harness, and the parity gate gets fixed before it ships

**Question.** The Rust crate and `parity.py` were built but ran nowhere automatic. Wiring them into
CI is the obvious next step — and doing it exposed that **the parity gate did not work**. A check that
cannot fail is worse than no check, because it manufactures confidence.

**Line:** no model line — tooling. · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

## The gate was broken, and it failed a real fault

`parity.py` compared violations **by count**, not by content, and never failed when the state was
actually invalid. Demonstrated on real state: a ledger row missing its required `ts` key made both
implementations report the same two violations — and `parity.py` printed

```
agree on every format, the ledger roll-up, and the presence board
```

and **exited 0**. Net effect: a state file violating the frozen contract, or a Rust implementation
that disagreed with Python about it, would have passed the CI job this unit adds.

Two defects, both fixed:

**1. It compared counts, not faults.** `{'problem_count': 2} == {'problem_count': 2}` is agreement
about arithmetic, not about what was found. It now compares the normalised fault text, so
*"both found the same thing wrong"* is what is actually asserted.

**2. It never failed on invalid state.** A gate that only compares two implementations passes
whenever both are wrong together. It now fails outright when the state violates the contract,
whichever implementation noticed.

A third, smaller fix while in there: Python renders a repr as `'ts'` and Rust as `"ts"` for the same
fault, and the first version of the content comparison flagged that as a disagreement. Comparing
quote style would make this a check on each language's formatting rather than on the fault — exactly
the noise that gets a gate ignored — so quote style is normalised away.

**Verified in both directions**, which is the part that matters:

```
valid state            -> agree on every format, ledger, presence, 6 cases   exit 0
a row missing `ts`     -> DISAGREEMENT: state violates the frozen contract    exit 1
```

## The CI job

`.github/workflows/harness.yml` gains `rust-harness`: install pinned Rust, cache cargo, then
`cargo fmt --check`, `clippy -D warnings`, `cargo test --locked`, `parity.py`, and `statecheck.py`.
The path filter now covers `crates/uor-r4-harness/**`, `Cargo.toml` and `Cargo.lock`, so the job runs
on this PR — because the PR also edits the workflow file. **The new job validates itself on the pull
request that introduces it**, rather than waiting for a later change to exercise it.

Every step was run locally first, from a **cold** build directory, to match what CI gets:

```
cargo fmt -p uor-r4-harness -- --check                        clean
cargo clippy -p uor-r4-harness --locked --all-targets -D warnings   0 findings
cargo test --locked -p uor-r4-harness                         19 passed + 1 doc-test
python3 scripts/harness/parity.py --json                      ok, 8 formats, 6 cases
python3 scripts/harness/statecheck.py                         pass
```

Cold build: 5.7 s. The crate depends only on `serde`/`serde_json`, so the job is quick despite
compiling, and the 20-minute timeout is margin rather than expectation.

`CARGO_TARGET_DIR=local/harness-target` is set on every cargo step and the directory is cached.
That is not a stylistic choice: **`target/` in the owner's checkout is a broken symlink** to a
removed archive on an external volume, so a job that relied on it would fail for a reason that has
nothing to do with the code.

## Also: my own stale worktree, removed with proof

The owner asked me to clear my stale worktree too. `deepseek/invariant-router` (1.3 GB) — the one I
declined to remove last unit because the file-level comparison was unreliable against a branch 40
commits behind. This time I checked it properly:

- **no claim on it** — nobody held the branch
- **clean tree** — zero uncommitted files
- **its two commits are superseded by content**: the `read_router` literals are present in `main`'s
  `cuda_stack_ops_parity.rs`, `temporal_cli.rs` and `d4_map_codec_adapter.rs` (arrived via #2245),
  and the `CellTree::route` change is in `main` (via #2247)

The two commits are archived as
[`deepseek_invariant-router-20261013.patch`](../../history/branch-archive/deepseek_invariant-router-20261013.patch)
with an INDEX row and its SHA256, so the exact historical form is restorable even though `main` is
the authority. Removing it also cleared a `status.no_orphan_branches` warning: the check went from
7 failures to 6.

**Disk: 33 GiB free**, close to the 30 GB floor. Codex's `geometric-path-repair` worktree still holds
12 uncommitted files and stays untouched, so the next safe reclaim depends on Codex committing.

## Limitations

- **The CI job is only as good as its first real run.** It is configured and locally simulated; the
  proof is the green job on this PR, and a green job that never fails is what this unit just fixed.
- `parity.py` covers **reading**. Writing — claims, presence, receipts — is still Python-only and
  uncompared, so a Rust writer could still diverge.
- The Python job and the Rust job both run on a PR touching the harness, so such a PR waits on two
  jobs. That is a deliberate cost: they check different things, and the honest alternative is
  dropping one.
- `gh-pages` is excluded from the orphan-branch rule by name. If the project ever publishes another
  machine-managed branch, it needs the same treatment or it will show up as noise.
