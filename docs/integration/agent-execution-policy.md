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
that target. **Owner direction (2026-09-23):** the programme's ultimate goal is a **fully
transformerless geometric language model in which geometry *replaces* floating-point matrix
multiplication**; the serving contract below is the intended end-state, not only a boundary on the
current prototype. Keep prior Python/dense artifacts as comparison evidence and do
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

[D8](DECISIONS.md#d8--correct-the-training-method-and-reference-ladder) makes a
working reference and an explicit differentiable training-to-serving bridge the
active implementation sequence. Use the reusable offline Rust autodiff tool;
verify the actual next-token gradient path, hard/relaxed discrepancy and serving
mask. Preserve native discrete scaffolds and old reference artifacts, but do not
resume local selector adjustments as the default programme. Complete the declared
rung through its decision with fixed evaluator identities and matched controls.
An integrity check or dependency does not establish language learning. Record
actual fitting, evaluation and local accelerator time separately from compilation
and orchestration, while retaining the cumulative ledger below.

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

## Progress control — owner correction, September 25

The principal investigator owns progress toward a useful geometric model. Passing
another test or completing another experiment is not itself the next objective.
Apply these rules before committing model compute; use the existing active issue
or current-state entry, without creating another tracking framework.

1. **Name the deliverable and the decision.** Keep one short work card: the
   missing model ability or implementation, the observed blocker, the proposed
   causal change, and what success, failure or an inconclusive result changes.
   A diagnostic is justified only when its answer changes implementation or
   adoption. If every outcome leads to the same next task, do that task directly.
2. **Preserve the active contract.** Read the accepted artifact and the current
   context/access contract from `current-state.md`. Before launch, compare the
   actual loaded training, evaluation and generation settings, including context
   length, direct-history access, admission budget, representation dimensions,
   data window and artifact lineage. These are separate quantities. Do not call
   matched sequence lengths preservation of memory access. Disclose a proposed
   reduction before execution; an owner-fixed requirement needs owner direction
   to change. Never silently reset learning or promote a diagnostic override.
3. **Spend on a discriminating change.** Link the previous result. A repeated
   fit or evaluation needs new causal evidence, the concrete changed mechanism,
   a predicted observable difference and a decision that the old evidence cannot
   settle. A nearby width, threshold, hash, seed or smaller horizon is not its own
   justification. Change one unresolved cause at a time; if interacting changes
   are necessary, declare the bundle and limit attribution accordingly. Reuse
   valid binaries, checkpoints and measurements. A corrected execution failure
   may resume its existing campaign; preserve and charge every attempt.
4. **Keep validation proportional and finite.** Name the checks that protect the
   changed arithmetic, causality, serialization or interface and the actual
   loaded behavior needed for this deliverable. Reuse valid prior results. A new
   failure is blocking only if it invalidates that behavior, the measurement or
   an applicable contract. Record unrelated failures for their owning work. Do
   not add a test, corpus, review, sweep or proof package merely because another
   possible uncertainty exists. Do not weaken frozen acceptance after seeing
   failure. Meeting the declared checks ends validation of that scope.
5. **Stop branches without a new cause.** When a sound experiment fails, retain
   its result and the last accepted model. Diagnose from existing evidence first.
   Reopen the branch only with the evidence required in rule 3. Otherwise park it
   and advance the highest-value independent prerequisite of the integrated
   model. An inconclusive measurement permits its named instrument repair, not
   an uncontrolled architecture search. This is a causal stop rule, not a fixed
   retry quota or an excuse to abandon a necessary implementation.
6. **Account for the whole cost and close the decision.** Project preparation,
   build, model work, agents, analysis and delivery against the existing budget.
   Reassess at the already scheduled checkpoint or first decisive result; do not
   add a checkpoint campaign. Report what now works, the retained artifact,
   limitations, costs and next implementation. Delegate only independent work
   with a concrete output that avoids duplicate investigations. More agents and
   more elapsed time are not evidence of progress.

These are mandatory agent execution rules mirrored in the machine policy and
loaded through repository `AGENTS.md` and the research skill. They are not an OS
sandbox or a technical interlock on someone invoking the Rust CLI directly.
Their concrete application to the September 25 correction is in
[D9](DECISIONS.md#d9--prevent-experiment-loops-and-preserve-the-context-contract).

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
