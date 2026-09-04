# Active UOR-R4 project track

This is the canonical build sequence for the UOR-R4 model and product. It
supersedes older next-action prose without erasing historical measurements,
negative results, or research records. Live GitHub decides which issue owns the
current stage; this document decides the order and the meaning of each
checkpoint.

## Goal and checkpoints

The project goal is useful, local intelligence whose serving path replaces
complete-prefix transformer attention, dense learned matrix intelligence, and
external model providers with bounded geometric state, routing, and table-native
execution.

- **Mechanical checkpoint.** A named mechanism executes on the real artifact
  and exposes enough direct behavior to decide the next implementation. This is
  engineering progress, not an intelligence or product claim.
- **Architectural alpha.** A source-free local artifact produces useful,
  prompt-dependent text through bounded recurrent geometric memory and bounded
  geometric selection. It does not retain or scan the complete prefix and makes
  no teacher, provider, or source-model call at runtime. Transitional f32 and
  dense nonlinear blocks may still remain.
- **Product alpha.** One local workbench exercises representative grounding and
  abstention, composition, identity-scoped memory, coding, and tool use with
  actual model behavior. Each behavior has a direct acceptance example and an
  honest unsupported case.
- **Release candidate.** The accepted product behavior runs through the Rust
  artifact/runtime path with bounded memory, table/integer serving operations,
  measured resource limits, installation, and rollback.
- **Release.** Targeted proofs, evidence reconciliation, broad QA,
  reproducibility, scorecards, and publication work are completed for the
  implementation that will ship.

## Ordered build sequence

1. **Fixed recurrent geometric memory.** Replace complete-prefix K/V storage
   with a small exact live window plus bounded multirate H4 summary state. Read
   before write and preserve a fixed storage bound. Compare real trajectories
   before fitting or evaluation.
2. **Sparse geometric attention.** Replace complete-prefix attention with a
   bounded geometric candidate selector and read operator. Keep the accepted
   causal Q/K/V path as a comparator while measuring whether geometric routing
   retrieves useful prior content. A bounded softmax comparator may remain
   during this stage; a full-prefix scan may not.
3. **Nonlinear geometric block.** Replace the dense SwiGLU/MLP intelligence
   path with a versioned finite R4 operator block or a separately typed E8/R8
   operator bank. Specify the state map, nonlinearity, residual/readout, and
   cost. H4/R4 and E8/R8 stay distinct unless an explicit bridge is implemented.
4. **Scale, data, and instruction behavior.** Increase model/data capacity only
   after the bounded architecture runs end to end. Train on open development
   data with declared resource limits; measure useful language, instruction
   following, retention, and composition before larger campaigns.
5. **Retrieval and tools.** Add typed local retrieval and tool contracts,
   explicit ambiguity/refusal, execution feedback, and result ingestion.
   Exercise real sources and tools; a registry lookup or route inspector alone
   is not tool-use capability.
6. **Representative product alpha.** Connect the model to the native local
   workbench and complete the representative grounding, composition, memory,
   coding, and tool-use behaviors.
7. **Rust/table lowering and optimization.** Preserve the accepted behavior
   while moving Python/f32 reference operations into the packed Rust runtime,
   then remove remaining multiply, float, allocation, and unbounded work from
   the deployed kernel.
8. **Release proof, evidence, and QA.** Reconcile claims, proofs, negative
   results, resource measurements, portability, security, broad QA, scorecards,
   and publication against the release candidate. This work does not gate the
   earlier build stages unless a concrete implementation decision needs it.

Fixed recurrent memory, sparse geometric attention, and the first nonlinear
geometric block are completed mechanical checkpoints under
[#973](https://github.com/UOR-Foundation/uor-r4/issues/973). The current stage
is bounded scale/data/instruction fitting of the assembled architecture. #954
and the old capability chain remain historical issue structure; they do not
insert a proof or bookkeeping campaign between these build stages.

## Current measured boundary

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
validation read, shorter dose, or additional retry followed. The direct stage-4
action is now a lean training forward that omits unused attention-weight outputs
and precomputes the metadata-only selector while preserving the current
recurrent computation graph and inference semantics.

## Research reservoirs

Research is consulted when an active implementation has a concrete unresolved
design question. It is not a serial gate or a license to import capability
claims.

| Reservoir | Candidate contribution | Current evidence boundary |
|---|---|---|
| SpiralCore v66 | Typed finite-state discipline, explicit refusal, canonical transition tables, and a labelled E8 action graph that may inform bounded operator-indexed routing | Executable browser reference and self-described fixtures; no recurrent language memory, learned attention, nonlinear R4/E8 model block, tool use, Rust lowering, or UOR product behavior has been independently measured here |
| HELM | Causal decoder, cache, and comparator semantics | External reference architecture; its mechanisms are not UOR capability evidence |
| W33 | Finite constructors, persistent graph ideas, and proof corrections | Research candidates only until mapped to an implemented UOR operator and directly tested |
| NEMESIS | Attributed mathematical and systems hypotheses | No license was found in the inspected tree; do not vendor it or treat illustrative code and complexity claims as verified |
| UOR standards and repositories | Typed identity, addressing, canonical artifacts, storage, and integer arithmetic components | Reuse only at a named seam with compatible semantics and costs |
| H4/zeta research | Typed R4 frame transport, chirality/cosine polarity, `Z[phi]` heatmaps, and fixed-zeta phase channels | Structural or control hypotheses until a matched intervention measures contribution to the active model behavior |

SpiralCore's E8 construction is a finite labelled transition graph and
operator-indexed routing candidate. It is not called attention until a UOR
implementation maps model state to candidates, aggregates values causally, and
measures language-relevant behavior. Its H4/R4 and E8/R8 constructions remain
typed separately. Its JavaScript tables are reference and fixture material,
not evidence of Rust, integer, allocation-free, or multiplication-free
lowering.

## Evidence and iteration rules

- Keep mathematical proof, measured behavior, and unverified hypothesis
  explicit. A direct behavior report is enough for normal development.
- A negative result binds the exact artifact, data population, operator,
  controls, budget, and decision rule that produced it. Preserve it. A
  materially versioned successor may re-enter when it names what changed and
  why that change could alter the result.
- `UNAVAILABLE` records an execution, source, or environment boundary. It is
  not positive or negative model evidence. Do not blindly retry an unchanged
  unavailable or negative configuration.
- Bounded iteration on open development data is allowed. Keep final held-out
  evaluation separate after design selection. Predeclare a decision only for a
  costly or release-bearing run, not for every code edit.
- Use one active implementation issue, an isolated worktree, the smallest
  direct check, and the protected pull-request path. Independent review is
  proportionate to novel causal/state code or explicit owner request; it is not
  mandatory paperwork for every change.
- Preserve unique artifacts, user material, and prior results. Defer broad
  proof, ledger, indexing, publication, and QA work until the release candidate
  unless the active code decision or owner explicitly needs one.
