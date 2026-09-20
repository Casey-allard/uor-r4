# Structural memory, retained Hopf fibers and conditional S7 research

Owner-directed addition, September 20, 2026. This is an adopted research direction and sequencing decision, not an implemented mechanism or measured advantage. Source baseline: `1dae322ce7770673c0046597c10f4361564cd3a0`. The ongoing [exact-occurrence reader experiment](deepseek-occurrence-reader-step-2026-09-20.md) remains first. The [canonical plan](project-track.md#structural-memory-and-geometric-representation-follow-up) owns sequencing; this note specifies the mathematical hypothesis, reuse opportunities and decisive comparisons. No model execution or resource extension is performed by this documentation change.

## Hypothesis and intended contribution

Local recency and structural persistence should have different update rules. Learn to route **occurrences in context**, rather than permanent word types, into bounded role/scope memory. Retain exact source identity and version separately from the geometric state used to select or transport it. A subject can survive irrelevant descriptive tokens, while a nested clause can retain another subject in a different scope. Roles may be learned latent classes; grammatical labels are useful diagnostics, not permission to insert an authored serving parser.

The first candidate is a banked view of the occurrence reader:

`address = (scope/frame, learned role, occurrence/version)`.

Keep several occurrences where required, causal scope decisions, an explicit uncertain/NoRead outcome, bounded capacity and declared eviction. A bank pointer alone is not persistent if its referenced ring slot is later overwritten: retain/pin/copy selected payloads within the same total memory budget, or treat their eviction as a real loss. Learning a good address and preserving the addressed content are separate obligations.

For disjoint banks, write isolation can be stated as

`project_bank(a, write_bank(b, M)) = project_bank(a, M)` for `a != b`,

apart from explicitly declared shared metadata and capacity eviction. This property prevents unrelated writes from changing a protected payload. It does not prove the learned router assigned the correct role or scope. Shared normalization, packed arithmetic carries, generic rotations and nonlinear mixing must not silently invalidate the claimed isolation.

## What the recalled Hopf loop actually implements

The first Hopf fibration is `S1 -> S3 -> S2`, with S1 the fiber, S3 the total space and S2 the base. It is not a lossless dimensional chain `S3 -> S2 -> S1 -> S3`. Reconstruction requires retained fiber and chart information, or retaining the original state; there is no global continuous product identification `S3 = S2 x S1`.

The existing [mathematics audit](architecture-2026-09/mathematics.md#supplement-trigonometry-s3s2s1-fibers-and-vector-bundles) and current source supply these concrete pieces:

| Source | Implemented role and reuse boundary |
| --- | --- |
| `crates/uor-r4-core/src/native_geometric/hopf_metric.rs`, `hopf_fiber_point`, `from_hopf_fiber`, Q30 counterparts | Actual projection, retained S2 base plus S1 phase/phasor, and reconstruction with a pole convention. The Q30 tests use numerical tolerances; comments saying "exact" do not establish bit-exact continuous-state recovery. Q30 helpers use integer multiplication/division and are not by themselves D0-b compliant. |
| `native_geometric/learner/binary_model.rs`, `jepa_trainer.rs`, `native_geometric/runtime.rs` | The optional older `geometric_prose_tables` path folds context into a root, projects it, predicts base/fiber, and lifts back to S3 for hierarchical shortlist routing. The owner's recalled observe/retain/update/reconstruct loop has a real source basis here. It is separate from the recent CPX3 reader experiment, and supplies no new language qualification. |
| `crates/uor-r4-core/src/prime_route_attention.rs`, `UnitS3Q30::hopf`, `rotate_common_fiber`, `SpinTorsionState` | Hopf observation, common phase action, retained signed S3 plus phase/torsion fields. The fiber field is not a verified universal inverse coordinate. Some helpers execute floating arithmetic/trigonometry despite integer-shaped storage. Reuse their mathematics or offline oracle, not an unreviewed serving call. |
| `crates/uor-r4-core/src/lib.rs`, `hopf_coordinate_components_scalar`, `hopf_phase_transport_components_scalar` | Local angular coordinates and phase transport. The S2 azimuth omits latitude and is undefined at poles; common phase is not derivable from the base. These scalar floating helpers do not establish a reversible universal cycle or compliant native serving. |
| `crates/uor-r4-core/src/canonical_lexical_ingestion.rs` | Typed complementary charts, orientation/fiber/torsion discipline and inverse witnesses for the declared discrete bridge. Reuse exact contracts without importing the older authored grammar into the BPE learner. |
| `crates/uor-r4-core/src/native_geometric/training.rs` and `native_geometric/learner/prefix_artifact.rs` | Existing offline compilation of finite H4 products/inverses and phase channels supplies a cheaper transport baseline. A bounded finite action can use lookup; arbitrary continuous rotations are not automatically closed in the 120-root set. |
| `native_geometric/memory_runtime.rs` and `source_routing.rs` | Sequence/slot checks, exact payload retrieval and source/query-local transport support the banked reader. These contracts separate admission, ranking, source retention and payload commit. |

Abbreviated paths in the table are under `crates/uor-r4-core/src/`. These are different implementation families. Their presence does not mean the current BPE predictor already executes a combined Hopf/fiber attention loop. Preserve signed state, exact payload references, chart/null cases and declared geometry identities when adapting a component. Source was inspected; existing tests were not rerun in this documentation task.

## S7 as a conditional relation-state representation

The relevant higher construction is the **quaternionic** Hopf fibration `S3 -> S7 -> S4`. For a normalized pair of quaternions `(a,b)`, one convention is

`H(a,b) = (2 a conjugate(b), |a|^2 - |b|^2)`.

Its output has unit norm in R5 because `4|a|^2|b|^2 + (|a|^2-|b|^2)^2 = (|a|^2+|b|^2)^2 = 1`. Simultaneous right multiplication `(a,b) -> (a u,b u)` by a unit quaternion leaves the base unchanged. The S3 fiber retains that common frame information. Base, fiber and local chart/transition data can represent the original point; discarding the fiber cannot. See [Baez, Projective Lines/Hopf bundles](https://math.ucr.edu/home/baez/octonions/node9.html) and [Mosseri and Dandoloff, Geometry of entangled states, Bloch spheres and Hopf fibrations](https://arxiv.org/abs/quant-ph/0108137).

**Engineering hypothesis:** a relation may benefit from an invariant relative observation plus retained frame state that supports later mutation, transport and composition. A shared-frame change should preserve the represented relation; changing one endpoint should change it when the task requires. The geometry does not itself assign linguistic meaning to these coordinates.

Start from the reduced finite construction before a general S7 implementation. If `g,h` are unit H4/2I states and `(a,b)=(g,h)/sqrt(2)`, the Hopf base is `(g h^-1,0)`: only an equatorial slice of S4, not coverage of full S7/S4. Its relation observation reduces to the already available finite relative-group lookup. Bind the action convention: this `g h^-1` is invariant under common right action, whereas a reused `g^-1 h` transport is invariant under common left action and conjugates under common right action. Align comparator and intervention conventions before declaring an invariance failure. That is an especially useful comparator and may be all the mechanism needed. Store finite indices; the normalization belongs to the mathematical embedding and need not run at serving. If a richer proposal adds amplitude bins or other coordinates, identify the additional distinctions and test whether they help beyond this baseline and an equally sized ordinary codebook.

Keep the project's golden/Galois-coupled `H4 + phi H4` icosian companion distinct from two independent quaternion endpoints. Neither eight stored coordinates nor an exploratory independently trained companion demonstrates a qualified S7 relationship memory. This construction uses associative quaternions; it does not require replacing the current group operation with nonassociative octonion multiplication.

## What a mutable harmonic wave would mean

One point on S7 and a function on S7 are different state objects. A truncated harmonic field has explicitly stored coefficients, for example `F(x)=sum c[l,alpha] Y[l,alpha](x)`. Its capacity and cost include every coefficient, precision bit, update and retrieval operation. Orthogonality is an integral property of functions; a single spatial sample is generally a mixture. A coefficient can be read by index if already stored in that basis; recovering it from samples requires an actual projection/reconstruction method.

On S7 the eigenvalues of `-Delta` are `l(l+6)`; many angular modes share a degree/frequency. A resonant frequency alone does not isolate the role. Updates must preserve the chosen decomposition, and finite sampling/quantization requires checking the actual weighted Gram matrix. E8 has 240 roots but only eight ambient dimensions; its roots are not 240 independent orthogonal registers. Base-256 supplies finite addresses, not additional dimensions. [Frye and Efthimiou, Spherical Harmonics in p Dimensions](https://arxiv.org/abs/1205.3548).

There are consequently two distinct optional experiments: a finite Hopf relation state, and a bank of mutable harmonic coefficients. Do not conflate them. A coefficient bank is worth implementing only with a specified write/read law and a capacity/interference or quality/cost hypothesis beyond direct role slots. Exact occurrences remain outside any lossy wave summary. If superposition requires more bytes or comparable projection work, include it in the comparison rather than treating it as free compression.

## Decisive evaluation and reuse decisions

After the current reader establishes a usable baseline or exposes a specific bottleneck, evaluate a small learned role/scope extension with:

- Fixed local suffix and query, changed distant binding/payload; actual generated answers must follow the changed source.
- Increasing irrelevant fillers, competing and nested subjects, the same token in changed roles, corrections/reassertions, and bounded-capacity/eviction cases. Separate role error, scope error, candidate absence, stale reference and ranking error.
- A shared-bank reader and a competent exact/latest baseline under the same total retained bytes and candidate/work bounds; ReadDisabled and role/scope-routing interventions. If a tradeoff cannot be exactly matched, report its quality/cost curve explicitly.
- Learned decisions from causal observed text with held-out bindings and structural recombinations. Oracle role labels can diagnose representation capacity, but their scores do not qualify the learned model.

Then test only the geometric addition warranted by the result. Hold occurrences and candidate support fixed for a transport/representation comparison. Compare existing finite H4 relative transport, explicit bank tags, and any proposed Hopf base-plus-fiber state. Change both endpoints by a common allowed action, change one endpoint, remove/permutate the retained fiber, and test chart boundaries where applicable. Preservation checks must use the candidate's declared finite domain. If only fixed lookup binding is needed, retain that helpful mechanism without building a full wave engine.

For a later harmonic arm, measure protected-channel changes under distractor writes, collision/capacity curves, quantization/saturation and actual query extraction cost. A pure base-only arm can diagnose what information the fiber restores, but an unfairly lossy baseline alone cannot establish S7 advantage. Existing geometry and an equally sized non-geometric codebook remain serious comparators.

Predeclare a practically useful target with uncertainty reporting before fresh evaluation. Separate exact correctness invariants from empirical quality thresholds. A near miss may justify one prospective revision with a causal rationale; preserve its original outcome. Stop the variant when its distinction is erased, its learning is unsupported, or its quality/cost is dominated. Reuse a helpful submechanism without promoting the whole hypothesis. No endless basis/dimension sweep or new broad qualification campaign is implied.

## Delivery and resource scope

#973 owns integration and these bounded research variants; #1139/#962 retain their broader binding/memory responsibilities, #963 measures complete cost and #964 owns declared invariants. All implementation/evaluation claims above are NOT_RUN. Useful prose, general reasoning/coding and energy advantage remain unqualified.

Before any later experiment, refresh the live cumulative model/storage ledger, project the complete build/fit/control/evaluation/continuation costs and record any necessary standing-authorized extension before use. Preserve the 128 MiB margin, negative artifacts, existing original checkouts and DeepSeek's active work. This note changes no training dose, runtime contract or active execution prompt. D0-b still permits bounded low-bit additions/subtractions/shifts/table reads, with no numerical-kernel multiplier or serving floating point. The roadmap can advance this hypothesis through a concrete, measured advantage; it does not make S7 a mandatory milestone for the language-model goal.
