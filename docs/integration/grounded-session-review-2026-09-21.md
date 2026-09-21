# Principal review: grounded computation and the next learned attention milestone

September 21, 2026. Reviews PR #1340, originally submitted as `17d5072c651a0574ff441309b05611fd399cbbec`, against corrected PR #1339 (`85df17394d3326cb30deae1d3b1ac5b895464ced`). This review owns interpretation and the [next execution brief](deepseek-learned-relation-control-step-2026-09-21.md); the [canonical plan](project-track.md) owns programme sequencing. Original artifacts and reports remain retained.

## Decision

**Retain the constructive geometric factorization and advance to learned relation-conditioned querying and content-dependent control inside an owned session.** The development transition graph now has an exact geometric realization. Another Q8 fitting campaign or a third host-written hop would not address the principal remaining gap: choosing which fact, operation and continuation the request and retrieved content require.

The component is meaningful progress toward the GLM. It is not yet learned general geometric attention, language reasoning or a complete inference-cost win. The fixed two-hop dependency is a useful retained control. The next milestone must complete a language-facing memory answer using a learned query/control policy on new supplied memories, on one actual loaded runtime path.

## Independent evidence and corrections

The retained [submission auditor](../../scripts/research/audit_grounded_session_submission.py) and [corrected replay auditor](../../scripts/research/audit_grounded_session_replay.py) are read-only receipt tools for the pinned local roots; they execute no model. The [original saved-data audit](../evidence/grounded-session-principal-review-2026-09-21.json) recounts all 1,440 arm/item records, checks all 1,296 grounded emitted-state products with exact signed-axis quaternion arithmetic and verifies all 672 development transitions. The 32 initial and 64 recurrent observed transition keys are consistent. Original four report seals, six source hashes and the retained release executable match. The original primary model execution took 2.869241292 seconds; all four retained executions total 11.549127584 seconds. These timings exclude preparation, builds and investigation; they are not complete serving latency.

| Arm | Development /288 | Length four /128 | Reversal /64 |
| --- | ---: | ---: | ---: |
| Grounded geometric factorization | 288 | 124 | 64 |
| Competent shared finite-state control | 288 | 124 | 64 |
| Newly fitted shared recurrence | 248 | 76 | 24 |

The third arm is a **new fit**, using all 288 development items as calibration, no probe, two restarts and seed `0xC0F40001`. PR #1339 used a 144/144 value-parity split, six restarts and seed `0xC0F30001`; its retained result was 186/48/10. Do not call the new fit the previous artifact or isolate a cause from changed optimization settings. The length-four/reversal panels are historically exposed combinations, not fresh final evaluation; four length-four failures are source-selection failures shared by the grounded and finite arms.

Material submitted-path defects required correction:

- Loaded grounded and finite objects were constructed but evaluation and the chain used their originals. The fitted recurrence was not exported/reloaded. Structural parity alone was not actual loaded serving.
- Program scoring passed detached `item.primitives` instead of deriving the declared instruction span from the observed prefix. Saved copies agreed, so counts remain valid, but this regressed the prior causal interface repair.
- The changed-first-source control changed payloads at absolute positions **2, 14 and 26**, including the newly selected second-hop value. Under a first-source-only edit, that second value stays **485**, with expected answer **4088**, instead of the reported **486 -> 4091**. Query and selection dependence were real; final-answer attribution was confounded.
- Second-selection correctness checked payload equality rather than occurrence identity, even though payloads repeat. Independent original recount verifies the four exact positions, but the instrument needed that stronger check.
- `first_lease` was captured after the second read. Resume equality was reported without saved snapshots or actual continuation records. The host owns the fixed schedule, so frame round-tripping did not establish an independently resumable learned controller.
- SourceLease stores a copied payload. Its unused live-ring helper compared only sequence and value, permitting a same-value replacement to look live. The correct scoped contract is an **owned immutable payload snapshot**, with separately checked exact provenance if resolving the current ring. A captured payload can legitimately remain usable after eviction.
- The Q8 isomorphism search assigned witness indices as group element IDs; the original witness happened to coincide with those IDs. Exact factorization also needed conflicting-label rejection and final consistency enforcement. Unsupported cyclic serving, malformed frame state/terminal tags and invalid state clamping require explicit errors.

The chain's first primitive 489 is the identity action. Disabling it can correctly leave the chain unchanged; this is an identity negative control, not evidence that a nonidentity first computation is necessary. Selected-source content still changes the second query; nonidentity ordered computation has separate program-panel evidence.

The corrections and executed verification are recorded in [principal checks](../evidence/grounded-session-principal-checks-2026-09-21.json) and the [corrected replay audit](../evidence/grounded-session-corrected-replay-audit-2026-09-21.json). They repair the exposed component evidence; they do not manufacture a fresh generalization result. The [result record](grounded-session-result-2026-09-21.md) distinguishes submission from correction.

The corrected exposed replay retains all 1,440 original item outputs and all panel totals. Eight bound checkpoint files verify across four chains. The isolated first payload edit at absolute position 2 changes the first result 4095 -> 4092, selected second payload occurrence 35 -> 26 and complete answer 4089 -> 4088, with the entire second-hop bank unchanged; both answers match independent expectations. Required first/second record removal and read-disabled controls stop with NoRead. Disabling the second payload update produces an incorrect answer 4088 instead of 4089. Identity first-computation removal correctly preserves output. The 13-file seal, six source hashes, three candidate artifacts and actual debug executable verify. Recorded replay elapsed 19.910024 seconds excludes build/preparation and is not optimized serving cost.

## What the mathematics establishes

Let `F_p` be the observed outcome permutation for primitive `p`. The finite development observations identify a regular Q8 action. The routine finds an isomorphism into a witnessed quaternion subgroup and coordinates satisfying

```text
Z[F_p(y)] = A[p] * Z[y]
E[x] = inverse(A[p]) * Z[F_initial(x,p)]
```

for all observed transitions. This is constructive learning from declared intermediate labels, not copying the generator's hidden coordinates. It is currently a **regular eight-state Q8-action factorizer**, not arbitrary finite-group discovery or a universal state-machine representation. Setting a reference outcome to identity removes common-right frame freedom (`Z_new[y] = Z[y] * inverse(Z[reference])`, leaving the left actions unchanged); subgroup automorphisms remain, with deterministic search selecting one representation. No unique semantic coordinate system is implied.

Group transport is invertible. World relations such as supervisor, current assignment or category membership need not be injective. Use geometry for query transformations, ordered computation and relational compatibility while exact memory and typed nonlinear operations implement the actual facts. Equal emitted tokens also need not imply equal hidden role, scope or control state. The current outcome alphabet is a useful finite computation fixture, not the general language-state ontology.

## Footprint and hardware interpretation

The grounded file is **137 serialized bytes**. The prior 102 was the method's parameter estimate (100 token/code bytes plus two declared fields), omitting 35 serialization bytes. The ordinary control is currently **1,668 bytes of JSON**. This supports shared factorization, not an encoding-normalized 16-fold compression claim. A compact ordinary table could use the same 80 token-ID bytes plus 96 one-byte transition destinations before headers; the geometric mapping uses those token IDs plus 20 action/state codes. Implement such packing only if a compression claim depends on it.

The common product/inverse tables occupy **15,480 bytes** in their fixed arrays, and common E/S/selector files occupy 454,788/53,555/4,408 bytes respectively, before runtime structures. Separate shared fixed assets, per-model parameters, per-session state and visited-candidate work. Full-vocabulary scoring, parsing, normalization, allocations and arithmetic elsewhere remain part of whole-path cost. No complete-path D0-b or physical energy qualification follows from this small geometric operator.

## Connection to the wider mechanism library

| Mechanism | Concrete role and sequencing |
| --- | --- |
| Exact prime/UOR/occurrence identities | Keep current records, versions and ownership distinguishable; identity is not semantic distance |
| Signed H4/Q8 and relative transport | Current exact ordered computation; next learned query/relation compatibility and a matched categorical control |
| Hopf observation and retained fiber/spin orientation | Q8's central sign is a concrete order distinction lost by a commuting quotient or base-only observation; preserve it where needed, without a physical quantum claim |
| Role/scope banks and typed state | Separate entity roles, multiple facts, current/previous values and continuation state when a single descriptor aliases them |
| Scalar or finite harmonic coefficient fields | Optional compact salience/compatibility summaries when interference or sharing is actually measured; finite orthogonality does not automatically supply learned syntactic routing or unlimited memory |
| Paired-H4/icosian E8 and S7 | Preserve the exact coupled construction; test only against a concrete lost distinction or cost bottleneck, with matched storage and causal controls |
| Existing owned objects and scheduling | Reuse ownership, resumable-action and feedback interfaces; historical authored byte/clause grammar is not learned BPE parsing |

The earlier mathematical corrections still apply: nonzero R8 vectors can be normalized to S7 while losing radius unless separately retained; E8 lattice/root vectors form a discrete structured subset. Quaternionic Hopf is S3 -> S7 -> S4; ordinary S3 -> S2 observation requires explicit fiber retention for reconstruction. G2/SU(3) is S6, not S7. Spin, scalar fields, primes and symmetry breaking become research mechanisms only when their representation, update and learned observable advantage are stated. There is no established supersymmetry/twin-prime language mechanism.

## Primary literature and implications

[Quaternion Knowledge Graph Embeddings, NeurIPS 2019](https://papers.nips.cc/paper_files/paper/2019/hash/d961e9f236177d65d21100592edb0769-Abstract.html) motivates learned quaternion relation scoring. Its continuous scorer is not an already compliant native kernel. [SpaceE](https://arxiv.org/abs/2204.10245) highlights the inability of injective rotation/translation representations to cover noninjective relations; its matrix solution is not adopted here. Together they favor geometric query compatibility plus exact typed facts, rather than forcing every relation into a permutation.

[PonderNet](https://arxiv.org/abs/2107.05407) motivates learning continuation from answer utility and computation cost. A native finite action policy can borrow that objective without importing its floating serving implementation. [Deep sequence models tend to memorize geometrically, v3, May 18 2026](https://arxiv.org/html/2510.26745v3) supports investigating shared relational geometry but explicitly distinguishes graphs memorized in weights from graphs supplied in context. Our next panel must vary the supplied memory graph and assignments so geometry cannot pass by remembering a frozen world.

Recent memory systems such as [Sparse Delta Memory](https://arxiv.org/html/2607.07386) and [HOLA](https://arxiv.org/html/2607.02303) are research comparisons for combining compressed state with selected memory access. Their matrix components and hardware results do not transfer to the GLM. No reviewed source justifies replacing the current native engine with a dense backbone. These are scoped literature motivations, not evidence of UOR-R4 predictive advantage.

## Programme adjustment

1. Preserve and reuse grounded shared computation, owned captured payloads and the corrected fixed dependent chain.
2. **Current milestone:** learn request/relation-conditioned queries and content-dependent Read/Apply/Emit/Stop in one real session; complete changed-memory answers on new worlds with competing plausible facts, resumed execution and matched controls.
3. Broaden that same path to varied language, persistent corrections, prose and executed Rust tasks. Retain useful partial operators when a larger experiment is negative; diagnose admission, representation, learning, control and emission separately.
4. Scale geometric access and qualify complete latency, memory traffic and physical energy at useful quality, then the API/session/product boundary. Cost profiling continues throughout; mathematical elegance alone does not guarantee cache residency or energy savings.

DeepSeek has substantive discretion over representations, fitting, development experiments and necessary diagnosed corrections. Codex retains principal architecture and evidence review. Use the standing owner authorization for prospectively recorded necessary local allowance extensions. No short timer or arbitrary retry quota substitutes for scientific judgment; actual hardware reserve, source preservation and external-cost boundaries remain binding.
