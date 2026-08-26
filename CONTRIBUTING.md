# Contributing to R⁴

[AGENTS.md](AGENTS.md) is the full operating manual — gates, normative
invariants, the κ re-pin procedure, long-run discipline. Read it before your
first change. The current architecture and issue order live in the
[Geometric Causal Decoder Roadmap](docs/geometric_causal_decoder_plan.md).
This file is the short version.

## The loop

1. **Assign yourself the issue.** In this repo assignment means
   *actively-being-worked-right-now*, not "someday" — queued work stays
   unassigned so anyone can pick it up. Never start an unassigned issue without
   assigning it first.
2. Branch `issue-<n>-<slug>` off `main`. **No direct pushes to `main`.**
3. Work, then run the focused checks below.
4. Open a PR. It merges through the queue; a queued PR's head branch is locked,
   so follow-up work goes on a fresh branch off `main` after it lands.
5. **Close the issue with the evidence** — the numbers, the verdict against the
   pre-declared exit rule, and the merge commit. Then unassign yourself.
6. Follow-up work discovered mid-stream gets **filed as an issue immediately**,
   not left in a PR body.

## Focused checks

Use the smallest check that exercises what changed:

```bash
cargo fmt --check
cargo check -p <touched-package> --all-targets --offline
cargo test -p <touched-package> --lib --offline
python3 scripts/check_claim_wording.py  # claims/docs only
```

Run cross-target or certification checks only when their contract is touched.
For example, a WASM-boundary change needs:

```bash
cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib
```

The substantive CI gate compiles the workspace and runs library/product-path
unit tests for Rust/build changes. Docs-only changes run claim wording only.
Temporary zero-work compatibility contexts mirror that result under the
repository ruleset's five legacy required names.
BDD, doctests, no_std, deterministic rebuild, κ, Gate C, all-features, WASM,
fuzz, Kani, conformance, and audit are nightly/manual certification—not routine
merge blockers.

Active decoder work also produces one bounded transcript or operator report
that exercises the changed behavior. Do not create a new test framework for
that smoke.

**Check the check.** Some ways these have silently lied before:

- `cargo … | grep …; echo $?` reports *grep's* exit status. Read
  `${PIPESTATUS[0]}` or run bare.
- After changing a public signature, `cargo clean -p <crate>` before the
  verifying run — stale test targets have "passed" pre-edit code four times.
- A κ test that finishes suspiciously fast has skipped. Confirm
  `/tmp/ref/out/model.bin` exists before trusting green.

## Execution-lane invariants

- **Active geometric decoder:** learned floating-point projections,
  `uor-matmul`, and allocation are allowed. Bind source, tokenizer, checkpoint,
  geometry, and decode identities; fixed inputs remain deterministic.
- **Frozen TLA/R4G1 runtime:** its multiplication-free, allocation-free,
  `no_std`, packed-format, and witness contracts remain in force. Do not weaken
  them when touching that lane.
- **Transformerless is not multiplication-free.** The active decoder earns the
  first term only with zero source-attention calls, no dense full-prefix Q·K
  matrix/softmax kernel, and bounded geometric support shown load-bearing by
  disabled/permuted interventions. Integer/table lowering is a separate
  post-viability decision.
- **Errors**: library boundaries return `Result` with focused error enums. No
  `unwrap`/`expect`/panic on recoverable paths.
- **No `unsafe`** in the portable runtime or the format crate.
- **Claim language**: `docs/formal_vocabulary.md` is normative, and
  `scripts/check_claim_wording.py` is CI-enforced. Exact-equivalence wording
  needs a linked proof artifact.

## If your change is product research

The experiment must be able to change the next decoder decision:

- **Name the active #949 child and the product decision.** Research without a
  reachable consumer is archived rather than activated.
- **Exercise the real token path first.** Geometry must run before token
  selection and emit its support/logit effect before a quality run.
- **Use the smallest falsifier.** Start with a tiny overfit, one layer, or a
  short student-prefix rollout before adding data or layers.
- **Use a non-degenerate null.** Active geometry is compared with disabled and
  shuffled/permuted geometry under equal budgets.
- **Include free-running output** whenever generation is in scope.
- **Pre-declare the exit rule, the null baseline and the falsifier** before you
  run anything. Write them in the issue.
- **Compute the reachability ceiling first.** If the ceiling is below the effect
  you are hoping for, do not launch. This is a five-minute calculation that has
  invalidated four-hour runs.
- **Run the issue-specific cheap instrument first and treat its verdict as
  binding.** For decoder work this is normally the source-control smoke,
  operator reachability probe, tiny-overfit gate, or short student-prefix
  rollout. The historical graph `capacity_scaling` instrument applies only to
  graph-capacity experiments.
- **Pre-declare what each outcome causes.** If the positive and negative branches
  lead to the same next action, the run has no decision value — drop it or
  redesign it.
- **Record negatives.** They are kept, not discarded, and several have redirected
  the whole programme.
- **Make sure your instrument can fail.** Assert the control arm is
  non-degenerate before comparing. An all-zero result across every arm is a
  harness bug until proven otherwise — seven have been found here.
- **Do not turn a negative into infrastructure by default.** A failed operator
  stops or returns to representation design; it does not automatically justify
  a larger corpus, graph format, proof lane, or benchmark suite.

For any run measured in hours, paste the run contract into the issue before
launching:

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
