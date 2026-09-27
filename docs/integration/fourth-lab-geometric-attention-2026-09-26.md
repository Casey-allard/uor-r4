# Fourth lab: geometric attention research and first implementation design

September 26, 2026. References #973 under #820. **Source/literature research and
implementation design; no new model, test, benchmark or capability result.**

The fourth lab should own a missing connection: **jointly learned finite
geometric attention inside the language graph**, carried to the existing integer
session. Preserve the full causal 256-token reader while establishing that
connection. This can proceed as an independent implementation alongside the
existing emission localization, dialogue/parallel-scan and session engineering
tracks. It is not a claim that absent geometric attention caused the current
prose failures, or permission to restart the completed exposure-only experiment.

## Authority, inspected revisions and limits

The initial owner checkout was `413a32fcf55f13a267862fd823109f8b459cafa9`.
After the principal refreshed origin, this review read the current authority and
changed model source at **`4b62608702481abd3f18915dc3718e311b86b8aa`** using
`git show origin/main:...`. The latest language continuation has therefore been
included; the September 25 next-fit wording in the owner checkout is stale.
Source line references below refer to `4b626087` unless explicitly branch-scoped.
README, canonical plan, current state, direction assessment and project map were
read, followed by the relevant model call chains and reusable geometry/import
audits. This is recursive source coverage of those decisions, not a claim that
every archived file or imported theorem was independently audited.

The latest [continuation result](language-continuation-result-2026-09-26.md)
records continuous prose 0/5 in both arms and integer prose 0/5 quaternion,
1/5 ordinary. All miss the fixed improvement criterion. NLL improved; that did
not qualify coherent language. Ordinary conversion/source failures and the
quaternion integer short-cycle remain separate observations. The accepted
September 25 parents remain retained. The existing-record emission diagnosis
is an active neighboring responsibility, not a completed causal explanation.

Read-only branch inspection observed:

| Working tree / revision | Observed responsibility | Integration consequence |
|---|---|---|
| `canonical-address-routing-20260925`, `50ac2527f95508de114080d36e76826d7781a3b5`, with dirty training files | Dialogue data/objective, cache comparison, then R2 scan cell | Reuse its data and eventual feature-generation interface after owner coordination; do not edit its live files or assume the scan is an equivalent execution of the retained cell. |
| `geometric-chatbot`, `474e98665e77c20283d9b63f5ebb5f20ace29b18`, with dirty integer/session files | Integer session, memory, chat surface and numerous adversarial checks | Obtain its final artifact/session contract before integration. Test count and a chatbot label do not establish learned dialogue. |
| `emission-localization-20260926`, `4b626087` | Same-checkpoint emission/selection localization, identified by the principal | Consume its finding; do not duplicate that diagnostic or alter its witnessed failures. |

These are source/worktree observations, not authoritative identification of which
external lab or person owns each process. They can change concurrently. The
shared issue work cards must resolve ownership before overlapping writes or
compute. No external lab was contacted by this review.

## What is actually implemented

| Mechanism | Current source evidence | What remains missing from this learned language path |
|---|---|---|
| Causal contextual reading | [`joint_model.rs`](../../crates/uor-r4-training/src/joint_model.rs), lines 975–1075: provisional state, learned Q/K, full previous-event dot product, learned age, NoRead and softmax | Finite H4/2I relative addressing or scoring is not this attention rule. Calling its Q/K vectors quaternion blocks does not change the executed score. |
| Geometric state transport | Same file, `transport_lanes_with_unit`, lines 1753 onward; `numerical_contract`, lines 1696–1715 | Signed quaternion left multiplication is in the actual recurrence. It is approximate numerical transport, not exact H4 or `Z[phi]` state. The ordinary arm changes the transport rule; it does not isolate a geometric attention kernel. |
| Joint language credit | Same file, state/write logic at 1083 onward and contract at 1714; `language_loss_reaches_query_key_value_and_recurrent_state` | This is the reusable learning advantage over A1–A4. A new hard geometric read must preserve credit through earlier writes, later state and actual emission. |
| Exact occurrence/token tape | `MemoryEvent`, lines 175–180; writes after read/update at 1104 onward | Occurrence identity is present. Rich durable subject/version/scope semantics from older native paths are not automatically integrated into this tape. |
| Vocabulary plus pointer-copy output | `output_distribution`, lines 1351 onward: normalized vocabulary mixture plus copy masses | NoRead disables both value feedback and copying, so its gain cannot isolate either path or geometry. A geometric reader still needs useful generated words beyond exact copying. |
| Actual integer execution | [`integer/model.rs`](../../crates/uor-r4-integer/src/model.rs), `matrix_work` at 234 and `step` at 276 | Four-bit learned affine maps use products tables and sums; attention remains full and variable-value products use declared integer routines. Dense parameter traffic and allocation persist. |
| Exact finite composition donor | [`group_table.rs`](../../crates/uor-r4-core/src/native_geometric/learner/group_table.rs), lines 40–70 and 154–223 | A signed 120-element composition/inverse table exists. Its current constructor uses floating root classification at initialization; a new standalone serving bundle should contain the bound table rather than silently invoking this constructor. |
| Earlier exact-address memory | [`geometric_attention.rs`](../../crates/uor-r4-core/src/native_geometric/learner/geometric_attention.rs), lines 129–154 | Token assignment is `token % 120`; ordered radix words preserve the assigned labels, not unique vocabulary identity above 120 tokens. This is a different model path. |
| Prime/zeta and ordered path attention | [`prime_route_geometric_attention.rs`](../../crates/uor-r4-core/src/prime_route_geometric_attention.rs), candidate-support types and `query_support_only`; [prior synthesis](principal-attention-plan-2026-09-24.md) | Real causal primitives exist, with bounded posting admission before geometry. They are not learned Q/K in the retained joint learner. Prime identity cannot be substituted for semantic proximity. |

The integer loader binds context/state/read widths separately: 256/256/64
(`integer/model.rs`, lines 155–158). The configuration also has a full-vocabulary
4096-by-256 tied embedding/output map (`integer/config.rs`, lines 59–84).
Reducing 64-coordinate attention scores alone will not remove this output
traffic, the recurrent maps, or 256-coordinate value accumulation. Whole-model
speed and D5 parameter sparsity need their own accounting.

## Mathematical checks that prevent misleading conclusions

1. **Quaternion scalar attention may be ordinary dot attention.**
   `Re(conj(q) * k) = q_0 k_0 + q_1 k_1 + q_2 k_2 + q_3 k_3`.
   Summing this over lanes exactly recovers the real dot product. A renamed
   implementation cannot establish a new inductive bias. The recent
   [shared-score quaternion paper](https://arxiv.org/html/2605.24920v1#S3.S2)
   makes the same equivalence explicit; its useful structural change is in
   quaternion parameter sharing, and its principal task is speech enhancement,
   with additional classification experiments. It is not evidence for this
   project's autoregressive prose or multiplier-free serving.
2. **A fixed linear function of the full relative quaternion is still bilinear.**
   For fixed `w`, `w · (conj(q) * k)` is `q^T M_w k`. With unconstrained learned
   Q/K projections, a suitable fixed factor can be absorbed into a projection.
   Its four signed coordinates preserve more information than one cosine, but
   this algebra alone does not establish an expressive improvement over dense
   attention. A nonlinear finite relation score and the finite coding/cost
   constraint supply the distinct research question below.
3. **Endpoint transport alone can be a change of frame.**
   If `U` is a representation, then
   `sum_i a_i U(q^-1 k_i)v_i = U(q)^-1 sum_i a_i U(k_i)v_i`.
   This factorization can be computationally useful. It is not automatically
   additional reasoning or nontrivial path-dependent geometry. Claiming a
   connection or holonomy needs actual edge/path data and its transformation
   law. [Gauge-equivariant networks](https://proceedings.mlr.press/v97/cohen19d.html)
   provide a precise frame-consistency precedent; they do not identify a natural
   physical gauge symmetry of text.
4. **One finite group state cannot encode an arbitrary history injectively.**
   A fold into 120 states has at most 120 outputs, however long its ordered
   input. Keep exact occurrence/token payloads separately. Likewise, the older
   source comment suggesting an injective single-root assignment for vocabularies
   larger than 120 cannot hold; multi-lane codes or external exact identity are
   required. This localized historical documentation defect does not warrant a
   separate bug campaign.
5. **Relabeling is not an adversarial geometry control.**
   Consistently permuting code labels, tables, encoder and decoder yields an
   isomorphic model. Equal performance is required, not evidence against
   geometry. An ordinary alternative must alter the operation class while
   preserving information, fitted opportunity and cost, or test a genuine
   group/composition property with an independently refitted comparator.
6. **Exact algebra and finite arithmetic have different claims.**
   Composition of bound group labels can be exact. Root coordinates, gains,
   value mixing and learned probability calculations can remain approximate.
   `Z[phi]` coefficient operations require explicit range/denominator bounds.
   Neither Hamilton multiplication nor an empirical energy score implies a
   Hamiltonian dynamical system or physical energy conservation.

## First operator: learned signed relative-group score, full admission

Implement a new, optional training module, provisionally
`crates/uor-r4-training/src/geometric_read.rs`, with a versioned read-kernel
configuration. Leave existing artifacts on their current kernel. The initial
integration replaces **only the score computation** and retains current exact
events, value/copy paths, NoRead, masks, objective and full256 access. Do not add
admission pruning, a new recurrence and a new output head in the same causal
change. Broader architectural discovery remains available after this comparison.

For the existing 64-coordinate query/key, use 16 signed four-coordinate lanes.
Let `G` be the existing 120-element binary icosahedral group, `c(g)` its declared
signed unit coordinates, and `Q` a nearest signed-root quantizer. For each lane:

```text
q_l = Q(normalize(q_raw,l));       k_i,l = Q(normalize(k_raw,i,l))
r_i,l = inverse(q_l) * k_i,l       // exact bound group-label table
score_i = sum_l E_theta,l(r_i,l) + age(t-i)
mass = softmax([learned_NoRead, score_0, ..., score_(t-1)])
read = sum_i mass_i * existing_value_i
```

This is a proposed new finite learned score, not an exact conversion of the
unconstrained baseline. In particular, unit coding removes query/key norms.
Measure that representational change explicitly against the norm-controlled
ordinary comparison; do not interpret a norm-removal loss as a failure of
noncommutativity. A later declared finite gain channel is possible if this is
the localized obstacle, but no gain sweep is part of the initial implementation.

`E_theta,l` should be a small **smooth nonlinear function** of `c(r)` during
training, evaluated on the 120 roots and compiled to one score table per lane
for hard execution. The final learned table entries use at most four-bit signed
codes with declared dyadic lane scales, consistent with D0-b; wider offline
scores are an explicitly separate learning view. Train with the declared hard
score quantizer in the forward path before claiming export retention. A shared
small function with lane parameters is an option;
its exact parameter count and function must be fixed in the implementation
card. A purely linear function would have the bilinear equivalence above.
An unconstrained direct table without a declared input gradient is insufficient
to train the root encoders.

The first feasible surrogate is explicit hard-forward, continuous-backward
root quantization. With `stop` denoting stop-gradient:

```text
q_ST = q_soft + stop(c(Q(q_soft)) - q_soft)
k_ST = k_soft + stop(c(Q(k_soft)) - k_soft)
r_smooth = conjugate(q_ST) * k_ST
r_ST = r_smooth + stop(c(inverse(q_code)*k_code) - r_smooth)
score = E_theta(r_ST)
```

Forward uses the actual hard relation. Backward uses the derivative of smooth
quaternion composition and the nonlinear score function; this is a **biased
surrogate**, not the derivative of the discontinuous quantizer. Do not pretend a
finite-difference mismatch at a root boundary disproves the declared estimator.
Check the smooth Jacobian and actual end-to-end credit, and track hard-path
learning/usage so that collapsed codes cannot pass as successful geometry.
Avoid a 120-by-120 soft categorical pair distribution per lane and event: it
would multiply training cost for an unnecessary initial relaxation.

Specify the zero-vector result and exact tie order; never fold antipodes.
Nearest-root selection can compare unnormalized dot products because positive
normalization does not change their order. This removes a normalization only
when the root norms and score convention are common. Bind the fixed root table
and the declared integer comparison precision in the serving artifact. Existing
finite-table evidence does not automatically validate a newly rounded encoder.

The value path initially stays unchanged. If full signed relations later prove
useful for interpreted content, the next distinct operator can expose relation
coordinates to the shared update or perform a bounded relation-conditioned
value action. Compare it against equal-sized ordinary features/actions. Pure
homomorphic endpoint transport has the factorization above; a nonlinear typed
action is a different hypothesis and must be named as such. Do not silently
claim it follows from a first score result.

Prime/UOR identity continues to own exact provenance and occurrence/version
addressing. Fixed zeta phase channels, ordered n-lets and exact paired-H4
representations remain architectural donors. None should be inserted as an
extra untrained feature merely to satisfy a naming checklist. Add a phase or
relation channel only with a declared task role, trainable causal use, and an
equal-cost ordinary-frequency/representation comparison when attributing it.

## Controls and learning questions

The original quaternion/Householder pair compares **state transport**; it is not
the entire attention control. The first implementation keeps one parent
recurrence fixed to isolate the new read kernel. Reuse the ordinary recurrence
once the read integration is operational and the complete cost allows it.

The immediate development comparison is baseline QK reading versus the signed
finite relation reader on the same data and complete generated continuations.
Include a norm-controlled ordinary reader where needed to identify what unit
coding removes. Use actual emitted continuation quality and changed-source
behavior, alongside natural likelihood. Any codebook distillation or teacher
term must be declared as offline supervision, with its data and cost charged.
It cannot replace next-token learning or supply serving responses.

The decisive questions are broad enough to change architecture:

- Does finite relative coding retain relevant distinctions and learn useful
  source-sensitive prose, or does its information loss dominate before the
  readout can use it?
- Does the nonlinear relation score improve complete generated behavior beyond
  its normalized dot baseline? If yes, is that from a useful finite interaction,
  extra parameters, better optimization, or a particular group operation?
- Does the new reader improve entity/role continuity and prompt-connected event
  progression, or only prediction of common/copied tokens? Consume the emission
  lab's per-token vocabulary/copy attribution where available.
- Does the compiled reader reduce measured whole-token cost after query/key
  coding, all value work and output work are included? A faster lookup alone is
  not the efficiency objective.
- Can the same learned reader retain behavior in integer execution and then in
  the shared session? A continuous positive is worth preserving even if its
  numerical conversion still needs a specific repair.

Only a result that makes group attribution decision-relevant calls for an
additional matched operation comparison, such as equally learned finite cyclic
relations or an unconstrained categorical relation model with an explicit work
budget. A cyclic group changes algebra; it is not equivalent to shuffling 2I
labels. Match encoder access, code cardinality, parameters, calibration and
tuning opportunity, and report remaining mismatches. No initial cross-product
of every codebook, loss, phase count, seed and recurrence is proposed.

Use open development to learn, inspect unexpected generations and revise the
mechanism deliberately. Keep the existing revealed regression populations and
failed outputs. A separately frozen final holdout follows design selection;
neither a new tiny authored puzzle nor this module's tests qualify prose, chat
or reasoning. Focused code checks cover actual algebra orientation, causality,
gradient connectivity, artifact identity and train/serve semantics. They are
implementation safeguards, not an ever-growing scientific acceptance funnel.

## Integration with the other tracks

The branch-scoped [R2 design](https://github.com/UOR-Foundation/uor-r4/blob/50ac2527f95508de114080d36e76826d7781a3b5/docs/integration/chat-r2-chunked-scan-design-2026-09-25.md)
correctly states that the retained state-dependent gate/transport/read cell is
not exactly a finite-state associative scan. Its proposed input-only affine
recurrence is a new model. Its local W32/64 reader plus optional blended linear
memory also changes the available exact source/copy path. A 256-token training
window alone cannot establish equality with full256 occurrence access.

Expose a read-kernel interface over causal query/key/value/event views so the
same geometry module can eventually consume either retained recurrent features
or the scan cell's features. Initially use the retained full256 features and
admission; later integration must name the new recurrence and source-access
contract. Neither lab should edit the other's dirty `joint_model.rs` or
`joint_parallel.rs` concurrently. New module ownership and a small reviewed
adapter are preferable to an uncoordinated merge of broad refactors.

The dialogue lab can supply licensed data, response masks and its established
development evaluator. Its branch results must be read at their actual scope
before adoption; improved training throughput alone does not establish a better
model. The session lab can supply persistence, token-boundary and loaded-bundle
interfaces. The emission lab can identify whether source-sensitive failures
occur before ranking, at vocabulary/copy mixture, or on a generated trajectory.
These findings should change the shared roadmap, not become three disconnected
definitions of success.

## Companion repositories and primary research

The reusable [import audit](architecture-2026-09/imports.md) already identifies
the strongest companions. This review reused its dated source/pin findings; it
did not independently fetch or build every donor. Before code adoption, refresh
the exact relevant file and license. No whole companion repository is promoted
to a working language model.

| Donor | Useful transfer | Boundary |
|---|---|---|
| UOR-ADDR / Framework | Typed immutable artifact, geometry and operator identity; exact page provenance | Canonical digest order is not semantic relevance. Ontology names are not learned numerical mechanisms. |
| Existing R4 2I/H4 and exact arithmetic | Signed finite action tables and declared coordinate conventions | Reuse this implementation before importing another group package; preserve exact-label versus approximate-coordinate claims. |
| W33 | Persistent path-copy pages and finite ordered operator examples | The existing mod9 model mapping is a retained negative. Its finite incidence geometry needs an explicit task encoding; it does not directly replace R4 state. |
| NEMESIS | Encoding/decoding/transition-fidelity questions | Prior audit found research documents and missing executable/language bridges; avoid unlicensed copying or adopting unproved constant-cost claims. |
| GoldSnnail / SpiralCore | Selected state-layout and finite-operator ideas | Reinspect the particular call chain; repository breadth is not model readiness. |

The original [quaternion NLP work](https://aclanthology.org/P19-1145/) is useful
for structured parameter sharing, but uses Hamilton tensor transformations and
does not satisfy this project's serving boundary unchanged. The
[Gated DeltaNet paper](https://arxiv.org/abs/2412.06464) separates adaptive
forgetting from targeted associative updates; this motivates distinguishing
retention, addressing and overwrite rules rather than treating every failure
as an attention-score defect. Its floating fast-weight mechanisms are research
comparators, not adopted runtime dependencies.

[Test-Time Training](https://arxiv.org/abs/2407.04620) makes the recurrent state
an adaptive model and therefore broadens the memory-capacity question. It does
not justify new runtime training or matrix operations here. Preserve the idea
for a specific future evidence gap instead of adding another engine now.
[T-MAC](https://arxiv.org/abs/2407.00088) supports lookup-based low-bit
execution, and [MatMul-free language modeling](https://arxiv.org/abs/2406.02528)
supports the feasibility of additive low-bit sequence models at their measured
scales. Neither gives geometric predictive advantage or terminal sparse
parameter access. Their quality and hardware conditions do not transfer to
UOR-R4 without measurement.

## Cost, outcomes and next concrete work

For 16 lanes, a padded 120-by-128 one-byte product table plus a 120-byte inverse
row is 15,480 bytes; packed 16-by-120 four-bit energy tables are another 960 bytes,
plus their lane scales.
These are derived table sizes, not measured model memory. Key/root coding,
gains if introduced, masks, all value work, output maps and provenance are
additional. Dynamic root search and a large autodiff graph may cost more than
the saved query-key products. Full-window reading remains linear in prior
events per token and quadratic across an unroll. This first implementation does
not satisfy terminal D5 sparsity by itself.

Before compiling or training, the principal must reserve the build/model slot
with the other labs and project the complete preparation, build, learner/control,
export, generation and delivery cycle against the live cumulative ledger and
physical storage. The standing authorization allows necessary local extensions
when prospectively recorded; it does not authorize paid compute or deletion.
No historical balance in this document is a current allowance.

The next concrete deliverable is the optional geometric read module, explicit
surrogate and finite export format with a small adapter to the common graph.
It should be compiled and exercised on an actual short loaded continuation as
soon as implementation allows, then evaluated at one prospectively declared
meaningful development exposure. The first work card must state its complete
cost and the outcomes below. It must not promise that a source-only module has
already improved language.

| Observed outcome | Programme decision |
|---|---|
| Useful full-context generated improvement retained through integer execution | Integrate with the shared dialogue/session path; assess real whole-token cost and only then select the next sparsity responsibility. |
| Equivalent language with materially reduced complete execution cost | Retain as a serving improvement; do not require unique semantic superiority to value useful geometry. |
| Continuous improvement with a localized hard/export loss | Preserve both artifacts and repair that identified numerical/representation boundary once under a concrete change; do not restart broad selector tuning. |
| Lower NLL with no meaningful output improvement | Use the emission/dialogue findings to choose an objective, data, capacity or state change; no automatic exposure tranche. |
| Finite coding destroys distinctions before learned scoring helps | Preserve the negative; decide on a justified richer code/gain representation or a different operator. Failure of this code does not reject all geometry. |
| No useful behavior or cost benefit under the declared comparison | Park this operator and advance the most informative independent programme dependency. Do not launch a grid search merely because remaining time exists. |

No build, model run, external paid compute, artifact mutation or changes to
other labs' files occurred in this research review. The blueprint is a proposed
native path toward geometric attention, useful inference, prose, chat and
reasoning; all empirical outcomes remain **NOT_RUN**.
