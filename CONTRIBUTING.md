# Contributing to R⁴

Follow [AGENTS.md](AGENTS.md), the [native geometric AI plan](docs/integration/project-track.md)
and the [current implementation](docs/integration/current-state.md). The stable
machine policy is [agent-execution-policy.json](docs/integration/agent-execution-policy.json).
Do not copy the current stage into another roadmap or restore old issue gates.

## Build the native model

Use Rust for preparation, training, artifact construction and inference.
Training may use floating point and matrix multiplication. The serving target
executes learned geometric operators through bounded state, routes and
integer/table lookup; a dense transformer hidden behind a lookup interface is
not the target.

Prime/ordered-n-let addresses, fixed zeta phases, R4/S3/H4 transport, exact
`Z[phi]` and orientation state, typed paired-H4/icosian geometry and UOR identity
are primary model mechanisms. Keep their architectural roles separate from
measured predictive contribution. Preserve historical Python/dense references
as evidence; add no Python model or product dependency.

Both conversation/memory and coding/reasoning must earn alpha through actual
model behavior. Implement small coherent steps in the native path. A request to
complete a whole plan authorizes its necessary successive tasks; the historical
one-task stop is not a default.

## Development and delivery

1. Refresh `origin/main` and relevant live issue state; work in an isolated full
   worktree. Preserve unrelated changes and unique research artifacts.
2. Implement the next useful native behavior. Reuse existing geometry and
   runtime parts before adding abstractions or another mechanism.
3. Compile and exercise the changed path. Use focused tests for concrete
   arithmetic, state, causal, serialization and interface risks.
4. Deliver through a protected pull request with actual commands, outcomes,
   limitations and the next action. Stage named files; do not push `main` or
   bypass branch protection. Compatibility check names alone are not QA.

Typical checks are `cargo fmt --check`, `cargo check -p <touched-package>
--all-targets --offline` and `cargo test -p <touched-package> <focused-test>
--offline`. Choose relevant checks; this is not a blanket full-suite ladder.
Broader checks run when the changed boundary or a release requires them.

## Learning and resources

Configure useful context, training and evaluation windows and CPU/thread,
wall-time, RAM, new-storage and checkpoint limits. Account cumulatively across
warmup, training, evaluation, retries and resumes. Diagnose and correct failures
within the remaining authorized budget; there is no global 15-minute cutoff or
one-retry quota. Stop/checkpoint at limits, avoid unchanged blind retries, and
obtain authorization before increasing the cumulative budget or external cost.

Open development evaluation is part of learning; final held-out evaluation
follows design selection. Keep prior results at their exact scope, and report
resource unavailability separately from model quality. Preserve proof, measured
behavior and hypothesis distinctions without creating a proof dossier or new
ledger for every edit.
