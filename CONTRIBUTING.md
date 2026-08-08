# Contributing to R⁴

[AGENTS.md](AGENTS.md) is the full operating manual — gates, normative
invariants, the κ re-pin procedure, long-run discipline. Read it before your
first change. This file is the short version.

## The loop

1. **Assign yourself the issue.** In this repo assignment means
   *actively-being-worked-right-now*, not "someday" — queued work stays
   unassigned so anyone can pick it up. Never start an unassigned issue without
   assigning it first.
2. Branch `issue-<n>-<slug>` off `main`. **No direct pushes to `main`.**
3. Work, then run the gates below.
4. Open a PR. It merges through the queue; a queued PR's head branch is locked,
   so follow-up work goes on a fresh branch off `main` after it lands.
5. **Close the issue with the evidence** — the numbers, the verdict against the
   pre-declared exit rule, and the merge commit. Then unassign yourself.
6. Follow-up work discovered mid-stream gets **filed as an issue immediately**,
   not left in a PR body.

## Gates

All clean before every commit:

```bash
cargo test --workspace --offline
cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
cargo fmt --check
cargo check -p uor-r4-graph-format --no-default-features
cargo check -p uor-r4-graph-format --no-default-features --features alloc
python3 scripts/check_claim_wording.py
```

Changes under `uor-r4-core` or `uor-r4-router` also need the wasm target —
clippy does not build it and the merge queue does:

```bash
cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib
```

Scope your local checks to what you touched; CI is the real gate. Running the
full ladder on every push wastes more time than it saves.

**Check the check.** Some ways these have silently lied before:

- `cargo … | grep …; echo $?` reports *grep's* exit status. Read
  `${PIPESTATUS[0]}` or run bare.
- After changing a public signature, `cargo clean -p <crate>` before the
  verifying run — stale test targets have "passed" pre-edit code four times.
- Lint the way CI lints. Local defaults mask `dead_code` and exit 0 on warnings.
- A κ test that finishes suspiciously fast has skipped. Confirm
  `/tmp/ref/out/model.bin` exists before trusting green.

## Normative invariants — do not weaken

- **Runtime kernel**: XOR/AND/OR/shift/rotate/popcount/integer add-sub/compare
  and table reads only. No multiply, divide or float in the deployed kernel,
  enforced by a machine-checked source scan. Compiler and certifier code may use
  floats and allocation; runtime code may not.
- **Allocation**: the prediction hot path is allocation-free in steady state,
  asserted by a test.
- **Determinism**: identical pinned inputs produce identical artifact bytes. No
  HashMap-iteration-order, clock or RNG dependence in compiler outputs.
- **Errors**: library boundaries return `Result` with focused error enums. No
  `unwrap`/`expect`/panic on recoverable paths.
- **No `unsafe`** in the portable runtime or the format crate.
- **Claim language**: `docs/formal_vocabulary.md` is normative, and
  `scripts/check_claim_wording.py` is CI-enforced. Exact-equivalence wording
  needs a linked proof artifact.

## If your change is a measurement

Most substantive work here is. The expectations:

- **Pre-declare the exit rule, the null baseline and the falsifier** before you
  run anything. Write them in the issue.
- **Compute the reachability ceiling first.** If the ceiling is below the effect
  you are hoping for, do not launch. This is a five-minute calculation that has
  invalidated four-hour runs.
- **Run the cheap instrument first and treat its verdict as binding.**
  `cargo test -p uor-r4-graph-certify --test capacity_scaling -- --ignored` takes
  ~12 minutes and prints a saturation verdict per structure.
- **Pre-declare what each outcome causes.** If the positive and negative branches
  lead to the same next action, the run has no decision value — drop it or
  redesign it.
- **Record negatives.** They are kept, not discarded, and several have redirected
  the whole programme.
- **Make sure your instrument can fail.** Assert the control arm is
  non-degenerate before comparing. An all-zero result across every arm is a
  harness bug until proven otherwise — seven have been found here.

Paste the run contract into the issue before launching:

```
metric to move:       <name, current value>
reachability ceiling: <arithmetic, with the numbers it came from>
instrument + verdict: <which cheap test, what it must report to proceed>
exit rule:            <threshold, pre-declared>
if positive:          <the next action>
if negative:          <the next action, and it must differ>
cost estimate:        <wall clock, and what else it blocks>
```

## Documentation

Records live in `docs/` as `<topic>_<issue>.md` and are **appended to, not
rewritten**, when a later result revises them — the history of what was believed
and when is part of the record. If your work changes a claim in `README.md`,
`docs/RESEARCH.md` or another record, correct it in place and say what revised
it.

## License

By contributing you agree your contributions are licensed under the MIT License.
