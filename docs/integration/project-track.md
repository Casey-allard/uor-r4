# Native geometric AI project plan

This is the canonical project goal and development plan, restored by owner
instruction on 2026-09-04. The [current state](current-state.md) names the actual
implementation and remaining work; live GitHub owns issue status. Other
roadmaps link here instead of copying a changing stage list. Earlier sequencing
and fixed experiment windows are historical, not instructions for new work.

## Goal

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

## Go-forward work

1. **Restore one native model path.** Evolve the existing Rust prime-route,
   fixed-zeta and geometric state components under #973. Connect text ingestion,
   causal updates, a learned geometric read/write/selection operator, artifact
   reload and decoded output. Reuse existing correct pieces before adding new
   geometry. A mechanical vertical slice is useful progress, not alpha.
2. **Learn useful behavior on the real path.** Train the Rust model on open
   development examples. Include retention across context changes, variable
   history lengths, prompt-dependent continuation, composition and small code
   tasks. Select meaningful configurable training/context/evaluation windows
   from the machine budget. Profile a measured bottleneck before inventing a
   replacement mechanism or changing the mathematical objective.
3. **Develop both alpha capabilities.** Conversation/memory and coding/reasoning
   progress against the same model and artifact. Add grounded answers,
   contradiction handling and abstention; add bounded multi-step composition
   and code repair with actual execution feedback. Increase task variety and
   context deliberately. Do not substitute canned replies, corpus recall alone,
   or a hidden provider for those behaviors.
4. **Integrate one Rust product.** Expose the actual model through the native
   CLI/service and local workbench. Exercise load, generate, cancel, memory
   persistence/isolation and controlled workspace operations. Product examples
   may develop alongside model work; a narrow four-fact reference or polished
   shell does not establish the broader alpha capabilities.
5. **Compile and optimize the learned geometric path.** Develop integer/table
   realization alongside native operators, with measured encode/operate/decode
   fidelity. Remove remaining inference-time dense transformer operations and
   unbounded work; measure the complete path, not only its K/V ledger. Rust is
   already the implementation language, not a final porting stage.
6. **Qualify a release.** Reconcile claims and preserved evidence, run relevant
   broader portability/security/resource checks, and complete installation,
   reproducibility and rollback. Formal proofs and publication support the
   implemented model when useful; they are not a serial prerequisite to building
   or training it.

These are dependent deliverables, not a hardcoded sequence of frozen research
rungs. Work may overlap when interfaces allow it. An explicit request to carry
out the whole plan authorizes continued progress across its necessary tasks;
there is no automatic one-task stop. Keep one coherent active objective and
coordinate independent subtasks without competing changes to the same model.

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
