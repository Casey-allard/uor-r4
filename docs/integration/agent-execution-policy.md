# Native geometric AI execution policy

The owner-directed mode is `native_geometric_ai`. The
[project plan](project-track.md) owns the goal and deliverables;
[current-state.md](current-state.md) owns the current implementation pointer.
The [machine policy](agent-execution-policy.json) captures stable invariants,
not a hardcoded stage order or copies of roadmap prose.

## Three-lab organization and shared operating policy (owner charter, 2026-09-28)

This section is the **single shared operating policy for every lab**. The labs' standing prompts restate it; they do not replace it. Where a prompt and this file disagree, this file on `main` wins until a protected PR changes it, except against a newer explicit owner instruction. Lab 1 records such an instruction here by protected PR. The owner's charter is recorded verbatim in [three-lab-charter-2026-09-28.md](three-lab-charter-2026-09-28.md).

### Organization

| Party | Owns | Does not own |
|---|---|---|
| **Owner (Casey)** | The mission and strategic authority. The hard constraints: D11 and the stable invariants in this file. Ratification of every `DECISIONS.md` entry. The organization, lane ownership and merge and review criteria in this section. Spending and destructive-action authority | — |
| **Lab 1, Claude main** (lead research lab) | The roadmap, the architecture, difficult mathematical decisions, shared interfaces and promotion. The integrated artifact contract (ROADMAP §2b). The learner, dialogue conditioning, learning objectives, persistent memory and the complete response path. The stack writer, loader and integer forward, absorbed from the retired cloud track. The QAT hook and its validation fine-tune (owner, 17:00 UTC) | Promotion of its own work without non-author review. Owner-level constraints |
| **Lab 2, OpenCode** | Geometric read/address research and its integration-ready implementation behind the stack's read interface: query formation, candidate admission, ranking, NoRead, value use and language-to-address generalization | The learner, tokenizer, memory semantics, training policy and serving contract. It proposes changes to these; Lab 1 decides, except for owner-level constraints (D11, `DECISIONS.md`), which go to the owner |
| **Lab 3, Anti-Gravity** | Numerical fidelity, codecs (including D4's training-aware discretization, through Lab 1's QAT hook), kernels, execution audits and measured local cost, coordinated with Lab 1's bundle and session | A second engine, format, frontend or deployment path |

**Not part of the three-lab organization, with no future assignments.** Their records are preserved:
- **The Claude cloud track**, stood down on 2026-09-28. Lab 1 absorbed its work and obligations; see ROADMAP §9, 2026-09-28.
- **The Codex lab (Lab 4).** Its allowance is exhausted, and its T4 study transferred to Lab 1 on 2026-09-28. It is assigned no work and returns only by owner direction.

### Standing operation

- **An ongoing goal, not one ticket.** Continue through investigation, implementation, testing, review, publication, integration and the next justified task. A blocked dependency blocks that task, not the lab: tell its owner and advance independent authorized work.
- **`main` is canonical** for accepted source, decisions and knowledge. Issues and PRs carry live activity and proposals, not automatic acceptance.
  - The latest explicit owner instruction overrides stale scheduling.
  - Verify consequential claims even when they are on `main`.
- **At startup or resumption:**
  - fetch `origin/main`;
  - read `AGENTS.md` and its reading order (README → project track → current state → model direction → project map), plus `ROADMAP.md`, `DECISIONS.md` and recent #820 and #973 activity (recent pages, not only the first);
  - inspect the task's source, artifacts, jobs and reservations. Reuse prior work.
- **Choose the highest-value unblocked task** in your lane and the roadmap. Record it in the existing issue with the progress-control work card below.
  - Routine work and bounded research inside a lane need no new owner approval.
  - Shared architecture and interface changes need Lab 1's documented decision.
- **Check the other labs' relevant changes** at startup, before decision-bearing runs, after important results or interface changes, and before merge or handoff. Never silently change a frozen experiment, or restart a valid job because `main` moved.
- **Give each relevant external or cross-lab finding one recorded disposition,** and name who acts. The dispositions are:
  - adopt;
  - run a bounded discriminating test;
  - defer, with a reason;
  - reject at a stated scope.

### Evidence integrity

- **Measurements come from actual runs**, written by the run itself into sealed report roots.
  - Readers of result files fail on a missing key; defaults never stand in for measurements.
  - Tests assert on measured values, and fail rather than skip when their fixture is missing.
- **Keep kinds of evidence separate:** proof, finite computation, synthetic fixtures, measured behavior and hypothesis.
  - A missing artifact means "unavailable" or "not run".
  - Expected unit-test constants are legitimate; invented model metrics are not.
  - Sealing establishes identity, not validity.
- **Before a decision-bearing experiment:**
  - declare its controls, splits, seeds, metrics, budget and consequences;
  - keep development separate from held-out qualification;
  - save recoverable checkpoints, and usable exports when the model will be needed;
  - preserve negatives;
  - retry only with a specific causal change.

### Gates promote; they never kill (owner, D12)

- A pre-registered gate decides only **whether a mechanism enters the served or main-line model now**. A miss keeps the mechanism active, with its next diagnosed step recorded.
- **Parking a mechanism family** needs a written root-cause case and the owner's OK. A near miss on one configuration of a mechanism still under construction is never grounds to retire the family.
- D9 still applies: every retry needs a real causal change, not an extra seed or dose.
- Earlier FAIL, DEAD and RETIRED labels on mechanisms read "not yet promoted at that scope". Mechanisms that demonstrated a real geometric capability stay available in the [geometric toolbox](geometric-toolbox-2026-09-28.md).
- **Recommend by the project's overall goal:** a learned, geometry-first language model on local hardware. Do not recommend by the narrowest gate.

### Standing merge and review criteria

Every PR names its lab and its class. Once its class's conditions hold, the approved merge path needs no further owner approval:
1. run `gh pr merge` into the queue once `mergeStateStatus` is CLEAN;
2. check that the squash commit's diff against its parent equals the PR's diff. The merge queue can merge several entries together, so `main`'s tree may legitimately differ from the branch's;
3. notify the consumer.

| Class | Covers | Merge when |
|---|---|---|
| **A. Lane work** | Code, tests, tooling and documents inside the lab's owned paths (listed in the lab's ROADMAP §4 subsection; otherwise the files its PR creates), with no result claim and no shared-interface change | The required checks pass, the PR lists the focused local checks run (commands and outcomes), and one non-author technical review approves |
| **B. Results** | Any measured number, fidelity or capability statement, or artifact promotion | Class A, plus sealed roots on disk and the headline numbers re-read from those roots by Lab 1. Lab 1's own results are re-read by another lab or an independent pass. Artifact promotion also needs Lab 1's documented decision. **Lab 3 results also need Lab 1 to re-run the gating measurement** from the committed code into a fresh root (owner, 2026-09-28 17:11 UTC). A failed gate merges as a negative; promotion is a separate step |
| **C. Shared interfaces and architecture** | The artifact contract (ROADMAP §2b), checkpoint, bundle and session formats, the read interface, cross-lab trainer and export hooks, `DECISIONS.md`, this policy and the serving contract | Class A (and B if it carries results), plus Lab 1's documented decision in ROADMAP §2b or §9. A change to `DECISIONS.md`, D11, this policy's stable invariants, or this section's organization and merge criteria also needs **the owner's recorded ratification** |

- **A non-author review** is a written review, linked from the PR, by another lab or by an independent reviewer pass that did not write the change. The review names who launched it. Lab 1's own class B and C changes get one too.
  - All labs share one GitHub account and the ruleset enforces no reviews, so this rule is procedural. Keep it anyway.
- **Never** push directly to `main`, bypass protection, use an admin merge or force-push shared work. A blocked merge names its cause and owner.
- **A review that lists required fixes is not an approval** until those fixes land.
- **Every PR is driven to merge** (owner, 2026-09-28: "always make sure PR's are being merged, including your own"). Lab 1 drives ready PRs through protected delivery, including its own, without waiting for owner prompts.
  - **No unconditional auto-merge** (owner, 2026-09-29: "No unconditional auto-merge, direct main push, admin bypass or force-push of shared work. Recheck the exact approved head and blockers before merging; verify the delivered patch and notify the consumer.").
  - The non-author review and the class conditions come first.
  - Immediately before merging, recheck that the head is the exact approved head and that no blocker is open. A changed head needs a fresh review.
  - After merging, check that the delivered squash patch equals the reviewed diff, then notify the consumer.
  - A finding after the merge becomes a follow-up PR.
  - **Exception:** a Lab 3 result (class B) merges only after Lab 1's gating re-run.

  No PR sits open without a named cause and owner.
- **Safe, inactive research and negative evidence** may merge without model promotion.

### Knowledge and resources

- **Keep knowledge linked,** recording scope, confidence, dependencies and consumer at each step:
  1. mission;
  2. blocker;
  3. mechanism or decision;
  4. source or artifact;
  5. experiment;
  6. adversarial finding;
  7. integration consequence;
  8. next action.

  Keep summaries compact and evidence immutable. Preserve superseded history.
- **Large payloads** live in approved storage (the owner SSD), with hashes and verified retrieval instructions.
- **Follow the shared machine protocol** (ROADMAP §6: one heavy job at a time through the model slot; light jobs at ≤ 2 threads) and charge the cumulative ledger.
  - No unapproved spending, provider changes or destructive cleanup.
  - Never force-interrupt another lab's job.
- **At a host, context or quota boundary,** publish a restartable handoff: the commit, artifact paths, live job ownership and the exact next action. Do not claim that execution continues without an authorized runner.

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
