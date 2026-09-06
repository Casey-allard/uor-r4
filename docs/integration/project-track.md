# Native geometric AI project plan

This is the canonical project goal and development plan, restored by owner
instruction on 2026-09-04. The [current state](current-state.md) names the actual
implementation and remaining work; live GitHub owns issue status. Other
roadmaps link here instead of copying a changing stage list. Earlier sequencing
and fixed experiment windows are historical, not instructions for new work.

## Goal

**Owner clarification after #1145:** the intended model is a learned
**geo-transformer LLM**: geometric routing replaces the expensive dense
implementation of contextual selection and transformation. Preserve language
learning, context combination and compositional processing while learning
where information belongs, which sources to access and which operators to
execute. Frontier-model capability on normal laptops is the ambition; current
capability and complete serving cost must be demonstrated. A small recall
system, a router around an external LLM, or a dense transformer re-encoded into
lookup tables does not complete this goal.

Build a useful local geometric AI in **Rust throughout preparation, training,
artifact construction, and inference**. Prime addresses, ordered prime context,
the fixed zeta-zero spectral grid, and R4/S3/H4 geometry are primary model
mechanisms. Typed paired-H4/icosian geometry, exact `Z[phi]` state, and UOR
identity remain part of the architecture, with their roles and implemented
boundaries explicit.

The model must learn and use these mechanisms to support both conversation with
memory and coding/reasoning. Their architectural priority is an owner decision;
their predictive contribution is an empirical question. An unsuccessful
operator or experiment does not demote the architecture to optional research.
A geometric trace or correct mathematical identity does not establish useful AI.

Rust training may use floating point, matrix multiplication, gradients, and
CPU libraries. The final inference path uses learned geometric operators,
bounded routing, state transitions, and integer/table lookup. Training may
learn coefficients, operator choices, and read/write behavior; inference must
execute the resulting geometric model. Storing dense transformer weights in a
table and continuing its dense attention/MLP computation is not this target.
Existing Python/dense references remain preserved comparison evidence, not the
primary implementation or a product dependency.

## Required mechanism roles

| Mechanism | Role in the model | Evidence needed when that role changes |
|---|---|---|
| Prime registry, semiprimes, ordered n-lets | Reversible lexical identity, factor overlap, ordered transition/context address | Roundtrip and order preservation; distinguish identity assignment from learned predictive relations |
| Fixed zeta zeros and phase channels | Artifact-bound spectral coordinates and local relative phase updates used by a learned geometric operator | Report the consumed channels; compare with the same operator with phase influence disabled or changed |
| R4/S3/H4, Hopf/fiber/torsion | Typed causal state, transport, bounded geometric memory and selection | State/update correctness and a useful behavioral comparison after learning |
| Exact `Z[phi]`, chirality and cosine polarity | Preserve radial and orientation information that scalar collapse would erase | Exact representation/reconstruction checks at the touched boundary |
| Paired H4/icosian bridge | Explicitly typed golden-coupled structural/storage or operator state | Declare the actual forward/inverse map; do not identify R4, H4, and E8 as the same space |
| Learned geometric operators | Learn how admitted state influences memory, selection and output | Improvement on open development tasks and measured resource use; preserve negative results |
| UOR identity and artifact format | Canonical serialization, integrity, versioning and reproducible loading | Existing typed identity APIs and artifact reload; digest bytes are not a semantic score |

The detailed architectural vocabulary remains in
[the geometric programme](../geometric_intelligence_programme.md#architecture-invariants).
Every model artifact declares which roles it actually implements. Missing roles
stay visible as missing work; an unrelated dense comparator cannot fill them.
An existing table/metric may be reused only when it computes the intended
operator at the declared fidelity and cost.

In particular, rewriting one H4 root into the eight integral coefficients of
its four `Z[phi]` coordinates and a golden/Galois companion is an invertible
representation, not two independently variable H4 factors or an orthogonal
Euclidean E8 isometry. A
unit-root norm is constant; a variable radial carrier must come from actual
state accumulation or a declared scale operator. Learned readout coefficients
are useful operator work, but do not by themselves learn memory writes or make
every primary geometric role effective. Keep these distinctions in model claims.

## Immediate build sequence

**Owner-adopted direction, clarified after #1145.** Retain the first two
completed bounded steps and focus the third on learned geometric routing and
selected transformations, with language and composition in its direct check.
Further NoWrite optimization is secondary. #973
remains the model parent and #820 the programme tracker. Live GitHub owns
readiness/blockers; [current-state.md](current-state.md) records the accepted
artifact and the earliest unmet step. The
[research direction review](../native_geometric_direction_review_973.md)
explains the evidence and alternatives. This is an implementation sequence,
not a claim that these mechanisms already work.

| Order | Native build issue | Required handoff |
|---|---|---|
| 1 | [#1137: role-aware source selection and causal response commitment](https://github.com/UOR-Foundation/uor-r4/issues/1137) | Learned source/NoRead choice transfers across wording and roles, stays attached to exact occurrence evidence, and drives complete generated answers |
| 2 | [#1138: exact relations with learned writes and updates](https://github.com/UOR-Foundation/uor-r4/issues/1138) | Useful facts and revisions survive raw-window eviction with exact recoverable values, bounded state and isolated causal persistence |
| 3 | [#1139: learned geo-transformer routing and selected computation](https://github.com/UOR-Foundation/uor-r4/issues/1139) | A jointly trained geometric block connects semantic placement, source admission and selected transformation to useful language/composition transfer, with its complete capability/work tradeoff compared to a matched sparse alternative |
| 4 | [#1140: typed operator composition for conversation and Rust reasoning](https://github.com/UOR-Foundation/uor-r4/issues/1140) | Learned operand/operator selection produces causally used intermediate values, correct grounded outputs and semantically verified generated Rust |

Native dependencies remain #1137 -> #1138 -> #1139 -> #1140, with #1137 and
#1138 complete at their recorded bounded scope. #1139 is immediate. Short
composition examples exercise its learned block during development; do not
postpone their learning signal until a separate routing speedup has passed.
#1140 owns broader multi-operation qualification after a useful block exists.
A predecessor must supply its accepted artifact and behavioral handoff;
closing an experiment with a negative
or unavailable result does not satisfy a capability prerequisite. Record the
specific redesign and update the dependency if a handoff fails. Do not silently
skip it, import a dense serving model, or treat issue closure as model quality.
Necessary fixes to an existing interface, invariant or resource bottleneck may
accompany a step; they do not start a competing product/research programme.

**Current handoff:** #1137 was delivered through PR #1142; exact relation
storage through #1143 and the accepted #1138 role-path repair through #1144
(`39e35c54`). [#1139 exact NoWrite admission](../native_geometric_relation_admission_1139.md)
now preserves 112 prior and 28 longer-context answers/write sequences and earlier
conversation/coding/session behavior. It removes most repeated writer scoring.
Retain the sparse artifact `067adbf0`; the matched geometric partition has no
established speed advantage. PR #1145 merged at `219f572f`. #1139's full handoff
remains unmet. Its immediate successor is now the learned geometric block below;
the previously proposed residual NoWrite score bound is secondary, unimplemented
and unqualified. Keep all earlier negatives and scope limits.

### Step 1: role-aware source choice shared with generation

The starting point was /4 memory, typed operators, lexical entry, retained-word
copy and completion. The preceding correction fixed unbound prefix scores, but
positional query/source features and pooled source votes do not provide a
general role representation. Required answer values are retained in the
observed failures; widening the window first does not address that cause.

Learn a candidate-specific ReadOccurrence(i) versus NoRead decision using
position-shared relative local role/context features, exact lexical equality,
ordered prime context and existing signed H4/zeta relations. Let entry and
copying consume that same occurrence/version. An observed transition commits
it; mismatching observation or invalidation clears it. Keep the initial
sixteen-word bound and the existing numeric and completion behavior.

Reuse #1073/#1077's association-preserving representations, role supervision
and owner/value contrasts. Their original dense soft readers and supplied
clauses are reference evidence, not a qualified hard/table implementation.
Use sparse additive scoring over ordered local n-lets and role/context features,
trained in Rust and exported to quantized tables. The bounded #1137 handoff
learns weights over these features; a separate semantic role codebook remains
unimplemented and is not required by that handoff. Construction labels may
supervise learning; serving receives raw text, not hand-parsed semantic answers
or a city-specific template.

Acceptance requires preservation of the existing 62 responses and eight binding
outputs, repaired open wording cases without lost abstention, causal
commit/snapshot checks, and actual generated-answer transfer on a small
predeclared first-use set. Vary wording, names/value assignments, source order,
role reversal, updates and absent entities. Check split/retention assertions
before opening evaluation. Set the exact acceptance rule before seeing those
results; report supported and unsupported outcomes separately. Preserve older
OPEN/first-use scope instead of relabeling it held out. Geometry sensitivity
and matched geometric advantage are separate results.

### Step 2: exact relational memory beyond the raw window

Learn which observations form associations and which later observations revise
them. A record retains selected role participants, exact value/span references,
source occurrence/version, scope, update order and typed geometric state.
Reusing the numeric commit law does not by itself establish semantic memory.

Demonstrate learning two facts, retaining the unrelated one, updating the other,
and answering both after their source leaves the immediate window. Cover
contradiction and unsupported outputs plus snapshot/restore and session
isolation. Keep exact old sources recoverable while an active relation points
to its revised value. Report storage, scans, writes and eviction explicitly.

Use recent exact occurrences, persistent relation records and coarse summaries
whose entries point to exact members. A transported average is not the only
surviving copy of an exact fact. Learn retention priorities when actual eviction
failures justify them; no ever-growing transcript scan disguised as bounded
state. Broader conversation, export/forget and product integration remain #962.

### Step 3: learn semantic placement and routed computation

Implement one small jointly trained angular routing block inside the existing
native language path. Learn token/ordered-n-let codes and contextual placement
within fixed geometric structure, source admission and a selected state
transformation. Prime/UOR identities remain exact; the current fixed
`prime % 120` lexical H4 placement is not learned semantic positioning. Use
typed independent state channels where needed, preserving orientation and
the distinction between hyperbolic H^4, S3 and the finite H4 root group.

The first integration connects contextual codes -> bounded angular/prime
shortlist -> selected payload -> learned integer/table transformation -> native
token prediction. Reuse exact occurrence/relational memory, causal commitment,
copying and typed values. Select before gathering expensive payloads or
executing operators. Query construction and output selection must also avoid
dense projections. A large table enumerating whole contexts is not the proposed
learner. Factorized local tables, signed permutations and small nonlinear
geometric maps are candidate building blocks, not already qualified operators.

Train placement and operator choices from raw-text language targets and bounded
binding/composition tasks. The direct check includes held-out prose/Rust token
prediction and first-use wording, role/operand changes and short dependent
operations. Existing OPEN data stays OPEN; reserve new final assessment after
design selection. Exercise the actual discrete serving forward during learning,
or explicitly measure the discrepancy of an offline soft surrogate. Do not
assume hard argmax preserves a dense soft reader.

Compare with the current fixed-feature model and an appropriate sparse
non-geometric selector at matched capacity and training budget. Report quality
and complete work together; geometric sensitivity and matched advantage remain
separate. A control must alter the relevant geometry/partition, not merely
relabel expert IDs. Retain a useful capability/work result, or revise the named
placement/operator failure. A failed attempt is not a passed handoff and does
not send the main task back to cache growth by default.

Count load/validation, encoding, contextual-code construction, route probes,
refinement/fallback, comparisons, gathers, transformations, writes and output.
Include index maintenance, offline compilation, artifact/state bytes, peak RAM
and whole-response latency. Cap refinement and report misses; do not claim
constant-time accurate retrieval for arbitrary populations. Coarse summaries
retain routing features and exact-member links; recent occurrences and exact
bindings retain source detail that the summaries cannot reconstruct.

### Reuse existing foundations at the matching boundary

| Existing source | Intended reuse | Current boundary |
|---|---|---|
| UOR/addr and ordered prime addresses | Stable object identity, exact references, deduplication and provenance of learned artifacts and retained values | Address equality is not semantic proximity; preserve typed identity domains |
| [NAF/GNAF slice](../../crates/uor-r4-naf/src/lib.rs) and [integration record](../gnaf_integration_653.md) | Canonical supported values, typed operation/result boundaries and scoped cost claims when a concrete adapter needs them | State/operator/plan capabilities are incomplete; the separate WASM-GEMM proof does not establish this model's correctness or optimality |
| [Graph compiler](../../crates/uor-r4-graph-compiler/src/lib.rs), [R4G1 format](../../crates/uor-r4-graph-format/src/lib.rs) and borrowed runtime | Compile learned routes/operators into reusable packed tables, explicit references and bounded execution; deduplicate shared data | Historical teacher observations/region covers do not compile an arbitrary frontier model into a small equivalent artifact; integrate only needed lowering seams |
| [XOR/popcount route attention](../../crates/uor-r4-graph-runtime/src/route_attention.rs) | Bounded relation comparisons, top-M selection and integer aggregation over learned or geometrically justified codes | The existing operator scans its declared candidates; no semantic meaning follows from digest-bit distance, and it is dormant in serving |
| [Pinned uor-matmul](https://github.com/UOR-Foundation/uor-matmul/tree/b13c98449948174f590e337c4dc25dfc394a07d0) | Offline Rust learning/reference arithmetic; inspect its Atlas lookup/add kernels for a bounded selected geometric operator when useful | The float path uses coded lookup/exact accumulation but still evaluates a mathematical matrix product. Removing multiply instructions does not by itself remove dense work or establish a speed advantage |

The library is already called by Rust training/reference code in
[`geometric_training.rs`](../../crates/uor-r4-model-source/src/geometric_training.rs).
This is not evidence that the current native joint learner uses it, or that a
new kernel would improve this laptop workload. Compare actual shapes, arithmetic,
scratch, packing, projection and accumulation cost before adoption. Necessary
training operations may use it under the existing offline allowance. A serving
use must be an explicit bounded operator selected by the geometric model, not
the unchanged dense transformer routed through a differently named library.

The angular/prime-router archives, uploaded project material and affiliated
repos remain available for concrete design questions. Read original algorithms
before adopting them. Reuse implementation, not a new broad proof/adapter
programme or inherited capability claims. No dependency pin changes here.

### Step 4: select, execute and compose typed operators

Learn which operands and operators are required before executing them. Start
with existing copy, exact arithmetic and bounded relation traversal. Commit
intermediate values that causally influence later choices and output through
the same native model. The current numeric path computes admitted additions
before selection; learned admission should remove unnecessary proposals.

Demonstrate at least two supported operations with an intermediate state, both
in grounded conversation and generated Rust under changed operands, names or
dependencies. Execute generated code and check the requested semantics;
compilation alone does not pass. Reuse syntax, compiler feedback and existing
typed planning/verifier components as supervision/validation without giving
serving an answer oracle. Preserve earlier binding/memory/routing behavior.
The accepted assembled artifact supplies broader correctness, reasoning,
conversation and coding qualification; it is not itself alpha.

### Cadence and resource envelope

The current owner cadence is one task and one agent, with the spending goal
"try to be as budget friendly as possible." Do not add a hard token cap,
parallel research branches or a receipt/benchmark framework. Continue useful
authorized work, checkpoint the result, and keep the next unmet step explicit.

At adoption the cumulative model ledger is 1,081.641/1,800 seconds, leaving
718.359 seconds. Refresh it before execution; do not reset it per issue. The
first admitted cycle uses at most 120 seconds model work and 1,200 seconds
engineering commands, a 4 GiB model RSS target and one model process. Charge
preparation, fitting, evaluation, generated-code execution, retries and resumes.
These are the current cycle envelope, not permanent global training limits or
fresh cumulative grants for every stage. Project the complete build, fit and
evaluation using current cache state and measured work before launching.

Necessary incremental storage increases are already owner-authorized; record
each against cumulative accounting and retain the 128 MiB stop margin. This does
not authorize deletion or paid external compute. Reuse existing reports,
checkpoints and the isolated full worktree; deliver through protected PRs.

## Broader programme after the immediate sequence

The goal remains useful local conversation/memory and coding/reasoning on the
same native geometric path. After the four accepted handoffs:

- #954 qualifies grounded correctness, contradiction handling and abstention;
  #955 broadens and qualifies multi-step reasoning; #1088 owns executable coding
  and controlled workspace capability.
- #962 owns broader multi-turn conversation, identity-scoped durable memory,
  restart, export/forget and the CLI/service/workbench integration those tasks
  need. Existing interfaces remain usable during the immediate steps.
- #963 owns broader complete-path optimization and serving realization; #964
  owns precise implemented guarantees, formal evidence and eventual research
  publication; #965 owns integrated capability acceptance, portability/security,
  governance, packaging, installation, rollback and release.
- The first learned nonlinear geometric transformation belongs in #1139's
  routed block. After useful transfer, #973 grows composable blocks, data and
  memory with measured capability/work scaling. Fixed quaternion-cube mechanics
  or more lookup rows do not establish a general learner. External mechanisms
  are consulted for concrete questions; broad donor surveys, publication and UI
  expansion are not immediate prerequisites.

### Consolidated issue responsibilities

The following issues are closed as superseded, not certified complete. Their
bodies/history are preserved with successor links; do not reopen them as parallel
queues merely because an old document names them.

| Superseded issue | Remaining responsibility and owner |
|---|---|
| #1083 identity/arithmetic integration | Touched artifact/session lineage, exact arithmetic and UOR invariants in #1137-#1140; remaining serving/guarantee closure in #963/#964 |
| #1084 separate CLI/service integration | Maintain the same model interface within each step; broader conversation/service integration in #962 and coding/workspace integration in #1088 |
| #1087 separate serving-contract qualification | Per-step integer/table and quantization checks; complete serving qualification in #963/#964 |
| #1089 publication | Evidence and eventual publication in #964, with release claims in #965 |
| #1090 separate capability scorecard | Actual behavior/cost results in each build issue; integrated capability and final acceptance in #965 |
| #1091 broad NEMESIS/W33 survey | On-demand mechanism selection within the relevant #973 build issue; preserve previous positive/negative scope |
| #940 dormant ruleset cleanup | Actual release-governance/check semantics in #965; no claim that admin cleanup or broad QA has run |

Broader #954 depends on accepted #1140, not closure of the whole #973 model
parent. Keep #954 -> #955 -> #962 -> #963 -> #964 -> #965 as the broader
qualification chain, with #1088 dependent on #955 and #962 and also required
by #965. Closed superseded administrative trackers are not live blockers.

## Alpha acceptance

Both capability groups are required. Agree on representative examples before
claiming alpha, then retain the actual inputs, outputs and limitations:

- **Conversation and memory:** prompt-dependent multi-turn answers; retaining
  and updating supplied facts across the chosen context window; surfacing
  contradictions; explicit unsupported-answer behavior; isolated persistent
  user/session memory.
- **Coding and reasoning:** compose multiple supported steps, use selected
  workspace context, propose a bounded code change, run the relevant check,
  and consume real execution feedback. Report success and failure on fresh
  examples separately from training examples.

For each group report task coverage, successful/failed examples, context length,
latency, peak RAM, artifact/state sizes and backend. A few toy successes,
source-free output, Rust compilation, or preservation of an old comparator does
not alone meet alpha. Final held-out evaluation follows design selection;
development evaluation is allowed throughout learning.

## Practical iteration and machine budget

The run configuration declares context/window lengths, training dose,
checkpoint and evaluation intervals, thread count, wall time, RAM and new-storage
limits. Account for their **cumulative** use across warmup, training, evaluation,
retries and resumed segments. Choose these values for the question and available
machine; the old 120-token, 128-update and 840-second experiment is not a global
limit. A projection informs scheduling and checkpointing; it is not evidence
that a model cannot learn.

Within the remaining authorized budget, inspect failures, correct a concrete
cause and rerun or resume when that can advance the decision. There is no
universal 15-minute cutoff or one-retry quota. Do not blindly repeat an unchanged
failure, silently increase the cumulative budget, or incur unauthorized external
cost. Save useful checkpoints and stop cleanly at the configured limits. Before
lengthy work, use a representative timing sample or existing measurements to
select a feasible run; do not build an elaborate supervision system for a short
experiment.

## Verification and preservation

Compile and exercise the changed Rust path. Use focused tests for its causal
state, arithmetic, serialization and interface risks, plus a representative
end-to-end behavior check when behavior changes. Broad workspace/release suites
run only when relevant. A compatibility status from the protected merge queue is
not a test result; report which commands actually ran.

Preserve unique artifacts, source changes, old Python references and all
positive, negative and unavailable evidence. A negative binds the exact
artifact, population, operator, controls, budget and decision that produced it.
A changed operator or a longer/different development window is a new declared
experiment, never a rewrite of the old verdict. Distinguish mathematical proof,
measured behavior and hypothesis. There is no requirement to add a new ledger,
ADR, proof dossier or exhaustive control matrix for every edit.

External programmes such as HELM, W33, NEMESIS and SpiralCore are optional
sources for specific questions. Core prime/zeta/R4/UOR architecture is not an
external donor. Import external claims only after source inspection and direct
measurement in this model. Deliver changes through protected pull requests and
keep the actual current task in [current-state.md](current-state.md).

## Historical mechanical checkpoints through PR #1124 (2026-09-04)

The full-cache checkpoint delivered by #1119 remains the accepted comparator:
ordinary learned Q/K/V/O and softmax execute through exact H4 frame transport
with one chronological K/V record per observed token.

`R4FixedRecurrentCausalKVBindingV1` now provides an unfitted mechanical
successor: eight exact live K/V records plus four chronological binary-age H4
summary banks. Persistent K/V storage is 2,304 f32 values (9,216 bytes), versus
23,040 values (92,160 bytes) in the 120-token comparator. The first eviction is
committed after its causal decision, and later decisions read the summaries.

In the frozen full-prompt, seed-9738, 16-token comparison,
`A purple turtle found a clock in the garden` and
`Albert Einstein was born in` each shared 12 generated tokens with the
comparator before diverging. Both recurrent runs evicted records, read summary
banks, stayed within 13 attention sources, and made zero teacher, provider,
future, or forbidden reads.
This is measured mechanism behavior. It does not establish better language,
long-context retention, geometric advantage, architectural alpha, or
table-native execution. The trained RoPE limit remains 120 positions.

`R4SparseGeometricCandidateSoftmaxKVBindingV1` now ranks the fixed twelve-slot
metadata directory with exact H4 inverse/product/root witnesses, admits at most
eight persistent records plus current, and only then gathers K/V for unchanged
learned Q/K softmax. On the same two no-fit prompts, peak attention sources fell
from 13 to 9 and aggregate materialized scores fell from 3,824 to 3,240. The
geometric set differed from age-only on 33/35 sparse decisions and admitted 55
summary records. Common generated prefixes against the fixed recurrent path
were 12 and 3 tokens. This completes the sparse mechanical checkpoint while
leaving useful retrieval and geometric advantage unestablished.

`R4H4FrameQuaternionCubeResidualV1` now replaces each executed dense SwiGLU
residual with twelve ordered R4 cells and a current-H4-frame-indexed
quaternion-cube map. The 120 signed frame indices form antipodal pairs for this
odd map, leaving at most 60 distinct operators. It keeps continuous f32 hidden state, adds no
parameter or persistent state, and retains the dense tensors only so the
accepted artifact remains a byte-identical comparator. Across the two no-fit
prompts it executed 1,272 R4 blocks and zero dense-MLP calls while preserving
the nine-source attention ceiling and all causal prohibitions. Its largest f32
block-norm error was `7.152557373046875e-07`; both continuations diverged from
the fitted dense comparator at the first generated token and were visibly
degraded. This completes a mechanical nonlinear checkpoint, not useful
language or a selected training architecture.

The subsequent bounded fit task returned
`RESOURCE_UNAVAILABLE_FULL_CONTEXT_CUBE_FIT`. The full 120-token recurrent graph
completed backward and reached update 8 in both admitted launches. The sole
resource correction was followed by an elapsed-to-update-one reduction from
`78.177` to `25.757` seconds, but the fixed 128-update dose still missed the
840-second completion projection. No fitted artifact, model-quality result,
validation read, shorter dose, or additional retry followed. The then-next Python action was a lean training forward that omits unused attention-weight outputs
and precomputes the metadata-only selector while preserving the current
recurrent computation graph and inference semantics.
