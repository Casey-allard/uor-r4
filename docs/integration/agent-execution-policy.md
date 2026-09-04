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
not add a Python model dependency.

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
stop/checkpoint at configured limits, and obtain explicit authorization for
external cost or a larger cumulative budget. Reuse measurements and checkpoints
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

PR and merge-group CI run formatting, the Rust architecture-policy check and
focused native model/context/allocation/CLI-service tests under one historical
required status name. Four other required names are explicit compatibility
acknowledgements. Broader legacy verification remains manually available for
relevant release work; neither those names nor unrun jobs certify capability.
Protected pull-request and merge-queue delivery remain in force.

Preserve source, unique artifacts and all earlier evidence. A negative retains
its exact artifact/data/operator/control/budget/decision scope. A changed
successor is allowed when its change and rationale are clear. Proof, measured
behavior and hypotheses remain distinct. Routine work does not require a new
proof package, ledger, ADR, replay dossier or duplicate status mirror.

Changing the project goal or these stable invariants requires owner direction
and protected delivery. Do not silently change them to make an experiment pass.
