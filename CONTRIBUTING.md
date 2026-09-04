# Contributing to R⁴

## Build-first architectural-alpha policy

The project is in **build-first architectural-alpha** mode. Follow the exact
[project track](docs/integration/project-track.md): fixed recurrent memory,
sparse geometric attention, nonlinear geometric block, scale/data/instruction,
retrieval/tools, product alpha, Rust/table lowering, then release proof and QA.
Read the binding policy in [AGENTS.md](AGENTS.md) and the small machine contract
in [agent-execution-policy.json](docs/integration/agent-execution-policy.json).

## Development loop

1. Pick one implementation task in the current project-track stage.
2. Start an isolated worktree from refreshed `origin/main`.
3. Implement the behavior in the actual product path.
4. Compile and run the smallest command that directly exercises it. Add a
   focused test only for a concrete regression risk.
5. Open a pull request that states the observed behavior, remaining limitation,
   and next implementation action. Merge through the protected queue.

Agents may build, test, train, fit, evaluate, run repository CLIs, models,
services, and browser flows, query the index, inspect original research, and
request proportionate review. SpiralCore, HELM, W33, NEMESIS, UOR, and H4/zeta
work are on-demand donor reservoirs for a concrete design seam. Broad formal
proof, ledger reconciliation, publication, programme-wide source mapping, and
release QA wait for the release candidate unless the current decision or owner
needs them.

Bounded open-data development may iterate. Keep final held-out evaluation after
design selection. A negative remains true for its exact configuration; a
materially versioned successor may re-enter with a named change and rationale.
`UNAVAILABLE` is not model evidence.

Automatic retries are zero. After a concrete failure, read the existing output,
make one direct source or input correction, and rerun once when that correction
plausibly addresses the failure. Do not create supervisors, watchdogs, receipt
harnesses, workspace capsules, or environment-probe campaigns.

A command expected to exceed 15 minutes, create more than 10 GiB, or incur an
external cost needs explicit owner or active-issue authorization, a hard limit,
and a stop condition.

## Checks

Use the smallest check that proves the changed path is usable. Typical examples
are:

```bash
cargo fmt --check
cargo check -p <touched-package> --all-targets --offline
cargo test -p <touched-package> <specific-test> --offline
```

These examples are not a mandatory ladder. Run full-workspace, cross-target,
certification, performance, fuzz, Kani, replay, or model-scale checks only when
the changed feature or an explicit release decision needs them.

## Technical boundaries

Keep the invariants of the code you touch. In particular, do not add floating
point, multiplication, allocation, or unsafe code to a runtime that explicitly
forbids it; keep deterministic artifact production deterministic; return focused
errors at library boundaries; and do not claim behavior that was not run.

Preserve unique artifacts, user material, and prior negative results. Old
research records in `docs/` remain historical evidence and do not create a
routine proof or bookkeeping gate for implementation.
