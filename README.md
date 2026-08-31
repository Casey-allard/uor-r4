# R⁴ — Geometric Intelligence on Local Hardware

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)](rust-toolchain.toml)

R⁴ is an open research project building a **transformerless local AI agent**.
Its goal is to replace transformer attention, mixture-of-experts routing, and
dense learned matrix operations in the serving path with deterministic
geometric routing and lookup.

That is a very real engineering goal, not a claim that the goal has already
been reached. The long-term target is frontier-like capability on ordinary
local hardware. The project is testing whether language context, inference,
and reasoning can emerge from routes through a canonical geometric memory. The
target serving engine uses no Ollama, hosted model, or source-model weights.

> **Current attention-to-intelligence checkpoint (2026-08-31):** ordinary
> learned causal Q/K/V attention with stable softmax is established as the
> equivalence baseline in coherent R4/Spin frames. The directly trained
> 7,155,360-parameter [#1014](https://github.com/UOR-Foundation/uor-r4/issues/1014)
> model now supplies the binding intervention evidence: enabled sealed-test
> NLL was `2.127407277216677`, while zeroing every attention output after
> `W_o` and before the residual raised it to `4.804799838144271`, a
> `2.6773925609275944`-nat penalty versus the frozen `0.10` minimum. Final
> enabled and attention-off Python/Rust top-1 matched with maximum logit deltas
> `0.00000762939453125` and `0.00001239776611328125`, both inside `0.005`,
> while all six layers passed exact causal/R4 audits with zero future reads.
> This establishes load-bearing ordinary causal attention at the declared
> learned R4/Spin scope; it is not a geometry-advantage result.
>
> That frozen campaign failed its full language-quality Definition of Done:
> enabled NLL exceeded the `1.50` ceiling and subject-or-scene retention was
> `3/5`, below `4/5`. All five outputs were valid UTF-8, avoided period-one
> through period-four loops, and replayed exactly. The campaign closes negative
> without rerun or tuning. Its one separately frozen quality-capacity successor,
> [#1017](https://github.com/UOR-Foundation/uor-r4/issues/1017), has now also
> completed. It reached `149,995,520` cumulative training tokens and selected
> development NLL `1.580241072373312`. Enabled-only Python/Rust parity passed
> with identical top-1 and maximum logit delta
> `0.0000057220458984375`; all six R4/Spin layers and every causal/external audit
> passed. The one-time fresh sealed reveal scored NLL
> `1.5727521962806827`, failing the strict `<1.50` gate, while all `5/5` fixed
> continuations passed subject-or-scene retention and all `5/5` normalized
> replays were exact. The complete #1017 verdict is therefore negative solely
> on NLL. Attention remains established, but dependable general language
> quality remains unresolved. There is no rerun, learning-rate adjustment, or
> further exposure extension for this 7.15M-parameter model. That successor is
> now frozen as [#1019](https://github.com/UOR-Foundation/uor-r4/issues/1019):
> one fresh 12-layer, 13,130,784-parameter model with seed 1019, exactly 16,800
> optimizer steps and 275,251,200 training tokens over the same attention and
> Rust evidence path. Its fresh population is `PASS`; the 400-step,
> 64-sequence MPS overfit smoke is `PASS` with `81.9752%` loss reduction; and
> random-export/all-12-layer Rust preflight parity is `PASS` with maximum absolute logit
> delta `0.0000443459`. The signed 200-step MPS probe passed memory at `21.03%`
> but failed time: its safety projection was `20.66 h`, above the `8 h` ceiling,
> so that frozen offline PyTorch/MPS implementation is terminal
> `UNAVAILABLE_HARDWARE_BUDGET`. Full training, final qualification, sealed
> reveal, generation, and replay remain `NOT_RUN`. This does not redirect UOR:
> the deployed architecture/runtime remains CPU-native; Apple Accelerate/BLAS
> and MPS are local offline training/compilation/test accelerators only; CUDA
> and external GPU execution are out of scope. A single isolated exact-shape
> MPS fast-path test (10 warmup plus 40 measured steps) combined fused AdamW
> with deferred logging and measured `4.485223 s/step`, slower than the signed
> `3.491307 s/step`; `fused=True` was removed immediately. This is a bounded
> fast-path negative, not a model result. #1019 closed without a full run.
> #954's first bounded grounding SFT then failed product transfer at `1/3`.
> Its `R4SourceSpanPointerV1` successor passed the 12/12 overfit preflight and
> Python/Rust score parity, but its sole 256-step fit stopped
> `FAIL_SOURCE_SPAN_POINTER_DEVELOPMENT_GATE_STOP`: answer, abstain, conflict,
> and supported-pointer development accuracy were `69.53125%`, `89.0625%`,
> `91.40625%`, and `94.53125%`, all below the frozen `>=95%` gates. No final
> pointer artifact was emitted and the reserved product probes were `NOT_RUN`.
> Its frozen source-relative successor, `R4SourceRelativeRelationHeadV1`
> (C1-SB2), is implemented and completed only its cheap matched-transfer
> preflight. The fitted families scored 12/12 positive relations, 20/20
> negatives, and 6/6 supported copies; the independently sealed families scored
> 5/12 positives, 14/20 negatives, and 0/6 copies. Same-source matched-pair,
> query-swap, duplicate-agreement, and distinct-conflict controls were false.
> C1-SB2 therefore stopped before Rust parity, the sole 512-step full fit,
> development, and product reveal; no final relation head was emitted. Do not
> tune or retry either revealed head. The next proposed #954 mechanism trains
> relation supervision into the representation through the established R4/Spin
> attention path while retaining exact-source-copy and typed-nonanswer semantics.
> #954's final source-free terminal remains blocked behind #973, and #955
> remains blocked behind #954.
> See the [#1017 record](docs/r4_softmax_quality_capacity_continuation_1017.md),
> [#1017 structured aggregate](docs/r4_softmax_quality_capacity_continuation_1017_raw.json),
> [#1019 frozen contract](docs/r4_softmax_parameter_capacity_1019.md),
> [#1019 structured contract](docs/r4_softmax_parameter_capacity_1019_raw.json),
> [#1019 signed preflight/admission result](docs/r4_softmax_parameter_capacity_preflight_1019_raw.json),
> [#1014 record](docs/r4_softmax_end_to_end_attention_1014.md) and
> [structured aggregate](docs/r4_softmax_end_to_end_attention_1014_raw.json).
>
> The completed #1017 checkpoint is the current working 7.15M
> coherent-generation prototype. If its local export exists, run it directly
> with `r4 generate --prompt "..."`; the alias defaults to
> `.uor-models/research/issue-1017/export`. #1019 closed without a full run and
> does not block using or productizing this
> bounded #1017 path. It remains source-backed, floating-point/matmul/softmax,
> and below the strict NLL target; it does not establish geometry advantage,
> transformerlessness, correctness, reasoning, frontier quality, browser/WASM
> readiness, or release readiness.
>
> The completed
> `R4SoftmaxTraceStudentV1` then compiled construction-side teacher traces into
> a source-free Q16 suffix artifact and showed bounded distillation relative to
> its count and document-permuted controls. Its autonomous continuation still
> entered a repetition loop. That result is not geometric attention, coherent
> generation, correctness, general-purpose inference, or reasoning.
>
> `R4SoftmaxTraceStateStudentV1` is now complete and negative at its frozen
> promotion gate. Its geometric arm moved covered CE only from `2.660721032` to
> `2.660705367`, changed no teacher or actual-next top-1 decision, separated
> from the transport-permuted control by only `0.000023848` nats versus the
> required `0.10`, and produced the identical period-two `, Scotland` loop.
> Exact replay and the zero-source/future-read audit passed, so this is a
> representation failure rather than an execution-integrity failure.
>
> [#1012](https://github.com/UOR-Foundation/uor-r4/issues/1012) is now measured
> at `INSUFFICIENT_SUPPORT_COVERAGE`: aggregate primary coverage was
> `0.6202622204224402`, but the minimum held-document fold covered only
> `0.3469116829611222`, below the frozen 50% floor. Boundary attribution is
> therefore forbidden. On the covered rows, the full current-step Q/K/V probe
> was also `0.0003463194386417179` nats worse than the suffix baseline with the
> required direction in `0/4` folds; both retained `14/26` teacher-top-1 and
> `6/26` actual-next top-1. The label-rotation control separated by
> `1.3807454322642605` nats in `4/4`, so the instrument was sensitive but did
> not demonstrate useful gain over suffix. Exact replay and zero source-model,
> future, or document-13 reads passed.
>
> The project will not expand support or build another observability ladder for
> this bounded current-step trace-distillation path. #1014 has now executed the
> direct-learning pivot and separates the established attention mechanism from
> the still-negative quality gate. See the
> [#1012 measured record](docs/r4_softmax_trace_observability_1012.md).
>
> The hosted GitHub Pages surface is currently a static visualization that
> reports WASM offline and has no functioning chat backend or compiled-artifact
> lowering. It is not a product proof and does not change the active research
> gate. No tag, release, hosted-chat, coherent-generation, correctness, or
> reasoning claim is authorized. See the
> [state-student result](docs/r4_softmax_trace_state_student_1011.md), the
> [#1012 measured record](docs/r4_softmax_trace_observability_1012.md), and the
> [#1014 result](docs/r4_softmax_end_to_end_attention_1014.md).

> **Prior #973 evidence chain leading to this checkpoint (2026-08-30):** routing, exact R4/spin
> state, least-cost selection, and multiscale hierarchy remain the geometric
> substrate, but routing is not being equated with attention. The first natural
> document-scale componentwise-Frechet placement was causally active and still
> harmful: 8.367592% versus frozen #953 at 12.221651%, with its shuffled and
> operator-permuted controls also slightly stronger. The first bounded
> `GeometricGatedDeltaRetentionR4V1` core then passed structural checks but was
> weaker than plain delta on its sealed synthetic fixture (16/28 versus 23/28
> next-token; 55/112 versus 98/112 association wins). Direct-attention V2 then
> appeared positive but is preserved as `NON_PROMOTABLE_BUDGET_MISMATCH`: its
> plain/current comparators had fewer effective degrees of freedom. The fresh,
> pre-reveal-kappa-bound 12-case V3 corrected every arm to normalized R4
> parameters. Full H4 scored 3/12, matched plain attention 12/12,
> current-token-only 6/12, and an inference-time coherent alternative-connection
> swap 10/12; that alternative was not separately trained.
> The direct learning/softmax/value path therefore works, but the current
> mixed-gauge H4 projection/connection/optimizer combination does not transfer;
> the exact H4 group action itself remains algebraically valid.
> `ConnectionGaugeCovarianceV4` then passed its construction/frame gate:
> H4-compatible, alternative-tangent, and fixed-frame arms each fit 16/16 with
> representation covariance. Its independently frozen Phase-III reveal did
> not establish held-out attention. All three main arms scored 13/24;
> current-only scored 12/24; and order-shuffled, value-permuted, and
> source-gauge-mismatch controls scored 13/24, 12/24, and 11/24. The sealed
> commitment and all causal/replay/geometry audits passed, so this is a clean
> functional negative, not an unavailable run. The subsequent #973 build is
> `R4SoftmaxReferenceGeneratorV1`, the provider-free `HELM-D-R4` reference path
> grounded in the official MIT HELM-D architectural source at commit
> `7501deca8f413848bfef804be64ce874b72a3cd7`. The active implementation credits
> and adapts HELM's attention seam and provenance; it does not port HELM's
> remaining geometric decoder stack. UOR's existing pinned SmolLM2
> `HuggingFaceLlamaOracle` supplies embeddings, RoPE, residual/RMSNorm, MLP,
> final normalization, and the language-model head. No HELM checkpoint or
> upstream generation code was executed in this gate. The released HELM generation/cache
> path is incomplete. Its checkpoint and full geometric
> decoder remain an optional external baseline behind a separate tokenizer and
> license gate, and are not directly an R4-block runtime. This decision neither
> vendors upstream code nor claims checkpoint parity. The credited attention
> seam preserves the complete
> learned causal Q/K/V, ordinary stable-softmax, value-aggregation, and output-
> projection path while splitting heads into R4 blocks, encoding them in exact
> cumulative Spin/H4 local frames, transporting K/V into the query frame, and
> mapping the aggregate back before `W_o`. That bounded first positive now
> passes: donor and coherent R4 matched all three held-out next-token top-1
> decisions and decoded `, and`; maximum/mean full-logit deltas were
> `1.049041748046875e-5` / `2.2742100540540378e-6`; donor and R4 replay were
> exact; the source-frame-permuted control decoded `[[` with a `23.0844`
> maximum-logit shift; and 2,700 key plus 2,700 value transports read no future
> position. This establishes preliminary ordinary softmax attention on a
> bounded real-language decoder in coherent R4/Spin frames; it is not a
> curvature-specific advantage. `IntrinsicLorentzR4AttentionV1` attempt 02 then
> completed construction fitting and validation but stopped unavailable before
> D3: Lorentz-barycenter covariance was `9.1214e-8` against the frozen `1e-8`
> ceiling. Its construction diagnostics were also materially worse than donor
> and flat R4, so there is no tolerance-only rerun and no intrinsic-attention
> claim. The source-faithful `HelmDLearnedManifoldR4ConstructionV2` qualifier
> then completed its repaired 120-frame preflight, fitting, checkpoint, and
> 64-position non-D3 construction validation. It returned the valid terminal
> `FAIL_HELM_D_MANIFOLD_CONSTRUCTION_REVISE_PROJECTION_SCORE_CENTROID_OR_TRAINING`:
> donor/gauge NLL was `3.667626`/`3.667626`, Euclidean was `4.483154`, and
> Lorentz was `7.710618`. All three destructive Lorentz controls were worse
> (`8.871399`–`9.466637`), establishing intervention sensitivity but not useful
> learned geometric attention; functional retention and matched parity failed.
> The separately frozen 8/8 contract for
> [score-by-readout localization](docs/helm_d_score_centroid_localization_973.md)
> stopped at its two-document preflight with
> `REJECT_TANGENT_READOUT_SELECT_SCORE_PREFLIGHT`: tangent readout increased
> normalized audit MSE on both documents (pooled ratio `1.0643688804269025`).
> The infrastructure, covariance, replay, causal-input, and work-ledger gates
> passed, so this is a measured mechanism rejection rather than an unavailable
> run. The maintainer now accepts ordinary dot-product/stable-softmax causal
> attention in coherent R4/Spin frames as the current reference baseline.
> Intrinsic score/readout, resonance, softmax-replacement, recurrent
> factorization, and exact-lowering research are **PARKED** rather than active.
> The provider-free-at-execution, source-backed native CPU
> `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) CLI gate has now passed. It
> retains the credited HELM attention seam and uses the
> pinned SmolLM2 `HuggingFaceLlamaOracle` for embeddings, RoPE,
> residual/RMSNorm, MLP, final normalization, and the language-model head in a
> CLI/evidence path. The frozen result passed decoded quality at 4/5 in both
> passes, replayed all 5/5 prompt reports exactly after deleting only timing,
> selected all 30 layers, recorded exact causal/projection/R4 audits with zero
> future reads, and matched the source donor for P1 through EOS and P2-P5 for
> all 32 retained tokens. Its terminal is
> `PASS_R4_SOFTMAX_REFERENCE_GENERATION_ADVANCE_NATIVE_PRODUCT_BRIDGE`. This
> reference is intentionally transformer-compatible, `f32`/multiply/alloc and
> source-weight backed; it is not yet table-native, multiply-free, or
> transformerless. Its explicit opt-in, loopback-only dedicated native HTTP
> endpoint now passes the frozen eight-token sunlight canary with the same token
> sequence, decoded text, decision CID, persistent-state CID, all-30-layer exact
> audits, and zero future reads as the CLI. Dashboard wiring, native-readiness
> gating, and static/WASM isolation checks pass; browser interaction/E2E is
> `NOT_RUN`. The feature is disabled by default and does not change the default
> engine. This remains a native CPU research reference,
> not a source-free, transformerless, static-WASM, release, or frontier-model
> result. #973 remains open for its intrinsic/source-free terminal; #954's
> cosine source-span pointer stopped at its development gate, and its implemented
> C1-SB2 relation head then stopped at failed matched-transfer preflight before
> Rust parity/full fit/development/product, without a final head. The proposed
> representation-trained successor retains the established R4/Spin attention
> plus exact-copy/typed-nonanswer seam. #954's final source-free terminal remains
> blocked behind #973, and #955 remains blocked behind #954.
> The trace/compiler rung
> that followed that checkpoint is now complete: `R4SoftmaxTeacherTraceV1`
> supplied construction traces and
> `R4SoftmaxTraceStudentV1` compiled a source-free Q16 suffix artifact with a
> bounded distillation effect, but its autonomous text looped. The subsequent
> `R4SoftmaxTraceStateStudentV1` recurrent rung also stopped: its minute CE
> change was not decision-bearing or materially geometry-dependent, and the
> same loop remained. The subsequent construction-only observability audit
> completed at `INSUFFICIENT_SUPPORT_COVERAGE` and cannot attribute a boundary.
> #1014 then directly trained the end-to-end R4/Spin causal-softmax model. Its
> `2.6773925609275944`-nat attention-off penalty and two-arm Rust parity
> establish load-bearing ordinary attention, while enabled NLL `2.127407` and
> subject/scene retention `3/5` fail the frozen quality DoD. That campaign is
> closed to rerun or tuning. #1017's fixed exposure-only successor improved the
> fresh sealed NLL to `1.5727521962806827` and prompt retention to `5/5`, with
> enabled Rust parity and exact replay, but still failed the strict `<1.50`
> quality gate. #1019 now freezes that capacity rung at twelve layers and
> 13,130,784 parameters over the qualified mechanism and runtime evidence path.
> Population, the 400-step MPS overfit smoke, and random-export/all-12-layer
> Rust parity passed, but the signed MPS probe stopped
> `UNAVAILABLE_HARDWARE_BUDGET` for the frozen eight-hour offline
> implementation, and full training through replay remains `NOT_RUN`. The
> fused-AdamW/deferred-logging fast path was slower (`4.485223` versus signed
> `3.491307 s/step`); #1019 closed without a full run. #954's cosine pointer
> stopped before final artifact or product reveal. Its implemented C1-SB2
> source-relative relation successor then failed matched-transfer preflight and
> stopped before Rust parity/full fit/development/product, without a final head.
> The next proposal trains relation supervision into the existing R4/Spin
> attention representation while retaining exact-copy/typed-nonanswer behavior.
> CUDA and external GPU execution are out of scope. No
> further 7.15M exposure or learning-rate tuning is authorized.
> Intrinsic/readout substitution, resonance, softmax replacement, scale, and
> product promotion remain parked. No tag, release, hosted
> promotion, or browser-WASM claim is authorized. See the
> [V4 connection-gauge record](docs/connection_gauge_covariance_v4_973.md), the
> [direct-attention history](docs/direct_causal_geometric_attention_973.md), the
> [resonance audit](docs/multi_resonance_attention_sieve_audit_973.md),
> [HELM-D-R4 result](docs/helm_d_r4_softmax_decoder_973.md),
> [localization result](docs/helm_d_score_centroid_localization_973.md),
> [generation result](docs/r4_softmax_reference_generation_973.md), the
> [compact attempt-01 aggregate](docs/r4_softmax_reference_generation_attempt_01_result_973.json),
> [native bridge result](docs/r4_softmax_reference_http_bridge_973.md),
> [#1014 end-to-end result](docs/r4_softmax_end_to_end_attention_1014.md),
> [ADR-0005](docs/adr/0005-predictive-geometric-connection-memory.md) and the
> [Geometric Intelligence Programme](docs/geometric_intelligence_programme.md).
>
> **HELM-D-R4 measured evidence status:** pinned-source provenance `PASS`;
> ordinary-donor reproduction `PASS`; transported-R4 parity and destructive
> control `PASS`; upstream HELM-D checkpoint parity `NOT_RUN`; intrinsic R4
> attention attempt 01 `UNAVAILABLE_PRE_REVEAL` from a checkpoint JSON
> round-trip defect; attempt 02 reached construction validation and terminated
> `UNAVAILABLE_INTRINSIC_LORENTZ_R4_STOP_BEFORE_HELD_OUT` on its covariance
> audit. The subsequent learned-manifold V2 Attempt 02 is a valid non-D3
> construction-validation negative: Lorentz NLL `7.710618`, Euclidean
> `4.483154`, donor
> `3.667626`; all destructive controls separated, all replays passed, causal
> reads were zero, and D3 is `NOT_RUN`. Result CID
> `blake3:9144913380c6ebdeebb5848138bc8e6642c1e7020d8e7a097aa3cd73cb829020`.
> The localization successor rejected tangent readout while retaining score as
> the only future intrinsic seam; that entire research lane is now parked.
> Localization result CID
> `blake3:79792c1a6e38733fd3eb925e364c87308ce26e02bea951f338466ab93481b374`.
> See the
> [intrinsic R4 record](docs/intrinsic_lorentz_r4_attention_973.md) and
> [learned-manifold record](docs/helm_d_learned_manifold_r4_construction_973.md).

> **Retained bounded-global evidence (2026-08-28):** #973's independently frozen
> `BoundedGlobalNoncommutingExactSpinR4V2` reached
> `RETAIN_BOUNDED_GLOBAL_NONCOMMUTING_EXACT_SPIN_ATTENTION_CONTINUE_CORPUS_INDUCTION`.
> The canonical population repair witnesses exact stored-S3-to-H4
> noncommutation, distinct nonidentity left-ordered folds, central Q29 phases,
> same-address class-result reuse, and incompatible unique candidate-relative
> winners under `C^-1*G` lexicographic least cost. With equal admitted support
> and executed work, the decoded matrix was real 2/2, identity-disabled 1/2,
> class/operator-permuted 0/2, and support-reversed real 2/2; support/work
> mismatches were zero and exact period-plus-EOS termination was 6/6. The
> target preimage was loaded once only after the target-free gate. Operator,
> population-audit, target-free-census, and decoded-smoke identities are
> `blake3:1cf08604fb4a1c545984f4cab41194e0ffcf1d7551b6e438ed57b49a0066a6e9`,
> `blake3:16ebc6d36f01e4cb324d3c46fc059aca4ffea84ba467e860b55f983cd83f4a9c`,
> `blake3:c3fb3568028f924fb12971c888193cc5780111a7af14503e240f39fbeb58dd4a`,
> and `blake3:41207999bb088e3b5f186cce983951cc27c2962d34ef8046a0beae4754b44218`.
> This establishes one bounded synthetic causal global geometric-attention
> witness, not corpus induction, general semantics, reasoning, correctness, or
> product readiness. Corpus induction was this result's contemporaneous next
> step; `ConnectionGaugeCovarianceV4` later preserved construction covariance
> but failed its held-out attention and control-separation gates. `HELM-D-R4`
> full-decoder softmax parity subsequently passed; intrinsic V1 stopped
> unavailable before D3; the source-faithful learned-manifold qualifier later
> completed with a valid functional-retention/parity negative; and the 8/8
> localization contract then stopped at its two-document preflight, rejecting
> tangent readout. The source-backed `R4SoftmaxReferenceGeneratorV1` generation
> gate and its explicit opt-in, loopback-only dedicated native HTTP endpoint
> subsequently passed without changing the default engine. Dashboard
> wiring/readiness and static/WASM-isolation checks passed; browser E2E remains
> `NOT_RUN`. The trace-capture/Q16 suffix-student successor later completed with
> bounded source-free distillation but looping output. Its recurrent state
> successor failed promotion, and the following #1012 observability audit
> completed at `INSUFFICIENT_SUPPORT_COVERAGE`. #1014 subsequently established
> load-bearing ordinary causal attention through its `2.677393`-nat
> attention-off intervention and Rust parity, but failed its full quality DoD
> at enabled NLL `2.127407` and subject/scene retention `3/5`. It closes
> negative. #1017's separate continuation then reached `5/5` retention but
> failed NLL only at `1.5727521962806827`. #1019 records an optional frozen
> 12-layer, 13,130,784-parameter successor. Its population, MPS overfit smoke,
> and random-export/all-12-layer Rust preflight parity passed, but its MPS hardware path
> stopped `UNAVAILABLE_HARDWARE_BUDGET` for the frozen eight-hour offline
> implementation; full training, final qualification, reveal, generation, and
> replay remain `NOT_RUN`. The fused-AdamW/deferred-logging fast path was slower
> (`4.485223` versus signed `3.491307 s/step`); #1019 closed without a full run.
> #954's cosine pointer stopped at its development gate; its implemented C1-SB2
> relation successor then failed matched-transfer preflight before Rust parity,
> full fit, development, or product reveal, and emitted no final head. The next
> proposal trains relation supervision into the existing R4/Spin attention
> representation while retaining exact-copy/typed-nonanswer behavior. CUDA and
> external GPU execution are out of scope. #954's final source-free terminal
> remains blocked behind #973, and #955 remains blocked behind #954.
> See the
> [bounded-global record](docs/bounded_global_exact_spin_attention_973.md).

> **Earlier bounded-global V1 negative (2026-08-28):** #973's independently frozen
> exact-spin contrast stopped at the target-free relation gate with
> `RETAIN_CONVERSATION_ONLY_REDESIGN_BOUNDED_GLOBAL_EXACT_SPIN_RELATION`.
> Both same-multiset snapshot carriers had distinct canonical epochs/roots,
> four references, three exact classes, and one byte-identical same-address
> class-result reuse, while sharing one byte-identical lower artifact and equal
> admitted support/work. But `Pavel` and `helix` map to the same H4 root and
> `prism` maps to identity, so both frozen orders produce the same complete
> `-1` fold, fiber, torsion, real `helix` role, and permuted `prism` role. The
> frozen incompatible-winner premise was false. Target loads were zero and the
> decoded smoke is `NOT_RUN`. Operator and target-free census identities are
> `blake3:f6b36cdf3e6cf96e1e9a345980843ee9eaffd25f5b864d4b4ed45a30ae6f746f`
> and `blake3:6c0a9f89a29584a09d917ae427a494b53c06b76e56482f665870ae86c1cd130a`.
> This rejected one frozen global relation, not geometry generally. The V1
> result remains append-only history; V2 supplied the noncommuting repair and
> historically advanced #973 to a corpus-induction gate. Later negative results
> supersede that action; `ConnectionGaugeCovarianceV4` later failed held-out
> attention at 13/24 with insufficient control separation. `HELM-D-R4`
> full-decoder softmax parity subsequently passed. See the
> [bounded-global record](docs/bounded_global_exact_spin_attention_973.md).

> **Retained conversation-scope result (2026-08-28):** Before the global V2
> qualification, #973 retained three narrow
> mechanisms: Gate 0's exact-candidate prior-prefix copy mechanism, one
> construction-bound exact-descriptor paragraph path selector, and now
> `ConversationEntitySpinPathR4V1` at
> `RETAIN_CONVERSATION_ENTITY_SPIN_PATH_ATTENTION_CONTINUE_BOUNDED_GLOBAL`.
> In the frozen conversation contrast, both held-out inputs had identical
> lexical multisets, immediately preceding/current turns, current-through-
> paragraph identities and ordered H4 states, global identity/state/snapshot,
> admitted candidates, and work. Only the older entity-to-descriptor binding
> changed. The decoded matrix was real 2/2, conversation-disabled 1/2, cross-
> turn-binding-permuted 0/2, and parsed-binding-row-reversed 2/2, with exact
> target-free and decoded replay. Neither candidate token occurred in the
> observed held-out conversations and Gate 0 abstained. The complete stored-
> spin lexicographic path was load-bearing, but this run did not separately
> qualify an H4-shell, fiber, or torsion coordinate. This is one bounded
> synthetic exact-descriptor cross-turn entity-role selector, not semantic or
> natural transfer, a geometric advantage over direct ordered binding lookup,
> a general entity/conversation model, or general conversation/global
> attention. The operator, target-free census, and decoded-smoke identities are
> respectively
> `blake3:343c961b06605f6ae9bb6160ac34a98224991715b706156349a8fd544b6dbb35`,
> `blake3:649d733a194469aa648101a873d9e2ee323266b18872ced412d1da2cc6a56635`,
> and `blake3:6930de3c07d30df4420bb68e60ea74531c8076516bcfef1c016240eddf1b9ca2`.
> The subsequent V1 bounded-global contrast failed its target-free relation
> gate; the independently frozen V2 repair later passed its bounded decoded
> contract. Later corpus placement and bounded recurrence results were negative;
> the direct reference has since run and V3 is negative. #973's
> `ConnectionGaugeCovarianceV4` construction/frame preflight is positive, but
> its independently frozen held-out reveal is negative at 13/24 for every main
> arm. The `HELM-D-R4` full-decoder softmax parity qualifier now passes;
> intrinsic V1 is unavailable before D3, and its source-faithful learned-manifold
> successor is now a valid negative. The localization attempt stopped at its
> two-document preflight and rejected tangent readout; provider-free-at-execution,
> source-backed autonomous
> `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) generation subsequently passed.
> Its opt-in, loopback-only dedicated native HTTP endpoint then passed exact CLI
> canary parity. Dashboard wiring/readiness and static/WASM-isolation checks
> passed; browser E2E remains `NOT_RUN`. The source-free trace/state attempts
> are preserved negatives; #954's cosine pointer is also a bounded negative.
> Its implemented C1-SB2 relation successor failed matched-transfer preflight
> before Rust parity/full fit/development/product and emitted no final head. The
> final source-free terminal remains blocked behind #973, and #955 remains
> blocked behind #954.
> See the
> [conversation record](docs/conversation_entity_spin_path_attention_973.md).

> **Accepted capability-first evidence (2026-08-28):** #953's frozen
> `MultiscaleCountRadiusR4V1` comparison is positive. Against #989's unchanged
> 99,362/446,342 (22.261404%) table reference, the construction-only R4 tie
> overlay scored 103,604/446,342 (23.211797%), +4,242 correct choices and
> +0.950392 percentage points. The declared-work ledger and candidate support
> matched at all teacher-forced positions.
> The fixed prompt still emitted 16 valid UTF-8 units, but geometry changed the
> bounded continuation from the date-fragment branch to
> `. It is the most important thing to do so. The first people to live`.
> Two complete executions produced byte-identical base artifacts, overlays, and
> reports; that external replay check promoted the reports' pending decision to
> the frozen positive terminal. This establishes only causal incremental value
> for the exact fixed-point R4 evidence-radius tie intervention over the frozen lexical
> table; it is not attention, semantics, correctness, reasoning, chat, or
> release evidence. See the
> [#953 evidence record](docs/source_free_table_geometric_intervention_953.md).

> **Evidence boundary before the predictive-memory reset:** the geometric storage/identity foundation, one bounded
> causal R4/S3 path selector, and reusable provider-free decode/render/append
> plumbing exist. The first #953 smoke was an exact lexical relabel of #969, so
> it did not qualify a natural grammar loop. `PrimaryThenAdjacentSpinFallbackV1`
> repaired the frozen agreement admission to exact `{still}` then `{run,runs}`
> support under equal work, but the one permitted four-arm run chose `still run`
> for both full-path prompts and `still runs` for both state-disabled prompts.
> The frozen `LocalSameObjectContextPlacementV1` preflight then reproduced 7/7
> construction prototypes with zero class collisions and zero
> padding-identity aliases, but real placement selected 0/2 intended candidates
> while the same-artifact placement-permuted and order-shuffled controls selected
> 2/2 and 1/2. Generation and replay were `NOT_RUN`; the terminal remains
> `REVISE_I1_GENERATOR_IN_PLACE`. Independent #983 then formed pure
> construction classes but transferred to 0/6 held-out decisions and closed at
> `UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER` before selection. #986 then closed
> `UNAVAILABLE_FRAME_OR_POPULATION`: the pinned raw corpus reproduced, but its
> exact #986 codec/pair commitment and a complete same-frame lexical SpiralCore
> operator map were unavailable. Placement, diffusion, Gate 0, calibration,
> sealed labels, selection, and #953 were `NOT_RUN`. The later #989 table reset
> supplied a working reference, and the one matched #953 R4 tie intervention
> has now passed its held-out and decoded-output contract. #973 Gate 0 has also
> retained one exact-candidate prior-prefix copy-attention mechanism. Its
> frozen paragraph slice retained one construction-bound exact-descriptor/
> entity-binding stored-phase path selector, and its frozen conversation slice
> retained one construction-bound exact-descriptor cross-turn entity-role path
> selector, each with the narrow boundary above. The first independent bounded-
> global exact-spin relation failed target-free because its swapped states
> commute; the independently frozen V2 repair then established one bounded
> noncommuting global mechanism. Later corpus-placement and recurrent results
> were negative. `ConnectionGaugeCovarianceV4` retained construction-scale
> representation covariance but failed held-out attention and control
> separation. `HELM-D-R4` full-decoder gauge-equivalent ordinary softmax now
> passes; intrinsic V1 stopped before D3, and the source-faithful
> learned-manifold qualifier is a valid non-D3 construction-validation negative,
> and the 8/8-contract attempt stopped at its two-document preflight, rejecting
> tangent readout. The provider-free-at-execution, source-backed
> `R4SoftmaxReferenceGeneratorV1`
> (`HELM-D-R4`) CLI gate has now passed using the accepted ordinary attention
> baseline and UOR's pinned SmolLM2 `HuggingFaceLlamaOracle` decoder path. Its
> explicit opt-in, loopback-only dedicated native HTTP endpoint now passes the
> frozen eight-token CLI-parity canary without changing the default engine.
> Dashboard wiring/readiness and static/WASM-isolation checks pass; browser E2E
> is `NOT_RUN`. The trace/state observability audit later completed at
> insufficient support; #1014 established load-bearing attention, and #1017
> closed NLL-only negative. #1019's optional frozen 12-layer,
> 13,130,784-parameter campaign recorded population, MPS overfit smoke, and
> random-export/all-12-layer Rust preflight parity passed; the MPS hardware path stopped
> `UNAVAILABLE_HARDWARE_BUDGET` for the frozen eight-hour offline
> implementation, so the full campaign remains `NOT_RUN`. The fused-AdamW/
> deferred-logging fast path was slower (`4.485223` versus signed
> `3.491307 s/step`); #1019 closed without a full run. #954's cosine pointer
> stopped before final artifact or reveal; its implemented C1-SB2 relation
> successor then failed matched-transfer preflight before Rust parity/full
> fit/development/product and emitted no final head. The next proposal trains
> relation supervision into the existing R4/Spin attention representation while
> retaining exact-copy/typed-nonanswer behavior. CUDA and external GPU execution
> are out of scope. #954's final source-free terminal remains blocked behind
> #973, and #955 remains blocked behind #954. General higher-scope attention,
> correct answers, and reasoning do not exist yet. The
> dashboard is an interactive window into the research substrate, not a
> frontier model or a ChatGPT replacement.

## Try the project

With Git and a current Rust toolchain installed:

```bash
git clone https://github.com/UOR-Foundation/uor-r4.git
cd uor-r4
cargo run --bin r4 -- demo
```

Open <http://127.0.0.1:8000>.

The dashboard lets you interact with the existing geometric router and inspect
its state. It is the quickest way to see the project in motion without
downloading or compiling a language model. A first Rust build may take longer
than five minutes on some machines; later launches reuse it.

To inspect one route from the command line instead:

```bash
cargo run --bin r4 -- route "geometry is the route"
```

To run the current working local 7.15M coherent-generation prototype:

```bash
r4 generate --prompt "Once upon a time in a quiet village"
```

`generate` defaults to `$UOR_MODEL_STORE/research/issue-1017/export` (or
`.uor-models/research/issue-1017/export` when the variable is unset). It requires
that local export and is a bounded #1017 prototype: provider-free at execution,
but still source-backed, floating-point/matmul/softmax, and below the strict
`<1.50` NLL target. #1019 was closed without another capacity run; it is not a
prerequisite for using or productizing this path.

The bounded `answer` interface now admits only an exact
`Where is the <subject>?` question and punctuation-terminated source spans:

```bash
r4 answer --source-file facts.txt --question "Where is the copper compass?" \
  --head /path/to/qualified-source-relation-head.json \
  --json-output grounded-answer.json
```

For a source-relative relation head, `answer` pairs each of two to eight exact
punctuation-terminated source sentences with the question, captures the #1017
model's normalized causal R4/Spin state at the final question token, and uses
the explicitly supplied qualified `--head` artifact to choose an exact source
span, typed abstention, or typed conflict. The source is content-addressed and
read again after evaluation to detect change. This is a fail-closed extractive
seam, not semantic-entailment or general-correctness evidence.

The first fixed #954 MPS fine-tune completed in 14 minutes 44 seconds on the
project M1, but its frozen Rust product population failed `1/3`: all three
prompts decoded `ABSTAIN`, so only the unsupported question passed. The command
therefore fails safely, but this checkpoint is not a usable answer model. It is
not rerun or tuned. The subsequent `R4SourceSpanPointerV1` preflight passed
12/12, and Python/Rust parity passed with maximum score delta
`1.234420776e-7` and maximum logit delta `1.428717041e-6`, both inside `0.01`.
The sole 256-step fit nevertheless missed every frozen development gate:
answer `89/128` (`69.53125%`), abstain `114/128` (`89.0625%`), conflict
`117/128` (`91.40625%`), and supported pointer `121/128` (`94.53125%`) versus
`>=95%` each. It stopped `FAIL_SOURCE_SPAN_POINTER_DEVELOPMENT_GATE_STOP`
before producing a final pointer artifact; the three reserved product probes
and browser/HTTP wiring are `NOT_RUN`. The implemented
`R4SourceRelativeRelationHeadV1` C1-SB2 successor then fit 12/12 positive
relations, 20/20 negatives, and 6/6 supported copies on its two fit families,
but transferred only 5/12 positives, 14/20 negatives, and 0/6 copies to its two
sealed families. Same-source matched-pair, query-swap, duplicate-agreement, and
distinct-conflict controls were false, so C1-SB2 stopped before Rust parity,
the sole 512-step full fit, development, and product reveal. It emitted no final
head. Consequently the default `r4 answer` surface is unavailable unless an
explicitly qualified relation-head artifact exists. Do not tune or retry either
revealed head. The next proposed #954 mechanism trains relation supervision
into the representation through the established R4/Spin attention path while
retaining exact-source-copy and typed-nonanswer semantics. #954's final
source-free terminal remains blocked behind #973, and #955 remains blocked
behind #954. See the
[#954 record](docs/r4_grounded_correctness_954.md) and
[C1-SB0 structured result](docs/r4_grounded_correctness_954_raw.json) plus the
[C1-SB1 pointer result](docs/r4_source_span_pointer_954_raw.json) and
[C1-SB2 relation result](docs/r4_source_relation_head_954_raw.json).

On Apple Silicon, build the opt-in CPU-BLAS version so local inference uses the
machine's Accelerate framework:

```bash
cargo build --release --offline --features local-inference-accelerate --bin r4
target/release/r4 generate --prompt "Once upon a time"
```

On the project M1, the same four-token #1017 prompt produced token IDs
`[14, 403, 285, 261]` and text `, there was a` under both exact `uor-matmul`
and Accelerate. Output and attention-audit CIDs were identical. Accelerate cut
measured generation from `3.060506042 s` to `0.116236875 s` (`26.33x`) and
end-to-end wall time from `3.41 s` to `0.52 s` (`6.56x`). The complete report
still differs intentionally because backend provenance and timing differ.

To run the qualified source-backed R4/Spin softmax reference generator from an
already-local pinned SmolLM2 snapshot:

```bash
cargo run --release --offline --bin r4 -- r4-softmax-generate \
  --source .uor-models/sources/smollm2-135m-instruct \
  --prompt "Explain in three short sentences why plants need sunlight." \
  --max-tokens 32 \
  --workers 4 \
  --json-output /tmp/r4-softmax-reference.json
```

This is the native `R4SoftmaxReferenceGeneratorV1` evidence surface. It has no
provider or network fallback and remains source-weight-backed, `f32`/matmul,
allocating, and Transformer-compatible. It is not the table-native runtime or
the browser-only WASM dashboard. See the
[generation record](docs/r4_softmax_reference_generation_973.md) and
[attempt-01 aggregate](docs/r4_softmax_reference_generation_attempt_01_result_973.json).

To expose the identical source-backed policy through its native research
bridge, opt in explicitly on a loopback address:

```bash
cargo run --release --offline --bin r4 -- \
  --host 127.0.0.1 --port 8000 serve \
  --enable-r4-softmax-reference \
  --r4-softmax-source .uor-models/sources/smollm2-135m-instruct \
  --r4-softmax-workers 8
```

The dashboard reveals the reference option only when the native server reports
the source ready. The dedicated API is
`POST /uor/v1/r4-softmax-reference/generate`; it is not `/api/chat` and does
not replace the default engine. The frozen eight-token sunlight request matched
the CLI's token sequence, decoded `Plants need sunlight to undergo
photosynthesis, a`, decision CID, persistent-state CID, all-30-layer audits,
and zero-future-read audit. The endpoint is disabled by default, loopback-only,
native CPU only, single-flight, and capped at 32 generated tokens. Static/WASM
builds reject it. Dashboard wiring/readiness and isolation checks passed, but
browser interaction/E2E remains `NOT_RUN`. See the
[native bridge result](docs/r4_softmax_reference_http_bridge_973.md).

To run the one fixed canonical-ingestion witness:

```bash
cargo run --bin r4 -- lexical-ingestion-witness
```

To compile and evaluate the established #989 source-free lexical table path:

```bash
cargo run --bin r4 -- source-free-table \
  --corpus /path/to/articles.jsonl \
  --prompt "The United States" \
  --continuation-cap 16 \
  --artifact-out /path/to/source-free-table.bin \
  --json
```

The corpus directory must also contain its pinned `manifest.json`. The command
uses only the D3 construction partition for its vocabulary and integer
unigram/bigram/trigram counts, evaluates held-out next-unit prediction, writes
the deterministic packed artifact, and emits the exact decoded continuation.
It is a statistical lexical baseline command, not an attention, semantic,
correctness, chat, or release surface.

To run the one frozen #953 comparison against that unchanged table baseline:

```bash
cargo run --bin r4 -- source-free-table \
  --corpus /path/to/articles.jsonl \
  --prompt "The United States" \
  --continuation-cap 16 \
  --artifact-out /path/to/source-free-table.bin \
  --geometric-intervention \
  --geometry-overlay-out /path/to/multiscale-count-radius-r4.bin \
  --json
```

`--geometric-intervention` enables only the frozen
`MultiscaleCountRadiusR4V1` tie-breaking overlay. Both arms retain the table's
first nonempty row, maximum-count tie set, lexical codec, decoder, and shared
declared-work ledger. The report compares held-out choices and both fixed-prompt
continuations; `--geometry-overlay-out` writes the deterministic overlay bound
to the base table artifact. The overlay is a bounded causal geometry experiment. Even a
positive comparison does not establish attention, semantics, correctness,
reasoning, chat quality, performance superiority, formal closure, or release
readiness.

To reproduce the bounded A1R associative ordered-summary decision:

```bash
cargo run --bin r4 -- associative-ordered-summary-a1r-probe
```

To reproduce the corrected A1P paired-H4-derived exact R4-heatmap
identifiability decision:

```bash
cargo run --bin r4 -- candidate-relative-identifiability-a1p-probe
```

To run the #953 decoded loop against a canonical route artifact:

```bash
cargo run --bin r4 -- bounded-geometric-generate \
  --artifact /path/to/canonical-route.json \
  --prompt "active agile athletes run" \
  --continuation-cap 2 --json
```

This research command loads no provider or source weights. It currently accepts
only a canonical artifact whose embedded construction/global input can fully
reconstruct the parent codec registry; subset-observation artifacts fail closed.
Plain output labels both the appendable continuation and typed stop reason;
`--json` emits the full deterministic witness. Trailing prompt whitespace is
also rejected fail closed so the lexical-boundary contract cannot silently
rewrite the prompt. The command is bounded to that reconstructed vocabulary and
the local #969 path; it is not `ask`, `chat`, or a correctness-qualified answer
surface.

The A1R command uses only the frozen construction/evaluation fixture and exact
finite tables. Its frozen report kappa is
`blake3:f0db7a5d5c81d51ebf3b4bf8a2715c4960ec16b14161e8bf7598d7b98c48c881`.
The associative state passed the declared scope, independent-global, fold,
incremental, and support invariants. The full arm produced distinct `ll`/`rr`
relative states on all 6 queries, but shortest Cayley distance mapped both to
energy 2 and tied every query. The terminal verdict is `RETAIN_STATE_ONLY`: it
does not generate text or establish full attention.

The A1P command preserves those six queries as regression-only evidence,
prepares construction and sealed-validation geometry/support without labels,
and derives S4 parity from each exact history and the frozen role order before
joining the separate label ledgers. Its paired contract computes
`X=C(H,c)`, `Y=C(P_c,c)`, and `D=X*Y^-1` in the signed `(1,i)` R4 chart. The
exact endpoint rule is `sin=±1, cos=0 -> 1` with chirality retained and
`sin=0, cos=±1 -> 0` with cosine polarity retained; `q0=q1=0` is typed-null
abstention, not a threshold shortcut. `q2` and `q3` remain in the full `D`
witness but are not scorer-key fields.

The target-free structural census covers 120×120 = 14,400 ordered pairs, 120
relative rows, 45 exact heatmap classes, and 480 typed-null pairs. Across 36
fixture decisions, 14 classes were exercised; construction coverage was 12/12
and pure, construction classes covered 10/12 validation decisions, the
no-class-splitting oracle ceiling was 10/12, strict construction transfer was
0/6, and eight heatmap classes were incompatible. The hard gate therefore
stops before scalar search; every downstream selection, control, and placement
row is `NOT_RUN_IDENTIFIABILITY_HARD_STOP`, not PASS. Its terminal literal is
`RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q`. Contract, universe, and report
kappas are
`blake3:2daacf538c022fab9580d1e124af6c18d0b06da04604fbc962a01bda57f08a98`,
`blake3:dca725c0ec6060166bcd0023df956e1ff029661b5fa7800ccb9f20808712b796`,
and `blake3:5f9239150dea8c0c27c4dfa6ad2e4d0068bc3d18afc127b315c0ec358ceddb3f`.
This negative is bounded to the paired-H4-derived heatmap readout. Fixed-zeta
phases, ordered n-lets, exact `phi` radial transport, and the typed
`sqrt(2) <-> 2i <-> [0,2]` adapters remain structural under
`STRUCTURAL_BINDING_ONLY_NO_ZETA_NLET_TO_PHI_EXPONENT_RULE`; they are not
scorer inputs. It does not establish attention or generation, and #969 becomes
the next stage only after protected #970 merge. #969 has since delivered one
bounded causal path selector. #953 has driven it through real decoded-loop
plumbing and tiered admission on the frozen preflight, but the natural agreement
run made the same full-path choice for both prompts and did not qualify a
natural grammar result.

The ingestion witness maps two turns of text through the pinned lexical codec,
prime/spin route state, canonical hierarchy manifest, strict reload, and exact
lexical reconstruction. It also exercises the declared fail-closed unknown-unit
path. It loads no model and establishes reversible state plumbing only—not
attention, inference, correctness, or reasoning.

The additive serving envelope is
`uor-r4.canonical-lexical-route-manifest/1`; it transitively embeds the frozen
`uor-r4.prime-route-spin-manifest/2` bytes. Its codec identity is
`uor-r4.unicode-lexical-runs/1`: UTF-8 identity normalization, caller-declared
sentence/paragraph/turn boundaries, canonical surface-byte vocabulary order,
and rejection of unknown units before mutation. The parent keeps the complete
codec route-address registry in stable lexical-unit order; the unchanged child
manifest contains only addresses witnessed by its causal sentences. The fixed
input ceiling is 8 turns, 32 paragraphs, 31 sentences, 128 units per sentence,
512 total units, and a 64-unit content-addressed global snapshot.

Downstream code consumes `CanonicalRouteArtifact::decode_canonical`,
`attention_consumer_trace`, `attention_consumer_trace_for_cursor`,
`attention_consumer_trace_with_ordered_h4`,
`incremental_update_trace`, `incremental_cursor`,
`lookup_shared_class_trace`, `scope_ceilings`, and `reconstruct_input`. The
attention handoff is ordered current, previous, last-two, sentence, paragraph,
conversation, then bounded global; the cursor resolver returns those same seven
slots and marks not-yet-established boundaries absent. S0 serializes state and
numeric geometry only: every candidate row ceiling is zero and marked
`NOT_IMPLEMENTED_S0_STATE_ONLY`. #952 established candidate/value reachability
but found its reusable summaries order-erasing. #967 landed the exact ordered
state repair but retained it as state only after the candidate tie. #970's
corrected paired-H4-derived exact R4-heatmap gate stopped at bounded readout
identifiability without searching another scalar. #969 then qualified one local
causal path selector, and #953 implemented the first bounded decoded
library/CLI plumbing. Its relabelled smoke terminated
`REVISE_I1_GENERATOR_IN_PLACE`. `PrimaryThenAdjacentSpinFallbackV1` then
recovered exact `{still}` then `{run,runs}` primary support while consulting
and truthfully tracing adjacent-spin rows, which remained non-admitting until
the primary tier was empty. The one permitted four-arm run produced `still run`
for both full-path prompts and `still runs`
for both state-disabled prompts, with deterministic replay. The terminal
remains `REVISE_I1_GENERATOR_IN_PLACE`. The first frozen local same-object,
order-sensitive candidate-placement preflight then failed before generation or
replay: real placement selected 0/2 intended candidates while its same-artifact cyclic
placement control selected 2/2. #983's later independent construction-return
classes then transferred to 0/6 held-out decisions. #986's later local
qualification stopped before geometry because neither its exact corpus/codec
population nor a complete lexical Cl(0,6)/SpiralCore frame was available.
#953's historical H4/placement fixtures remain untouched. The later B0 reset
accepted a separate fixed-point R4 table-tie intervention and closed #953 at
its positive terminal. #973 Gate 0 has since retained one bounded prior-prefix
copy mechanism. Its frozen paragraph and conversation slices retained one
exact-descriptor/entity-binding path selector apiece at their respective
   scopes. The first bounded-global exact-spin relation failed target-free; its
   independently frozen V2 noncommuting repair then passed the bounded decoded
   contract. The first natural corpus placement later failed in PR #997, and
   the first bounded gated-delta core trailed plain delta on its sealed smoke.
   #973 now owns the accepted direct transported Q/K/V/O softmax reference.
   Intrinsic/readout alternatives, resonance-based softmax replacement,
   full-model recurrent lowering, and exact deployment are parked. The
   provider-free-at-execution, source-backed `R4SoftmaxReferenceGeneratorV1`
   (`HELM-D-R4`) generation gate and opt-in, loopback-only dedicated native
   HTTP endpoint now pass. Dashboard wiring/readiness and static/WASM-isolation
   checks pass; the hosted Pages surface is static, currently reports WASM
   offline, and has no working chat backend/artifact lowering. The Q16 suffix
   trace student completed with bounded distillation but looping output; its
   recurrent `R4SoftmaxTraceStateStudentV1` successor then failed to produce a
   material or selection-bearing effect. The subsequent construction-only
   observability audit completed at `INSUFFICIENT_SUPPORT_COVERAGE` and cannot
   attribute a boundary. #1014 then directly trained the frozen R4/Spin
   causal-softmax model: its `2.677393`-nat attention-off penalty and two-arm
   Rust parity establish attention, but enabled NLL `2.127407` and
   subject/scene retention `3/5` fail its quality DoD. Close that exact campaign
   without tuning. #1017's separate exposure continuation then passed retention
   `5/5` and all mechanical gates but failed only sealed NLL at
   `1.5727521962806827`. #1019 now freezes that 12-layer parameter-capacity
   contract. Population, smoke, and random-export parity passed, but MPS
   admission stopped `UNAVAILABLE_HARDWARE_BUDGET` for the frozen eight-hour
   offline implementation; full training through replay remains `NOT_RUN`.
   The fused-AdamW/deferred-logging fast path was slower (`4.485223` versus
   signed `3.491307 s/step`); #1019 closed without a full run. #954's cosine
   pointer stopped before final artifact or product reveal; its implemented
   C1-SB2 relation successor then failed matched-transfer preflight before Rust
   parity/full fit/development/product and emitted no final head. The next
   proposal trains relation supervision into the existing R4/Spin attention
   representation while retaining exact-copy/typed-nonanswer behavior. CUDA and
   external GPU execution are out of scope. #954's final source-free terminal
   stays blocked behind #973, and #955 remains blocked behind #954.
See the [append-only #953 record](docs/local_geometric_generation_953.md).
See the [accepted table-tie record](docs/source_free_table_geometric_intervention_953.md).
See the [#973 Gate 0 record](docs/prior_sentence_count_radius_attention_973.md).
See the [#973 paragraph record](docs/paragraph_entity_spin_path_attention_973.md).
See the [#973 conversation record](docs/conversation_entity_spin_path_attention_973.md).
See the [append-only #973 bounded-global record](docs/bounded_global_exact_spin_attention_973.md).
See the [#986 evidence record](docs/corpus_signed_transport_attention_986.md)
for the exact feasibility boundary and deliberately unrun stages.
Stored H4/Hopf/zeta/icosian and related route fields remain
structural state, diagnostics, or controls unless the owning stage qualifies a
specific term.

These commands exercise the no-model research substrate. `demo` does not start
the historical artifact-discovery server, and `route` does not claim to answer
the prompt; it exposes how the current geometry represents it.

The browser-only WASM visualization is published at
[uor-foundation.github.io/uor-r4](https://uor-foundation.github.io/uor-r4/),
but the hosted Pages deployment currently reports WASM offline and cannot run
chat: it has neither the native reference backend nor a lowered compiled student
artifact. With `just` and `wasm-pack` installed, `just wasm-dashboard` builds the
local visualization surface without model weights. Neither surface is evidence
for attention, coherent generation, inference, or reasoning.

## What R⁴ is trying to build

The central hypothesis is simple:

> **The geometry is the route, and the data is the location.**

Text is reversibly assigned to canonical geometric addresses. As a sequence
unfolds, its route carries local and accumulated context. A bounded geometric
query evaluates possible next locations, chooses an admitted least-cost route,
and decodes that location back to text.

```text
text
  → reversible lexical address
  → prime / semiprime route
  → spin, phase, torsion, and radial state
  → current + sentence + conversation + global context
  → bounded next-route selection
  → text
```

The working design brings together:

- primes and semiprimes as addressable atoms and route experts;
- spherical harmonics as the working description of related spin states;
- fixed zeta-zero channels with changing phase and torsion;
- S³/R⁴ transport, Hopf projection, and golden-ratio radial shells;
- a paired-H4/E8 bridge for coupled geometric state; and
- recursive context at route, sentence, paragraph, conversation, and global
  scopes.

Kappa provides canonical identity and serialization. It is not itself the
tokenizer, semantic distance, attention mechanism, or language model. A pinned
lexical codec supplies reversible text boundaries; the intelligence must come
from the geometry.

## What exists now

The current foundation can represent and rebuild prime-route state, preserve
transported trajectory and overlapping context summaries, and perform bounded
deterministic candidate lookup.

It has **not** yet demonstrated:

- prompt-to-answer source-free chat;
- recursive geometric attention that generalizes beyond recall;
- a qualified natural grammatical generation loop;
- correctness and calibrated abstention;
- multi-step reasoning; or
- frontier-class capability or an energy advantage.

Earlier compiler, graph, proof, conformance, and teacher-derived systems remain
in the repository as research evidence and reusable components. They are not
the current product path and are not prerequisites for trying the dashboard.

## Current roadmap

The programme is deliberately sequential so that infrastructure and testing do
not become substitutes for working intelligence:

1. **Retain the established source-free table baseline (#989)** — 22.261404%
   held-out top-1 versus 5.413561% unigram on 446,342 known targets, exact
   bounded decoding, and byte-identical replay. Preserve its artifact and claim
   boundary as a statistical lexical reference.
2. **Retain the accepted R4 tie intervention (#953)** — 23.211797% held-out
   top-1, +4,242 correct choices over the unchanged table, a distinct bounded
   continuation, matched support and declared-work ledger, and byte-identical
   replay.
3. **Compile from the qualified `R4SoftmaxReferenceGeneratorV1` oracle (#973)** — retain the literal causal Q/K/V/O
   scaffold and V4's positive construction-scale connection-gauge covariance,
   but preserve its terminal held-out negative: H4, alternative, and plain were
   each 13/24 and the destructive controls did not separate. Pin and reproduce
   HELM-D as the bounded architectural reference, then preserve a frozen
   ordinary full decoder's learned Q/K/V,
   ordinary stable softmax, value aggregation, and output projection while
   splitting heads into R4 blocks, binding exact cumulative Spin/H4 frames,
   transporting every causal K/V pair into the query frame, and mapping the
   aggregate back before `W_o`. Require numerical/behavioral parity first on
   frozen real next-token loss, top-1, and decoded output against equal-budget
   plain controls. The first coefficient-only `acosh^2`/centroid intrinsic arm
   stopped unavailable at construction covariance and was diagnostically worse
   than donor and flat R4. The next source-faithful learned-manifold qualifier
   completed validly but failed functional retention and matched parity:
   Lorentz NLL `7.710618`, Euclidean `4.483154`, donor `3.667626`, while all
   geometry-destroying controls were worse than coherent Lorentz. The 8/8
   contract's score/readout attempt stopped at its two-document preflight and
   rejected tangent readout: pooled
   normalized audit-MSE ratio `1.0643688804269025`. Accept ordinary
   dot-product/stable-softmax causal attention in coherent R4/Spin frames as
   the current baseline. Park intrinsic score/readout, resonance,
   softmax-replacement, recurrence, and exact lowering. The smallest
   provider-free-at-execution, source-backed native CPU
   `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) path now passes its native CLI
   gate while retaining the credited HELM
   attention seam and using UOR's pinned
   SmolLM2 `HuggingFaceLlamaOracle` for embeddings, RoPE, residual/RMSNorm,
   MLP, final normalization, and the language-model head. The frozen gate
   recorded 4/5 quality in both passes, 5/5 exact replay after deleting timing,
   exact all-layer audits, zero future reads, and source-donor reproduction.
   Its explicit opt-in, loopback-only dedicated native HTTP endpoint now passes
   the frozen eight-token canary with exact CLI token, text, decision/state CID,
   all-layer-audit, and causal-read parity, without changing the default engine.
   Dashboard wiring/readiness and static/WASM-isolation checks pass; browser
   interaction/E2E is `NOT_RUN`.
   That trace/compiler rung produced `R4SoftmaxTraceStudentV1`, and the next
   recurrent `R4SoftmaxTraceStateStudentV1` rung completed with exact causal
   execution but no material control separation, no changed decision, and the
   same loop. Its #1012 full-trace/signed-reduction/state/readout audit then
   completed at `INSUFFICIENT_SUPPORT_COVERAGE`; it cannot localize signal loss
   and will not be expanded or repeated. #1014 then established load-bearing
   ordinary causal attention with a `2.677393`-nat attention-off penalty and
   exact Rust parity, but failed its complete quality gate at enabled NLL
   `2.127407` and prompt retention `3/5`. Close that campaign without rerun or
   tuning. #1017 then completed the one frozen exposure continuation: NLL
   `1.5727521962806827` failed the strict `<1.50` gate, while retention, parity,
   causal audits, and replay passed. #1019 now freezes an optional 12-layer,
   13,130,784-parameter increase over the same mechanism. Its population, MPS
   overfit smoke, and random-export/all-12-layer Rust preflight parity passed, but MPS
   admission stopped `UNAVAILABLE_HARDWARE_BUDGET` for the frozen eight-hour
   offline implementation; the full train/final-qualification/reveal/
   generation/replay path remains `NOT_RUN`, with no further 7.15M exposure or
   LR tuning. The fused-AdamW/deferred-logging fast path was slower (`4.485223`
   versus signed `3.491307 s/step`); #1019 closed without a full run. #954's
   cosine pointer stopped before final artifact or product reveal. Its
   implemented C1-SB2 relation successor then failed matched-transfer preflight
   before Rust parity/full fit/development/product and emitted no final head. The
   next proposal trains relation supervision into the existing R4/Spin attention
   representation while retaining exact-copy/typed-nonanswer behavior. CUDA and
   external GPU execution are out of scope.
   Do not resume resonance substitutes. Product development continues through
   `r4 generate`, but no production-readiness or release claim follows yet. This intermediate
   reference is transformer-compatible, `f32`/multiply/alloc and source-weight
   backed—not table-native, multiply-free, or transformerless. No tag, release,
   static web, or browser-WASM claim follows from the result.
4. **Establish correctness** — relevance, contradiction handling, and honest
   abstention.
5. **Establish reasoning** — bounded multi-step route composition.
6. **Connect and ship the accepted engine** — chat integration, measured
   optimization, and only then release QA.

The CLI and WASM dashboard remain usable research surfaces throughout this
sequence so each new mechanism can become visible before the final engine is
complete.

The active dependency chain is tracked in
[#820](https://github.com/UOR-Foundation/uor-r4/issues/820). #989 established
the frozen table reference, #953 established one matched R4 tie intervention
over it, and #973 retained one bounded prior-prefix copy mechanism plus bounded
exact-descriptor/entity-binding path selectors at paragraph and conversation
scope. Its first bounded-global relation remains closed-negative history; the
independently frozen V2 repair passed its bounded contract; and PR #997 rejected
the first natural componentwise placement. A bounded gated-delta core is
structurally implemented but negative against plain delta on its sealed smoke.
Direct-attention V2 is non-promotable; its equal-manifold-budget V3 rejects the
tested mixed-gauge H4 projection/connection/optimizer combination against a
working plain arm. Its `10/12` alternative-connection score is diagnostic only
because that arm was swapped at inference time rather than trained separately.
Connection/gauge Phase I is positive within #973, but its protected Phase-III
held-out reveal is negative: every main arm scored 13/24 and the destructive
controls failed to separate. `HELM-D-R4` source-pinned full-decoder softmax
parity in transported R4/Spin frames now passes. The first intrinsic
distance/centroid V1 attempt is unavailable before D3. The following
source-faithful HELM-D learned-manifold construction qualifier completed as a
valid functional-retention/parity negative despite clear destructive-control
separation. Its 8/8-contract score-by-readout attempt stopped at the
two-document preflight and returned
`REJECT_TANGENT_READOUT_SELECT_SCORE_PREFLIGHT`. Ordinary dot-product/stable-
softmax causal attention in coherent R4/Spin frames is therefore the accepted
current baseline; intrinsic/readout alternatives, resonance-based softmax
replacement, full-model recurrent lowering, and exact deployment are parked. The provider-free-at-execution,
source-backed `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) generation gate and
its explicit opt-in, loopback-only dedicated native HTTP endpoint now pass,
with no default-engine change. Dashboard wiring/readiness and
static/WASM-isolation checks pass, but the hosted Pages deployment is static,
currently reports WASM offline, and lacks a working chat backend/artifact
lowering. The source-free Q16 suffix trace student is complete and boundedly
positive but loops; `R4SoftmaxTraceStateStudentV1` also completed and failed
its material, decision, and cycle gates. The bounded construction-only
observability audit then completed at `INSUFFICIENT_SUPPORT_COVERAGE`; no
boundary attribution follows. #1014 subsequently established load-bearing
ordinary causal attention in the trained R4/Spin path through a `2.677393`-nat
attention-off penalty and exact Rust parity. Its full quality DoD is negative:
enabled NLL `2.127407` exceeded `1.50`, and subject/scene retention was `3/5`
versus `4/5`. #1017's separately frozen continuation improved those measurements
to NLL `1.5727521962806827` and retention `5/5`, but its full DoD remains
negative solely on the strict NLL ceiling. #1019's optional frozen 12-layer,
13,130,784-parameter campaign uses the same attention/runtime path.
Its population, MPS overfit smoke, and random-export/all-12-layer Rust preflight parity
passed, but the signed MPS probe stopped `UNAVAILABLE_HARDWARE_BUDGET` for the
frozen eight-hour offline implementation; the full campaign remains `NOT_RUN`.
UOR's deployed architecture/runtime remains CPU-native. Apple Accelerate/BLAS
and MPS are local offline accelerators only. The fused-AdamW/deferred-logging
fast path was slower (`4.485223` versus signed `3.491307 s/step`); #1019 closed
without a full run. #954's cosine pointer stopped before final artifact or
product reveal. Its implemented C1-SB2 relation successor then failed
matched-transfer preflight before Rust parity/full fit/development/product and
emitted no final head. The next proposal trains relation supervision into the
existing R4/Spin attention representation while retaining
exact-copy/typed-nonanswer behavior. CUDA and external GPU execution are out of
scope. #954's final source-free terminal remains blocked behind #973, and #955
remains blocked behind #954. The exact contract is
[ADR-0005](docs/adr/0005-predictive-geometric-connection-memory.md).

## Find your way around

- `src/` — the `r4` executable, local server, chat shell, and WASM surface.
- `crates/uor-r4-core` — current geometric route/manifest foundation plus
  preserved runtime research.
- `crates/uor-r4-router` — geometric router, memory, and dashboard backend.
- `crates/uor-r4-graph-*` — preserved graph-format/compiler/runtime research.
- `docs/` — current programme, mathematical decisions, evidence, and archive.

Start with the [documentation guide](docs/README.md). The
[R4 Intelligence Completion Plan](docs/r4_intelligence_completion_plan.md) is
the post-v0.1 sequencing authority and readable mirror of programme root #820;
the [Geometric Intelligence Programme](docs/geometric_intelligence_programme.md)
defines its architecture and claim boundaries. Historical records remain
available through the documentation guide without dominating the front door.

## Contributing

This is an obscure and ambitious research problem, and useful contributions
are welcome. The most valuable work advances the first unblocked roadmap stage
and produces an observable user-facing capability. Expensive experiments and
broad QA stay dormant unless a current decision truly requires them.

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a change.

## License

MIT — see [LICENSE](LICENSE). © 2026 UOR Foundation.
