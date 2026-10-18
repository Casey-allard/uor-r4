# Joining the UOR-R4 research labs

> **Current M1 handoff (18 October):** the original ordinary-path dose remains [owner-stopped before Q1](attention-dose1-strategic-stop-2026-10-18/README.md). Both arms of the separate [rows Dose2](attention-rows-q1-diagnosis-2026-10-18/README.md) stopped at the registered Q1 admission-collapse gate after 4,096 batches, with P_succ target admission 2,034 → 225 treatment /205 control. [Scientific review](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6248429872) finds configured admission collapse in both arms and no implementation or identity fault; no registered efficacy hypothesis fires. These counts are not accuracy or a treatment win. Fresh stays sealed; no KEEP or mechanism-negative verdict follows. The pod was down at 07:53:30 UTC, and #2652/#2653 are merged. A shared vector/combine pathway is plausible, not causally established. See [STATUS](../../STATUS.md) for the next-decision boundary.
>
> **Current K16 preparation:** the existing FIT A/R redirect on `rows_v1`, seeds1/2,
> retains its conditional release and all registered qualification predicates.
> The [timing adapter and packet sequence](k16-rows-timing-source-2026-10-18/README.md)
> are source-only; actual new-head qualification, checkpoint-0 A/R100-sample timing
> and accepted measured D-Packet remain pending. The [owner Q4 timing waiver6251027125](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6251027125) removes Q4/importer from the critical path.
>
> [D37](../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised) permits offline floating point, retaining matmul/GEMM and softmax/squashing bans, GPU-only model compute and separately qualified D11 export. Other training, M2 and Stage B holds remain.
>
> Work claims and reviewer availability are renewable and belong in the live harness/issue. Start with the [session contract](session-goal.md), [protocol](protocol.md), [current architecture](../architecture.md) and [discrete crate](../../crates/uor-r4-discrete/README.md); the dated packets below preserve historical scope.

> **Current owner direction, October 1 — D19:** grounded conversation and durable
> memory lead the active canonical plan. Codex rejoins Claude and OpenCode–DeepSeek
> for the owner-authorized continuation; other historical lab assignments are
> not reactivated. The owner's newer exact-head review rule (self-review allowed),
> actual scoped compile/tests and protected merges supersede conflicting older
> council/coordinator delivery prerequisites below. The delivery-evidence check
> is advisory; use the documented manual fallback when deployment is unverified.
> Scientific integrity, D11/D5, unique-material preservation, live job ownership
> and resource admission remain. See [D19](../integration/DECISIONS.md#d19--grounded-conversation-and-durable-memory-first).

Any authorized lab can join, leave and return. GitHub holds accepted decisions,
source, research records and live coordination. No provider or chat is the
permanent director. The shared machine limits concurrent execution, not the
number of labs or independent research questions.

Read these in order:

1. The root [AGENTS.md](../../AGENTS.md), [README](../../README.md) and
   [STATUS](../../STATUS.md).
2. The [active canonical plan](../integration/project-track.md),
   [integration queue and branch dispositions](integration-queue.md), and
   [shared lab protocol](protocol.md).
3. The live [tracker #2028](https://github.com/UOR-Foundation/uor-r4/issues/2028),
   the relevant milestone issue and its latest comments; then the relevant
   [current result](../integration/current-state.md) and
   [decisions](../integration/DECISIONS.md).
4. Your task's actual source, callers, data, artifact manifests and nearest
   historical positive/negative attempts. The [project map](../PROJECT_MAP.md),
   [source audit](../integration/architecture-2026-09/README.md) and
   [geometric toolbox](../integration/geometric-toolbox-2026-09-28.md) are indexes.

Start with the [extended joining goal](prompts/join.md). Available client packets
are [Codex](prompts/codex.md), [Claude](prompts/claude.md),
[Anti-Gravity](prompts/antigravity.md) and [OpenCode/DeepSeek](prompts/opencode-deepseek.md).
They are persistent missions with finite child tasks; each child still needs its
live claim, dependencies and resource admission. The
[client continuation notes](client-continuation.md) distinguish installed hooks,
manual activation, quota loss and verified live continuation. OpenCode can use
its [continuous startup packet](prompts/opencode-continuous-start.md). Additional labs use the same joining goal with their own
capability declaration and milestone claim. The [client adapter contract](adapters.md)
keeps the packets portable. [Operations](operations.md) covers the host,
recovery, storage, rollout and administrative checks.
Shared Runpod GPU pods follow one cadence and one tool, `scripts/pod/uor-pod`: see [compute](compute.md).

The September 30 goals begin with integration of existing work. A pushed branch,
finished run or queued PR is a checkpoint, not the end of a lab's responsibility.
Each lab follows its current change through review, verified protected delivery
or a recorded successor/dormant disposition, then takes the next eligible task.
The full goal files are self-contained for clients that need a pasted prompt;
publishing them does not by itself start those clients.

The shared protocol supersedes older fixed lab counts, provider-specific
leadership, one-ticket stopping rules and duplicate operational policies.
Historical records retain their measured scope. A new prompt cannot change D11,
invent a result, reinterpret a failed gate as passed, or authorize paid compute
or deletion of unique material.
