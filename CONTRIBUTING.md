# Contributing to R⁴

## Build-first pre-alpha policy

The project is in **build-first pre-alpha** mode. The immediate goal is to load
a source-free model artifact and produce prompt-dependent text through the
repository CLI or local service on target local hardware. Read the binding
policy in [AGENTS.md](AGENTS.md) and the small machine contract in
[agent-execution-policy.json](docs/integration/agent-execution-policy.json).

## Development loop

1. Pick one implementation task that moves the working product toward the
   pre-alpha goal.
2. Start an isolated worktree from refreshed `origin/main`.
3. Implement the behavior in the actual product path.
4. Compile and run the smallest command that directly exercises it. Add a
   focused test only for a concrete regression risk.
5. Open a pull request that states the observed behavior, remaining limitation,
   and next implementation action. Merge through the protected queue.

Agents may build, test, train, fit, evaluate, and run repository CLIs, models,
services, and browser flows. Formal proof work, claim and evidence ledgers,
knowledge-index maintenance, experiment freezes, replay packages, independent
review, publication, external-theory mapping, duplicate status documents, and
broad release QA are deferred until the pre-alpha goal is working. Only an
explicit owner instruction can activate one of those activities earlier.

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
research records in `docs/` remain historical evidence, but routine pre-alpha
development does not update or reproduce them.
