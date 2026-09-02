# AGENTS.md — uor-r4

Guidance for agents (human or otherwise) working in this repository.
**Post-v0.1 intelligence sequencing is authoritative in
[`docs/r4_intelligence_completion_plan.md`](docs/r4_intelligence_completion_plan.md),
the readable mirror of programme root #820.** The
[`Geometric Intelligence Programme`](docs/geometric_intelligence_programme.md)
is the current architecture and claim-boundary companion.

**Current English construction diagnostic (2026-09-02):** #1065 completed
`CONSTRUCTION_DIAGNOSTIC_COMPLETE`, with descriptive focus `QUESTION_READOUT`.
The retained #1063 model reproduced its entire construction score exactly:
`2,396/8,192 = 29.2480%`, including full logits, predictions, attention and NLL.
Changing the question left the prediction unchanged in
`3,974/4,096 = 97.0215%` of construction pairs; only `20/4,096` pairs had both
answers correct. Target-logit changes were positive in 2,040 pairs and negative
in 2,056. See the [#1065 diagnostic record](docs/r4_zoology_english_diagnostic_1065.md).

Of 8,192 answers, 6,905 selected a location in the history and 1,287 answered
`unknown`; none selected an absent location or other vocabulary token. The
largest displayed-slot selection share was 27.7625%, against balanced 25%
target exposure. Pooled q0 in-history errors were same-owner 841, same-object
834, unrelated 578. No overall position or attribute-confound majority fired;
type-specific attribute effects remain visible in the full record. This
localizes the next investigation behaviorally without proving an internal cause.

The run and exact fresh-process replay took `3.43 s` combined with peak RSS
`0.775 GiB` on eight Apple Accelerate threads. Training updates, new development
decisions, development/checkpoint/frame payload reads and geometry changes were
zero. #1063's completed 3,920-update fit and held-out negatives remain unchanged:
`218/1,024` supported answers, `0/256` complete groups, `37/256` unknown answers.
Its conditional R4/control remain `NOT_RUN_ENGLISH_BINDING_MISS`.

The next recommendation is one separately frozen readout-placement learning
experiment: a fresh matched fit with the supervised answer readout at the
queried object (position 37) instead of the constant colon (40). Keep the cell,
construction rows/labels, seed, optimizer and dose fixed; report both question
types separately. This is an explicit answer-readout task, with new unrevealed
development data required for a new transfer claim. The diagnostic does not
establish that this change will repair learning.

[#1061](docs/r4_zoology_exact_coherent_inference_1061.md) remains established: plain and coherent R4 both scored
`8,071/8,192 = 98.5229%`, with identical predictions and an `86.2061` percentage
points lost under the transport control. #1059's `11,900/12,000 = 99.1667%` preservation
also remains intact. More geometry is deferred. General English understanding,
H4 superiority, softmax removal, reasoning and chat readiness remain
unestablished; #973 stays open and #954 remains blocked.

The earlier #973 V5 terminal is the independently verified predictive
write/binding campaign. It completed
`PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY` at result CID
`blake3:6c67544d675eafcb8eb9c0dabb93617e3f6c3295af812e8acbb687107c010a74`;
exact independent replay passed at verification CID
`blake3:567cf336eb05c3ec562aef7135f6fb35b580d02c758b0e79f2508cae57065f5d`.
Integrity and fresh-language nonregression passed. The geometric arm reached
gain `0.03896945868086732` with `375/512` wins and beat V1 and pooled, but
missed the frozen `0.04332169878499658` absolute capacity floor. Geometry
attribution also failed: its gain margin over independently fitted plain delta
was `0.023929811749894725`, below `0.025341569256760274`, and its own NLL was
worse; the transport-permuted comparison passed. Delta-overwrite attribution
failed against independently fitted additive at gain
`-0.006512463228773413` and `234/512` paired improvements. The original
scoring-harness tail-batch failure remains recorded as `NOT_RUN`; its repair
performed zero retraining and zero optimizer steps and reused the three frozen
arms for scoring and exact replay. The predeclared action is
`STOP_WITHOUT_GENERATION`: retire this write/binding law. No generation,
reasoning, or integer/table lowering follows. Ordinary softmax and qualified
retained attention remain established and the larger programme continues, but
#954 remains blocked. See the
[binding #973 V5 record](docs/r4_predictive_block_delta_binding_prompt_capacity_973.md).

The qualified retained mechanism remains
[`R4RetainedLanguagePathV1`](docs/r4_retained_language_path_v1_973.md); its
retained state remains a causally load-bearing, competitive source-free
attention path. Its sole layerwise-normalized candidate,
`R4LayerwiseNormalizedRetainedReadoutLanguagePathV1`, is terminal
`LAYERWISE_NORMALIZED_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL`, not PASS. With
V1's representation, recurrence, initialization, seed `9738`, `252,160` learned
parameters, `23,040` f32 state values, data/order, and `2,730`-step/
`5,241,600`-decision optimizer dose fixed, the exact readout
`E @ [N(h) + (g / sqrt(2)) * (N(a1) + N(a2))]` used fixed `g=1` versus the
equal-work `g=0` control, zero new parameters/state, and one vocabulary matmul.
Candidate prompt gain was `0.02869802096506591` versus matched V1 at
`0.007331623694789724` (delta `0.021366397270276186`), with `339/512` wins and
own NLL `3.479876528760464` versus `3.6930405921095097`. It missed the frozen
absolute `0.04332169878499658` and incremental `0.025341569256760274` gain
floors. Fresh held-out NLL/top-1 improved to
`3.712641167679153`/`31.661826%` from
`3.8850003882891597`/`29.728138%`; state-off cost
`1.3495375636624845` NLL and `20,595` correct decisions. Mechanics, replay, and
all `13/13` fresh-process verification comparisons passed.
Candidate/population/reveal/result/verification CIDs are respectively
`blake3:8d31e15c355aade1ccc2592dc5fb1caf14a5f056862621e7b467858569a1c1e4`,
`blake3:165be397b73041afd39aa65ae796400ea539399f8586729ad19a168c4daa9e93`,
`blake3:079bee84db32513c5d6c0cb54cbff1e70b163902efa934d950204090985b3f5a`,
`blake3:35396bd6e64fc2c0bc7d86a84cc9e212ed913ce28e5353f5f2b8212b4cf2c532`,
and `blake3:3f316541dbab8061ed5ba891bf6a47ef22c55bca21fba01f6f97dbb3cb8497aa`.
See the [binding #973 layerwise-readout record](docs/r4_layerwise_normalized_retained_readout_prompt_capacity_973.md).
Generation, reasoning, lowering, and geometry-native lowering are `NOT_RUN`;
coherence, H4 superiority, exact/table lowering, browser readiness, and release
readiness remain unestablished.

The preceding separately frozen learned-associative successor completed
`LEARNED_ASSOCIATIVE_READOUT_NO_CAPACITY` at result CID
`blake3:cedba37738ee249457bb589f716ee75afb16a0c4937c2a22ae9f917dd3eb97c1`;
fresh-process verification passed at CID
`blake3:443d711ce9a228e26e2eb2eebb55c582848424e2677c3473d41deaf8afd69ec7`.
Across the frozen 512 prompt directions, geometric gain was
`0.0063767854348491465` with `299/512` wins, pooled gain was
`0.010263234571452827` with `324/512` wins, and V1 gain was
`0.006423652456300697`; neither learned arm met the absolute
`0.04332169878499658` or incremental `0.025341569256760274` capacity floor.
On 247,920 fresh-language decisions, pooled NLL/top-1 was
`3.8737562215878296`/`30.042756%` versus V1 at
`3.9036360153193317`/`29.628509%`; pooled state-off degraded to
`4.239191759767437`/`22.859793%` and lost `17,808` correct decisions. Preserve
that load-bearing fresh-language signal only as the matched non-geometric
control. Geometry attribution failed: geometric-minus-pooled gain was
`-0.0038864491366036808` with `209/512` paired improvements, and
geometric-minus-deranged gain was `-0.0002888663472835149` with `251/512`, both
below the required `308/512`. Mechanics, replay, causal access, and independent
verification passed, but no associative prompt capacity or geometry advantage
was established. That result motivated the terminal V5 write/binding campaign
reported above. Do not tune or retry this readout, and do not run generation
from it. #954's final source-free correctness terminal remains blocked; no
C1-SB6 is authorized. See the
[binding learned-associative record](docs/r4_learned_associative_readout_prompt_capacity_973.md).
ADR-0005 and the
append-only [#954 grounded-correctness record](docs/r4_grounded_correctness_954.md)
remain historical mechanism/evidence context. The
earlier frozen [#1019 parameter-capacity contract](docs/r4_softmax_parameter_capacity_1019.md)
and its [signed preflight result](docs/r4_softmax_parameter_capacity_preflight_1019_raw.json)
remain reference history.
It keeps the established ordinary causal R4/Spin Q/K/V plus stable-softmax
mechanism and changes only decoder depth from six to twelve layers: exactly
13,130,784 parameters, seed 1019, 16,800 optimizer steps, and 275,251,200
training tokens. Its population, fixed overfit smoke, and random-export
all-twelve-layer Rust preflight parity passed. The signed MPS probe passed memory at
`21.03%` but stopped `UNAVAILABLE_HARDWARE_BUDGET` on time at a safety-projected
`20.66 h` against the `8 h` ceiling. That terminal applies only to the frozen
offline PyTorch/MPS implementation; full training, final parity, reveal,
generation, and replay remain `NOT_RUN`. UOR's deployed architecture and
runtime remain CPU-native. Apple Accelerate/BLAS and MPS are permitted only for
local offline training, compilation, and bounded tests; CUDA and external GPU
execution are out of scope. A single isolated exact-shape MPS fast-path test
(10 warmup plus 40 measured steps) combined fused AdamW with deferred logging
and measured `4.485223 s/step`, slower than the signed `3.491307 s/step`;
`fused=True` was removed immediately. This is a bounded fast-path negative, not
a model result. Preserve the passed population, smoke, and parity artifacts,
but stop #1019 tuning and full-run work; #1019 is optional and paused. #954's
first grounding SFT failed `1/3`; `R4SourceSpanPointerV1` then passed 12/12
overfit and Python/Rust parity but failed all four frozen development gates
after its sole 256-step fit. The terminal is
`FAIL_SOURCE_SPAN_POINTER_DEVELOPMENT_GATE_STOP`; no final pointer artifact was
emitted, and product probes plus browser/HTTP wiring were `NOT_RUN`. Do not tune
or retry the revealed cosine head. Its frozen source-relative successor,
`R4SourceRelativeRelationHeadV1` (C1-SB2), is implemented and completed only its
cheap matched-transfer preflight. The fitted families scored 12/12 positive
relations, 20/20 negatives, and 6/6 supported copies; the independently sealed
families scored 5/12 positives, 14/20 negatives, and 0/6 copies. Same-source
matched-pair, query-swap, duplicate-agreement, and distinct-conflict controls
were false, so the run stopped before Rust parity, the sole 512-step full fit,
development, and product reveal. No final relation head exists. C1-SB3 then
moved supervision into all six attention layers and transferred most relations,
but missed its exact gate. C1-SB4's independently frozen full-source,
record-level structured-margin successor also failed: exact records were
`70/126` fit and `35/63` sealed; positive groups were exact, negative-group
specificity was `394/478` and `197/239`, and same-source query relocation was
not exact. It stopped before Rust parity, checkpoint emission, development, or
the four committed product probes; do not tune or retry it. C1-SB5
`R4PairedQueryCandidateMatrixV1` then fit all `56/56` paired records but reached
only `14/28` exact sealed pairs. Query-row-swap equivariance was bit-exact;
pair-mean-query and inference-time attention-off controls were each `0/28`.
The product population remained unopened, and checkpoint/binding-head emission,
Rust parity, development, and product evaluation were `NOT_RUN`. Terminal
`FAIL_PAIRED_QUERY_BINDING_PREFLIGHT` retires C1-SB5 without retry. It preserves
only bounded source-backed attention evidence; it does not establish generation,
reasoning, correctness, or a source-free runtime. #954's final source-free
terminal remains blocked behind #973, and #955 remains blocked behind #954. The
#1017 `r4 generate` path remains the working
ordinary-softmax generation prototype.
Prototype iteration uses
one targeted compile plus one real behavior check; do not run a broad local
suite or add a permanent gate until the mechanism is useful. The existing
mandatory merge-queue CI remains the single integration boundary rather than an
every-iteration loop. On the project M1, the opt-in
`local-inference-accelerate` CPU-BLAS build preserved the four generated token
IDs, output CID, and attention-audit CID while reducing internal generation
from `3.060506042 s` to `0.116236875 s`; use it for local #1017 inference while
keeping exact `uor-matmul` as the portable default. Offline
teacher/compiler floats, matrix operations, and softmax are allowed; deployed
runtime remains exact and source-free. The hosted Pages build is static,
currently reports WASM offline, and has no functioning chat backend/artifact
lowering; do not treat it as product evidence or let it replace this research
gate.

The H4-only `DirectCausalGeometricAttentionR4V1` scaffold now exists, but its first
8-case V2 result is `NON_PROMOTABLE_BUDGET_MISMATCH`: the nominally matched
plain/current arms had fewer effective degrees of freedom. The corrected,
pre-reveal-kappa-bound 12-case V3 used normalized R4 parameters for every arm.
It returned full H4 3/12, matched plain 12/12, current-only 6/12, an
inference-time coherent alternative-connection swap 10/12, key-isometry 7/12, order-shuffled 5/12,
and value-permuted 8/12. Thus the direct learning/softmax/value path works, but
the current mixed-gauge H4 parameterization/conditioning does not transfer and
is not a geometric oracle; the exact group action itself remains algebraically
valid. `ConnectionGaugeCovarianceV4` subsequently passed construction covariance
but failed its protected held-out gate: all three main arms were 13/24 and the
order/value/gauge destructive controls did not separate. V4 is frozen negative
evidence and must not be retuned. The #973 reference is `HELM-D-R4`, pinned to
the official MIT HELM-D architectural source at
`7501deca8f413848bfef804be64ce874b72a3cd7`. The now-qualified
`R4SoftmaxReferenceGeneratorV1` credits and adapts HELM's attention seam and
provenance; it does not port HELM's remaining geometric decoder stack. UOR's
existing pinned SmolLM2 `HuggingFaceLlamaOracle` supplies embeddings, RoPE,
residual/RMSNorm, MLP, final normalization, and the language-model head. No
HELM checkpoint or upstream generation code was executed in this gate. The released HELM
generation/cache path is incomplete. Its checkpoint and full
geometric decoder remain an optional external baseline behind a separate
tokenizer and license gate, and are not directly an R4-block runtime; do not
vendor upstream code or claim checkpoint parity in this documentation change.
Its bounded full-decoder run
preserves learned causal Q/K/V, ordinary stable softmax, value aggregation, and
`W_o` while transporting R4 blocks through exact cumulative Spin/H4 frames. It
passes exact donor/R4 replay, real-language behavioral parity, a live
frame-permutation control, and zero future reads. This establishes ordinary
softmax attention in R4/Spin frames, not geometric advantage.
`IntrinsicLorentzR4AttentionV1` attempt 02 subsequently stopped
`UNAVAILABLE_INTRINSIC_LORENTZ_R4_STOP_BEFORE_HELD_OUT`: its construction
barycenter-covariance audit measured `9.121400701417315e-08` against the frozen
`1e-08` ceiling, while diagnostic curved NLL was worse than both donor and flat.
D3 remained sealed. Source-faithful learned-manifold V2 then completed one valid
non-D3 construction-validation run at
`FAIL_HELM_D_MANIFOLD_CONSTRUCTION_REVISE_PROJECTION_SCORE_CENTROID_OR_TRAINING`.
Donor/gauge parity, deterministic replay, exact causal work, and all three
destructive-control separations passed, but learned-Lorentz NLL
`7.71061809923296` failed donor retention (`3.667626465210025`) and matched
learned-Euclidean parity (`4.483153905078387`). The controls establish
sensitivity only, not useful Lorentz attention. The 8/8-contract localization
attempt stopped at its two-document preflight with
`REJECT_TANGENT_READOUT_SELECT_SCORE_PREFLIGHT`: tangent readout increased
normalized audit MSE on both documents (pooled ratio `1.0643688804269025`).
Ordinary dot-product/stable-softmax causal attention in coherent R4/Spin frames
is the accepted current baseline. Intrinsic score/readout alternatives,
resonance-based softmax replacement, full-model recurrent lowering, and exact
deployment are parked. The provider-
free-at-execution, source-backed native CPU `R4SoftmaxReferenceGeneratorV1`
(`HELM-D-R4`) CLI gate passes:
4/5 frozen quality in both passes, 5/5 exact replay after timing removal, all
30 layers with exact causal/projection/R4 audits and zero future reads, and
source-donor reproduction (P1 through EOS; P2-P5 all 32 tokens). The terminal
is `PASS_R4_SOFTMAX_REFERENCE_GENERATION_ADVANCE_NATIVE_PRODUCT_BRIDGE`. Its
explicit opt-in, loopback-only dedicated native HTTP endpoint now passes the
frozen eight-token sunlight canary with the same token sequence, decoded text,
decision CID, persistent-state CID, all-30-layer exact audits, and zero future
reads as the CLI. Dashboard wiring, native-readiness gating, and static/WASM
isolation checks pass, but hosted Pages remains static/offline without a
functioning chat backend/artifact lowering. The feature is disabled by default
and does not change the default engine. The teacher-trace/Q16 suffix student,
its recurrent state successor, and #1012's observability rung are complete
bounded negatives. #1014 established load-bearing attention; #1017 closed
NLL-only negative and remains the working bounded generator; #1019 is an
optional frozen 12-layer parameter-capacity improvement whose model-side
population/smoke/parity subgates
passed, MPS is `UNAVAILABLE_HARDWARE_BUDGET` for the frozen eight-hour offline
implementation, and the full campaign remains `NOT_RUN`. The subsequent fused-
AdamW/deferred-logging fast path was slower (`4.485223` versus signed
`3.491307 s/step`), so `fused=True` was removed and #1019 is optional/paused.
The #1017 `r4 generate` path remains the working ordinary-softmax
raw-continuation prototype.
#1041 bounds its product presentation to raw single-turn story continuation:
do not add a source-backed history serializer or multi-turn/chat adapter around
that checkpoint.
#954 C1-SB2 through C1-SB5 are bounded negatives, not active answer artifacts.
C1-SB4's full-source structured-margin arm reached only `70/126` fit
and `35/63` sealed exact records and stopped before Rust/checkpoint/product.
Its product text remains unopened and it must not be retried. C1-SB5 then fit
`56/56` pairs but reached only `14/28` sealed; its products stayed unopened and
the rung retired before checkpoint/head/Rust/development work. CUDA and external
GPU execution are out of scope. This reference remains
transformer-compatible and `f32`/multiply/alloc/source-weight backed—not
table-native, multiply-free, or transformerless. It does not establish geometry
advantage, softmax removal, correctness, reasoning, frontier quality, release
readiness, or a static-WASM decoder. Product work does not wait for #1019: use
the bounded #1017 `r4 generate` path while keeping its claim limits explicit.
Do not resume resonance substitutes. Do not tune
the revealed V2/V3/V4 or learned-manifold
fixtures, relax the V1 covariance bound, or scale #997's rejected
componentwise-Frechet placement. The binding records are
[`docs/helm_d_r4_softmax_decoder_973.md`](docs/helm_d_r4_softmax_decoder_973.md)
and the completed
[`score-by-readout localization`](docs/helm_d_score_centroid_localization_973.md);
the [generation record](docs/r4_softmax_reference_generation_973.md), its
[compact aggregate](docs/r4_softmax_reference_generation_attempt_01_result_973.json),
the [native bridge result](docs/r4_softmax_reference_http_bridge_973.md),
and the V1/V2 negative records remain linked from there.
Intrinsic/readout alternatives, multi-resonance softmax replacement,
full-model recurrent lowering, and exact deployment are parked; #954's C1-SB2
preflight stopped before Rust parity/full fit/development/product, with no final
head. Its final source-free terminal remains blocked behind #973, and #955
remains blocked behind #954. Implementation progress is not a result.
The geometric causal decoder plan, prior S0–S7 completion plan, and
graph-compiler implementation plan are retained as historical
engineering/evidence records; none decides what is built next. Native GitHub
relationships now mirror the programme: #961 closed with reversible S0 state;
#952 stopped at `REDESIGN_ORDERED_ROUTE_SUMMARY`; #967 repaired the ordered
state but terminated `RETAIN_STATE_ONLY`; #970's corrected, target-free A1P
gate produced the bounded paired-H4-derived exact R4-heatmap result
`RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q` and closed through protected
PR #972. #969's mechanism-first pivot delivered one causal R4/S3 least-cost
route-attention mechanism and one matched two-unit decoded smoke at
`PROCEED_TO_I1_WITH_CAUSAL_R4_PATH_ATTENTION`. #953 implemented the first
bounded provider-free decode/render/append loop, but its initial smoke was an
exact rank-preserving lexical relabel of #969 and terminated
`REVISE_I1_GENERATOR_IN_PLACE`: it did not supply incompatible natural choices
or qualify grammar. `PrimaryThenAdjacentSpinFallbackV1` then repaired the
separately frozen natural agreement admission: I1/I2/ordered-sentence plus
divisor form the primary tier; adjacent-spin rows are always consulted and
report physical presence truthfully, but do not admit while primary support is
non-empty. The preflight recovered exact `{still}` then `{run,runs}` support
under equal work. One permitted four-arm run produced left/full `still run`,
right/full `still run`, and both state-disabled arms `still runs`, with
deterministic replay, so the terminal remains `REVISE_I1_GENERATOR_IN_PLACE`.
The first frozen local same-object, order-sensitive candidate-placement
preflight then reproduced 7/7 construction prototypes with zero class
collisions, but real placement selected 0/2 intended candidates while the
same-artifact placement-permuted control selected 2/2. Generation and replay
were `NOT_RUN`. #983 then froze `ConstructionCausalReturnV1` on an independent
three-family, six-decision population. Its usable construction classes were
pure, but construction coverage and the sealed strict ceiling were both 0/6.
It stopped `UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER` before a deployed selector,
payload inversion, or #953 generation. #983 is now closed as bounded negative
evidence. #986 then executed the one frozen `CorpusSignedTransportV1`
feasibility contract. The pinned raw corpus reproduced, but no exact
corpus-scale codec/three-way pair commitment existed, and the exact SpiralCore
control still supplied no complete same-frame lexical `O(x)` map or
compiler/query frame identity. #986 therefore closed
`UNAVAILABLE_FRAME_OR_POPULATION` before placement, diffusion, Gate 0, labels,
selection, or the historical #953 path. The later capability-first reset first
established B0/#989 and then executed the one permitted matched #953
intervention. `MultiscaleCountRadiusR4V1` improved held-out top-1 from
22.261404% to 23.211797% (+0.950392 percentage points, +4,242 correct) with
zero candidate-support/declared-work-ledger mismatches and byte-identical
replay. The external replay adjudication promoted each report's pending verdict
to `PROCEED_TO_A1Q_H_WITH_BOUNDED_SOURCE_FREE_GEOMETRIC_GENERATION`. This does not
rehabilitate the failed #983/#986 representations or establish semantics,
attention, correctness, reasoning, chat, or release readiness. #973 Gate 0 then
retained `PriorSentenceCountRadiusR4V1` at
`RETAIN_GATE0_PRIOR_SENTENCE_ATTENTION_CONTINUE_PARAGRAPH_CONVERSATION`. On two
synthetic D3-partitioned histories with identical local #953 evidence, its
prefix-before-final-period coordinate made exact earlier-candidate state
causally load-bearing: real decoded ` tea.` / ` coffee.` (2/2), scope-disabled
decoded ` coffee.` / ` coffee.` (1/2), and candidate-permuted decoded
` coffee.` / ` tea.` (0/2), with zero support/work mismatches and exact replay.
This establishes only one bounded lexical copy-attention mechanism, not
semantic or general paragraph/conversation/global attention. The next frozen
two-case synthetic paragraph slice then retained
`ParagraphEntitySpinPathR4V1` at
`RETAIN_PARAGRAPH_ENTITY_SPIN_PATH_ATTENTION_CONTINUE_CONVERSATION`. Neither
candidate token occurred in either prompt, the prior-copy operator abstained,
and #953 support/work stayed unchanged. The decoded matrix was real 2/2,
paragraph-disabled 1/2, entity-binding-permuted 0/2, and parsed-fact-vector-
reversed 2/2. Every candidate-relative H4 shell was `Coincident`; fiber was the
first sufficient discriminator, while torsion was retained and audited but did
not decide the ranking. The stored phases retain their upstream lexical
unit-ID/prime provenance; the new ranker adds no prime/hash placement and
does not establish prime/index-independent or intrinsic geometry. It also does
not establish semantic/paraphrase or natural-distribution transfer, anti-recall
beyond exact candidate absence, a general entity model, or general paragraph,
conversation, or global attention. The operator, target-free census, and
decoded-smoke identities are respectively
`blake3:9221efa7ad952e4890aae335970418b38ec93beb8cb4de65c5aa1d8c67f70afd`,
`blake3:515720686b96dbebc2f055f9a21d3f0684f76092018381c836b51abf47a4d197`,
and `blake3:0ba32e5fe26f1280ec2eef2b115023de52f2ef946882352311dcecc531d76a32`.
#973 then retained `ConversationEntitySpinPathR4V1` at
`RETAIN_CONVERSATION_ENTITY_SPIN_PATH_ATTENTION_CONTINUE_BOUNDED_GLOBAL`.
With current-through-paragraph and global identities/ordered states fixed, only
the older conversation binding changed. Its decoded matrix was real 2/2,
conversation-disabled 1/2, cross-turn-binding-permuted 0/2, and parsed-binding-
row-reversed 2/2, with zero support/work mismatches and exact target-free and
decoded replay. The complete stored-spin lexicographic path was load-bearing;
the run did not separately qualify an individual H4-shell, fiber, or torsion
coordinate. This remains one bounded synthetic, construction-bound exact-
descriptor cross-turn entity-role selector, not semantic, natural, general
conversation, or global attention. The operator, target-free census, and
decoded-smoke identities are respectively
`blake3:343c961b06605f6ae9bb6160ac34a98224991715b706156349a8fd544b6dbb35`,
`blake3:649d733a194469aa648101a873d9e2ee323266b18872ced412d1da2cc6a56635`,
and `blake3:6930de3c07d30df4420bb68e60ea74531c8076516bcfef1c016240eddf1b9ca2`.
The first independently frozen bounded-global exact-spin contrast stopped
target-free at
`RETAIN_CONVERSATION_ONLY_REDESIGN_BOUNDED_GLOBAL_EXACT_SPIN_RELATION`.
Its detached carriers retained four references, three classes, one same-address
reuse, distinct epochs/roots, one common lower artifact, and equal support/work,
but `Pavel`/`helix` share one H4 root and `prism` is identity, so both swapped
orders finish at the identical complete `-1`/fiber/torsion state. Real roles
were `helix/helix`; permuted roles were `prism/prism`. Target loads were zero
and decoded execution is `NOT_RUN`. Operator and census identities are
`blake3:f6b36cdf3e6cf96e1e9a345980843ee9eaffd25f5b864d4b4ed45a30ae6f746f`
and `blake3:6c0a9f89a29584a09d917ae427a494b53c06b76e56482f665870ae86c1cd130a`.
#973's independently frozen V2 repair subsequently retained one bounded
noncommuting exact-spin mechanism. Its first natural document-scale corpus
placement then passed target-free reachability but scored 2,931/35,028
(8.367592%), below unchanged #953 at 4,281/35,028 (12.221651%) and below both
order-shuffled and operator-permuted controls. The terminal is
`RETAIN_BOUNDED_GLOBAL_ONLY_REDESIGN_CORPUS_SPIN_PLACEMENT`. The bounded
gated-delta core later trailed plain delta on its sealed smoke. Direct-attention
V2 is non-promotable and equal-manifold-budget V3 isolated the connection/gauge
seam. V4 preserved construction covariance but failed held-out functional
binding. ADR-0005's `HELM-D-R4` reference/parity path subsequently passed and
remains qualified. Intrinsic V1 attempt 02 stopped unavailable before D3 on its
covariance audit. Source-faithful learned-manifold V2 then produced a valid
non-D3 construction-validation negative: learned Lorentz failed retention and
matched parity, while its controls established sensitivity only. The 8/8
contract's attempt stopped at its two-document preflight and rejected tangent
readout. Provider-free-at-execution, source-backed
`R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) generation and its explicit
opt-in, loopback-only dedicated native HTTP endpoint now pass. Dashboard
wiring/readiness and static/WASM-isolation checks pass, while hosted Pages
remains static/offline without a functioning chat backend/artifact lowering.
The Q16 suffix trace student, its recurrent state successor, and #1012's
observability rung are complete bounded negatives. #1014 established
load-bearing attention and #1017 closed NLL-only negative. #1019's optional,
paused 12-layer parameter-capacity campaign recorded population, fixed
overfit smoke, and random-export all-twelve-layer Rust preflight parity passed;
MPS stopped `UNAVAILABLE_HARDWARE_BUDGET` on time for the frozen eight-hour
offline implementation, and full training through replay remains `NOT_RUN`.
The fused-AdamW/deferred-logging fast path was slower (`4.485223` versus signed
`3.491307 s/step`), so #1019 tuning/full-run work stops and remains
optional/paused. The #1017 `r4 generate` path remains the working prototype;
#954 C1-SB2 and C1-SB3 are preserved negatives. C1-SB4 then trained the frozen
full-source structured-margin attention representation but reached only
`70/126` fit and `35/63` sealed exact records; it stopped before Rust parity,
checkpoint emission, development, or its unopened products. Do not retry it.
C1-SB5 subsequently tested that paired-query contrast: fit was `56/56`, sealed
was `14/28`, row-swap equivariance was bit-exact, and mean-query plus
attention-off controls were each `0/28`. Its products stayed unopened and the
rung retired before checkpoint/head/Rust/development work.
CUDA and external GPU execution are out of scope.
Intrinsic/readout, resonance, recurrence,
and lowering are parked. D3 remains `NOT_RUN`; #954's final source-free
terminal remains blocked behind #973, and #955 remains blocked behind #954.
Do not add a second #953 intervention or reuse the #983/#986 populations. The
complete #973 Gate 0 record is
[`docs/prior_sentence_count_radius_attention_973.md`](docs/prior_sentence_count_radius_attention_973.md).
The complete paragraph record is
[`docs/paragraph_entity_spin_path_attention_973.md`](docs/paragraph_entity_spin_path_attention_973.md).
The complete conversation record is
[`docs/conversation_entity_spin_path_attention_973.md`](docs/conversation_entity_spin_path_attention_973.md).
The complete bounded-global target-free negative record is
[`docs/bounded_global_exact_spin_attention_973.md`](docs/bounded_global_exact_spin_attention_973.md).
The complete #986 result and nonclaim boundary is
[`docs/corpus_signed_transport_attention_986.md`](docs/corpus_signed_transport_attention_986.md).
Terminology lives in
`docs/transformerless/GLOSSARY.md`. Keep this file current when conventions
change.

## Capability-first baseline and geometric increment — #989/#953 established; #973 native reference generation qualified

Effective 2026-08-28, #989 established the deterministic source-free
table-native lexical baseline at
`ESTABLISH_TABLE_NATIVE_LEXICAL_BASELINE`. Across 446,342 held-out known-target
positions, the table scored 99,362 (22.261404%) versus 24,163 (5.413561%) for
unigram, an uplift of +16.847843 percentage points. The 35,655,288-byte
artifact is frozen at
`blake3:ccdc399731cb866a329be478467a434cda4e445813421e5d17c21ccc87288297`;
two complete executions produced identical report bytes and artifacts.

The one permitted matched #953 intervention is now accepted. Its separate
24,250,680-byte overlay is frozen at
`blake3:914126a311c3984d1482258a8f0a7fa2e34896540d502d19f1d9076fbd4a9b76`.
Across the same held-out positions, it scored 103,604/446,342 (23.211797%), a
net +4,242 correct and +0.950392 percentage points over #989. It changed 56,280
known-target choices, with 6,753 geometry-correct versus 2,511 baseline-correct
among those changes. Candidate support and the declared-work ledger matched at
all 446,342 teacher-forced positions and through the first free-running
divergence. The structural source-closure counters were all zero, and two
complete executions produced identical table, overlay, and report bytes. The
external byte comparison promoted the reports' pending verdict to the frozen
positive terminal.

This establishes a bounded source-free geometric increment over a statistical
lexical baseline. It does not establish semantics, broad attention, general
coherence, correctness, reasoning, chat, performance, or release readiness.
The #989 table remains the frozen non-geometric reference; no second #953
formula, axis, prompt, or corpus run is permitted. #973 Gate 0 has retained one
exact-candidate prior-prefix copy mechanism. Its frozen paragraph slice
retained one construction-bound exact-descriptor/entity-binding stored-phase
path selector with absent candidate tokens and unchanged #953 support/work.
Its independently frozen conversation slice then retained one construction-
bound exact-descriptor cross-turn entity-role stored-spin path selector while
all lower scopes and global state stayed fixed. These are not semantic,
paraphrastic, natural-distribution, or general higher-scope evidence. One
independently frozen bounded-global exact-spin contrast then failed target-free
because its swapped states commute. V2 repaired that exact relation and
retained a bounded synthetic mechanism. PR #997 then rejected the first natural
componentwise-Frechet document placement against #953 and both destructive
controls. The first bounded `GeometricGatedDeltaRetentionR4V1` core then passed
its structural smoke but showed no advantage on its sealed construction
fixture: full geometric was 16/28 next-token and 55/112 association wins versus
plain delta at 23/28 and 98/112. This does not falsify geometric attention;
without a literal dense attention reference, the result confounds geometry,
compression, training, and recurrent factorization. Direct-attention V2 was
then rejected as `NON_PROMOTABLE_BUDGET_MISMATCH`. Its fresh equal-manifold-budget V3
returned full H4 3/12 against matched plain 12/12 and current-only 6/12; an
inference-time coherent alternative-connection swap returned 10/12. The ordinary learning
path is viable, but the current H4 parameter-gauge/conditioning seam is not.
V4 later preserved construction covariance but failed held-out functional
binding and destructive-control separation. The exposed `HELM-D-R4`
source-pinned full-decoder ordinary-softmax parity in transported R4/Spin frames
remains qualified on bounded real causal language. Intrinsic Lorentz V1 attempt 02
reached construction validation but stopped unavailable before D3 because its
barycenter covariance exceeded the frozen ceiling; its curved NLL was also
diagnostically worse than donor and equal-capacity flat. Source-faithful
learned-manifold V2 then completed a valid non-D3 construction-validation
negative: learned Lorentz failed retention and matched parity, although its
controls established sensitivity. The 8/8-contract attempt stopped at its
two-document preflight and rejected tangent readout. Provider-free-at-execution,
source-backed `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) generation and its
opt-in, loopback-only dedicated native HTTP endpoint now pass; dashboard
wiring/readiness and static/WASM-isolation checks pass while hosted Pages
remains static/offline without a functioning chat backend/artifact lowering;
the Q16 suffix trace student and `R4SoftmaxTraceStateStudentV1` are complete
bounded negatives. #973's recovered full construction run now qualifies a
bounded causal retained-attention component, but the exact decoder recipe did
not satisfy its frozen generalization criterion. Formal H4 specificity remained
`NOT_EVALUATED`; diagnostic scrambled-transport CE was `0.033049` nats better.
The subsequent `R4RetainedLanguagePathV1` qualified, but its paired-H4
addressing successor failed prompt capacity despite slightly better fresh-
language metrics and fewer construction collisions. The direct retained-state
readout then improved prompt and fresh-language metrics but missed both frozen
gain floors. Its sole layerwise-normalized successor also improved prompt and
fresh-language metrics but missed the same absolute and incremental gain
floors. The parameter-free readout ladder is closed. The separately frozen
learned-associative campaign then completed
`LEARNED_ASSOCIATIVE_READOUT_NO_CAPACITY` (result
`blake3:cedba37738ee249457bb589f716ee75afb16a0c4937c2a22ae9f917dd3eb97c1`;
verified
`blake3:443d711ce9a228e26e2eb2eebb55c582848424e2677c3473d41deaf8afd69ec7`).
Its pooled arm's load-bearing fresh-language improvement remains a
non-geometric control, not prompt-capacity evidence. The independently frozen
V5 predictive write/binding successor then stopped
`PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY` with exact independent replay. Its
fresh-language and integrity gates passed, but capacity, geometry attribution,
and delta-overwrite attribution did not; retire that law and
`STOP_WITHOUT_GENERATION`;
new route families, intrinsic/readout alternatives, resonance-based softmax
replacement, unqualified scale, and exact deployment remain parked. D3 remains
`NOT_RUN`; #954 and
later-stage work remain blocked.
See the
[#989 evidence record](docs/source_free_table_baseline_989.md) and
[#953 evidence record](docs/source_free_table_geometric_intervention_953.md),
then the [#973 Gate 0 record](docs/prior_sentence_count_radius_attention_973.md)
and [#973 paragraph record](docs/paragraph_entity_spin_path_attention_973.md),
then the
[#973 conversation record](docs/conversation_entity_spin_path_attention_973.md)
and [#973 bounded-global negative record](docs/bounded_global_exact_spin_attention_973.md).

## What this repo is

A local, CPU-first **geometric intelligence programme**. Geometry is the route
and the route is the data location. The active lane uses a pinned lexical codec,
registered prime atoms, semiprime transitions including `p^2` self-loops,
ordered n-lets, fixed-zeta R4/S3 state, Hopf observation, torsion, exact
`Z[phi]` radial shells, and the required structural/storage project bridge
`E8 = H4 x H4`, realized in code and serialization by the golden/Galois-coupled
icosian pair `H4 ⊕ phi H4`. The qualified local attention mechanism is
narrower: natural schema-2 adjacency supplies candidates, an ordered
unit-quaternion path on S3 supplies causal prefix memory, and exact path closure
plus lease age selects or abstains. #953 wraps that selector in one bounded
canonical decode/render/append loop. H4 is an exact finite S3 codebook here;
`H4 ⊕ phi H4` / E8 remains structural storage and control rather than the
attention score.

#986 did not assume that identity geometry was semantic geometry. It stopped
before constructing either semantic placement or the signed transport readout:
the population commitment and complete lexical SpiralCore operator frame were
unavailable. Its exact finite algebra remains addressing/transport/control
substrate only. No table-value or geometric arm ran, so neither may be promoted
from #986.

The intended destination is frontier-like useful local intelligence without
transformers, MoE/sparse learned routing, or dense matrix intelligence. That is
an aspirational research target, not a current capability claim. Spherical
harmonics are the project-level model for overlapping spin-state storage and
transport; R4/S3 and Hopf/S2 are the bounded compute/observation charts used to
operate on that field. Exact operator sharing reuses the existing signed
S3/Hopf/fiber/torsion `shared_class_kappa`; the current Hopf-octant/torsion
`SpinSector` is only a coarse lookup bucket. Direction-sensitive relations use
either a new exact `SpinTorsionState` relative relation or an explicitly bound
spin-to-H4 map; the existing H4 relative witness is prime-derived route state.
A versioned exact-spin global result is an immutable overlay over lawful
candidates, not candidate injection or a corpus broadcast. Call it harmonic
only after its identity binds basis, mode order, coefficients, quantization,
and transition law.

Corpus scale follows mechanism qualification; it never substitutes for it.
#986 was one bounded local placement/transport qualification, but it stopped
before freezing its CID-disjoint split. Within #973, `HELM-D-R4` now establishes
ordinary inclusive causal attention with learned Q/K/V/O, causal all-prefix
logits, R4/Spin frame transport, stable softmax, and transported value
aggregation. Intrinsic Lorentz V1 did not clear construction validation and
never opened D3. Source-faithful learned-manifold V2 then failed donor retention
and matched Euclidean parity on valid non-D3 construction validation, while its
controls established sensitivity only. The 8/8-contract attempt stopped at its
two-document preflight and rejected tangent readout. Provider-free-at-execution,
source-backed `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) generation and its
opt-in, loopback-only dedicated native HTTP endpoint now pass; dashboard
wiring/readiness and static/WASM-isolation checks pass while hosted Pages
remains static/offline without a functioning chat backend/artifact lowering;
the Q16 suffix trace student and `R4SoftmaxTraceStateStudentV1` are complete
bounded negatives. #973's CPU recovery qualifies a bounded causal
retained-attention component while rejecting promotion of the exact
complete-decoder recipe; no H4-specific advantage was established. The later
language-path baseline qualified, while paired-H4, direct/layerwise readout,
learned-associative, and finally V5 predictive write/binding capacity failed
their frozen promotion gates. The V5 action is `STOP_WITHOUT_GENERATION`;
intrinsic/readout alternatives, paired-E8, resonance-based softmax replacement,
and exact deployment are parked. Actual paired-E8 hierarchy, fiber, and torsion binding remains
`NOT_IMPLEMENTED`. Transport
overhead is reported explicitly rather than called equal work. The old
recurrent gated-delta core remains a negative historical comparator; it is not
an active retry lane. The terminal independently frozen group-addressed cell used
matched H4, cyclic, and destructive scrambled actions instead.
No construction result alone authorizes the held-out D3 join.
Freeze the operator family/schema, objective, scope semantics, neighborhood
contract, and induction rule; vary only declared parameter values under new
artifact/operator identities, and rerun the bounded gate for every structural
or placement epoch. Additional rows, hits, or trace activity are capacity/recall
unless the real arm changes a held-out anti-recall choice and matched controls do
not. A selector may provisionally append one candidate admitted from its cloned
observed state, but actual future/target data never enter inference. Deeper
hypothetical branches, rollback, and comparison belong to #955 after #954. Do
not launch corpus expansion while the required scope gate is negative or
blocked.

The active autonomous reference may execute pinned local source weights; it is
explicitly not the final serving path. That final path loads no source weights
and contains no transformer/self-attention, dense
matrix intelligence kernel, MoE, or sparse learned router. The learned
four-coordinate mixer remains only the negative G0/G1 comparator recorded by
#950/#951; #958 is retained positive foundation evidence at
`RETAIN_STORAGE_RECALL_ONLY` scope.

The multiplication-free TLA/R4G1 compiler, packed graph runtime, certifier,
proof assets, and dashboard remain in the repository as working historical
components and research comparators. They are not the active intelligence
sequencing path.

## Workspace layout

- `crates/uor-r4-core` — active prime-route math/manifest/attention foundation + historical transformerless runtime
- `crates/uor-r4-router` — active geometric memory/router + historical word-Markov decoder and dashboard backend
- `crates/uor-r4-graph-format` — R4G1 packed artifact format, two-stage validation, borrowed `GraphView`
- `crates/uor-r4-graph-compiler` — offline graph-compiler stages (observation, cover induction, packing)
- `crates/uor-r4-graph-certify` — offline certification/measurement (Gate C `score` harness, `score_runtime` reference scorer, certificates)
- `crates/uor-r4-graph-runtime` — `no_std` allocation-free R4G1 graph runtime (engine, routing, patch chains)
- `crates/uor-r4-graph-cli` — `r4 transformerless …` CLI stage dispatch (convert-r4g1, scenarios, corpus tools)
- `crates/uor-r4-model-source` — offline source teacher/comparator and historical forward/KV/trace runtime
- `crates/uor-r4-proof-model` — executable proof obligations + proof-status matrix
- `crates/uor-r4-api` — typed compile + engine library façade for downstream consumers (wraps the CLI-shaped stages; see its README)
- root package `uor-r4-wasm-router` — façade + `r4` CLI + local server/chat
- `docs/` — plan, RFC (`transformerless/R4G1.md`), baseline, threat model, explainers,
  and the per-issue measurement records (`docs/<topic>_<issue>.md`)

Documentation entry points, in the order a newcomer should read them:
`README.md` (what it is, quickstart, CLI/HTTP/config reference) →
`docs/r4_intelligence_completion_plan.md` (authoritative sequencing) →
`docs/geometric_intelligence_programme.md` (architecture and claim boundaries)
→ `CONTRIBUTING.md` (the short form of this file) → this
file (the full operating manual) → `docs/RESEARCH.md` (what is measured, closed
and open) →
`docs/MODEL_LIFECYCLE.md` (active decoder and historical compile lanes) →
`docs/CONFIGURATION.md` (every environment knob).

**Keep them true.** When a measurement revises a claim, correct it where it is
asserted — README, `docs/RESEARCH.md`, and the record itself — rather than
letting a superseded number survive because it lives in three places. Records in
`docs/` are appended to, not rewritten: the history of what was believed and when
is part of the evidence.

UOR standards (`uor-addr`, `UOR-Framework`) are **pinned git dependencies** in
`Cargo.toml` — a fresh clone builds with no extra checkouts. The
`uor_standards/` directory is legacy material excluded from the workspace
build (`Cargo.toml` `exclude`); its `.gitignore` entry blocks new additions,
but ~1,100 legacy files remain tracked in the tree (recorded 2026-08-18,
baseline audit).

## Decision checks (dormant by default)

```bash
cargo fmt --check
cargo check -p <touched-package> --all-targets --offline
cargo test -p <touched-package> --lib --offline
python3 scripts/check_claim_wording.py      # when claims/docs change
```

These commands are references, not automatic pre-commit work. Testing and QA
remain dormant until a product or release issue names the exact check, decision,
fixture identity, outcome actions, and resource budget. Do not run a focused
test merely because code changed. Do not run broad suites to create confidence
without a decision they can change.

Source-free attention probes, anti-recall controls, bounded product
transcripts, and serving censuses are activated by their programme stage.
Workspace, BDD, doctest, no_std, deterministic-rebuild, kappa, Gate C,
all-features, WASM, fuzz, Kani, conformance, audit, and corpus-scale suites stay
dormant unless the active product/release decision explicitly requires them.
Automatic QA is disabled. Pull-request and merge-group events emit only five
instantaneous ruleset-transport acknowledgements with no checkout or
verification work. They exist because immutable ruleset `19597522` requires
the historical names and queue; they are explicitly **not PASS evidence**.

The toolchain is pinned in `rust-toolchain.toml`: rustup-managed `cargo`
resolves the pin automatically, so an activated local check and the manually
dispatched workflow use the same toolchain. Caveat: a non-rustup Rust earlier
in `PATH` (e.g. Homebrew)
ignores the pin — verify `which cargo` resolves to `~/.cargo/bin/cargo`,
or run gates as `rustup run stable cargo …`. Bump the pin in a dedicated
PR (a bump can shift libm-sensitive teacher logprobs — see Gate E below).

## Execution-lane invariants (do not conflate)

- **Active geometric intelligence path:** compiler-side floating point and
  allocation are allowed while constructing witnessed charts, but source
  weights, source residual/MLP/LM-head execution, `uor-matmul` intelligence
  projections, transformers, dense matrix intelligence, MoE, and sparse learned
  routers are not serving dependencies. The pinned lexical codec may load
  vocabulary/normalization data without weights. Route manifests, hierarchy
  state, chart selection, and decode settings remain deterministic, and library
  boundaries retain typed errors.
- **Frozen TLA/R4G1 runtime:** XOR/AND/OR/shift/rotate/popcount/int
  add-sub/compare/table reads only. No multiply, divide, or float in its
  normative kernel; its steady-state prediction path remains allocation-free.
  Do not weaken those scoped guarantees while changing decoder code.
- **Transformerless is not multiplication-free.** A decoder may be called
  transformerless only when it invokes no source-attention operator, contains
  no dense full-prefix Q·K matrix/softmax kernel, and uses bounded geometric
  support shown load-bearing by disabled/permuted interventions.
  P-4/table lowering is a later, separately triggered decision.
- **Artifact determinism:** identical pinned compiler inputs still produce
  identical historical artifact bytes. New route manifests and transitional
  decoder checkpoints must bind their source, tokenizer, compiler or training
  configuration, and semantic parameters.
- **Errors**: library boundaries return `Result` with focused error enums;
  no `unwrap`/`expect`/panic on recoverable paths. No unsafe in the portable
  runtime or the format crate (`#![forbid(unsafe_code)]` there).
- **Claim language**: `docs/formal_vocabulary.md` (v0.1.0+) is normative —
  equations are labeled Definition/Objective/Guarantee/Assumption/Empirical
  Criterion, guarantees carry a proof-matrix status, and
  `python3 scripts/check_claim_wording.py` (available but dormant unless a
  product/release decision activates it) blocks
  "machine-verified"/exact-equivalence wording without a linked proof artifact.

## Active product and research rules

- Follow the reconciled #820 dependency chain. #961 closed GI-1/S0 lexical
  geometry at reversible-state scope. #952's A1.0 gate preserved candidate and
  value reachability but stopped before a scorer because the reusable
  non-digest summaries erase earlier order. #967's A1R delivery added the exact
  associative ordered fold and passed the frozen scope, global, fold,
  incremental, and support contracts. Its full arm produced distinct `ll`/`rr`
  relative states on 6/6 queries and changed the same-candidate state in 5/6
  paired comparisons, but the scalar shortest-Cayley-distance readout collapsed
  both candidates to energy 2 and tied on 6/6. Its terminal verdict is
  `RETAIN_STATE_ONLY`
  (report `blake3:f0db7a5d5c81d51ebf3b4bf8a2715c4960ec16b14161e8bf7598d7b98c48c881`).
  #970's corrected target-free preflight then enumerated the complete paired-H4
  domain: 120×120 = 14,400 ordered pairs, 120 relative `D=X*Y^-1` rows, 45
  exact signed `(1,i)` R4-heatmap classes, and 480 typed-null pairs. Across 36
  fixture decisions it exercised 14 classes; construction coverage was 12/12
  and pure, construction classes covered 10/12 validation decisions, the
  no-class-splitting oracle ceiling was 10/12, strict construction transfer was
  0/6, and eight exact heatmap classes were incompatible. The hard gate stopped
  before scalar search; every downstream selection, control, and placement row
  is `NOT_RUN_IDENTIFIABILITY_HARD_STOP`, not PASS. The terminal literal is
  `RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q`. Its contract, universe, and
  report identities are respectively
  `blake3:2daacf538c022fab9580d1e124af6c18d0b06da04604fbc962a01bda57f08a98`,
  `blake3:dca725c0ec6060166bcd0023df956e1ff029661b5fa7800ccb9f20808712b796`,
  and `blake3:5f9239150dea8c0c27c4dfa6ad2e4d0068bc3d18afc127b315c0ec358ceddb3f`.
  This is only a bounded heatmap-readout identifiability negative: fixed-zeta
  phases, ordered n-lets, exact `phi` radial transport, and typed geometry
  adapters remain structural, diagnostic, or control state. It does not
  promote attention or generation. #970 closed through protected PR #972.
  #969 then delivered one causal R4/S3 least-cost route-attention mechanism and
  one matched decoded smoke. #953 has implemented a bounded source-free
  library/CLI decode/render/append loop. Its rank-preserving relabel smoke
  terminated `REVISE_I1_GENERATOR_IN_PLACE`; the later
  `PrimaryThenAdjacentSpinFallbackV1` repair recovered exact `{still}` then
  `{run,runs}` primary support under equal work while still consulting and
  truthfully tracing non-admitting adjacent-spin rows. The one permitted
  four-arm run produced `still run` for both full-path prompts and `still runs`
  for both state-disabled prompts, with deterministic replay. The terminal
  therefore remains `REVISE_I1_GENERATOR_IN_PLACE`, now localized to
  candidate-relative representation/scoring rather than admission or state
  starvation. The first frozen local candidate-placement preflight then
  reproduced 7/7 construction prototypes with zero class collisions, but real
  placement selected 0/2 intended candidates while its same-artifact cyclic
  placement control selected 2/2; generation and replay were `NOT_RUN`. #953
  was then blocked by #983, whose independently frozen
  `ConstructionCausalReturnV1` produced pure usable construction classes but
  transferred to 0/6 held-out decisions. #983 stopped
  `UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER` before deployed selection, payload
  inversion, or #953 generation and is now closed as bounded negative evidence.
  #986 then stopped `UNAVAILABLE_FRAME_OR_POPULATION`: its raw corpus
  reproduced, but the exact population/codec commitment and a complete
  same-frame lexical SpiralCore operator map were unavailable. Placement,
  Gate 0, labels, selection, and the historical #953 path were `NOT_RUN`. The
  later B0/#989 reset and separate matched #953 table-tie intervention have
  since passed. #973 Gate 0 has now retained one bounded exact-candidate
  prior-prefix copy-attention mechanism, the frozen paragraph slice retained
  one exact-descriptor/entity-binding stored-phase path selector, and the
  frozen conversation slice retained one exact-descriptor cross-turn entity-
  role stored-spin path selector. The first bounded-global relation failed
  target-free; its V2 repair later passed, the first natural placement failed,
  and the bounded gated-delta core trailed plain delta. Direct-attention V2 was
  non-promotable; equal-manifold-budget V3 then rejected the current mixed-gauge
  H4 projection/connection/optimizer combination against a working plain path
  and an inference-time coherent alternative-connection swap. V4 then passed
  construction covariance but failed held-out functional binding. `HELM-D-R4`
  ordinary-softmax parity in transported R4/Spin frames subsequently passed and
  remains qualified. Intrinsic Lorentz V1 attempt 02 stopped unavailable before
  D3 on covariance, with diagnostic NLL worse than donor and flat.
  Source-faithful learned-manifold V2 then produced a valid non-D3
  construction-validation negative: learned Lorentz failed retention and
  matched parity, while its controls established sensitivity only. The 8/8
  contract's attempt stopped at its two-document preflight and rejected tangent
  readout. Provider-free-at-execution, source-backed
  `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) generation and its explicit
  opt-in, loopback-only dedicated native HTTP endpoint now pass. Dashboard
  wiring/readiness and static/WASM-isolation checks pass; hosted Pages remains
  static/offline without a functioning chat backend/artifact lowering. The Q16
  suffix trace student is complete with bounded distillation but looping output.
  #1019 is an optional frozen 12-layer, 13,130,784-parameter ordinary-softmax
  R4/Spin quality-capacity improvement. The model-side
  population/smoke/parity subgates passed; MPS stopped
  `UNAVAILABLE_HARDWARE_BUDGET` on time for the frozen eight-hour offline
  implementation, and full training through replay remains `NOT_RUN`. The
  fused-AdamW/deferred-logging fast path was slower (`4.485223` versus signed
  `3.491307 s/step`), so #1019 tuning/full-run work stops and remains
  optional/paused. The #1017 `r4 generate` path remains the working prototype;
  #954 C1-SB2 and C1-SB3 are preserved negatives. C1-SB4's independently frozen
  full-source structured-margin attention arm reached only `70/126` fit and
  `35/63` sealed exact records and stopped before Rust/checkpoint/development/
  product; do not retry it. C1-SB5 then fit `56/56` paired records but reached
  only `14/28` sealed, with bit-exact row-swap equivariance and `0/28`
  mean-query/attention-off controls. Its products remained unopened and the rung
  retired before checkpoint/head/Rust/development work. CUDA and external GPU
  execution are out of scope.
  Intrinsic/readout alternatives,
  resonance-based softmax replacement, full-model recurrent lowering, and
  exact deployment are parked. D3 remains `NOT_RUN`.
  #954's final source-free terminal remains blocked behind #973, and #955 remains
  blocked behind #954. #954 and #955 own correctness and reasoning respectively.
  #962 owns durable multi-turn CLI/HTTP chat, persistence, isolation, and
  hive-memory; #963–#965 then own optimization, formal closure, and release.
- Sequence strictly from the current reset: working source-free table baseline
  (#989, established) → one matched geometric intervention (#953, accepted) →
  direct geometric attention (#973: retained bounded scope evidence → H4
  scaffold/V4 held-out negatives → HELM-D-R4 full-decoder softmax parity →
  intrinsic Lorentz V1 construction-unavailable → valid non-D3
  learned-manifold V2 construction-validation negative → tangent-readout
  localization rejection → qualified provider-free-at-execution,
  source-backed `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) generation →
  verified opt-in, loopback-only dedicated native HTTP endpoint (dashboard
  wiring/readiness and WASM isolation PASS; hosted Pages static/offline without
  chat backend/artifact lowering) → construction-only layerwise oracle traces
  [COMPLETE] → source-free Q16 suffix student [BOUNDED DISTILLATION; LOOPING] →
  `R4SoftmaxTraceStateStudentV1` [COMPLETE; FAIL PROMOTION] → #1012
  observability [COMPLETE; INSUFFICIENT SUPPORT] → #1014 direct attention
  [ATTENTION PASS; QUALITY FAIL] → #1017 exposure continuation [NLL-ONLY FAIL]
  → #1019 frozen 12-layer parameter-capacity campaign [MODEL SUBGATES PASS;
  FROZEN OFFLINE MPS IMPLEMENTATION OVER 8 H; FUSED FAST PATH SLOWER; OPTIONAL/
  PAUSED; FULL CAMPAIGN NOT_RUN; CUDA/EXTERNAL GPU OUT OF SCOPE]) →
  working bounded #1017 `r4 generate` prototype → #973 qualified retained
  language path → rejected paired-H4/direct/layerwise/learned-associative
  capacity seams → V5 predictive write/binding
  [`PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY`; `STOP_WITHOUT_GENERATION`] →
  #954 source-span pointer development negative → C1-SB2 source-relative
  relation preflight negative → C1-SB3 bounded attention transfer / exact
  preflight negative → C1-SB4 structured-margin negative → C1-SB5 paired-query
  binding negative → final source-free correctness terminal [BLOCKED BEHIND
  #973] → reasoning [BLOCKED ON POSITIVE CORRECTNESS] → optimization/purity/
  release. The older placement/transport
  sequence is retained as evidence, not an active implementation queue.
- Kappa is canonical identity/serialization, never the tokenizer or semantic
  distance. The pinned lexical codec is provenance-bound but opens no weights.
- Preserve the project shorthand `E8 = H4 x H4`. Its concrete implementation
  and serialization is the golden/Galois-coupled icosian pair
  `H4 ⊕ phi H4` with fixed basis, glue, forward map, and inverse witness.
- Keep **required structural/storage representation** distinct from a
  **qualified semantic scoring term**. Paired-H4/icosian coordinates may remain
  mandatory for canonical storage, address reconstruction, and inverse
  witnesses without being valid ranking features. Hopf, H4, zeta, icosian,
  SpiralCore, trajectory, hypersphere, winding/window, projection-energy,
  shared-factor, and resonance terms remain storage fields, diagnostics, or
  future hypotheses. #969 establishes only its local ordered-S3 path mechanism
  as load-bearing. #953's historical path code establishes reusable
  decoded-loop plumbing and tiered admission, but its unchanged full-path
  choice did not establish natural grammar. The separate B0 table-tie
  intervention now establishes only its bounded geometric accuracy increment
  and decoded comparison. #973 Gate 0 adds one exact-candidate prior-prefix
  copy mechanism. Its paragraph slice adds only one construction-bound exact-
  descriptor/entity-binding stored-phase path selector, and its conversation
  slice adds only one construction-bound exact-descriptor cross-turn entity-
  role stored-spin path selector. They do not qualify a general entity,
  paragraph, or conversation model. The first global exact-spin relation failed
  target-free; only a newly frozen #973 repair may qualify bounded-global state.
- Keep **candidate admission** separate from **harmonic influence**. In #953,
  `PrimaryThenAdjacentSpinFallbackV1` uses I1/I2/ordered-sentence plus divisor
  as the primary admission tier. Adjacent-spin rows are always consulted and
  report physical presence truthfully, but admit candidates only when that tier
  is empty. Do not delete or disguise a physical adjacent-spin hit.
  A newly frozen #973 repair may apply one global-epoch/operator-bound result to every immutable
  reference in the same exact signed-S3/Hopf/fiber/torsion class. Similar but
  non-identical states require a separately frozen finite relative-angular
  kernel built independently over exact classes. The existing adjacent-spin
  rows remain retrieval fallback/diagnostics, not operator coefficients.
  Neither operator mechanism may widen #953 support.
- An exact kappa miss must not collapse unseen global history to a suffix-only
  default, but global ordered-state behavior is tested on an independently
  frozen global-snapshot permutation rather than by mutating session history.
- The completed #969 evidence compares exactly full retained path, last-only,
  and state-disabled. #953's repaired natural agreement run carried full path
  and state-disabled arms under equal support/work: both full-path prompts chose
  `still run`, while both disabled prompts chose `still runs`. That deterministic
  negative localized the next revision to candidate-relative
  representation/scoring; it did not qualify incompatible natural choices.
  Do not retroactively add a broad construction/validation programme, channel
  census, weight sweep, control matrix, or higher-scope fixture to the completed
  #969 smoke or #981 tiered-admission decision. The frozen #953 placement
  revision permitted exactly one tiny pre-frozen construction/evaluation
  separation: construction-only observed transitions compiled the overlay and
  a label-free, selection-blind raw relation census froze before expected
  continuations were attached; frozen evaluation labels could not tune it.
  Full-history disjointness did not supply operative-representation anti-recall:
  the decisive suffixes exactly recalled shorter construction subhistories. Its
  exact preflight selected the opposite candidate on both prompts while the
  placement-permuted control selected both intended candidates, so it stopped
  before decoded generation or replay. That failure does not authorize a second
  representation, a wider split, or broader scope under the same contract.
- Teacher output may label or compare only after a source-free report freezes.
  It is never substituted for the product response.
- #973 Gate 0 selections emit their exact prefix-coverage and matched-work
  witnesses. The retained paragraph selector emits its exact two-fact binding,
  stored-phase path, and matched-work witnesses. The retained conversation
  selector emits its cross-turn binding, lower/global-scope isolation, stored-
  spin path, and matched-work witnesses. The first global relation emitted its
  detached-carrier/exact-class census and commuting-fold negative; later
  hierarchy selections must emit
  their corresponding scope coverage witness. Exact recall,
  grammatical generation,
  correctness, and reasoning remain separate gates.
- Start with the smallest product artifact that can falsify the stage. A
  negative stops or redesigns it; it does not authorize a larger harness.
- Do not add a graph section, proof lane, benchmark framework, BDD suite, or
  corpus-scale run before the active product decision requires it.
- Testing/QA is dormant by default. Activate only named product/release checks;
  missing or unrun evidence remains `NOT_RUN` or `UNAVAILABLE`.

## Historical release-only κ reproduction reference (dormant)

Do not run this during ordinary development. It is retained only for a future
release issue that explicitly activates the cross-platform κ decision.

- Setup (once per machine): `curl -sL -o /tmp/run.com
  https://github.com/trholding/llama2.c/releases/download/experimental/run.com
  && cd /tmp && unzip -o run.com out/model.bin -d ref`
- Run: `TLESS_CANONICAL_DETERMINISTIC=1 cargo test -p uor-r4-core --release
  --offline --test kappa_reproduction -- --ignored` (the canonical mode is
  required for the cross-platform Gate E claim; check
  /tmp/ref/out/model.bin exists before trusting a green result).
- The certificate fixture is re-pinned under the portable canonical math path.
  Legacy accelerated teacher builds remain platform-sensitive and are not the
  cross-platform reproducibility claim.
- Re-pinning is a **maintainer decision**, done via
  `dump_baseline_kappa` (`--nocapture`) → review diff → adopt →
  `TLESS_REPIN_WRITE=1` regenerates the fixture container. Compiler redesigns
  legitimately change κs; drift from nondeterminism never does — investigate
  first (double-compile determinism check), then re-pin.

## Historical TLA/R4G1 teacher-parity certification (dormant)

The commands below are records of the historical lane. They require an
explicit product/release decision before execution.

The suite below remains valid for the frozen compiled-runtime evidence lane. It
is not an entry gate for geometric-decoder development and must not be added to
routine decoder PRs.

`features/suites/teacher_parity_benchmarks.feature` (steps in `tests/bdd.rs`)
runs the live SmolLM2-135M teacher against both compiled runtimes (legacy TLS
store and R4G1 graph) on teacher-forced accuracy (top-1 / top-8 recall /
Δbits), generation speed, and kernel invariants (zero-multiply op census,
zero-alloc hot path, witness self-consistency), κ-pinning every input. A
corpus-replay scenario (S6) additionally measures in-distribution top-1
against the recorded teacher labels in the bundle's `corpus.meta` /
`corpus.records` through the deployed paths — no live teacher — reporting
next to Gate C's anchors (Gate C scores a held-out partition with the
compiler-side plain baseline; S6 replays recorded positions, so its ~0.43
figures sit above the 0.181 anchor by construction). It runs
in the default `cargo test --test bdd` when `.uor-models/sources/
smollm2-135m-instruct` and the compiled bundle are present. If a conditional
fixture is absent, that evidence is **UNAVAILABLE** even when the enclosing test
process exits successfully; never report the unexercised parity scenario as
PASS.
Budgets: `R4_PARITY_POSITIONS` (256), `R4_PARITY_GEN_TOKENS` (8, a hard
adaptive ceiling), `R4_PARITY_RUNS` (1), `R4_PARITY_CORPUS_POSITIONS` (1000).
Thresholds are pinned empirical floors with ~20%
margin; the ~1% top-1 figures are out-of-distribution honesty, not a bug —
the suite's 8 prompts are novel text, unlike Gate C's same-corpus replay
(see the comment above the constants in `tests/bdd.rs`).

The fixture-present live-teacher work is required to be an exact-parallel,
multi-stream host measurement, not a single-stream latency benchmark hidden
behind an intra-forward thread pool. `S = R4_PARITY_STREAMS` is the independent
private-state trajectory/batch width; `W = R4_PARITY_WORKERS` is the one
persistent exact output-row worker pool. Scientific coverage stays fixed at
eight canonical lanes in an `S = 8` shared-weight batch. `S` and `W` are
independent: the bounded tuner compares the host's all-logical-CPU width with
its four-worker candidate (deduplicated when equal) over the same eight-lane
work and selects the faster exact point. On the binding M1 these candidates are
`W = 8` and `W = 4`; neither width is a utilization quota or performance goal.
A physical teacher
batch must advance all `S` states through shared immutable weights while the
`W` pool divides output rows only; no worker may split or reassociate a row's
pinned exact dot-product reduction. Compiled candidates must receive the same
lane seeds and logical workload, and all results must reduce in canonical
prompt/position order. The shared teacher transcript also retains the S4 prefix
states, eliminating duplicate teacher prefill and the independent S4 warm-up.

Every live run must emit flushed JSONL progress events, deterministic evidence,
and a final JSON report with fixture identities/status, actual tokenized work,
configured/effective/current/peak stream and worker occupancy, complete
physical-batch/logical-forward/matrix/tile/cell/scalar-term accounting,
per-lane state/output identities, elapsed/rate/ETA basis, CPU/RSS readings, a
retained-workspace capacity/growth ledger, and a typed final `PASS`, `FAIL`,
`UNAVAILABLE`, `ABORTED`, or `NOT_RUN` verdict. Model, transpose/output, and
per-worker exact scratch buffers are prepared outside timed work; any capacity
growth during a measured forward fails the steady-state evidence. A heartbeat
must continue while an individual exact forward is in flight; its liveness and
ETA use monotonic in-flight exact scalar-term progress (worker-task progress is
the fallback), while completed-forward throughput remains a separate rate. The
bounded live tuner compares equal S=8 work at W=available/W=4 without full-model
candidate warm-ups, establishes exact trace equality plus owner-plan
reconciliation, and selects the faster exact point. W=1/2/4/8 equality remains
a focused structural gate. Speedup and CPU utilization are recorded diagnostics
rather than admission floors. Full work launches only when the selected exact
point has complete evidence and a safety-adjusted projection below the
configured hard wall ceiling, capped at eight hours. S4 starts with one causal
decode step per lane and extends through 2, 4, then 8 only while more work can
change its verdict. Any missing or failed evidence refuses the full run. See
`docs/teacher_parity_parallelism_932.md` and `docs/CONFIGURATION.md`.

The exact teacher, pinned `uor-matmul` crates, and both compiled S4 engine paths
have narrow `profile.test.package` opt-level 3 overrides in the root manifest.
Do not remove them and then interpret an opt-level-0 BDD rate as serving
performance. The rest of the workspace retains the normal test profile.

Before spending any live-teacher work, run
`R4_PARITY_PREFLIGHT_ONLY=1 cargo test --test bdd --offline`. This teacher-free
gate parses the tokenizer and every compiled prerequisite, exercises all eight
canonical legacy and graph seeds through typed deployed decisions, and writes a
content-bound `uor-r4.teacher-parity-preflight/1` success or refusal artifact
before exiting. The ordinary BDD fixture loader publishes the same artifact
before it can open the teacher. Refusals retain the exact reason, safe input
paths/CIDs, `teacher_source_opened=false`, and `teacher_forwards=0`; an
unwritable artifact path is itself a visible failure. A failed preflight blocks
the tuner and full suite; it is not bypassed as a fixture skip. The artifact's
`authorizing_contract_cid` binds the current executor, BDD, model, manifest,
and toolchain sources. Direct tuner invocation validates that binding plus the
selected paths and current compiled-input plus complete production-admission
CIDs before loading teacher weights.

## Process conventions

- **PR workflow; queue as transport only.** Do not push directly to `main`;
  use a named branch and PR. Ruleset `19597522` cannot currently be edited, so
  the repository emits its five required names as explicit no-QA
  acknowledgements and uses the forced merge queue only to transport the
  reviewed commit. Never report those acknowledgements as tests or PASS.
- **Dormant governance cleanup (#940).** A future administrator may remove the
  obsolete ruleset and its queue. Until then, contributors with write-only
  permission use the transparent transport shim; they must not fabricate
  external statuses, use `--admin`, or reactivate development QA.
- **Release activation only.** A future product-ready release issue may
  manually dispatch a bounded product/release QA scope and may propose a new
  minimal release ruleset. The old always-on research queue is not restored by
  default.
- **Per issue**: assign yourself (WIP signal) → branch `issue-<n>-<slug>` →
  work + produce the declared product/release evidence → run only checks the
  issue explicitly activated → open PR → merge through the protected path →
  close the issue with its evidence and merge commit. Milestones mirror plan
  phases.
- **PR review** (incl. Copilot-generated): review the changed path and its
  declared evidence before merge. Run no QA by habit; use only the activated
  product/release checks. Resolve conflicts
  hunk-by-hunk — whole-file `checkout --theirs/--ours` has silently dropped
  upstream features before (the TLA5 incident).
- **Committing while subagents work in-tree**: add files **by name**, never
  `git add -A` — in-flight agent work (unregistered modules, half-written
  tests) must not be swept into unrelated commits (the cover.rs incident).
- **Tests that encode era sensitivity**: `src/tless_uor.rs`
  `indexing_and_generation_update_store` asserts resolution depths that depend
  on the fixture artifact's class signatures — update the expected depths with
  an era note whenever the fixture is regenerated.
- **ScoreQ**: there are intentionally two compatible definitions in the frozen
  graph lane
  (`uor-r4-graph-format::ScoreQ` wire newtype; `uor-r4-core::score_q::ScoreQ`
  with compiler-side f32 conversions). Do not add a third or prioritize their
  consolidation ahead of the active route-native intelligence sequence.

## Long-run discipline (process amendment, 2026-08-06)

Compiles and Gate C runs at corpus scale cost hours. The waste is never the
run itself; it is launching one whose result could not have changed what we
do next. Three gates, in order, before any run measured in hours:

**One — reachability arithmetic.** From numbers already in hand, compute the
ceiling on the metric the run intends to move, and write it in the run
contract. Worked example (#460, 2026-08-06): the record showed 97.9% of
held-out positions resolving as ExactContext, so at most 2.1% ever touch the
graph path, so ANY cover-side change is capped at about 2.1pp of headline
movement. That is a five-minute calculation and it invalidates a four-hour
run. If the ceiling is below the effect you are hoping for, do not launch.

**Two — the cheap instrument is a hard gate.** Where an instrument exists
that reports the structural precondition, it runs FIRST and its verdict is
binding. For graph experiments,
`cargo test -p uor-r4-graph-certify --test capacity_scaling -- --ignored`
takes about twelve minutes and prints a SATURATION verdict per structure. For
decoder experiments, use the issue's tiny-overfit, reachability, or short-rollout
preflight instead; do not run a graph instrument that cannot decide the decoder
question. If the relevant instrument fails, the long run does not launch. On
2026-08-06 the graph instrument reported
`records_per_full_key: 36.02 SATURATED` and `exct.supported_record_fraction:
0.9882 SATURATED` before a multi-hour Gate C run that then confirmed exactly
what those two lines already implied.

**Three — pre-declare the decision, not just the exit rule.** Exit rules
("positive if at least 2pp") say how to read the number. A run contract also
says what each outcome CAUSES. If the positive and the negative branch lead
to the same next action, the run has no decision value; drop it or redesign
it until they differ.

**Run contract** — paste into the issue before launching, and post the
outcome against it afterwards:

    metric to move:      <name, current value>
    reachability ceiling: <arithmetic, with the numbers it came from>
    instrument + verdict: <which cheap test, what it must report to proceed>
    exit rule:           <threshold, pre-declared>
    if positive:         <the next action>
    if negative:         <the next action, and it must differ>
    cost estimate:       <wall-clock, and what else it blocks>

**Calibrate substantial offline compute; never silently accept one-core
execution.** Before launching a deterministic offline training, compilation, or
measurement job, predeclare a small set of materially plausible, scientifically
eligible plans, then benchmark one representative unit: CPU BLAS/Apple
Accelerate, MPS only when the frozen contract allows it, and selected
intra/inter-op thread and process/worker counts.
Select the measured-fast stable plan that preserves the declared result and fits
memory. Maximum threads or concurrent arms are not automatically faster; use
sequential arms when shared-memory contention wins. Record hardware, backend and
BLAS provider, intra/inter-op threads, worker processes, utilization, unit
timings, determinism/equivalence evidence, and the selected plan in the run
contract. A substantial job must not default to one core without measured
evidence that one core is fastest or a scientific constraint requiring it.
CUDA is eligible only when the active issue explicitly places it in scope.
Offline acceleration never changes the CPU/table-native deployed-runtime target.

**A long run must be observable before it starts.** Anything expected to exceed
15 minutes needs a finite work denominator, completed/remaining units,
throughput, an ETA derived from that denominator, durable checkpoints, and a
typed terminal report. A missing denominator, absent ETA, non-resumable
checkpoint, or worker setting that has not passed a one-worker/four-worker
semantic-equivalence, useful-worker utilization, and measured wall-time
improvement canary prevents launch. #958's final schema-2 complete-manifest
canary passed on 2026-08-26; its exact binding may be reused only while semantic
inputs and workload shape remain unchanged. A change must re-establish the
binding before the dependent product decision. Performance
evidence comes from release builds; a debug run cannot authorize larger work.
Eight hours is a hard kill ceiling, not an estimate. Reaching it stops the run
and records `ABORTED`, `NOT_RUN`, or the last completed bounded result; never
continue because the process may be nearly finished.

**Cross-target checks are scoped certification.** A native workspace check does
not build WASM. Activate `cargo check --target wasm32-unknown-unknown -p
uor-r4-wasm-router --lib` only when a product/release decision explicitly names
the WASM boundary; it is not routine implementation work. A
filesystem-touching helper gated `#[cfg(not(target_arch = "wasm32"))]` needs a
wasm counterpart, or every caller has to become cfg-aware; prefer the
counterpart. This was found the expensive way on PR #470, where PR checks were
green and the queue build failed.

**The forced queue is transport, not QA.** The five compatibility jobs perform
no checkout or verification and carry no product evidence. A silent or blocked
merge command is not authorization to use `--admin`, run dormant QA, or
manufacture external contexts; inspect the queue/check state and keep the
status language exact.

**Issue hygiene that goes with it.** Every issue filed mid-run gets an owner
and a named next action, or it gets closed with its record. Assignment means
actively-working-now; unassign when a track parks so the board reads true for
everyone. A PR that ships only part of an issue's scope says "References #N",
never "Closes #N" — GitHub will auto-close the issue on merge and the
unfinished half loses its home.

## Batch flow for small issues (process amendment, 2026-07-29)

Small, low-risk issues (docs, help text, certifier-side rows, test
harnesses, telemetry) are worked on ONE integration branch (`batch-N`)
with one commit per issue (message refs `#N`). Run only product/release checks
explicitly activated by the batch contract, once per batch of 3-6 issues—not
per issue. Do not turn authoring feedback into an implicit compile/test gate;
all other certification remains dormant.
Runtime-kernel and serving-semantics changes still get individual PRs.
Measurement runs are background science with scheduled harvests; they
never sit between two pieces of code work.

## Things that bite

- `/tmp/ref/out/model.bin` disappears on reboot/periodic /tmp cleanup — κ tests
  may still exit successfully without exercising reproduction; the Gate E
  evidence is **UNAVAILABLE**, not PASS.
- `crates/uor-r4-graph-format/fuzz/target` must never be committed (gitignored).
- Fuzz targets need nightly (`cargo +nightly fuzz run …`); the stable
  deterministic mutation smoke runs under plain `cargo test`.
- The on-disk compiled store in `.uor-models/` predates the u32 token
  migration (TLS1-u16); `runtime::parse_store_legacy_u16` reads it, and a full
  recompile is needed to refresh it.
- After deleting a git worktree, cached rlibs in the shared `target/` can
  carry the dead worktree's baked paths and poison the local register gates
  (#788, AUD-VER-001). `repo_root()` now resolves at runtime, but any other
  compile-time `env!("CARGO_MANIFEST_DIR")` user (fixture-loading tests) has
  the same hazard — `cargo clean -p repo-model -p repo-conformance -p xtask`
  clears the register gates; when in doubt, clean the crate whose test reads
  a repo path.
- `cargo test` is fail-fast at the test-binary level: one poisoned binary
  hides every suite after it. Use `cargo test --workspace --no-fail-fast`
  for local gate runs so a single bad binary cannot mask the rest
  (AUD-VER-002).
