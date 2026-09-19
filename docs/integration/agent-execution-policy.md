# Native geometric AI execution policy

The owner-directed mode is `native_geometric_ai`. The
[project plan](project-track.md) owns the goal and deliverables;
[current-state.md](current-state.md) owns the current implementation pointer.
The [machine policy](agent-execution-policy.json) captures stable invariants,
not a hardcoded stage order or copies of roadmap prose.

## Architecture and scope

Prepare data, train, construct artifacts and run inference in Rust. Training may
use floating point and matrix multiplication. Final inference executes learned
geometric operators through bounded routes, state transitions and integer/table
lookup; a dense transformer stored behind a lookup interface does not satisfy
that target. Keep prior Python/dense artifacts as comparison evidence and do
not add a Python model dependency. Owner-adopted [D0-b](DECISIONS.md) permits
bounded <=4-bit integer/ternary linear maps implemented by add/subtract/shift/lookup.
The declared numerical kernel must execute no multiplier instruction. Served
computation must use no floating-point or transcendental arithmetic. This allowance does not make a linear map cease to
be a mathematical contraction, or establish an efficiency advantage. Geometric
routing remains the preferred direction; deterministic address/page selection is permitted. The current design prioritizes shared typed operators;
the owner retains expert gates as a conditional future option if a concrete
capability need and complete laptop-cost comparison justify adoption. See the
[owner-requested architecture review](architecture-2026-09/README.md).

Primes and ordered prime context, fixed zeta-zero phases, R4/S3/H4 transport,
exact `Z[phi]` and orientation state, the typed paired-H4/icosian bridge and UOR
identity are primary architecture. Their architectural roles and measured
predictive contribution are separate facts. External research is optional
support for a concrete design question.

Both conversation/memory and coding/reasoning are alpha goals. Continue within
the owner's authorized objective, including necessary successive tasks when the
whole plan is requested. An old one-task stop or historical blocker does not
shrink that authorization. Use an isolated full worktree, coordinate file
ownership and deliver through protected pull requests.

## Learning and budget

Use configurable context, training and evaluation windows. Declare the run's
CPU/thread, wall-time, RAM, new-storage, checkpoint and evaluation settings and
charge their cumulative use across preparation, training, evaluation, retries
and resumes. Select a meaningful window that fits the machine instead of
changing the model to satisfy an arbitrary short historical experiment.

A failed command permits diagnosis, correction and another run or resume within
the remaining authorized budget when that can change the outcome. There is no
global 15-minute cutoff or one-retry quota. Avoid unchanged blind retries,
stop/checkpoint at configured limits, and apply the standing local-extension
authorization below before a necessary cumulative increase. External cost
requires separate explicit authorization. Reuse measurements and checkpoints
when their inputs remain valid.

Open development evaluation is part of learning. Keep final held-out evaluation
separate until design selection. Resource unavailability is an execution result,
not evidence against the model's ability to learn.

## Verification and records

Compile and exercise the changed Rust path. Focus tests on real arithmetic,
state/causal, serialization and interface risks; run relevant broader checks
when the change or release needs them. Neither a blanket full-suite ritual nor
an echo-only queue status substitutes for the behavior check. Report actual
commands, outcomes and remaining limitations.

Following the owner-directed CI change in PR #1163, PR and merge-group CI
provide five explicit compatibility acknowledgements for the historical required
status names. They execute no formatting, Clippy or model tests. Actual focused
Rust/model validation is local; the native suite and broader legacy verification
remain available through manual workflow dispatch. Neither status names nor
unrun jobs certify capability.
Protected pull-request and merge-queue delivery remain in force.

Preserve source, unique artifacts and all earlier evidence. A negative retains
its exact artifact/data/operator/control/budget/decision scope. A changed
successor is allowed when its change and rationale are clear. Proof, measured
behavior and hypotheses remain distinct. Routine work does not require a new
proof package, ledger, ADR, replay dossier or duplicate status mirror.

Changing the project goal or these stable invariants requires owner direction
and protected delivery. Do not silently change them to make an experiment pass.

## Discovery and presentation

Use the product name **UOR-R4 Geometric Language Model**. The canonical plan owns ordered issue responsibilities, current-state owns changing evidence, and `docs/PROJECT_MAP.md` locates code/research/artifacts. Reuse `docs/integration/model-direction-2026-09.md` for the latest source-derived recommendation; it adds no model execution. Preserve historical/imported READMEs and their names; the complete README inventory records their disposition. Do not reinterpret finite contextual attention as general prose/reasoning or a transformer architecture.


**Standing owner authorization (2026-09-06):** necessary local model/time/storage allowance extensions are already authorized. Record the complete projection, reason, increment and updated cumulative limit before using each extension; retain cumulative charges and the 128 MiB storage stop margin. Do not ask the owner to approve the same class of necessary increase again. This authorizes neither destructive deletion nor paid/external compute, and does not require spending unused allowance.
