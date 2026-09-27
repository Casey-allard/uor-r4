# Fourth research lab — UOR-R4 Geometric Language Model

Established by owner direction on September 26, 2026. The fourth lab is the
Codex research and integration team alongside Google, OpenCode/DeepSeek/Kimi,
and Claude. The shared objective is a useful native geometric language model
with efficient geometric attention, integer inference, prose, chat and reasoning.
The owner authorizes autonomous technical decisions and successive necessary
work within that objective. The owner can redirect the programme at any time.

**Owner clarification, September 27:** “Keep the native, multiplier-free serving
target.” This lab retains the native geometric model and D0-b additive serving
contract. Offline Rust training may use floating point and matrix multiplication.
D10's converted-transformer backbone and hardware serving-multiplier exceptions
are not adopted by this fourth lab. Preserve that other session's historical
record and source as research/comparators; this clarification does not claim to
rewrite instructions in a separate lab. Existing prototype exceptions and costs
remain explicit, and no prototype is promoted by this policy statement.
[Shared owner clarification](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5852795279).

## Start and recover context

Read the root [AGENTS.md](../AGENTS.md), [README](../README.md),
[canonical roadmap](../docs/integration/project-track.md),
[current state](../docs/integration/current-state.md),
[direction](../docs/integration/model-direction-2026-09.md), and
[project map](../docs/PROJECT_MAP.md). Refresh `origin/main` and live issues
[#820](https://github.com/UOR-Foundation/uor-r4/issues/820) and
[#973](https://github.com/UOR-Foundation/uor-r4/issues/973). The current owner
request supersedes old one-milestone stopping instructions; it does not turn
historical negative results into positives or authorize paid compute/deletion.

Run `bash scripts/research-lab-context.sh --github` in the chosen isolated
worktree for a compact, read-only context receipt. It records live revisions,
dirty paths, authority hashes, worktrees, physical space and recent issue/PR
state. It neither launches a model nor certifies complete understanding.
Read the actual documents and source linked by the receipt.

Use the [architecture audit](../docs/integration/architecture-2026-09/README.md),
[research archive](../research/README.md), and `uor_knowledge` for navigation.
The knowledge service is a historical snapshot, including old current-state
copies. Verify every decision-relevant result at its source revision. Follow a
mechanism recursively through caller, representation, training objective,
serialization, serving and actual output, then its historical experiments and
upstream source. Stop traversal when remaining unknowns cannot affect the next
decision; record those unknowns. No expert is assumed to know the entire repo.

## Expert bench and execution

These are research responsibilities, not a claim that twelve independent
humans or twelve models are continuously running. The host currently supports
the principal plus three concurrent Codex specialists. Rotate specialists in
dependency waves; give each a complete packet and meaningful design discretion.
Use the configured DeepSeek route for external specialist work when available
within existing authorization. Never relabel a Codex response as DeepSeek, and
do not start paid providers or change another lab's configuration. Kimi routing
inside the other lab remains that lab's responsibility.

| Responsibility | Questions and concrete deliverables |
|---|---|
| Principal investigator / synthesis | Maintain the programme objective, compare alternatives, select the next integrated deliverable and resolve disagreements with evidence. |
| Geometry, algebra and topology | Derive R4/S3/H4 and paired-icosian operations, orientation/frame invariants and exact `Z[phi]` bounds; distinguish representation from a useful learned operator. |
| Attention, information and memory | Study query-dependent geometric compatibility, transported values, exact occurrence/version access and information discarded by a representation. |
| Learning and optimization | Inspect language-to-context gradients, recurrence, state, objective/data mixture, optimization continuity and hard/relaxed retention. |
| Numerical methods and compilers | Establish quantization/error budgets, integer operators, serialization and emitted-instruction scope. |
| Rust and systems architecture | Build shared interfaces, typed errors, reusable operators, safe sessions and training/serving protocol alignment. |
| Data and linguistics | Review provenance, licensing, tokenization, discourse/entity consistency, turn supervision and train/dev/heldout separation. |
| Reasoning and programming languages | Develop compositional state/operator needs, actual generated-program execution and transfer beyond authored examples. |
| Statistics and experimental design | Choose informative comparisons, matched ordinary controls, uncertainty, seeds and complete denominators. |
| Hardware, performance and energy | Measure complete M1 work, access sparsity, latency/RAM/physical energy at comparable output quality; coordinate shared resources. |
| History and primary literature | Recover prior positives/negatives and source revisions; inspect companion repositories before proposing reuse. |
| Adversarial review and delivery | Seek counterexamples, shortcuts and integration failures; inspect actual artifacts/output, source claims and protected delivery. |

Every consequential design receives both a constructive proposal and an
independent challenge. Independence means a distinct evidence review, not just
a second model agreeing. Preserve disagreements and their consequences. The
principal reads the decisive source/output and owns the final technical choice.

## Shared lab coordination

The owner selected **shared GitHub issues and isolated worktrees**. Use #820 for
programme ownership and #973 for the current model integration work. Other
existing issues retain their responsibilities; do not create a second roadmap.
Before editing, post a short claim containing lab, branch/base, owned paths,
concrete deliverable, dependencies, resource needs and next decision. A claim
records this lab's intent; it does not command or lock another lab.

Inspect live worktrees, dirty files, PRs and running model/build jobs before
choosing work. Treat unconfirmed provider ownership as unknown. Coordinate an
adapter boundary before modifying another lab's active files. Keep competing
hypotheses in separate branches/artifacts. Integrate useful parts after reviewing
their training/serving contract and actual evidence. Source inspection can
justify a bug report; it cannot establish unexecuted model quality.

Post concise findings and requests on the owning issue/PR. Share source-linked
evidence and preserve each lab's independent research freedom. Never kill
another lab's job, reset its checkout, discard its results or impose this lab's
protocol on its private sessions. Record unresolved integration dependencies and
advance work that does not depend on them.

## Discovery and engineering

Keep exploration broad and promotion precise. Open development permits new
objectives, data mixtures, geometric operators and alternative architectures.
An experiment starts with a scientific question and a decision it can change;
it need not satisfy a tiny authored fixture before useful research proceeds.
Retain novel mechanisms and unexpected negatives at their measured scope.

Use a short work card: integrated deliverable; observed blocker; causal change;
fixed conditions; distinct success/failure/inconclusive decisions; necessary
checks; complete cost. Do not require a new report framework or a separate
approval for ordinary technical choices. Use existing records where possible.

Tests protect actual contracts and causal measurements. Choose focused checks
for changed arithmetic, state, format or interface risks. A test that only
asserts today's implementation is not evidence that it matches training or
language intent. Read actual generated text and execute generated programs when
those capabilities are claimed. Final heldout evaluation follows design
selection; exploratory results remain exploratory.

For a bug, trace the complete affected path and ask which model conclusion or
user behavior it invalidates. Fix it when it blocks that deliverable or a real
contract. Park unrelated cleanup with a source pointer. Once declared risks are
resolved, stop testing and deliver. Planned controls, independent seeds and
replications can resolve a declared uncertainty within the same study; they do
not require changing its mechanism. Restarting a failed development cycle
requires new causal evidence, a concrete correction and distinct decisions,
as specified by D9. A negative result can redirect the roadmap without
disproving all geometry.

Retain exact identities, source/artifact bindings, old parents and failed
candidates. Preserve full256 as the retained comparison baseline; an alternative
scan/local-memory architecture must explicitly name its changed context and
state contract. Do not present it as an equivalent optimization by default.

## Resource and delivery discipline

Before build/model work, read the shared ledger and current physical storage,
then project total preparation/build/learning/evaluation/review/delivery cost,
threads, RAM and new retained/temporary bytes. One cargo process at a time;
coordinate accelerator/model work across labs. Model/training/accelerator time
is reported separately from orchestration. Local extensions are already owner
authorized when recorded prospectively; paid/external compute and destructive
actions remain outside this authorization. Retain the 128 MiB stop margin and
any stronger active physical reserve. A ledger field changed by an unknown
writer is not an unexplained allowance for this lab.

Use a full isolated `codex/` branch/worktree; managed Codex worktrees are suitable.
Stage named paths, execute relevant checks and create protected PRs. No direct
main push or bypass. Merge requests follow the owner's repository permission;
a PR being open/queued is not a delivered merge. Queue acknowledgment statuses
are not executed tests. Update claims at their actual source, and report
source/tree identity, output, limitations, costs and the next decision.

The active Codex goal drives successive work. This file does not create a
background daemon, scheduled worker or an always-running expert team. At a
handoff, record completed work, current ownership, running jobs, resource cursor
and the concrete next action. On resume, refresh these instead of replaying a
stale plan. Set up scheduled wakeups only when needed and avoid duplicating an
active goal or another lab's monitor.

## Specialist task packet

```text
Role and decision to resolve:
Programme objective and owner priority:
Worktree, branch/base SHA, current issue/PR, dirty/other-lab ownership:
Authority documents and exact source/caller/evidence/parent paths:
Prior result, hypothesis, strongest alternatives and predicted differences:
Training/evaluation/session context, access, dimensions and protocol identity:
Owned files and allowed mutations; integration boundary:
Data/artifact/control identity; development versus final scope:
Complete resource projection and existing active jobs:
Return: conclusion, contrary evidence, exact source/results, uncertainty,
decision changed, preserved artifacts and the next implementable step.
```

First source reviews: [geometric attention](../docs/integration/fourth-lab-geometric-attention-2026-09-26.md),
[cross-lab integration](../docs/integration/fourth-lab-integration-review-2026-09-26.md),
and [shared dialogue protocol](../docs/integration/dialogue-protocol-contract-2026-09-26.md).
The [establishment receipt](../docs/evidence/fourth-lab-establishment-2026-09-26.json)
binds the first executed protocol checks, actual tokenizer witness and resource
accounting. It is a dated receipt; refresh live state before successive work.
