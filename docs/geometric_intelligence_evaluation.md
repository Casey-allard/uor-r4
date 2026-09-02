# Geometric Intelligence Evaluation Policy

- **Status:** Normative experiment policy; mechanisms remain dormant unless a
  named product decision activates the smallest relevant probe.
- **Applies to:** source-free lexical prediction, geometric recall, geometric
  attention, inference, correctness, bounded reasoning, and provider-free
  serving.
- **Architecture:** [ADR-0004](adr/0004-geometric-intelligence-route-hierarchy.md)
- **Latest terminal mechanism:**
  [#973 predictive block-delta result](r4_predictive_block_delta_binding_prompt_capacity_973.md)
- **Historical mechanism family:**
  [ADR-0005](adr/0005-predictive-geometric-connection-memory.md)
- **Vocabulary:** [Formal Vocabulary](formal_vocabulary.md) and the
  [Glossary](transformerless/GLOSSARY.md)

## Outcome

Evaluation exists to answer one decision at a time. The repository does not
accumulate indiscriminate gates, run every available test because it exists, or
make an hours-long run the price of learning whether a mechanism is reachable.
Experimental evaluations are dormant by default. Activate only the smallest
probe whose possible outcomes cause different next actions.

### Active decision — predictive block-delta terminal/#973

The independently frozen `R4PredictiveBlockDeltaBindingV1` write/binding-law
campaign completed `PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY`. On 512 sealed V5
prompt directions, the geometric arm produced gain
`0.03896945868086732`, `375/512` directional wins, and own-prompt NLL
`3.5419674206289073`. It passed the V1/pooled incremental, wins, NLL,
fresh-language, state-load, and integrity gates, but missed the frozen absolute
gain floor `0.04332169878499658`. The binding action is therefore
`STOP_WITHOUT_GENERATION`.

The geometry comparison was mixed but did not qualify. Geometric minus the
independently fitted plain arm was `0.023929811749894725`, below
`0.025341569256760274`, and geometric own-prompt NLL was worse. Geometric
minus transport-permuted was `0.03181032686529761` with `310/512` paired
improvements and passed its gates. Since both comparisons were required,
geometry attribution is negative. Delta overwrite was also not attributed:
geometric minus independently fitted additive was
`-0.006512463228773413`, with `234/512` paired improvements.

Fresh geometric NLL/top-1 was `3.84055165318221` / `30.979348%`; fresh and
all integrity gates passed. A scoring-only recovery repaired a variable-tail
work-audit assertion without creating an optimizer or refitting an arm. Result
CID:
`blake3:6c67544d675eafcb8eb9c0dabb93617e3f6c3295af812e8acbb687107c010a74`;
scoring CID:
`blake3:44f8941d24a99fc230710fd700e7a7b13cee87587bfbe4e13bf7b095222e2ee6`;
recovery CID:
`blake3:7b76e36e44798bebf184ece08fdd8a2065bdd370106b5d64d5fae4c59dc6d88b`;
exact-replay verification CID:
`blake3:567cf336eb05c3ec562aef7135f6fb35b580d02c758b0e79f2508cae57065f5d`.
See the [predictive block-delta record](r4_predictive_block_delta_binding_prompt_capacity_973.md).

This rejects only this predictive block-delta law. Preserve qualified V1 and
the established ordinary-softmax attention evidence. Generation, coherent
text, reasoning, exact/geometry-native lowering, #954, C1-SB6, browser
promotion, and release remain blocked or `NOT_RUN`.

### Prior #973 evidence retained

PR #997 proved reachability but rejected its componentwise-Frechet placement:
real 2,931/35,028 (8.367592%), unchanged #953 4,281/35,028 (12.221651%),
order-shuffled 2,934/35,028, and operator-permuted 2,966/35,028. The first
bounded `GeometricGatedDeltaRetentionR4V1` core then passed structural checks
but did not beat plain delta on its sealed synthetic construction fixture:
16/28 versus 23/28 next-token and 55/112 versus 98/112 association wins. This
is `NO_ADVANTAGE_ON_THIS_FIXTURE`; held-out corpus, autonomous decode, and exact
lowering remain `NOT_RUN`.

The literal one-head `DirectCausalGeometricAttentionR4V1` scaffold now exists.
Its V2 result is `NON_PROMOTABLE_BUDGET_MISMATCH`; V3 returned full H4 3/12
against matched plain 12/12. `ConnectionGaugeCovarianceV4` then passed its
construction/frame/gradient gate but failed held-out functional binding: all
three main arms scored 13/24 and order/value/gauge destructive controls retained
nearly all performance. V4 is frozen negative evidence.

The positive reference is `HELM-D-R4`; this establishes preliminary ordinary
softmax attention on a bounded full decoder in coherent R4/Spin frames. The
reference pins and audits the official MIT HELM-D
source at `7501deca8f413848bfef804be64ce874b72a3cd7` as the architectural
reference. The source-faithful HELM-D implementation uses its declared
Lorentz inner-product distance surrogate
`2c + 2c<q,k>_L`, learned scale, ordinary causal softmax, and normalized Lorentz
centroid; it is not described as computing `arcosh` geodesic distance squared.
Then freeze an ordinary full-decoder donor and build one gauge-equivalent
R4/Spin arm: retain all learned Q/K/V and `W_o`, ordinary compatibility, stable
softmax, and linear value aggregation; split heads into R4 blocks; encode exact cumulative Spin/H4 local
frames; transport every causal K/V block into the query frame; map the result
back before unchanged `W_o`.

Freeze a bounded real causal-language train/validation contract before outcomes.
The ordinary donor and R4-frame arm hold source weights, learned parameters,
updates, causal support, decoding, and raw parameter budget fixed. Report
transport overhead separately. The first gate requires ordinary-donor/reference
parity, zero future reads, deterministic replay, and numerical plus behavioral
parity on next-token loss, top-1, and exact decoded output. Its equal-work
source-frame-permuted intervention must break parity so a bypassed transport
seam cannot pass. Failure of donor parity or failure to retain language behavior
is the falsifier and stops before intrinsic R4 training. A parity positive is
not geometric advantage.

That first gate is now positive. Pinned source provenance, ordinary-donor
replay, transported-R4 numerical/behavioral parity, the destructive control,
and the causal ledger passed on the bounded held-out run. See the
[`HELM-D-R4 record`](helm_d_r4_softmax_decoder_973.md) and
[`machine result`](helm_d_r4_softmax_decoder_result_973.json). Upstream HELM-D
checkpoint parity remains `NOT_RUN`; no HELM checkpoint or code executed in the
UOR generation or bridge gates, and no upstream result is inherited.
`IntrinsicLorentzR4AttentionV1` attempt 02
reached construction validation and stopped unavailable before D3 because its
barycenter-covariance audit missed the frozen ceiling. Its diagnostic NLL also
trailed donor and flat R4. Intrinsic attention is therefore not established;
learned-manifold V2 then failed retention and matched parity. The 8/8-contract
score/readout attempt stopped at its two-document preflight and rejected
tangent readout (pooled normalized audit-MSE ratio `1.0643688804269025`). Ordinary dot-product/stable-
softmax causal attention in coherent R4/Spin frames is now the accepted
baseline. Intrinsic score/readout alternatives, multi-resonance replacement,
whole-decoder recurrent lowering, and D3 are parked or `NOT_RUN`.

The provider-free autonomous `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`)
native CLI gate now passes with HELM-D credited only as the MIT architectural
reference and UOR's pinned SmolLM2 `HuggingFaceLlamaOracle` for
embeddings, RoPE, residual/RMSNorm, MLP, final normalization, and the
language-model head. It produced 4/5 frozen quality in both passes, 5/5 exact
replay after deleting timing, all 30 layers with exact causal/projection/R4
audits and zero future reads, and reproduced the source donor for P1 through
EOS and P2-P5 for all 32 retained tokens. Its terminal is
`PASS_R4_SOFTMAX_REFERENCE_GENERATION_ADVANCE_NATIVE_PRODUCT_BRIDGE`. This reference remains
transformer-compatible and `f32`/multiply/alloc/source-weight backed—not
table-native, multiply-free, or transformerless. One dedicated opt-in,
loopback-only native HTTP endpoint for the exact same policy now also passes:
its frozen eight-token prompt matches the CLI token sequence, decision CID, and
state CID, all 30 layers retain exact audits, and future reads remain zero.
Dashboard wiring/static native-readiness and WASM-isolation checks pass, but the
hosted Pages deployment is static, currently reports WASM offline, and has no
functioning chat backend or compiled-artifact lowering; the default engine is
unchanged. See the
[`bridge record`](r4_softmax_reference_http_bridge_973.md) and
[`structured result`](r4_softmax_reference_http_bridge_result_973.json).

`R4SoftmaxTeacherTraceV1` and `R4SoftmaxTraceStudentV1` have now completed that
first trace/compiler gate. The source-free Q16 suffix artifact shows bounded
distillation against its count and document-permuted controls, but its
autonomous continuation loops. This result is not geometric attention,
coherent generation, correctness, general-purpose inference, or reasoning. See the
[`trace-student record`](r4_softmax_trace_student_973.md).

`R4SoftmaxTraceStateStudentV1` then completed its three sealed phases and
stopped
`STOP_R4_SOFTMAX_TRACE_STATE_STUDENT_REPAIR_OR_RETIRE_REPRESENTATION`. Every
arm covered the same `422,875` Q16 teacher mass on the same nine positions:

| Arm | Covered CE (nats) | Teacher top-1 | Actual-next top-1 |
|---|---:|---:|---:|
| Frozen suffix | 2.660721032 | 3/9 | 2/9 |
| Plain recurrent | 2.660770919 | 3/9 | 2/9 |
| Geometric recurrent | 2.660705367 | 3/9 | 2/9 |
| Transport-permuted | 2.660729215 | 3/9 | 2/9 |

The geometric-to-permuted CE margin was only `0.000023848` nats against the
frozen `0.10` materiality requirement. No teacher or actual-next top-1 decision
changed, and every arm retained the same period-two `, Scotland` continuation.
Exact artifact reload, causal replay, zero-source execution, input provenance,
and matched work ledgers passed. This is a valid negative for the frozen 4D
signed-reduction/token-derived state representation, not a failure of execution
integrity and not a falsification of ordinary R4/Spin softmax attention. The
state artifact is
`blake3:b617fc38e7bef1cdea76991f6e5e7cc653118451d63bcbd595f8ffd7e247ae7b`;
the construction freeze is
`blake3:67cf67bb46b94cf5644b8dde286e89adb7e49159b3749790dffb500d8047fedb`;
the source-free seal is
`blake3:64587526f7883ab046e884a28b6af7e9e89818c9ead2039f8c995de7fb483060`;
and the result is
`blake3:dc04a8a8b21750799db2d451c8237d1e62cf90ffa74561fb54272b1e9704c824`.
The complete evidence is recorded in the
[`state-student record`](r4_softmax_trace_state_student_1011.md) and
[`structured result`](r4_softmax_trace_state_student_1011_raw.json).

The next gate at #1011 close was
[#1012](https://github.com/UOR-Foundation/uor-r4/issues/1012), one
construction-only, leave-one-document-out observability audit measuring the
same teacher-relative candidate loss at four
explicit boundaries: the full ordered final-layer Q/K/V trace blocks; the fixed
576-to-4 signed reduction; the token-derived role maps and recurrent state
features; and the fitted residual readout/logit scale. The result selects only
the first failed boundary: replace the reduction if full traces transfer but
the reduction does not; repair context-conditioned role induction if the
reduction transfers but state features do not; repair readout calibration if
state features transfer but logits remain inert; or stop trace distillation and
train under a fresh independently frozen holdout if even full traces fail.
Document 13 is already revealed and cannot promote a repaired mechanism again.

#1012 subsequently completed at `INSUFFICIENT_SUPPORT_COVERAGE`: aggregate
primary coverage was `0.6202622204224402`, but the minimum fold was
`0.3469116829611222`, below the frozen 50% floor, so no boundary attribution is
licensed. On the covered rows the full Q/K/V probe returned CE
`2.215410922655504` versus suffix `2.215064603216862`, with the required
improvement direction in `0/4`; the fixed label control separated by
`1.3807454322642605` nats in `4/4`. Exact replay and zero
source/document-13 reads passed. Support will not be expanded and another
localization ladder will not run.
[#1014](https://github.com/UOR-Foundation/uor-r4/issues/1014) subsequently
completed the direct end-to-end evaluation. Enabled sealed-test NLL was
`2.127407277216677`; zeroing every attention output after `W_o` and before the
residual raised NLL to `4.804799838144271`. The
`2.6773925609275944`-nat intervention passes the `0.10` attention rule, and
two-arm Rust parity plus exact six-layer causal/R4 audits pass. Ordinary causal
attention is established as load-bearing at this learned R4/Spin scope. The
full quality DoD fails because enabled NLL exceeds `1.50` and subject/scene
retention is `3/5` rather than `4/5`; seeded replay is exact `5/5`. Close this
campaign without rerun or tuning. [#1017](https://github.com/UOR-Foundation/uor-r4/issues/1017)
then completed its one separately frozen exposure continuation at
`149,995,520` cumulative tokens. Enabled Rust parity, all mechanical gates,
subject-or-scene retention `5/5`, and normalized replay `5/5` passed; fresh
sealed NLL `1.5727521962806827` failed the strict `<1.50` gate. Its overall
verdict is negative solely on NLL. The next evaluation is now specified under
[#1019](https://github.com/UOR-Foundation/uor-r4/issues/1019): one frozen
12-layer, 13,130,784-parameter campaign with seed 1019, 16,800 steps, and
275,251,200 tokens over the same mechanism. Exact population, 400-step
fixed-sequence overfit, and random-export all-twelve-layer Rust preflight parity passed.
The signed MPS gate stopped `UNAVAILABLE_HARDWARE_BUDGET` on time: its
`20.66 h` safety projection exceeded the `8 h` ceiling, while memory passed at
`21.03%`. That terminal applies only to the frozen offline PyTorch/MPS
implementation. Full training, final parity, reveal, generation, and replay
remain `NOT_RUN`. A single isolated exact-shape MPS fast-path test (10 warmup
plus 40 measured steps) combined fused AdamW with deferred logging and measured
`4.485223 s/step`, slower than the signed `3.491307 s/step`; `fused=True` was
removed immediately. This is a bounded fast-path negative, not a model result.
#1019 tuning/full-run work stops and remains optional/paused. At that
checkpoint, the active next step was the working #1017 `r4 generate` path;
#1041 later bounded it to raw single-turn story continuation after `2/3`
narrative and `0/2` supplied-history results (see the
[#1041 record](r4_softmax_local_normal_use_1041.md)). UOR's deployed architecture/runtime
remains CPU-native; Apple Accelerate/BLAS and MPS are local offline accelerators
only; CUDA and external GPU execution are out of scope. The MPS stop
is not a model-quality negative, leaves the full-scale capacity hypothesis
untested, and does not revoke the established attention result. This is not
another attention diagnostic or more 7.15M exposure/LR tuning. See the
[#1012 record](r4_softmax_trace_observability_1012.md),
[#1014 record](r4_softmax_end_to_end_attention_1014.md), and
[#1017 record](r4_softmax_quality_capacity_continuation_1017.md), and the
[#1019 frozen contract](r4_softmax_parameter_capacity_1019.md) plus its
[observed preflight](r4_softmax_parameter_capacity_preflight_1019_raw.json).

Offline floats, matrix operations, and softmax remain allowed at the
teacher/compiler boundary; the deployed destination remains exact,
integer/table-native, and source-free. Intrinsic/readout substitution,
resonance, new state dimensions, corpus scale, softmax replacement, exact
lowering, tag/release, hosted promotion, general-generation/reasoning, and
static-WASM evidence remain ineligible. None of this alone closes #973 or
unblocks #954. See the
[`generation record`](r4_softmax_reference_generation_973.md) and
[`compact aggregate`](r4_softmax_reference_generation_attempt_01_result_973.json).

### Established product decision — B0/#989

#989 returned `ESTABLISH_TABLE_NATIVE_LEXICAL_BASELINE`. The deterministic
construction-trained table scored 99,362/446,342 (22.261404%) held-out top-1
versus 24,163/446,342 (5.413561%) for unigram, a +16.847843 percentage-point
uplift. Trigram, bigram, and unigram selection counts were 319,336, 108,738,
and 18,268 respectively. The fixed prompt emitted 16 valid UTF-8 units, stopped
at the cap, and did not enter a period-1/2 cycle. Two complete executions
emitted identical reports and artifacts. The binding record is
[#989](source_free_table_baseline_989.md).

This is an established statistical lexical prediction and bounded-decoding
baseline, not semantics, attention, geometry, correctness, reasoning, chat,
performance, or release evidence. Its exact artifact, corpus, support, decode,
and work were frozen as the non-geometric reference for the one later accepted
#953 intervention. Unrelated geometric, teacher, broad-QA, and release probes
remain dormant.

Lexical ingestion, canonical serialization, registered-address membership,
and rebuild witnesses are prerequisite plumbing, not inference. The delivery
sequence is fixed: A1R/#967 repaired ordered state but retained it as state only;
A1P/#970 produced the local paired-H4/R4-heatmap identifiability result
`RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q` before scalar design because
eight retained heatmap classes aliased outcomes and construction transfer was
0/6. This is a readout-identifiability negative only; paired H4 and its exact R4
heatmap remain structural/control state. #970 closed through protected PR #972.
#969's mechanism-first pivot delivered one causal R4/S3 least-cost
route-attention mechanism and one matched two-unit decoded smoke at
`PROCEED_TO_I1_WITH_CAUSAL_R4_PATH_ATTENTION`. #953 implemented a bounded
decoded-loop path, but its first smoke was an exact rank-preserving lexical
relabel of #969 and terminated `REVISE_I1_GENERATOR_IN_PLACE`. Its frozen
natural agreement follow-up first stopped at support admission under the old
flat union. The versioned tiered repair then produced exact `{still}` and
`{run,runs}` support with matched work and preserved #969's empty-primary
fallback. The single permitted four-arm run still selected `still run` for both
full-path prompt orders; state-disabled selected `still runs` for both. #953
remains at the historical terminal `REVISE_I1_GENERATOR_IN_PLACE`, but its
labels, geometry, and failed representations are now known. It is therefore a
parked, unassigned integration regression, not an independent discovery
population. #983 then tested `ConstructionCausalReturnV1` on a separate
three-family, six-decision population. Its usable construction classes were
pure but reached 0/6 held-out decisions; the sealed strict ceiling was also
0/6. It stopped `UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER` before a deployed
selector or payload inversion and is now closed bounded negative evidence.
#986 then closed `UNAVAILABLE_FRAME_OR_POPULATION` before placement because its
exact population/codec commitment and complete lexical SpiralCore frame were
unavailable. Gate 0, labels, selection, and #953 were `NOT_RUN`. The later
established B0/#989 result exposed one matched #953 intervention, which has
since been accepted. #973 then retained its bounded Gate 0, paragraph, and
conversation mechanisms. Its first bounded-global exact-spin relation failed
target-free because the frozen swapped states commute. The independently
frozen V2 repair then passed its bounded noncommuting decoded contract. PR #997
then rejected the first natural document placement. Its first bounded
gated-delta core later trailed plain delta. #973 continues to block #954 while
it owns the accepted ordinary-attention reference, the qualified provider-free
autonomous generation gate, the qualified dedicated opt-in loopback-only native
HTTP endpoint, and checked dashboard wiring/static native-readiness and WASM
isolation. Hosted Pages remains static/offline without a functioning chat
backend/artifact lowering. The source-free Q16 suffix trace student is complete
with bounded distillation but looping autonomous output;
`R4SoftmaxTraceStateStudentV1` is complete with `FAIL_PROMOTION`. The
construction-only leave-one-document-out observability audit completed at
`INSUFFICIENT_SUPPORT_COVERAGE` without boundary attribution. Direct end-to-end
causal-softmax attention training in R4 coordinates on a fresh untouched split,
including autonomous decoded generation, is active; support expansion and
another localization ladder are not.
Intrinsic/readout, new state dimensions, corpus scale, resonance/recurrent
lowering, and final requalification are parked.

A negative result remains evidence about its declared mechanism and
distribution. It does not erase a separately established storage, identity,
recall, or efficiency result. Conversely, a useful substrate does not imply
attention, inference, correctness, reasoning, or product readiness.

## 1. Evidence ladder and delivery order

The current ladder starts with the established B0/#989 reference. The older
mechanism stages below remain as historical provenance and future
dependencies; they are not a competing queue.

In this policy, the **accepted local semantic selector lineage** can resume only
through the one permitted #953 intervention matched to the established #989
reference. #983 and #986 remain in evidence provenance; neither a failed
experiment nor a conditional issue number is automatically a serving consumer.

0. **Source-free table baseline (B0/#989; established)** — construction-only
   integer lexical counts drove 22.261404% held-out top-1 versus 5.413561%
   unigram, exact decode, a bounded 16-unit continuation, and byte-identical
   replay. Geometry was absent. The artifact is the fixed reference used by the
   one accepted matched #953 intervention; later stages remain sequenced.

1. **Local route-attention mechanism (#969)** — natural schema-2 adjacency,
   causal ordered R4/S3 path state, retained prefix memory, and deterministic
   exact path closure change one decoded choice under matched natural support
   and group-comparison budgets.
2. **Construction-transfer gate (#983)** — one independently frozen
   candidate-hypothetical causal-return mechanism had to transfer pure
   construction-derived classes to six held-out natural choices under equal
   support/work and causal derangements; the observed result was 0/6.
3. **Semantic-placement and signed-transport gate (#986)** — one CID-disjoint
   source-free corpus placement must transfer before one candidate-relative
   signed zero-sum exact transport arm may compete with a matched table-native
   semantic-value comparator and causal controls.
4. **Inference/generation (#953)** — the real decoder drives the accepted local
   semantic selector through one bounded provider-free autoregressive lexical
   loop on the evaluated library/CLI path.
5. **Higher-scope attention (#973)** — paragraph, conversation, and bounded
   global state change selection through the accepted #953 decoded loop.
6. **Correctness/abstention (#954)** — the accepted local semantic selector,
   #953, and #973 path satisfies an independent oracle or constraint while its
   #969/#983/#986 provenance remains bound and typed abstentions are explicit.
7. **Reasoning (#955)** — every step invokes the accepted selector/
   #953/#973/#954 consumer; novel multi-step tasks expose typed intermediate transitions,
   constraint preservation, alternative/counterfactual comparison, and a
   checkable result. #952 is not the reasoning consumer.

Within #973, after the accepted local semantic selector and #953 qualify, and
every required #973 scope qualifies or an explicit native revision changes
that scope, run the separately frozen
higher-scope/corpus-scale offline induction ladder and requalify its final
artifact. The ladder is part of #973's terminal before #954; it is not a new
serving-time corpus reader or a shortcut around the issue order. More rows,
exact hits, or trace activity are capacity/recall results unless a
candidate-relative effect transfers to the held-out anti-recall partition under
matched controls.

#962 later integrates the accepted selector/#953/#973 path into product chat
and memory. #964 binds evidence provenance #970 → #969 → #983 → #986, plus
the actual accepted selector → #953 → #973 serving path; #965 qualifies only
that exact release path.

Passing an earlier stage never promotes a later claim. Codec coverage is not
attention; exact recall is not geometric attention; partial local/sentence
attention is not a completed recursive hierarchy; attention is not coherent
inference; fluent inference is not correctness; and correctness on recalled
items is not reasoning.

### Ancestor evidence that must be preserved

The prior evidence record
[`prime_router_geometric_context_evidence.md`](prime_router_geometric_context_evidence.md)
is an empirical prior, not a current pass. It reports coordinate-tracking
accuracy falling from `1.0000` to `0.3027` when the initial trajectory was
masked and to `0.2612` under a last-state-only ablation. Its delayed-trainable
probe reports the signal becoming readable at the transported final state. The
same ancestor route used a session hypersphere vector, winding/window state,
projection energy, shared-prime factors, cosine resonance, and accumulated Hopf
phase.

Therefore the local prototype cannot replace the transported path with only the
last coordinate and still claim continuity with the original mechanism. The
full ordered path must remain available. The other historical channels are
hypotheses, not mandatory preflight work. The ancestor used Ollama for
language generation and did not report held-out next-token quality, so it does
not establish current inference, correctness, or reasoning.

## 2. Dormant-by-default rule

Experimental test harnesses, corpus evaluations, teacher comparisons, and
release sweeps MUST be dormant by default. They run only when an experiment
card names:

- the exact decision the result can change;
- the smallest population that can falsify the mechanism;
- the artifacts and source partitions involved;
- positive, negative, unavailable, and stop actions; and
- the maximum time, disk, memory, and worker budget.

Fast structural unit tests may protect an already selected representation.
Their existence does not authorize every empirical suite, every corpus, or an
exhaustive workspace run. Adding a required merge gate is a separate maintainer
decision made only after the behavior is product-reachable and the gate has a
stable denominator and acceptable cost.

## 3. Minimum decision-bearing probe

Every experiment begins with the cheapest probe that can distinguish its
outcomes. The order is:

1. derive a reachability ceiling from evidence already available;
2. inspect one bounded trace or fixture that exercises the intended branch;
3. run a tiny deterministic matched-control sample;
4. expand only if the preceding result leaves the product decision unresolved.

An experiment has no decision value when its positive and negative outcomes
lead to the same action. Do not run it. Redesign the question or proceed with
the already-supported action, while recording the skipped work as `NOT_RUN`.

Thresholds are instruments, not project goals. If a frozen threshold measures
the wrong mechanism, preserve the observed verdict and repair the next
experiment contract; do not reinterpret a miss as a pass, and do not let an
irrelevant metric erase a load-bearing result established by a more direct
intervention.

### A1R/#967 frozen scope result

The frozen #952 histories intentionally match current route, previous route,
last-two suffix, length/multiset/boundary shape, and the immutable `gg` global
project snapshot. A1R therefore expects current, previous, last-two, and global
state to be equal, while sentence, paragraph, and conversation ordered state
must differ. Equality at the control scopes is a fixture requirement, not a
failure of the repair.

Global behavior is a separate intervention. A claim about global ordered state
requires a construction-independent global-snapshot permutation fixture in
which only global order differs while current-through-conversation scopes,
candidate support, payloads, and work budgets remain matched. Evaluation may
not mutate global state from the conversation merely to manufacture a
difference. The #967 report satisfied this scope mask, the independent-global
intervention—including distinct content-derived global epochs with matched
lower inputs, rows, support, budgets, and denominator—exact group/fold laws,
incremental reproduction, and candidate support invariants. Its report kappa is
`blake3:f0db7a5d5c81d51ebf3b4bf8a2715c4960ec16b14161e8bf7598d7b98c48c881`.

The full arm produced distinct `ll`/`rr` relative states on all 6 queries and
changed the same-candidate relative state in 5/6 paired comparisons. The scalar
shortest-Cayley-distance readout nevertheless mapped both candidates to energy
2 and tied on all 6/6 queries. The terminal verdict is `RETAIN_STATE_ONLY`:
ordered state is retained, but attention, generation, correctness, and reasoning
remain unestablished. The corrected local A1P/#970 probe subsequently found
eight incompatible exact paired-H4-derived R4-heatmap classes and stopped before
readout or placement. This narrows only the frozen heatmap readout; it does not
reject paired H4 or the other declared channels. #970 is closed through
protected PR #972; #969's later mechanism-first result is recorded below.

The legacy additive state was bound and remained equal, but its ranking arm is
`NOT_EXERCISED`: no additive candidate scorer was predeclared. The exact-recall
arm exercised six `NO_EXACT_HIT` abstentions rather than six ties. Canonical
address order reported `rr` as a diagnostic tie-break token for the full arm;
it never converted a tie into a selection. The verdict does not depend on the
unavailable additive arm: the report binds the 6/6 full tie and
`any_exercised_control_not_weaker = true` separately.

### A1P/#970 corrected construction/validation contract

#970 is an identifiability decision before it is a metric decision. The six
visible A1R labels remain an unchanged regression/root-cause fixture. Passing
those six labels with a replacement scalar can never establish semantic value,
and they cannot train, orient, or select the new readout. Their #952 partition
kappa remains
`blake3:d008b82eda9b16b102cf4c7ffa4a47a40ad514b30f0763ed3f46c0ebae3e277b`;
their historical result remains bound by #967 report kappa
`blake3:f0db7a5d5c81d51ebf3b4bf8a2715c4960ec16b14161e8bf7598d7b98c48c881`.
The old denominator remains three contrasts/six queries, and the frozen
shortest-Cayley outcome remains 0/6 selections, 6/6 ties, and 6/6 abstentions.

The independent pre-evidence fixtures are:

| Fixture | ID | Kappa | Histories / intended observations |
|---|---|---|---|
| Construction | `A1P-CONSTRUCTION-1` | `blake3:fb5f27fc1107f527d616f32affa8eba1746a2f60cfdb95ddbb21a0e493299652` | `aa bb cc dd qq -> ll`; `aa cc bb dd qq -> rr`; `bb aa cc dd qq -> rr`; `bb cc aa dd qq -> ll`; `cc aa bb dd qq -> ll`; `cc bb aa dd qq -> rr` |
| Sealed validation | `A1P-VALIDATION-1` | `blake3:ecbe8b404e7542d801ff4b4e66c91a41f90158d84efa484dc4edb53aff38b602` | `aa cc dd bb qq -> ll`; `aa dd cc bb qq -> rr`; `cc aa dd bb qq -> rr`; `cc dd aa bb qq -> ll`; `dd aa cc bb qq -> ll`; `dd cc aa bb qq -> rr` |

Construction, validation, and regression end in `dd qq`, `bb qq`, and `cc qq`,
respectively; all exact five-unit histories differ. The natural candidate set is
always `{ll, rr}`, with construction predecessors `P_ll = uu` and `P_rr = vv`.
Observation continuations are a separate kappa-bound ledger and are not compiled
into the schema-2 candidate manifest. Direct/exact or divisor continuation hits,
target injection, future events, or a candidate union other than natural
`{ll, rr}` invalidates the contract.

Construction observations determine the rule rather than attaching targets to
unrelated histories. The predeclared family is the two-element abelianization of
permutations on ordered roles `[aa, bb, cc, dd]`: construction may select only
the trivial map or the sign homomorphism `S4 -> C2`. The first construction
observation rejects the trivial map and fixes candidate orientation; the
remaining five must confirm `even -> ll` and `odd -> rr`. At validation time the
scorer receives only exact construction-derived state classes, never role
names, parity, token spelling or IDs, prime/address order, or labels. Validation
labels are public for the
identifiability ceiling; validation scorer outputs remain sealed until the
construction-to-readout compiler is frozen. No #969 sentence, paragraph,
conversation, or global fixture may be consumed, inspected, or exposed.

Parity is not trusted from the fixture literal: it is derived by inversion count
from each exact history and frozen role order `[aa,bb,cc,dd]`, then checked
against the serialized literal. Construction and validation geometry and
natural support are prepared through a target-free input with no observed-next
field; both populations are complete before their separate label ledgers are
joined for purity and ceiling calculations.

The corrected public contract retains the complete ordered witness

```text
X(H,c) = C(H,c)
Y(P_c,c) = C(P_c,c)
D(H,c) = X(H,c) * Y(P_c,c)^-1.
```

Writing `D=(q0+q1 i+q2 j+q3 k)/2`, with every coordinate in exact `Z[phi]`,
the scorer key is the signed `(1,i)` R4 heatmap
`(sin=q0/2, cos=q1/2, activation=q0^2/4, chirality, cosine polarity, chart
status)`. The full `D` witness retains `q2/q3`; neither operand is discarded or
made label-bearing. The exact endpoint mapping is `sin=+1,cos=0 -> 1`,
`sin=-1,cos=0 -> 1`, `sin=0,cos=+1 -> 0`, and `sin=0,cos=-1 -> 0`, with signed
orientation retained. Non-landmarks remain exact classes, while `q0=q1=0` is a
typed-null abstention. There is no float, fitted threshold, or Q1.30 shortcut.

The same contract carries the immutable fixed-zeta identity, ordered
multiplicity-preserving prime n-lets, golden radial maps
`(a,b)*phi=(b,a+b)` and `(a,b)*phi^-1=(b-a,a)`, and the typed
`Euclidean sqrt(2) <-> complex 2i <-> Riemannian [0,2]` adapter. These are
structural bindings, not scorer shortcuts. No zeta/n-let-to-`phi` shell-exponent
rule has been supplied or established.

Before choosing a scalar or opening validation scorer output, enumerate all
120×120 ordered `(X,Y)` pairs and the exact heatmap classes for every
inherited-regression, construction, and validation candidate decision. Keep all
three fixture denominators separate and keep the structural universe independent
of the 36 exercised decisions. Report:

1. construction class coverage and per-class binary candidate-outcome purity;
2. validation candidate-class coverage;
3. the no-class-splitting oracle ceiling,
   `sum_E max_y count_validation(E,y) / 12`;
4. the construction-transfer selection ceiling: validation queries on which the
   construction class map makes the intended candidate uniquely positive and
   the alternative uniquely negative, divided by six, with unseen classes
   abstaining; and
5. the same coverage, purity, aliasing, and ceilings for the additive comparator
   class `(A*(H), A*(c), A*(P_c))`, using the existing additive non-digest
   summary and excluding spelling, occurrence/position identifiers, lexical-unit
   ID, prime, address index, boundary/chain identity, kappa/digests, and
   provenance.

If any retained exact class requires incompatible outcomes, construction is
impure, or construction observations define no rule that transfers to
validation, stop immediately as
`RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q`. Do not search another metric,
add fields, enlarge H4, expand the corpus, or execute downstream selection arms.
The inherited/new union gate is binding even if another scalar could fit the six
visible regression labels.

Only an identifiable union authorizes exactly one deterministic bounded readout
compiled from construction observations using exact integer and finite-table
operations. Freeze its algorithm and serialized artifact before opening
validation outputs. Apply the same class-to-outcome compiler to the additive
comparator. Preserve candidate rows, support, admission, payload inversion,
source partition, row/candidate ceilings, causality, and work. Expose only an
API-neutral serializable `select_or_abstain` result and minimal trace. Report the
old-six regression, construction, and validation separately, and execute the
final authorized probe twice with byte-identical inputs, report, and kappa.

The required #970 arms are the full paired-H4-derived exact R4 heatmap,
current-only, additive with its compiled scorer, factor/count-only,
deterministic geometry permutation,
candidate relabeling, prime-assignment permutation, hierarchy-disabled, and
exact-recall-only. A placement arm exists only if identifiability proves
placement is the isolated defect; readout and placement may not both be tuned.
Candidate relabeling applies `rho = (ll rr)(uu vv)` to construction evidence,
predecessors, and validation labels before recompilation. Prime-assignment
permutation is `pi_p = (aa bb cc dd gg qq)(ll rr uu vv)` with
`leaf_pi(t) = leaf(pi_p(t))`; it changes only H4 leaf binding after the unchanged
natural candidate/support path. Geometry permutation conjugates construction and
validation by the existing A1R first-noncentral conjugator under root-table kappa
`blake3:8d33d62a239fb8001fea2bd14a9a5ec7321d0f07d81c74a5715eaeb3df53aa76`.
These are equivariance audits: the compiled result must follow transformed
construction evidence exactly.

Each fixture has six histories, twelve candidate decisions, seven row reads per
query / 42 total, candidate-entry ceiling 56/query, candidate ceiling eight, and
maximum two admitted candidates. The complete preflight is bounded to 18 fixed
histories and 36 candidate decisions plus one target-free exhaustive structural
census of 14,400 ordered pairs, its complete 120-root relative image, 45 exact
heatmap classes, and 480 typed-null pairs. Every authorized arm reports
selections, ties, abstentions, exact hits, support
equality, work, and explicit status. `NOT_EXERCISED`, `NOT_RUN`, `UNAVAILABLE`,
and `INVALID` are not passes. A hard-gate stop records downstream arms as
`NOT_RUN_IDENTIFIABILITY_HARD_STOP`; that is a valid bounded
readout-identifiability negative, not unavailable evidence or a claim that the
underlying structures have no value.

The terminal decisions are exact:

- `PROMOTE_H4_READOUT_CANDIDATE_TO_A1Q` admits only the paired-H4-derived exact
  R4-heatmap candidate term to #969 after strict 6/6 sealed validation, all
  required controls, strict
  superiority to every matched baseline, candidate/prime/geometry equivariance,
  equal support/admission/payload/causality/work, and deterministic selector and
  trace bytes. It establishes neither attention nor generation.
- `RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q` closes this exact heatmap
  readout search after aliasing, no transferable rule, a held-out
  miss/tie/abstention, an inert
  readout, or failure to beat an exercised comparator. Both H4 operands and the
  derived heatmap remain structural, provenance, diagnostic, or control state;
  protected delivery exposed #969 without approving another paired-heatmap
  scalar-readout chain. #969 was later pivoted to the single causal path
  mechanism above.
- `UNAVAILABLE_NO_INDEPENDENT_HOLDOUT` or `INVALID_CONTRACT` would have left
  #970 open and #969 blocked with the exact blocker published.

Both historical valid outcomes exposed only #969 after protected delivery.
The observed negative closed through PR #972; it did not itself expose #953,
which at that checkpoint remained blocked by #969.

### Observed #970 outcome

The corrected local preflight reached the valid negative without a scalar
search. Its structural universe is independent of the fixture decisions:

| Structural measure | Result |
|---|---:|
| Ordered paired-H4 witnesses | 14,400 / 14,400 |
| Exact relative `D=X*Y^-1` image | 120 / 120 |
| Pair multiplicity per relative row | 120 |
| Exact signed R4 heatmap classes | 45 |
| Typed-null ordered pairs | 480 |

| Measure | Paired-H4-derived exact R4 heatmap | Existing additive comparator |
|---|---:|---:|
| Exact classes across 36 decisions | 14 | 2 |
| Construction decisions | 12 / 12 pure | 12 / 12 across 2 impure classes |
| Validation coverage by construction | 10 / 12 | 12 / 12 |
| No-class-splitting ceiling | 10 / 12 | 6 / 12 |
| Strict construction-transfer ceiling | 0 / 6 | 0 / 6 |
| Incompatible retained classes | 8 | 2 |

No readout was compiled and every downstream arm is
`NOT_RUN_IDENTIFIABILITY_HARD_STOP`. The exact contract, complete pair-universe
census, and byte-identical double-run report kappas are respectively
`blake3:2daacf538c022fab9580d1e124af6c18d0b06da04604fbc962a01bda57f08a98`,
`blake3:dca725c0ec6060166bcd0023df956e1ff029661b5fa7800ccb9f20808712b796`,
and
`blake3:5f9239150dea8c0c27c4dfa6ad2e4d0068bc3d18afc127b315c0ec358ceddb3f`.
This is only a bounded readout-identifiability negative. #970 is closed through
protected PR #972 and the measurement remains append-only history.
See the [append-only A1P record](candidate_relative_identifiability_a1p_970.md).

### A1Q-L/#969 causal path mechanism and decoded smoke

#969 freezes exactly one mechanism before the smoke:

```text
A(i,j)  = natural schema-2 candidate adjacency
P(0)    = identity
P(k+1)  = P(k) composed with route(x_k)
M(t)    = retained causal prefix states before P(t)
Q_t(c)  = P(t) composed with route(c)
cost(c) = minimum exact (round-S3 closure shell, causal lease age) over M(t)
```

Exact candidate-cost ties abstain. H4 is the exact finite S3 codebook; the
paired-H4/E8 store is not the selector. Then run one bounded 2–8-unit decoded
smoke:

```text
bytes -> lexical routes -> admitted candidates -> frozen select/abstain
      -> payload decode -> incremental state update
```

The smoke has identical natural support and group-comparison budgets, exact
payload decode, incremental state reproduction, and deterministic replay. A
miss keeps #969 open for direct mechanism revision; it does not create another
gate issue. Paragraph, conversation, and global work belongs to #973 only after
an accepted local semantic selector and #953 qualify. The accepted #969 result
is identity-derived predecessor evidence for #983; it no longer directly
authorizes work on #953.

### A1Q-L2/#983 construction-transferred candidate-conditioned local attention

The predeclared #983 contract froze exactly one past-only mechanism before
implementation:

```text
P_0        = identity
P_i        = P_(i-1) * L(x_i)
S_i(H)     = P_i^-1 * P_t
R_i(H,c)   = ((S_i * L(c)) * S_i^-1) * L(c)^-1,  0 <= i < t
```

For every already-admitted candidate, `ConstructionCausalReturnV1` retains exact
signed-H4 relation equality, round-S3 shell, past-only lease age, relation
multiplicity, and exact witnesses. `R_min` selects one populated event by shell
ascending, multiplicity descending, then past-only lease age ascending. Only an
impure construction `R_min` class promotes to `R_full`, the complete eight-slot
occupancy-tagged ordered word. Unseen, impure, multiply mapped, malformed, or
ambiguous classes abstain. Compiler and query must bind the same codec,
manifest, address mapping, exact tables, policy, ceilings, class order, and
occupancy rules or return `UNAVAILABLE_FRAME_MISMATCH`. No scalar, threshold,
sign, shell order, placement, or width may be selected from validation outcomes.

The frozen population is exactly three natural candidate-pair families
(`is/are`, `has/have`, `was/were`), two construction transitions per candidate,
and one sealed matched validation pair per family, for six held-out decisions.
Each pair preserves token multiset, length, trailing-four suffix, exactly two
naturally admitted candidates, support, and work while an earlier causal
controller differs. Controlling validation lexemes are absent from their
construction family. Complete histories, shared suffixes, ordered witnesses,
and complete operative candidate-interaction prototypes do not cross the
partition. `run/runs`, all #953 surfaces and witnesses, the #970 population,
candidate injection, and actual future routes are forbidden.

Gate 0 is label-free with respect to sealed validation outcomes. It freezes
separate fixture/partition, codec/vocabulary, construction artifact, mechanism
policy, validation input, raw census, construction-label join,
validation-label join, and final outcome identities. Before selector execution
it reports same-frame reproduction, natural support equality, complete
`R_min`/`R_full` inventories, construction purity/promotion/coverage,
construction-to-validation coverage, operative raw-history/suffix/route/
representation overlap, populated/padded aliases, real/control ceilings, exact
work, and zero source/provider/teacher/future-route inputs. Freeze the raw-census
identity before the sealed validation-label join is loaded.

Hard-stop unless every usable class is construction-pure, all six validation
decisions have both candidate actions covered, the real strict ceiling is 6/6,
operative prototype recall and populated/padded aliases are zero, the real
ceiling strictly exceeds every causal derangement, and every arm preserves
support and declared work. Failure is one of
`UNAVAILABLE_FRAME_MISMATCH`, `UNAVAILABLE_NO_OPERATIVE_ANTI_RECALL`,
`UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER`, or
`REDESIGN_CANDIDATE_CONDITIONED_CAUSAL_RETURN_REPRESENTATION`; no selector,
second representation, or replacement population then runs.

Only a passing Gate 0 permits one frozen `SELECT`/`REJECT` run. A positive
requires 6/6 held-out choices, no real tie/abstention, every causal negative
strictly below real, coherent relabeling equivariance, exact incremental
reproduction, byte-identical rebuild/outcome replay, exact payload inversion,
source/provider absence, and no candidate injection or semantic tiebreak. Its
only positive terminal is
`PROCEED_TO_I1_WITH_CONSTRUCTION_TRANSFERRED_LOCAL_GEOMETRIC_ATTENTION`.
It establishes only bounded, source-free, construction-transferred local
geometric attention on this population. Decoded generation and all #953 work
are `NOT_RUN`; positive protected delivery authorizes a later session to apply
the algorithm unchanged to #953 after a new label-free preflight.

**Observed #983 result, 2026-08-28.** The frozen construction artifact formed
21 `R_min` and 24 `R_full` classes from 24 construction rows. Every usable
class was pure, but the real arm structurally covered 0/6 held-out decisions.
After the separately sealed validation-label join was attached, the offline
no-class-splitting lookup also reached 0/6, with six abstentions; every one of
the eleven causal controls and the count-only comparator was likewise 0/6.
The raw census and sealed outcome identities are respectively
`blake3:5e970efe79c13d38e02eab6ff60642d3d449ce9dc571af6425b16d0d94858017`
and
`blake3:58fba09dba1b9245cb62a73bf8e3ac153242dc0730e3df7586446aa2820d4587`.
The binding terminal is `UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER`. The hard
gate stopped before any deployed selector or payload inversion; #953
generation was `NOT_RUN`. This is a negative for this frozen representation
and placement, not a general rejection of H4, paired-H4/E8, trigonometric,
prime/semiprime, or alternative corpus-induced candidate-relative mechanisms.
See
[`construction_causal_return_attention_983.md`](construction_causal_return_attention_983.md).

### A1Q-L3/#986 corpus-induced harmonic signed transport qualification

`CorpusSignedTransportV1` is the only authorized successor. It first asks
whether construction-only corpus induction produces transferable semantic
placement/value, including a static self-plus-six harmonic link-state record
for every route, then whether exact ordered Cl(0,6)/SpiralCore transport adds a
candidate-relative signed zero-sum causal effect beyond that value. The local
link-state record and fixed candidate-rooted diffusion are construction
substrate, not attention by themselves.

Freeze by document/CID one induction partition, exactly 16 matched calibration
pairs (32 decisions), and exactly 32 matched sealed-test pairs (64 decisions).
Within each pair, candidates, length, multiset, last-two suffix, support, and
work match while a predeclared earlier-order intervention differs. A separate
split builder enumerates every eligible pair-CID tuple, applies the sealed
incompatible-naturally-admitted-route predicate, sorts by the frozen partition/
candidate-set/structure/decision-CID key, and greedily accepts a pair only when
neither decision CID has already appeared before taking the first 16 or 32.
It commits the pair IDs, intervention map, and sealed label join together. Pair
IDs and intervention metadata exist only in the audit/statistics harness;
placement, link/operator construction, diffusion, scoring, margin calibration,
abstention, and selection receive histories and natural candidates only.
Insufficient vertex-disjoint pairs are unavailable; the key and sample do not
change. Every
decision has a 4–8-unit history, 2–8 naturally admitted candidates,
construction support for each placement, and no complete-history,
trailing-four, ordered-route, operative-key, document/CID, or prior-fixture
overlap. Validation inputs contain histories and candidates only. Use the
pinned zeta list if referenced and never recalculate it. Construction and
rebuilds are deterministic multithreaded content partitions with ordered
reductions.

Before any placement, link, harmonic, or score diagnostic, one immutable pre-
geometry CID binds that population and sealed commitment, the implementation
commit, formulas/quantization, operators, controls, thresholds, 32-pair
randomization blocks, and exact work ledger. A Gate 0 miss is terminal; #986
does not re-freeze or retry.

Gate 0 loads no calibration/test labels. It binds corpus, split, codec,
placement, `HarmonicLinkState7`, operator, policy, and work identities and
requires 100% selected-history/candidate placement and link-state coverage,
operative directed reachability for every decision/candidate pair (positive
quantized `H_c^6` on at least one occupied history, nonzero occupied-history
spread, and distinct field vectors for at least two candidates), nondegenerate
`H^1`/`H^6` difference, nonzero `g(c)` spread, nondegenerate rank/score spread,
unchanged natural support, operative
anti-recall, populated/padded separation, same-frame reproduction, equal arm
work, source closure, incremental/full equality, and two byte-identical
multithreaded builds. Exact pre-label identities bind the PPMI smoothing/shift,
window weights, factorization and degeneracy handling, basis/sign convention,
radius map, PPMI and Euclidean-control peer selection/ties/nulls/weights, row
normalization, fixed canonical-float restart/iteration, final-only `H^1`/`H^6`
quantization, operator map, MAD-zero rule, balanced ternarization, and margin
calibration. Any miss returns
`UNAVAILABLE_FRAME_OR_POPULATION` before selection.

Calibration fits exactly two abstention margins with one frozen deterministic
rule and equal budget: `delta_F` on the unablated two-component full arm, shared
by every geometric control, and `delta_T` on the unablated one-component table
comparator, shared by its order/recall controls. Each family uses an ordered 33-
position list: zero followed by one top-two gap per decision in CID order,
retaining duplicate positions; it maximizes selected precision subject to
24/32 coverage, then ties by more correct decisions and smaller margin. If no
candidate meets the floor, zero freezes and that family cannot take its
positive terminal. No control calibrates itself. Label-free `M_b` and `M_g`
are computed once from canonically pooled unablated induction values and reused
unchanged by the applicable arms; controls do not self-normalize. Both margins
and their transcript freeze before one sealed test, which then
compares the full arm with separate placement-only and link-only permutations,
address-only placement/links, harmonic-link-disabled, matched weighted six-
Euclidean-nearest, direct-link `H^1`, transport-disabled, order-deranged,
last-only, state-disabled, candidate/operator-mapping-permuted, absolute-only,
positive-only, and exact-recall/count controls,
plus a table-native semantic-value comparator. All arms receive the same
natural candidates, information, link slots, and declared work. Every control
per candidate performs six canonical sweeps over every artifact route and six
padded slots, then the same eight padded history/operator operations. Disabled
and direct arms substitute their frozen value only at the scoring boundary.
Abstention is incorrect.

Let `U=(1/64) sum_j 1/|C_j|`. The positive geometric terminal requires
calibration eligibility, coverage at least 48/64, accuracy at least `U+0.15`,
and at least 7/64 advantage over
every named placement, link, address, distance, direct-link, transport, order,
last, state, operator-mapping, absolute-only, positive-only, and recall/count
control. Each exact paired randomization test flips whole 32-pair IDs, not 64
dependent decisions, and must give `p <= 0.05`. Both decisions must be correct
in the sealed direction on at least 24/32
predeclared pairs while last-only and state-disabled do not reproduce both
correct decisions on those same pairs, and at least 4/64 advantage over the
table comparator under the same exact 32-pair-blocked randomization test with
`p <= 0.05`.
Determinism, equivariance, work, and source boundaries must also pass. The table
comparator transfers only if it is calibration-eligible and independently clears 48/64 coverage,
`U+0.15`, and a 7/64 advantage over its order-deranged and recall/count
controls under the same pair-blocked `p <= 0.05` test. Do not weaken a threshold after labels load.

Precedence is exhaustive. Contamination returns `INVALID_CONTRACT`; a failed
pre-sealed integrity/reachability gate returns
`UNAVAILABLE_FRAME_OR_POPULATION`; otherwise, if the full arm passes, return
`PROCEED_TO_I1_WITH_CORPUS_SIGNED_TRANSPORT_ATTENTION`. If corpus/table value
independently transfers but the full arm does not, return
`RETAIN_GEOMETRY_AS_TRANSPORT_ADVANCE_TABLE_VALUE_QUALIFIER`. If the full arm
and table comparator both fail a valid sealed test, return
`REDESIGN_CORPUS_OBJECTIVE_OR_PLACEMENT`. Every nonpositive result stops; no
re-freeze, retry, second representation, or #953 generation runs in #986. Any
repair requires a freshly frozen successor. The complete contract is the
[#986 plan](corpus_induced_signed_transport_attention_plan_986.md).

**Observed #986 result, 2026-08-28.** The pinned raw corpus reproduced its
content identity and 3,000-document census, but no source-free corpus-scale
codec or exact induction/calibration/sealed pair commitment was available. The
exact 64-state SpiralCore operator/table reproduced at control scope, while
chart transport remained `NOT_ESTABLISHED` and no complete lexical `O(x)` or
compiler/query frame identity existed. The prerequisite certificate
`blake3:3fff541e4ac37193babaacd25227019fb401950ccdd936ab38ac46c6c2916337`
therefore records `UNAVAILABLE_FRAME_OR_POPULATION` before placement. Gate 0,
calibration, sealed labels, full/table/control arms, coverage, paired tests,
payload replay, and #953 were `NOT_RUN`. See the
[#986 evidence record](corpus_signed_transport_attention_986.md).

### I1/#953 bounded decoded generation contract

#953 owns the preserved production vertical slice, not a qualification matrix.
Its implementation executes this source-free loop through the reusable core and
an explicitly research-scoped CLI entrypoint:

```text
prompt bytes -> canonical lexical routes -> natural schema-2 admission
             -> full causal-path select-or-abstain -> exact payload inversion
             -> deterministic boundary rendering -> append selected route
             -> punctuation, abstention, or bounded-cap termination
```

Only after that loop exists, freeze one tiny deterministic construction corpus,
one matched natural prompt pair, one artifact identity, at most eight observed
lexical units including prompt and continuation, two to four emitted units, and
one deterministic stop rule. Construction may supply bounded predecessor/
successor or second-order admission support, but it must not store or select an
exact full-history continuation for either prompt.

The matched prompts require incompatible natural lexical choices while holding
the decisive candidate union and comparison budget equal. Compare exactly two
arms: full #969 causal-path selection and state-disabled selection with
identical natural support and work shape. At most one grammar-disabled
diagnostic is permitted, only after a concrete output defect is observed and
only to localize that seam.

The positive gate requires all of the following:

- the full path changes the selected route;
- distinct, short output that is exactly decodable and grammatical only at the
  bounded level claimed;
- exact route-to-payload inversion for every admitted and selected route;
- deterministic punctuation, abstention, or cap termination;
- byte-identical trace and report bytes across two executions;
- no period-1 through period-4 cycle; and
- no provider, source weights, target row, future event, or exact full-history
  continuation.

Stop on support drift, unequal work, an inert path intervention, decode failure,
nondeterminism, a short cycle, or exceeding the eight-unit state bound. The
historical direct-repair rule produced the observed record below. It is now
superseded for forward sequencing: another representation judged on the known
#953 population would risk post-hoc fixture tuning. #983 supplied the completed
independent negative; #986 then supplied a completed unavailable population/
frame result and #953 remains untouched.

The implemented repair has one frozen admission policy. I1/last-one, I2/last-two,
ordered-sentence, and divisor rows form the primary tier. With non-empty primary
support, adjacent-spin rows are consulted and their physical entries counted
but examined/admitted as zero. Only an empty primary tier activates bounded
adjacent-spin fallback. The trace binds the policy identity and distinguishes
row slot/key, consulted, physical-row-present, entries available/examined,
fallback activated, and entries admitted. The repaired preflight preserved PR
#978's frozen construction, surfaces, ordering, route placement, identities,
expected support, and work before exposing H4 selection.

The frozen terminals are:

- `PROCEED_TO_A1Q_H_WITH_BOUNDED_SOURCE_FREE_GEOMETRIC_GENERATION` — the full
  causal path drives one bounded natural decoded loop under the complete frozen
  contract; and
- `REVISE_I1_GENERATOR_IN_PLACE` — a valid run localizes a generator seam that
  must be repaired before #953 can close.

Invalid or unavailable work leaves #953 open. Established B0/#989 now permits
one #953 intervention against its fixed table reference. A later positive #953
result qualifies only that matched intervention plus #953's decoded
grammar/sentence loop. Paragraph,
conversation, and global influence remain inert until #973; correctness,
reasoning, product chat, optimization, formal closure, and release remain
downstream.

#### Observed #953 outcome

The frozen lexical pair `active agile athletes run` / `agile active athletes
run` produced full-path continuations `slowly carefully` / `carefully slowly`.
State-disabled produced `slowly slowly` for both. At the decisive position all
four arms admitted exactly `{carefully, slowly}`, read seven rows and two
entries, used four prefix keys per candidate, and performed eight H4
comparisons. Step two preserved the same support with five keys and ten
comparisons. No direct I1, I2, or ordered-sentence row hit.

All admitted and selected addresses inverted exactly; each selected route was
appended, both full outputs terminated at the frozen two-unit cap with six
total observed units, and no period-1 through period-4 three-repeat cycle was
present. Complete reports reproduced byte-for-byte. Construction artifact
`blake3:411f091f9455dd401711861db6db534482780f4b07645454c6bc1579072cc0ad`
and canonical smoke record
`blake3:f8738ae16585b5817108ad6c8bc1ec7aee93f9d5a6cacffaa3aa084bb643cf72`
bind the result.

Independent audit found that the fixture preserves #969's vocabulary ranks,
construction topology, prompt ranks, outputs, and costs exactly. Every
contextual row misses, and neither prompt linguistically requires one admitted
adverb over the other. The smoke therefore cannot satisfy the incompatible
natural-choice or bounded-grammar clauses. The terminal is
`REVISE_I1_GENERATOR_IN_PLACE`; #953 remains open and #973 remains blocked. The
complete evidence and then-current in-place action are in the
[#953 record](local_geometric_generation_953.md).

The next frozen natural agreement contrast failed earlier, at the support-only
preflight. Both prompts had identical count/work shape, but one adjacent-spin
row expanded the expected `{still}` and `{run,runs}` direct-plus-divisor unions
to the same five candidates. The preflight record is
`blake3:70375921e267b5ceff2198f879356cfb42dd6907accc0c2b720fc8b89b59b271`.
H4 path costs, selector outputs, and all four generator arms are
`NOT_RUN_SUPPORT_PREFLIGHT_HARD_STOP`; the localized repair is a primary
direct-plus-divisor admission tier with adjacent-spin rows used only when that
primary tier is empty. That statement remains the historical old-policy stop.

The versioned `PrimaryThenAdjacentSpinFallbackV1` repair subsequently produced
the exact frozen support. At the two steps, all seven slots were consulted; 8
then 11 entries were physically available, while only 3 then 6 primary entries
were examined and admitted. The physical adjacent row exposed five available
entries at both steps but examined/admitted zero because fallback was inactive.
The left/right candidate unions, source counts, keys, and comparison budgets
matched. Repaired support record:
`blake3:aab38fc513521cdd495bad74cc4a87754ec43ecdef5cb6e098b101412d3d7fe9`.
The #969 regression also retained active adjacent fallback and its prior record.

The one permitted four-arm run then decoded `still run` for both full-path
prompts and `still runs` for both state-disabled prompts. Exact inversion,
append, bounded termination, no short cycle, source/provider closure, matched
support/work, and byte-identical replay passed. The right full-path arm matched
its frozen continuation, the left did not, and the two full-path choices were
not distinct. Record:
`blake3:dfe03d4c56f7e5e9cf48d524f2f0b10482c4b3b85fae152dd29c64543caa0b79`.
The terminal remains `REVISE_I1_GENERATOR_IN_PLACE`.

That next hypothesis was frozen and executed as
`LocalSameObjectContextPlacementV1`. Each decisive history obtained `still`
from causal singleton support before the `{run,runs}` query; no candidate was
appended as an inference input. All controls reused the same overlay artifact,
prompt support, admission, and work. The label-free, selection-blind raw census
reproduced 7/7 construction trajectories with zero class collisions and zero
padding-identity aliases. Real placement selected 0/2 intended candidates,
while the placement-permuted and order-shuffled controls selected 2/2 and 1/2.
Complete held-out histories were absent from construction, but the operative
suffixes exactly recalled shorter construction subhistories, so the partition
was not operative-representation anti-recall. Generator execution and replay
were `NOT_RUN`. The corrected label-free input, raw-census, and label-attached
outcome identities are bound in the
[#953 record](local_geometric_generation_953.md). This observed chronology is
append-only. #953's fixture, overlay, generator, and records remain untouched
under #986; #973 and downstream #954 remain blocked.

### A1Q-H/#973 exact-spin global operator qualification

The V1 contract and negative below remain append-only evidence. The
independently frozen V2 repair subsequently satisfied the bounded-global
decision without changing V1. Later corpus-placement, gated-delta, and direct
V3 results were negative. `ConnectionGaugeCovarianceV4` subsequently passed
construction covariance but failed held-out functional binding. `HELM-D-R4`
architecture audit, ordinary-donor reproduction, and full-decoder R4/Spin
softmax parity then passed and remains qualified. Intrinsic V1 is closed
unavailable before D3. Source-faithful
[`HelmDLearnedManifoldR4ConstructionV2`](helm_d_learned_manifold_r4_construction_973.md)
then completed a valid non-D3 construction-validation negative: learned Lorentz
failed retention and matched parity while its controls established sensitivity
only. The 8/8-contract attempt stopped at its two-document preflight and
rejected tangent readout. Provider-free autonomous
`R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) generation now passes. The single
dedicated opt-in, loopback-only native HTTP endpoint also passes the frozen
eight-token CLI-parity canary, with no default-engine change. Dashboard
wiring/static native-readiness and WASM-isolation checks pass; hosted Pages is
static/offline without a functioning chat backend/artifact lowering. The
source-free Q16 suffix trace student is complete with bounded distillation but
looping output. Its recurrent state successor is complete and failed
promotion. The construction-only leave-one-document-out observability audit
then completed at `INSUFFICIENT_SUPPORT_COVERAGE` without boundary attribution.
The #1014 end-to-end campaign then established load-bearing ordinary causal
attention through a `2.677393`-nat attention-off penalty and exact Rust parity,
but its enabled NLL `2.127407` and subject/scene retention `3/5` failed the full
quality DoD. #1017's separately frozen continuation then passed retention
`5/5`, parity, audits, and replay, but failed solely on NLL
`1.5727521962806827`. #1019 now specifies an optional evaluation action: a
frozen 12-layer, 13,130,784-parameter capacity improvement over that exact
mechanism.
Population, 400-step overfit, and random-export all-twelve-layer Rust preflight parity
passed. MPS stopped `UNAVAILABLE_HARDWARE_BUDGET` on time (`20.66 h > 8 h`),
with memory passing at `21.03%`. That terminal applies only to the frozen
offline implementation. Full training, final parity, reveal, generation, and
replay remain `NOT_RUN`. Its fused-AdamW/deferred-logging fast path was slower,
so #1019 is optional/paused. #1017 remains the working source-backed
`r4 generate` path; #973's retained language path qualified, its paired-H4
capacity successor failed, its direct retained readout is a directional
`PARTIAL`, and the independently frozen layerwise-normalized readout is also
terminal `PARTIAL`. The parameter-free readout ladder is closed. The learned
candidate-leaf associative successor subsequently completed
`LEARNED_ASSOCIATIVE_READOUT_NO_CAPACITY`: its pooled control was strongest
but below both capacity floors, and geometry attribution failed. The one
independently frozen write/binding-law successor subsequently completed
`PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY`: geometric gain
`0.03896945868086732` missed the absolute capacity floor, the plain comparison
missed geometry attribution, and full delta did not beat independently fitted
additive. Fresh-language and integrity gates passed. This exact predictive
block-delta law stops without generation; ordinary-softmax and qualified
retained-attention evidence remain unchanged.
CUDA and external GPU execution are out of scope. D3 remains `NOT_RUN`;
intrinsic/readout alternatives,
resonance-based softmax replacement, whole-decoder recurrent lowering, and
exact deployment are parked.

The earlier #989-matched #953 dependency qualified; the retained-language-path
and paired-H4 results linked above now constrain #973's readout-seam mechanism.
Operator influence consumes #953-admitted support;
it does not participate in admission. The existing adjacent-spin retrieval rows
remain fallback/diagnostic data, not operator coefficients; any neighbor
operator is compiled independently over exact classes and admitted candidates.

The first global decision reused one existing exact `shared_class_kappa` over
full signed S3 orientation, checked Hopf observation, fiber, and torsion.
Hopf/S2 is not class identity by itself, and `q`/`-q` remain distinct. The
fixture contains enough ordered global state to produce a witnessed
non-identity transition, at least two immutable address references in the same
exact class, and one different `shared_class_kappa` intentionally occupying the
same current `SpinSector`. The operator
result is bound by global root/epoch kappa, operator kappa, chart/table
identities, and exact class and is reused without changing route, payload, or
kappa identity.
The final candidate score remains query-specific. If angular direction enters
the dynamic relation, #973 binds either a new exact `SpinTorsionState` relative
relation or an explicit spin-to-H4 map. The existing prime-derived relative H4
quaternion remains a separate route witness.

Compare exactly matched real, identity-disabled, and deterministic
class/operator-permuted arms. Row reads, admitted candidate addresses, payloads,
lower-scope state, candidate/operator ceilings, and observed work MUST be
byte-identical. The report separately records base local cost, exact class,
operator input/result, operator contribution, transformed candidate-relative
cost, reuse count, selection or abstention, and exact decoded output. The real
arm must change the predeclared candidate and decoded output, while the
identity-disabled and class/operator-permuted arms do not reproduce that effect
under identical support and work. Distinct stored state, a reuse hit, or a
non-zero trace alone is insufficient.

Before freezing a scalar readout, enumerate the transformed candidate-state
classes and stop if any retained class requires incompatible outcomes. One
fan-out witness MUST show one class-operator evaluation reused by multiple
references with byte-identical overlay state. Share or precompute only that
class result; do not share the query-specific candidate score. The current H4 leaf is assigned
from the prime rather than `SpinTorsionState`, so the report must bind the
spin/operator-to-candidate relation. The existing prime-derived H4 relative
witness does not establish stored-spin direction without an explicit
spin-to-H4 map.

Only after that exact-class decision may #973 freeze a finite,
orientation-aware relative-angular neighbor kernel. It may influence already
admitted candidates but cannot inject candidates, collapse antipodal states via
Hopf projection, or enumerate the corpus. Current procedural spin placement
remains a threat to semantic interpretation and is exercised by the
class/operator permutation control. H4 group action is not by itself evidence
of a spherical-harmonic field. The operator kappa must bind basis, mode order,
coefficients, quantization, and transition law before the prototype is called
harmonic. #973 will freeze its own terminal literal before running this
contract.

The now-positive bounded V2 subprobe cannot close #973. Paragraph and
conversation are retained. The ordinary attention reference is now accepted;
provider-free autonomous generation and its dedicated opt-in, loopback-only
native HTTP endpoint now pass. Dashboard wiring/static native-readiness and
WASM-isolation checks pass; hosted Pages is static/offline without a functioning
chat backend/artifact lowering. The source-free Q16 suffix trace student is
complete with bounded distillation but looping output.
`R4SoftmaxTraceStateStudentV1` is complete with `FAIL_PROMOTION`; the
construction-only leave-one-document-out observability audit completed at
`INSUFFICIENT_SUPPORT_COVERAGE` without boundary attribution; #1014 then
established load-bearing ordinary causal attention through its
`2.677393`-nat attention-off intervention and exact Rust parity, while failing
its complete quality DoD at enabled NLL `2.127407` and prompt retention `3/5`.
#1017's separate continuation then closed NLL-only negative at
`1.5727521962806827`, with retention `5/5` and all other gates passing. #1019 is
an optional, paused frozen 12-layer, 13,130,784-parameter campaign over that
unchanged mechanism. Population, 400-step overfit, and random-export
all-twelve-layer Rust parity passed. MPS is `UNAVAILABLE_HARDWARE_BUDGET` on
time (`20.66 h > 8 h`) with memory passing at `21.03%`. That terminal applies
only to the frozen offline implementation. Full training, final parity, reveal,
generation, and replay remain `NOT_RUN`. Its fused-AdamW/deferred-logging fast
path was slower, so #1019 is optional/paused. At that checkpoint, the active
product step became the #1017 `r4 generate` path. CUDA and external GPU execution are out of scope,
while intrinsic/readout alternatives,
resonance-based softmax replacement, whole-decoder recurrent lowering, and
final requalification are parked.

#### Observed bounded-global V1 target-free terminal (2026-08-28)

The first frozen global decision reached
`RETAIN_CONVERSATION_ONLY_REDESIGN_BOUNDED_GLOBAL_EXACT_SPIN_RELATION` before
target attachment. Its detached snapshot carriers had distinct epochs/roots,
four references, three exact classes, and one same-address result reuse, while
one byte-identical lower artifact and equal support/work were preserved. The
operative relation was nevertheless identical: `Pavel` and `helix` map to
`q=(1+i+j+k)/2`, `prism` maps to identity, and both `q*q*q*1` and `q*q*1*q`
finish at the same complete `-1`/fiber/torsion state. Real roles were
`helix/helix`; permuted roles were `prism/prism`. Target loads were zero and
decoded evaluation is `NOT_RUN`. See the
[#973 bounded-global record](bounded_global_exact_spin_attention_973.md).

This is a relation/population falsification, not a global-attention result. It
remains V1 history; the independently frozen V2 repair below supplied the
required noncommuting distinct folds and incompatible target-free winners.

#### Observed bounded-global V2 positive terminal (2026-08-28)

`BoundedGlobalNoncommutingExactSpinR4V2` canonically enumerated the
construction population and selected the first pair jointly satisfying direct
exact H4 `A*B != B*A`, distinct nonidentity complete left folds, central Q29
phase composition, one same-address exact-class reuse, and incompatible strict
candidate-relative winners under `C^-1*G` lexicographic least cost. The exact
score path is protected by a typed/source firewall and an injected forbidden-
read falsifier. Its zero forbidden-read trace is structural evidence, not
dynamic instruction counting.

The target-free gate completed with zero target loads. The committed target
preimage
`blake3:b7340c776e005c32316de793b332e3f218b1fad757c77044b0fa2e70fc308354`
was then loaded exactly once. Real decoded behavior was 2/2,
identity-disabled 1/2, class/operator-permuted 0/2, and support-reversed real
2/2; support/work mismatches were zero and exact period-plus-EOS termination
was 6/6. Exact replay identities are:

```text
operator:         blake3:1cf08604fb4a1c545984f4cab41194e0ffcf1d7551b6e438ed57b49a0066a6e9
population audit: blake3:16ebc6d36f01e4cb324d3c46fc059aca4ffea84ba467e860b55f983cd83f4a9c
target-free:      blake3:c3fb3568028f924fb12971c888193cc5780111a7af14503e240f39fbeb58dd4a
decoded smoke:    blake3:41207999bb088e3b5f186cce983951cc27c2962d34ef8046a0beae4754b44218
```

The binding terminal is
`RETAIN_BOUNDED_GLOBAL_NONCOMMUTING_EXACT_SPIN_ATTENTION_CONTINUE_CORPUS_INDUCTION`.
This establishes one bounded synthetic causal global geometric-attention
witness only. It does not establish corpus induction, semantic or natural
transfer, general attention, correctness, reasoning, or product readiness.
That next corpus-induction gate is now completed negative in PR #997. V4 later
preserved construction covariance but failed held-out functional binding. The
ADR-0005's pinned HELM-D provenance, frozen ordinary donor, and full-decoder
gauge-equivalent ordinary-softmax parity in transported R4/Spin frames remain
qualified. Intrinsic V1 is unavailable before D3; source-faithful
learned-manifold V2 is now a valid non-D3 construction-validation negative.
Its controls established sensitivity, but learned Lorentz failed retention and
matched parity. The 8/8-contract attempt stopped at its two-document preflight
and rejected tangent readout.
Provider-free autonomous `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`)
generation now passes, and its explicit opt-in, loopback-only native
HTTP endpoint passes the frozen eight-token CLI-parity canary. Dashboard
wiring/static native-readiness and WASM-isolation checks pass; hosted Pages is
static/offline without a functioning chat backend/artifact lowering. The
source-free Q16 suffix trace student is complete with bounded distillation but
looping output. Its recurrent state successor is complete and failed promotion.
The construction-only leave-one-document-out observability audit subsequently
completed at `INSUFFICIENT_SUPPORT_COVERAGE` without boundary attribution. The
subsequent #1014 campaign established load-bearing ordinary causal attention
through a `2.677393`-nat attention-off penalty and exact Rust parity, but failed
its full quality DoD at enabled NLL `2.127407` and prompt retention `3/5`. The
subsequent #1017 continuation failed solely on NLL `1.5727521962806827` while
retention, parity, audits, and replay passed. #1019 records an optional, paused
12-layer, 13,130,784-parameter campaign over that unchanged
mechanism. Population, 400-step overfit, and random-export all-twelve-layer Rust
parity passed; MPS is `UNAVAILABLE_HARDWARE_BUDGET` on time
(`20.66 h > 8 h`) with memory passing at `21.03%`. That terminal applies only
to the frozen offline implementation. Full training, final parity, reveal,
generation, and replay remain `NOT_RUN`. Its fused-AdamW/deferred-logging fast
path was slower, so #1019 is optional/paused. At that checkpoint, the active
product step became the #1017 `r4 generate` path. CUDA and external GPU execution are out of scope;
intrinsic/readout,
multi-resonance, and recurrent lowering are parked. D3 remains `NOT_RUN` and
#954 remains blocked. See the
[#973 bounded-global record](bounded_global_exact_spin_attention_973.md).

## 4. Matched controls

An attention, inference, or reasoning comparison MUST use controls with the
same causal information and declared budget. At minimum report:

- identical row types, row ceilings, candidate-entry ceilings, and output
  ceiling;
- identical prefix, lexical coverage, hierarchy scopes, and absence of actual
  future routes, events, target labels, teacher continuations, or provider text;
- any one-step candidate projection derived deterministically from the same
  observed state and already-admitted support;
- identical compiler artifacts except for the named intervention;
- candidate support before and after any common pre-geometric admission; and
- work performed, not merely requested worker count.

For #969, run exactly three arms with identical natural support and
group-comparison budgets: full retained causal path, last-only path, and
state-disabled. No construction/validation split, channel census, weight sweep,
permutation matrix, or higher-scope fixture is part of this prototype decision.
A control that sees future tokens, additional retrieval, or a larger candidate
set is invalid.

For #983, every arm uses the same natural support and performs eight typed
prefix slots, two candidate evaluations, two prototype/class slots per
candidate, and the same table-operation shape per decision. The encoder
declares two payload-inversion slots per decision, but Gate 0 performs none;
payload inversion remains selector-only and was `NOT_RUN`. Occupancy-false
padding is a typed identity no-op and may not alias an
occupied identity result. Negative controls are state-disabled, last-only,
order-shuffled history, causal-return/lease-disabled, construction
content/current-pairing shuffle, candidate/prototype placement permutation,
prime-placement permutation, exact-recall-only, content swap, construction-key
shuffle, and incoherent candidate relabeling. Positive controls are coherent
full-artifact candidate relabeling, exact incremental/full-history equality,
and byte-identical rebuild/outcome replay. The separate count-only last-anchor
comparator cannot admit candidates, rank geometric candidates, break ties,
populate relation classes, or be described as attention.

For #986, every arm receives the same natural candidates, semantic-placement
reads, causal slots, operator slots, and declared work. Required interventions
are stratified placement permutation, address-only placement, identity
transport, order derangement, last-only, state-disabled, candidate/operator
permutation, absolute/positive-only weights, and recall/count-only lookup. The
table-native semantic-value comparator receives the same candidates and work
but no geometric transport. Two content-partitioned multithreaded builds,
incremental/full equality, and coherent relabeling are positive controls. A
single-worker long run is not an authorized reproducibility arm.

When #953 is later unblocked, run exactly the full-path and state-disabled arms with identical
natural support and work. Do not carry #969's last-only arm or the obsolete
six-comparator I1 matrix forward. A single grammar-disabled diagnostic is
allowed only to localize an already observed output defect.

For #973's global exact-spin operator decisions, run real, identity-disabled,
and deterministic class/operator-permuted arms with identical admission and work.
At least two references share one exact class so reuse is exercised; a different
exact class in the same current `SpinSector` checks that the operator is not
aliasing the coarse Hopf-octant/torsion bucket. A one-unit registration-only
global snapshot or an identity transition cannot qualify the operator.
The first frozen pair satisfied the class/reuse population but failed this gate
because both complete order folds were identical; its decoded arm remains
`NOT_RUN`.

A working decoded intervention records
`PROCEED_TO_I1_WITH_CAUSAL_R4_PATH_ATTENTION`. This establishes only a
load-bearing identity-derived mechanism. It is predecessor evidence for #983,
not current permission to resume #953.

The observed smoke uses `aa bb dd qq` versus `bb aa dd qq` with natural
`{ll, rr}` support. Full retained path decodes `rr ll` versus `ll rr`; both first
winners use retained non-identity prefix 1. Last-only abstains on both first
choices and state-disabled chooses `rr` on both. The canonical byte-identical
record kappa is
`blake3:60360a9e22a56ea4af363e43f7103bb8104d015d58feb582d921fc17afaf207f`.
See the [#969 record](local_geometric_attention_969.md).

If count and source breadth admit support before geometric scoring, reports
MUST say so. “Least energy” then means least declared energy among admitted
candidates, never among the full untruncated union.

## 5. Anti-recall protocol

This section does not apply to #969's mechanism-only smoke; that result is not a
semantic generalization claim. Later geometric attention and reasoning claims
require evidence beyond stored continuation lookup. Their evaluation partition MUST be separated before route rows,
continuation counts, or hierarchy summaries are compiled. Partition by source,
conversation, or task family when adjacent records could leak the answer.

For every evaluated position report:

- whether its exact local, sentence, paragraph, conversation, and global keys
  were present in construction data;
- whether the selected continuation appeared under an exact or backoff key;
- lexical-codec coverage and unknown-unit handling;
- duplicate or near-duplicate source status; and
- whether the item is classified as recall, recombination, or anti-recall.

An anti-recall slice excludes exact continuation hits and must contain novel
ordered combinations or counterfactual constraints while retaining enough
registered lexical coverage to make the geometry reachable. Randomly hiding a
row after compiling from the same answer is an ablation, not held-out evidence.

Full-history disjointness is insufficient. The exact operative representation
must also exclude recalled construction suffixes, ordered route witnesses, raw
prototypes, and complete candidate-interaction witnesses. Report every overlap
at the actual retained width and class resolution. A shorter construction
subhistory recalled by the decisive representation is recall even when the
complete validation prompt is absent. #983 additionally requires controlling
validation lexemes to be absent from their construction family and freezes its
raw selection-blind census before sealed validation labels are attached.

On this slice, exact hierarchy kappas must miss by construction while bounded
overlapping summaries remain available. Report whether shared-prime factors,
cosine resonance, projection energy, winding/window compatibility, accumulated
Hopf phase, transported hypersphere trajectory, and paired-H4/E8 coordinates
recover locality. Report unqualified terms as storage fields, diagnostics, or
controls rather than folding them into a winning aggregate score. A candidate
found only through digest equality is recall and does not satisfy this
requirement.

A harmonic-neighbor channel may change the state or cost of a candidate already
admitted by an independent lawful row. That is an influence hypothesis, not
candidate admission and not proof of anti-recall selection until the matched
operator controls change the selected route and decoded output.

Corpus scale receives the same treatment. A larger construction partition may
compile more transition support, multiscale summaries, versioned placement
overlays that preserve immutable route/payload identity, or operator statistics,
but increased lookup density is not emergent attention.
Construction may compile causal prefix-to-observed-next-route examples;
validation/test continuations and runtime future data may not tune it.
Within each corpus rung, real and control arms hold candidate support and work
fixed: the real arm must change the predeclared route and decoded output, while
scope-disabled, order-shuffled, and operator-permuted arms do not reproduce the
effect. Coverage and support changes between rungs are reported separately as
capacity/recall; an operator-only scale question freezes admission across all
rungs. Exact recall, recombination, and anti-recall results remain separate at
every corpus size.

## 6. Correctness and abstention

Correctness requires an independent evaluator fixed before results are read:
an executable answer checker, formal constraint, source-backed fact set,
deterministic simulation, or reviewed rubric with disagreement reporting.
Teacher agreement and fluent grammar may be useful diagnostics but are not
general correctness oracles.

Every correctness report includes:

- total eligible items;
- answered, correct, incorrect, and abstained counts;
- accuracy conditional on answered items;
- correct answers divided by all eligible items;
- abstention rate and false-answer rate; and
- results split by recall/recombination/anti-recall status.

Abstention is a typed output with a reason such as lexical-uncovered,
route-uncovered, contradictory support, insufficient evidence, or resource
ceiling. It is not silently converted to a correct answer, a blank string, or a
provider fallback.

## 7. Long-run preflight

No hours-long run starts until all fields below are written into its run
contract and the cheap preflight permits launch:

```text
decision:                 exact product/research choice this run can change
metric and denominator:   current numerator/denominator and final denominator
reachability ceiling:     arithmetic from already observed routing populations
small probe:              command/artifact, result, and why expansion is needed
matched controls:         information, row, candidate, and work budgets
parallelism proof:        active workers, assigned/completed work, wall-time gain
reuse plan:               existing routes, tables, traces, caches, and CIDs reused
checkpoint contract:      interval, atomic files, resume identity, completed shards
progress contract:        completed/total units, rate, ETA method, heartbeat
resource contract:        disk headroom, peak memory, worker count, hard wall time
if positive:              distinct next action
if negative:              different next action
if unavailable:           action without inventing a result
stop conditions:          saturation, no reachability, bad ETA, errors, resource cap
```

Corpus expansion uses a predeclared increasing, preferably log-spaced ladder
rather than one immediate maximum-scale run. Document, conversation, and task
splits freeze before route rows or operator statistics are constructed. The
operator family/schema, basis and mode order, objective, quantization, scope
semantics, neighborhood contract, and induction rule freeze before the ladder;
only declared statistic/coefficient values vary under a new artifact/operator
kappa per rung. A structural or placement-epoch change reruns the bounded #973
qualification. The accepted small mechanism remains a regression at every rung,
and expansion stops when the preceding rung cannot change the named decision or
only improves exact/backoff coverage.

Requested parallelism is not proven parallelism. Before a long run, a bounded
canary must show positive assigned and completed work on every intended worker,
byte-identical semantic output across worker counts, and a measured wall-time
improvement on the same workload. Reuse previously compiled routes, zeta
tables, lexical artifacts, content-addressed shards, and completed checkpoints;
do not recompute work whose identity already matches.

Progress reports use a real denominator: completed units divided by total
units, plus an observed rate and ETA method. “Still running” and “likely almost
done” are not progress. Crossing the hard wall time causes an orderly
checkpoint and stop. Apparent closeness is never grounds for an undeclared
extension.

Check disk headroom before launch and during checkpoint commits. A checkpoint
is useful only if it is atomic, content-addressed, records completed shard IDs,
and resumes without repeating accepted work.

## 8. Product-ready boundary and release QA

Broad release QA begins only after a product-facing path is ready for a release
decision. Product-ready here means all of the following are exercised together:

- real CLI, HTTP, or chat input uses the pinned lexical codec;
- route hierarchy state persists across at least one multi-turn interaction;
- all five hierarchy scopes have bounded coverage and matched-control evidence,
  including held-out overlap behavior when exact keys miss;
- next-token inference produces decodable, prompt-responsive output;
- provider-free mode fails closed rather than calling an external generator;
- correctness and abstention have declared evaluators and denominators; and
- the build has bounded progress, checkpoint, resource, and provenance records.

Before that boundary, run focused structural checks and the smallest empirical
probe owned by the active decision. Do not spend hours on workspace-wide,
corpus-scale, cross-platform, fuzz, formal, or release-package sweeps for a path
that cannot yet deliver a real provider-free answer.

After the boundary, release QA may activate the repository's full required
matrix, but each check still names the contract it protects. Expensive
certification runs once in the protected release/merge context when possible,
not repeatedly during exploratory iteration.

## 9. Result vocabulary

Use these outcomes literally:

- `PASS` — the predeclared criterion passed on the declared population.
- `FAIL` — it ran and did not pass.
- `NOT_RUN` — the experiment was intentionally not launched.
- `UNAVAILABLE` — a required fixture, provider, oracle, or platform was absent.
- `NOT_EXERCISED` — the run completed but the named branch received zero valid
  opportunities.
- `PROMOTE_H4_READOUT_CANDIDATE_TO_A1Q` — historical #970 positive terminal:
  its independent construction and
  sealed validation contract passed; only the paired-H4-derived exact R4
  heatmap term advances to #969 after protected delivery.
- `RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q` — observed historical #970
  probe produced a valid readout-identifiability negative; both H4 operands and
  the derived heatmap
  remain structural/control state; protected PR #972 delivered this terminal.
- `UNAVAILABLE_NO_INDEPENDENT_HOLDOUT` — under the historical #970 contract,
  lack of independent evidence would have left #970 open and #969 blocked.
- `INVALID_CONTRACT` — under the historical #970 contract, a violation would
  have left #970 open and #969 blocked.
- `PROCEED_TO_I1_WITH_CAUSAL_R4_PATH_ATTENTION` — #969's full causal path
  changes a matched naturally admitted decoded continuation while last-only and
  state-disabled are weaker under the same natural support and group-comparison
  budget. It is direct predecessor evidence for #983 but establishes neither
  semantic intelligence nor coherent generation.
- `PROCEED_TO_I1_WITH_CONSTRUCTION_TRANSFERRED_LOCAL_GEOMETRIC_ATTENTION` —
  #983's frozen construction-derived selector transfers to all six held-out
  natural choices with no real tie/abstention, every causal negative strictly
  below real, equal support/work, exact replay/inversion, operative anti-recall,
  and no source, provider, future route, candidate injection, or semantic
  tiebreak. It authorizes only a later unchanged application to #953.
- `UNAVAILABLE_FRAME_MISMATCH` — #983 compiler/query identities or exact frames
  differ; stop before selection.
- `UNAVAILABLE_NO_OPERATIVE_ANTI_RECALL` — #983's retained representation
  recalls an operative construction prototype; stop before selection.
- `UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER` — no usable construction-derived
  rule reaches the held-out decisions; stop before selection.
- `REDESIGN_CANDIDATE_CONDITIONED_CAUSAL_RETURN_REPRESENTATION` — #983's frozen
  classes, transfer ceiling, padding, derangement, or equal-work gate fails;
  retain the evidence and do not try a second representation in the issue.
- `PROCEED_TO_I1_WITH_CORPUS_SIGNED_TRANSPORT_ATTENTION` — #986's corpus
  placement transfers and its signed exact-transport arm clears every causal
  threshold while beating the matched table-native value comparator by at
  least 4/64. Apply only the frozen scoring semantics to #953 after a fresh
  label-free preflight.
- `RETAIN_GEOMETRY_AS_TRANSPORT_ADVANCE_TABLE_VALUE_QUALIFIER` — corpus/table
  value transfers but signed transport supplies no predeclared causal uplift.
  Retain exact geometry as address/transport and create one table-value
  qualifier; do not call the table result geometric attention.
- `REDESIGN_CORPUS_OBJECTIVE_OR_PLACEMENT` — #986's semantic placement fails
  against its stratified permutation/address controls; stop before #953.
- `UNAVAILABLE_FRAME_OR_POPULATION` — #986 cannot freeze a complete
  CID-disjoint population, placement, frame, natural support, or equal-work
  contract; stop before labels or selection.
- `PROCEED_TO_A1Q_H_WITH_BOUNDED_SOURCE_FREE_GEOMETRIC_GENERATION` — #953's
  full causal path drives one bounded natural decoded loop under equal support
  and work, exact inversion, deterministic termination/replay, and the declared
  source-free boundary. It exposes #973 but establishes none of its higher-scope
  attention, correctness, or reasoning claims.
- `REVISE_I1_GENERATOR_IN_PLACE` — a valid #953 smoke localizes a generator
  defect. This remains #953's historical terminal, but its former immediate
  in-place-repair action is superseded by completed-negative #983 and closed-
  unavailable #986 because the known population is no longer independent. The
  established B0/#989 result supersedes their former fresh-successor handoff:
  permit exactly one #953 intervention against the fixed table reference.

Never convert a missing fixture, empty denominator, skipped test, or unrelated
green suite into `PASS`. Every report preserves artifact kappas, partition CIDs,
control identity, denominators, and the action caused by the result.

## #973 layerwise-normalized readout result (2026-09-01)

The frozen `R4LayerwiseNormalizedRetainedReadoutLanguagePathV1` campaign ran
one Apple Accelerate CPU trajectory with four threads: `2,730` optimizer steps,
`5,241,600` token presentations, and `1,447.763973 s` total elapsed time. The
candidate artifact (`blake3:8d31e15c355aade1ccc2592dc5fb1caf14a5f056862621e7b467858569a1c1e4`)
was fixed before V3 reveal
`blake3:079bee84db32513c5d6c0cb54cbff1e70b163902efa934d950204090985b3f5a`.

The prompt decision over 512 directions / 8,192 target tokens was:

| Measure | Layerwise candidate | Frozen V1 | Frozen requirement |
|---|---:|---:|---:|
| Mean gain `G` | `0.0286980210` | `0.0073316237` | candidate `>= 0.0433216988` |
| Incremental gain | `0.0213663973` | — | `>= 0.0253415693` |
| Directional wins | `339/512` | `298/512` | candidate `>= 308/512` |
| Own-prompt NLL | `3.4798765288` | `3.6930405921` | candidate no worse |
| State-off gain | `0` | `0` | collapse within `1e-7` |

Thus wins, own-NLL, state-off, replay, and forbidden-read gates passed, while
both prompt effect-size gates failed. Fresh-language evaluation separately
passed all eight gates: candidate NLL/top-1 were `3.7126411677` / `31.661826%`
versus V1 `3.8850003883` / `29.728138%`; initial-to-final NLL improved by
`4.6111841143`; state removal cost `1.3495375637` nats and `20,595` correct
decisions; forbidden reads were zero.

Terminal:
`LAYERWISE_NORMALIZED_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL`. Result CID:
`blake3:35396bd6e64fc2c0bc7d86a84cc9e212ed913ce28e5353f5f2b8212b4cf2c532`.
Fresh-process verification reproduced all 13 declared comparisons, created no
optimizer, executed zero optimizer steps, and scored zero training batches;
verification CID
`blake3:3f316541dbab8061ed5ba891bf6a47ef22c55bca21fba01f6f97dbb3cb8497aa`.

The exact decision caused by this valid miss was to end parameter-free readout
variants. At that checkpoint, `R4LearnedCandidateLeafAssociativeReadoutV1`
became the independently frozen successor. Its completed result is recorded
below; this paragraph preserves the decision state before that campaign ran.
Do not retry, tune `g`, add a third normalization placement, generate from this
candidate, widen it, or lower it. Candidate generation, reasoning, and
exact/geometry-native lowering are `NOT_RUN`; #954 remains blocked.

## #973 learned associative readout result (2026-09-01)

`R4LearnedCandidateLeafAssociativeReadoutV1` completed against frozen V1, an
equal-parameter address-blind pooled control, and a fixed-leaf derangement:

| Prompt measure | Geometric | Frozen V1 | Pooled |
|---|---:|---:|---:|
| Mean gain | `0.00637679` | `0.00642365` | `0.01026323` |
| Directional wins | `299/512` | `308/512` | `324/512` |
| Own-prompt NLL | `3.710383` | `3.712799` | `3.682891` |

Pooled was `PROMPT_CONDITIONING_PARTIAL`, not a capacity pass: its absolute
gain was below `0.04332170`, and its `0.00383958` increment over V1 was below
the required `0.02534157`. The overall terminal is
`LEARNED_ASSOCIATIVE_READOUT_NO_CAPACITY`.

The separate terminal was `GEOMETRY_ATTRIBUTION_FAIL`. Geometric minus pooled gain was
`-0.00388645` with `209/512` paired directional improvements; geometric minus
deranged gain was `-0.00028887` with `251/512`. Pooled is therefore retained as
the non-geometric control signal, not promoted as geometric attention evidence.

Fresh-language NLL/top-1 was `3.903636` / `29.6285%` for V1, `3.901412` /
`29.6342%` for geometric, and `3.873756` / `30.0428%` for pooled; both learned
arms passed their fresh gates. Mechanics and exact independent verification
passed. Result CID:
`blake3:cedba37738ee249457bb589f716ee75afb16a0c4937c2a22ae9f917dd3eb97c1`;
verification CID:
`blake3:443d711ce9a228e26e2eb2eebb55c582848424e2677c3473d41deaf8afd69ec7`.

The frozen decision is no generation and no further readout retry over the same
retained values. At that checkpoint the next #973 mechanism had to alter the
retained-value write/binding law under a new independent freeze, using the
pooled arm as the matched control; that successor is recorded below. Reasoning,
exact/geometry-native lowering, #954, and C1-SB6 remain blocked or `NOT_RUN`.

## #973 predictive block-delta terminal (2026-09-01)

`R4PredictiveBlockDeltaBindingV1` executed the independently frozen write-law
successor with three separately fitted, equal-parameter arms: canonical H4
full delta, identity/plain full delta, and canonical H4 additive/no-overwrite.
All arms completed exactly `2,730` construction steps before reveal. Terminal
scoring and independent verification created no optimizer and performed zero
training steps.

| Prompt measure | Geometric | Plain | Additive | Frozen requirement |
|---|---:|---:|---:|---:|
| Mean gain | `0.0389694587` | `0.0150396469` | `0.0454819219` | geometric `>= 0.0433216988` |
| Directional wins | `375/512` | `309/512` | `368/512` | geometric `>= 308/512` |
| Own-prompt NLL | `3.5419674206` | `3.5184441975` | `3.5523845836` | comparator-specific nonregression |

The geometric arm passed its incremental comparisons with immutable V1 and
pooled, wins, NLL, state-load, fresh-language, and integrity criteria. It
failed the absolute gain floor, so capacity is negative and the terminal is
`PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY`.

Geometry attribution also failed because geometric-minus-plain gain was
`0.023929811749894725`, below `0.025341569256760274`, and geometric own NLL was
worse than plain. The independent transport-permuted comparison did pass at
gain margin `0.03181032686529761` with `310/512` paired improvements and no NLL
regression. Since both controls were mandatory, that partial separation is not
a geometry result. Delta-overwrite attribution failed at
`-0.006512463228773413` gain versus independently fitted additive and
`234/512` paired improvements.

Fresh geometric NLL/top-1 was `3.84055165318221` / `30.979348%`, and the
fresh-language and all integrity gates passed. The first scoring process
stopped on a variable-size final-batch work-ledger assertion before writing a
scientific result. Recovery
`blake3:7b76e36e44798bebf184ece08fdd8a2065bdd370106b5d64d5fae4c59dc6d88b`
bound the unchanged fitted artifacts and repaired scoring only. Result CID:
`blake3:6c67544d675eafcb8eb9c0dabb93617e3f6c3295af812e8acbb687107c010a74`;
scoring CID:
`blake3:44f8941d24a99fc230710fd700e7a7b13cee87587bfbe4e13bf7b095222e2ee6`;
exact-replay verification CID:
`blake3:567cf336eb05c3ec562aef7135f6fb35b580d02c758b0e79f2508cae57065f5d`.

The predeclared action is `STOP_WITHOUT_GENERATION`. Retire this exact
predictive block-delta write/binding law without corpus expansion, generation,
or lowering. Preserve ordinary causal-softmax attention and qualified V1 at
their established scopes. Coherent generation from this cell, reasoning,
integer/table lowering, release, and #954/C1-SB6 progression are `NOT_RUN` or
blocked.
