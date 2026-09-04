# Roadmap

**Active track (2026-09-04):** The earlier artifact-only generation target was
a mechanical checkpoint. It is complete, but it is not architectural alpha,
product alpha, or release readiness. The exact definitions, research-reservoir
rules, and evidence boundaries live in the
[active project track](docs/integration/project-track.md). The machine policy is
`build_first_architectural_alpha`.

## Project sequence

1. **Fixed recurrent geometric memory — implemented mechanical checkpoint.**
   `R4FixedRecurrentCausalKVBindingV1` replaces the 120-slot full K/V cache
   with eight exact live records and four H4-local summary banks. Its 9,216-byte
   f32 K/V state is constant. The no-fit two-prompt comparison executed and
   read summaries after eviction; quality and long-context retention remain
   unestablished.
2. **Sparse geometric attention — next.** Implement bounded
   geometry-selected candidates and a bounded read operator without scanning
   the complete prefix. Compare against the accepted causal Q/K/V reference.
3. **Nonlinear geometric block.** Replace dense SwiGLU/MLP intelligence with a
   versioned R4 operator or separately typed E8/R8 operator bank and a measured
   nonlinear state transition.
4. **Scale, data, and instruction behavior.** Grow the bounded architecture and
   train on open development data until useful language, instruction,
   retention, and composition are measured.
5. **Retrieval and tools.** Add typed retrieval, ambiguity/refusal, real tool
   execution, feedback, and result ingestion.
6. **Representative product alpha.** Exercise grounding/abstention,
   composition, identity memory, coding, and tools through one local workbench.
7. **Rust/table lowering and optimization.** Preserve accepted behavior through
   the packed Rust runtime, then remove remaining float, multiply, allocation,
   and unbounded serving work.
8. **Release proof, evidence, and QA.** Reconcile proofs, claims, negative
   results, resources, portability, security, broad QA, scorecards, and
   publication against the actual release candidate.

[#973](https://github.com/UOR-Foundation/uor-r4/issues/973) owns the current
model track. #954 and the older capability chain remain historical issue
structure, not a proof queue between build stages. The #1107 workbench source
remains an unbuilt historical candidate; it returns when the model reaches the
product-alpha stage.

SpiralCore, HELM, W33, NEMESIS, UOR ecosystem work, and H4/zeta research are
on-demand donor reservoirs. SpiralCore's finite labelled E8 transitions may
inform sparse operator routing, and its typed state/refusal/table discipline may
inform contracts, but the attached browser implementation is not measured UOR
attention, recurrent memory, nonlinear model execution, tool use, or Rust/table
lowering. H4/R4 and E8/R8 remain typed separately.

Negative results retain their exact historical scope. A materially versioned
successor may re-enter with a named change and reason; `UNAVAILABLE` is not
model evidence. Bounded open-data development may iterate, while final held-out
evaluation and broad release evidence wait until design selection and release
candidate respectively.

## Historical roadmap

The sections below preserve older baselines and delivered components. Their old
next-action and proof-process wording does not override the current sequence or
the [build-first policy](docs/integration/agent-execution-policy.md).

## Historical established baseline and then-permitted successor

- [x] **#989 B0 source-free table-native lexical baseline — established.** The
  3,000-document D3 run produced 116,061 lexical routes and a 35,655,288-byte
  artifact at
  `blake3:ccdc399731cb866a329be478467a434cda4e445813421e5d17c21ccc87288297`.
  Table top-1 was 22.261404% versus 5.413561% unigram, the fixed prompt emitted
  16 valid UTF-8 units without a period-1/2 cycle, and full replay was byte
  identical. This establishes statistical lexical prediction and decoding
  only.
- [x] **#953 — accepted matched geometric intervention.**
  `MultiscaleCountRadiusR4V1` improved the frozen held-out table result by 4,242
  correct routes (+0.950392 pp), retained support/work equality and deterministic
  replay, and closed at
  `PROCEED_TO_A1Q_H_WITH_BOUNDED_SOURCE_FREE_GEOMETRIC_GENERATION`.

## Completed negative

- [x] **#986 A1Q-L3 corpus-induced signed transport qualification** — closed
  `UNAVAILABLE_FRAME_OR_POPULATION`. The source corpus and finite SpiralCore
  control reproduced, but the exact #986 population/codec commitment and
  lexical operator frame were unavailable. Gate 0, labels, all selection arms,
  replay, and #953 were `NOT_RUN`.

- [x] **#983 A1Q-L2 construction-transferred candidate-conditioned geometric
  attention** — closed at `UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER`: pure usable
  construction classes, exact controls, and zero operative recall did not
  transfer to any of six held-out decisions. The deployed selector, payload
  inversion, and #953 were `NOT_RUN`. Its successor handoff is #986; the failed
  #983 representation and evidence remain unchanged.

## Historical #973 experiment sequence

- [x] **#973 paired-H4 prompt-responsive-capacity arm — rejected.**
  `R4RetainedLanguagePathV1` is terminal `RETAINED_LANGUAGE_PATH_PASS`: held-out
  retained NLL `3.899862`, ordinary-control NLL `3.903394`, and state-off NLL
  `4.234849` with 16,660 lost top-1 decisions. Its retained-only successor
  completed `5/5` bounded local decodes with valid raw UTF-8, exact replay, and
  zero forbidden, future, source-data, target, teacher, or provider reads/calls.
  Autonomous retained decoding is established, but all five outputs drift from
  their prompts. The one frozen paired-H4 successor then slightly improved
  fresh-language NLL/top-1 while reducing structural repeats `97.5477%`, yet
  its prompt contrast was worse than V1 (`0.0062477543` versus `0.0063672952`;
  `282/512` wins). It terminated `PAIRED_H4_PROMPT_CAPACITY_FAIL` at
  `blake3:508a4ff352f1e533d669d9616f65b972b0f13e8efe35867b7b095281ad940274`.
  Preserve V1; reject and do not generate from the paired candidate. See the
  [V1 binding record](docs/r4_retained_language_path_v1_973.md) and
  [paired-H4 machine result](docs/r4_paired_h4_prompt_capacity_result_973_raw.json).

- [x] **#973 direct retained-readout seam — directional PARTIAL.** With V1's
  representation, recurrence, `252,160` parameters, `23,040` f32 state values,
  seed/data/order, and `2,730` steps fixed, candidate gain was
  `0.02158978940594819` versus matched V1 `0.007630419823799905` (delta
  `0.013959369582148285`), with `343/512` wins. Fresh held-out NLL/top-1 was
  `3.7374367988736603`/`31.542433%` versus
  `3.9010778352651876`/`29.632946%`; state-off cost `1.1234286047020587` NLL
  and `20,179` decisions. The absolute `0.04332169878499658` and incremental
  `0.025341569256760274` gain floors were missed. Mechanics/replay passed;
  generation/lowering are `NOT_RUN`. Result and verification CIDs are
  `blake3:71dd85e610dcc50b74cb2bb2068e5a1a433ac5df5db2a4f8fde22fb41735889c`
  and `blake3:b8ad3b6fa6d6ab9e429b3bd8d2a5060215d15230cd272e7272f27b7eef54785b`.
  See the [binding readout record](docs/r4_direct_retained_readout_prompt_capacity_973.md).

- [x] **#973 layerwise-normalized retained readout — directional PARTIAL.**
  With the exact fixed formula
  `E @ [N(h) + (g / sqrt(2)) * (N(a1) + N(a2))]`, candidate `g=1` produced
  prompt gain `0.02869802096506591` versus equal-work V1 `g=0` at
  `0.007331623694789724` (delta `0.021366397270276186`) and `339/512` wins.
  Fresh held-out NLL/top-1 improved to `3.712641167679153`/`31.661826%` versus
  `3.8850003882891597`/`29.728138%`; state removal cost `1.3495375636624845`
  nats and `20,595` decisions. It missed both frozen gain floors. Mechanics,
  replay, and the `13/13` independent verifier passed; generation, reasoning,
  and lowering are `NOT_RUN`. Result and verification CIDs are
  `blake3:35396bd6e64fc2c0bc7d86a84cc9e212ed913ce28e5353f5f2b8212b4cf2c532`
  and `blake3:3f316541dbab8061ed5ba891bf6a47ef22c55bca21fba01f6f97dbb3cb8497aa`.
  See the [binding layerwise-readout record](docs/r4_layerwise_normalized_retained_readout_prompt_capacity_973.md).

- [x] **#973 learned associative binding/readout — no capacity.** The frozen
  campaign completed `LEARNED_ASSOCIATIVE_READOUT_NO_CAPACITY`; independent
  verification passed. Neither learned arm met prompt-capacity, and geometry
  attribution failed. Preserve the pooled fresh-language signal only as a
  non-geometric control. Result and verification CIDs are
  `blake3:cedba37738ee249457bb589f716ee75afb16a0c4937c2a22ae9f917dd3eb97c1`
  and
  `blake3:443d711ce9a228e26e2eb2eebb55c582848424e2677c3473d41deaf8afd69ec7`.
  No tuning, retry, generation, or lowering is authorized from this rung.

- [x] **#973 predictive retained value write/binding law — terminal no
  capacity.** The independently frozen V5 campaign completed
  `PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY` and exact independent replay. The
  geometric arm gained `0.03896945868086732` with `375/512` wins and beat V1
  and pooled, but missed the `0.04332169878499658` absolute floor. Its margin
  over plain delta was `0.023929811749894725`, below
  `0.025341569256760274`, with worse own NLL; transport permutation passed.
  Delta overwrite lost to independently fitted additive at
  `-0.006512463228773413` and `234/512`. Integrity and fresh-language
  nonregression passed. Preserve the first scoring attempt as a harness-only
  `NOT_RUN`; recovery reused the frozen arms with zero retraining. Result and
  verification CIDs are
  `blake3:6c67544d675eafcb8eb9c0dabb93617e3f6c3295af812e8acbb687107c010a74`
  and
  `blake3:567cf336eb05c3ec562aef7135f6fb35b580d02c758b0e79f2508cae57065f5d`.
  Retire this law and `STOP_WITHOUT_GENERATION`; no generation, reasoning, or
  integer/table lowering is authorized. See the
  [V5 record](docs/r4_predictive_block_delta_binding_prompt_capacity_973.md).

- [ ] **#954 grounded correctness: C1-SB5 retired negative; final source-free
  terminal blocked behind #973** — retain the
  bounded positives, #997 placement negative, bounded gated-delta negative, V2
  budget invalidation, equal-manifold-budget V3 mixed-gauge H4 negative, and
  V4's held-out 13/24 functional/control negative. The positive attention
  reference is `HELM-D-R4`; the active generator is
  `R4SoftmaxReferenceGeneratorV1`: the pinned HELM-D provenance and frozen ordinary
  full-decoder donor, preserve learned
  Q/K/V, ordinary stable softmax, value aggregation, and `W_o` while transporting
  R4-block K/V through exact cumulative Spin/H4 frames; numerical plus
  real-language behavioral parity now pass. Preserve intrinsic Lorentz V1
  attempt 02 as unavailable before D3 on covariance, with diagnostic curved NLL
  worse than donor and flat. Source-faithful learned-manifold V2 then produced a
  valid non-D3 construction-validation negative: learned Lorentz failed donor
  retention and matched Euclidean parity, although donor/gauge parity and
  destructive-control sensitivity passed. The 8/8-contract attempt stopped at
  its two-document preflight and rejected tangent readout. Accept ordinary dot-product/stable-softmax causal
  attention in coherent R4/Spin frames as the baseline. The smallest provider-
  free autonomous `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) native CLI
  path now passes with HELM-D credited only as the architectural reference and
  UOR's pinned SmolLM2 `HuggingFaceLlamaOracle` decoder path. Its exact policy
  also passes through a dedicated opt-in, loopback-only native HTTP endpoint
  with no default-engine change: the frozen eight-token prompt matches the CLI
  tokens and CIDs, audits all 30 layers exactly, and reads no future token.
  Dashboard wiring/static native-readiness and WASM-isolation checks pass, but
  hosted Pages currently has no working chat backend/artifact lowering.
  `R4SoftmaxTeacherTraceV1` and the source-free Q16 suffix student are complete:
  bounded distillation is positive, autonomous generation still loops; the
  state-student and observability successors closed negative. #1014 established
  load-bearing ordinary attention. #1017 then completed at `149,995,520`
  cumulative tokens with NLL `1.5727521962806827` as its sole failed gate and
  retention/parity/audits/replay passing. #1019 freezes an optional successor:
  depth 12, width 288, six 48-wide heads with twelve R4 blocks per head,
  13,130,784 parameters, seed 1019, 16,800 steps, and 275,251,200 tokens.
  Fresh development and confirmation tranches passed create-once population
  validation. The 400-step, 64-sequence MPS overfit smoke passed with
  `81.9752%` loss reduction, and random-export/all-12-layer Rust preflight parity passed
  with maximum absolute logit delta `0.0000443459`. The signed 200-step MPS
  probe passed memory at `21.03%` but failed time at a safety-projected
  `20.66 h` against the `8 h` ceiling, ending that frozen offline implementation
  `UNAVAILABLE_HARDWARE_BUDGET`. Full training, final trained-checkpoint
  qualification, the strict `<1.50` sealed-NLL reveal, generation, and replay
  remain `NOT_RUN`. Its fused-AdamW/deferred-logging fast path was slower
  (`4.485223` versus signed `3.491307 s/step`), and #1019 closed without a full
  run. #954's fixed 384-step MPS grounding SFT then completed in `883.773549 s`,
  but all three frozen Accelerate-backed Rust prompts decoded `ABSTAIN`; only
  the unsupported prompt passed (`1/3`). Do not rerun or tune that SFT. Its
  `R4SourceSpanPointerV1` successor passed 12/12 overfit and Python/Rust parity,
  then missed the frozen `>=95%` development gates at answer `89/128`, abstain
  `114/128`, conflict `117/128`, and supported pointer `121/128`. No final head
  was emitted; the product probes and browser/HTTP wiring are `NOT_RUN`. Do not
  tune or retry the revealed cosine pointer. The independently frozen
  `R4SourceRelativeRelationHeadV1` successor then passed its construction fit at
  `12/12` positive, `20/20` negative, and `6/6` copy, but sealed transfer reached
  only `5/12`, `14/20`, and `0/6`; every semantic control except candidate
  order failed. It stopped before Python/Rust parity, the full fit, development,
  or product reveal and emitted no final head. C1-SB3 then produced bounded
  attention-representation transfer but missed exact promotion. C1-SB4's
  independently frozen full-source record-margin successor failed at `70/126`
  fit and `35/63` sealed exact records and stopped before Rust/checkpoint/
  product; no retry is authorized. C1-SB5 then fit all `56/56` paired records
  but reached only `14/28` sealed. Its row-swap control was bit-exact;
  mean-query and attention-off were `0/28`. Products remained unopened and no
  checkpoint/head/Rust/development stage followed; retire the rung without
  retry. No product wiring is active. Apple
  Accelerate/BLAS and MPS remain local offline accelerators only; CUDA and
  external GPU execution are out of scope. See the
  [frozen contract](docs/r4_softmax_parameter_capacity_1019.md) and
  [signed preflight/admission result](docs/r4_softmax_parameter_capacity_preflight_1019_raw.json),
  then the [#954 record](docs/r4_grounded_correctness_954.md) and
  [C1-SB4 aggregate](docs/r4_joint_candidate_margin_954_raw.json), followed by
  the [C1-SB5 aggregate](docs/r4_paired_query_binding_954_raw.json).
  Do not extend
  exposure or tune learning rate on the 7.15M checkpoint. Intrinsic score/readout
  alternatives, resonance-based softmax replacement, full-model recurrent
  lowering, and exact deployment are parked. D3 remains `NOT_RUN`; #954 remains
  open, its final source-free terminal remains behind #973, and #955 remains
  blocked.

## Landed

- [x] **#969 causal R4/S3 path attention** —
  *`PROCEED_TO_I1_WITH_CAUSAL_R4_PATH_ATTENTION`, 2026-08-27*. One exact
  identity-derived local path selector changed a matched two-unit decoded
  continuation under equal natural support and work. It established a
  load-bearing local mechanism only. See the
  [#969 record](docs/local_geometric_attention_969.md).

- [x] **#952 A1.0 ordered-state/value gate** —
  *`REDESIGN_ORDERED_ROUTE_SUMMARY`, 2026-08-27*. Three of three frozen
  matched contrasts collided across all seven hierarchy levels and all 46
  non-digest fields, so no scorer was built. The real schema-2 path naturally
  admitted both continuations under ceiling eight, inverted both exact values,
  and reproduced incremental next state. Exact H4 and SpiralCore finite tables
  closed as controls only. At that checkpoint, repair was #967 and #969 still
  blocked #953. See the
  [A1.0 record](docs/recursive_geometric_attention_a1_952.md).

- [x] **#967 A1R associative ordered-state repair** —
  *`RETAIN_STATE_ONLY`, 2026-08-27*. The exact H4 fold passed the frozen scope,
  independent-global, group/fold, incremental, and support invariants. The full
  arm produced distinct candidate-relative states, but shortest Cayley distance
  mapped both to energy 2 and tied on 6/6, so no attention or generation claim
  follows. Report kappa:
  `blake3:f0db7a5d5c81d51ebf3b4bf8a2715c4960ec16b14161e8bf7598d7b98c48c881`.
  #970 subsequently found eight incompatible exact paired-H4-derived R4-heatmap
  classes and a 0/6 strict transfer ceiling, retaining the heatmap and auxiliary
  channels as structural/control state while stopping before readout or
  placement. #970 closed through protected PR #972; #969 then delivered the one
  causal path mechanism directly. See the
  [A1R record](docs/associative_ordered_route_summaries_a1r_967.md).

- [x] **#961 reversible lexical geometry/state plumbing** —
  *`PASS_REVERSIBLE_STATE_PLUMBING_ONLY`, 2026-08-27*. The pinned codec,
  address-to-payload inversion, canonical hierarchy, incremental consumer
  trace, and concrete paired-H4 witness landed without claiming attention or
  generation.

- [x] **#958 fixed-zeta prime-route foundation** —
  *`RETAIN_STORAGE_RECALL_ONLY`, 2026-08-26*. Source-free algebra, the complete
  schema-2 manifest/rebuild witness, bounded candidate mechanics, matched
  controls, and worker compilation landed. The product probe and teacher
  comparison were correctly `NOT_RUN`; no lexical generation loop or API/chat
  caller existed. This is the retained foundation for GI-1/#961, not evidence
  against geometric intelligence. See the
  [qualification](docs/prime_route_attention_qualification_958.md).

- [x] **#932 exact-parallel live-teacher parity instrument** — *negative
  prerequisite outcome, 2026-08-25*. Deterministic scheduling, durable
  observability, planted negatives, and fail-closed preflight are implemented.
  The selected canonical 135M bundle predates #933's schema-2 production
  envelope, so the preflight refused before opening teacher weights and the
  live tuner/parity marathon did not run. See the
  [append-only record](docs/teacher_parity_parallelism_932.md).

- [x] **#933 normative R4G1 serving and deployed-quality reconciliation** —
  *RATIFY, 2026-08-25*. One `R4G1Runtime` candidate/token owner now reaches the
  production adapters; the canonical schema-2 bundle clears same-position TLA
  and the frozen +20‰ sections-absent gate with zero surface/binding/witness
  failures and strict empty-store admission. The result is scoped to its exact
  measured envelope.

- [x] **Wrap transformerless into r4** — *done 2026-07-18*. The full
  transformerless program is integrated into
  [`uor-r4-core::transformerless`](crates/uor-r4-core/src/transformerless),
  rebased onto the UOR substrate (`src/tless_uor.rs`: uor-addr addressing,
  `TlessAxis`, per-prediction `Grounded` witnesses), and exposed at
  `/api/tless/{predict,index,generate}`. The old repository is superseded.
- [x] **`up`/`run` locally** — `./uor-r4-cli` orchestrates download → compile →
  score → serve → interactive client in one command.
- [x] **API `/v1/chat`** — OpenAI-compatible `/v1/chat/completions`, plus
  `/v1/models`, `/v1/status`, `/v1/reload`, `/v1/corpus`.
- [x] **Web-based chatbot** — the browser dashboard (`index.html`) with engine
  selector, telemetry, tokens/sec speed metric, semantic map and dev-mode
  toggle.
- [x] **Attestable / audit logging / provenance** — UOR attestation envelopes
  (`uor_address`, `artifact_cid`, `store_cid`, `attestation_cid`),
  `POST /api/uor/verify`, per-turn audit log rendered by `r4 audit`.
- [x] **GNAF vendored (#653 phase 1, PR #742)** — the pinned
  `WASM-GEMM-GNAF` Lean4 proof project is vendored at
  `proofs/wasm-gemm-gnaf/` with full provenance
  (`docs/gnaf_import_provenance.md`). It is a proof that a WASM GEMM kernel
  is cost-optimal, unrelated to text generation; source and provenance only
  — not wired into any pipeline, not in the deployed dependency graph. Phase
  2 (integration matrix, PR #765) landed and #653 closed 2026-08-17; its
  "defer-open" items are not part of the active geometric-decoder programme.
- [x] **Ready-by-default serving epic (#655, closed 2026-08-19)** — A–F
  complete: uor-matmul-owned compile arithmetic; shared bundle loader +
  sidecar manifest/verifier; Production/Experimental engine profiles
  (Production default, r4g1-only admission, typed declines); **`r4` as the
  canonical served identity** on every surface (`uor-r4` a deprecated request
  alias); **seeded sampling as the decode default** (pinned seed, greedy
  opt-out; 15/15 valid on the declared in-domain canary vs 0/15 greedy); the
  CLI ask path applying the **same deployed D4 abstention policy** as the
  server tier (#811). Execution records:
  `docs/serving_default_flip_655_f.md` + the #655 close comment.
- [x] **Release v0.1 + pipeline (#741, closed 2026-08-19)** — tag-triggered
  workflow (draft release binding code SHA + contract version; native CLI for
  linux x86_64 + macOS arm64; wasm frontend), the packaged model bundle as an
  attested GitHub Release asset, and the explicit hard-verified
  `r4 install-release --tag v0.1` fetch, proven end to end against the live
  release. `docs/RELEASE_PIPELINE.md`.

## Capabilities

- [~] Text-based AI — the repository has the retained #958 route/storage
  foundation, working geometric memory/retrieval, and historical compiled
  runtimes, but no source-free geometric decoder has established coherent
  product behavior. GI-1/#961 closed the lexical route loop, while #952 showed
  that its reusable non-digest hierarchy summaries erase earlier order. #967
  repaired that representation but retained it as state only; #970 then found
  the bounded paired-H4-derived exact R4-heatmap readout non-identifiable on its
  frozen union. #969 then qualified one local causal selector, and #953 has
  implemented its decoded-loop plumbing while the tiered policy passed its
  frozen preflight. The repaired natural agreement run still made the same
  full-path choice for both prompts, producing the historical
  `REVISE_I1_GENERATOR_IN_PLACE` terminal. Its first bounded
  construction-induced placement preflight then
  failed at a frozen-contract real-placement ceiling of 0/2 while the cyclic
  placement-permuted control reached 2/2; generation and replay were `NOT_RUN`.
  #983 then tested `ConstructionCausalReturnV1` on an independent natural
  population and stopped `UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER` at 0/6
  before its deployed selector. #986 then stopped
  `UNAVAILABLE_FRAME_OR_POPULATION` before geometry because its exact
  population/codec commitment and complete lexical SpiralCore frame were
  unavailable. B0/#989 and the later accepted #953 intervention then exposed
  #973. Its bounded paragraph, conversation, and V2 global mechanisms are
  retained; its first document-scope corpus placement passed target-free but
  failed held-out promotion. The bounded gated-delta core then trailed plain
  delta on its sealed smoke. `HELM-D-R4` ordinary-softmax parity remains
  qualified; intrinsic Lorentz V1 attempt 02 stopped unavailable before D3;
  and source-faithful learned-manifold V2 is a valid non-D3
  construction-validation negative. Its controls established sensitivity, but
  learned Lorentz failed retention and parity. The 8/8-contract attempt stopped
  at its two-document preflight and rejected tangent readout. Provider-free
  autonomous `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) generation now passes;
  its dedicated opt-in, loopback-only native HTTP endpoint now passes at the
  frozen eight-token parity scope; dashboard wiring/static native-readiness and
  WASM-isolation checks pass while hosted Pages remains static/offline without
  a functioning chat backend/artifact lowering; the source-free Q16 suffix
  trace student is complete with bounded distillation but looping output;
  the state student and observability audit closed negative; #1014 established
  load-bearing attention; and #1017 closed NLL-only negative after passing
  retention, parity, audits, and replay. #1019 is an optional frozen
  parameter-capacity improvement: population/smoke/random-export parity passed,
  MPS admission stopped `UNAVAILABLE_HARDWARE_BUDGET` for the frozen eight-hour
  offline implementation, and the full campaign remains `NOT_RUN`. Its fused-
  AdamW/deferred-logging fast path was slower, and #1019 closed without a full
  run. #954's first fixed grounding SFT completed but failed its frozen Rust
  population `1/3`. Its cosine source-span pointer then passed preflight/parity
  but failed the frozen development gate before a final artifact or product
  reveal. The independently frozen `R4SourceRelativeRelationHeadV1` successor
  then passed construction fit at `12/12` positive, `20/20` negative, and
  `6/6` copy but failed sealed transfer at `5/12`, `14/20`, and `0/6`; every
  semantic control except candidate order failed. It stopped before parity,
  full fit, development, or product reveal and emitted no final head. C1-SB3
  then produced bounded attention-representation transfer but missed exact
  promotion. C1-SB4's full-source record-margin successor failed at `70/126`
  fit and `35/63` sealed exact records and stopped before Rust/checkpoint/
  product; do not retry it. C1-SB5 then fit `56/56` pairs but reached `14/28`
  sealed and retired before checkpoint/head/Rust/development; products remained
  unopened.
  CUDA and external GPU execution are out of scope. Intrinsic score/readout alternatives,
  resonance-based softmax replacement, full-model recurrent lowering, and exact
  deployment are parked. D3 remains `NOT_RUN`; #954 remains open, its final
  source-free terminal remains behind #973, and #955 remains blocked.
  Coherent product behavior, correctness, and reasoning remain unestablished.
  Historical canaries and pointwise results remain scoped evidence, not a
  claim that the current product works. See
  [Which track can actually produce coherent text](docs/RESEARCH.md#which-track-can-actually-produce-coherent-text--the-honest-current-answer).
- [ ] Image
- [ ] Audio

Stretch:

- [ ] Video
- [ ] vLLM

## Tooling

- [ ] agentic harness
- [ ] agentic tooling (loop-based calls)
- [ ] `SKILL.md`
- [ ] collaborative prompting
- [ ] Tauri desktop shell

## Cloud service

- [ ] `deploy` (to cloud) — command-based; stack
- [ ] analytics
- [ ] persistent and transient VMs
- [ ] egress/ingress replacement

## Self-host

- [ ] deployment to your own cloud — tooling

## Hologram

- [ ] app SDK

## AI features

- [ ] Identity
- [ ] RBAC / policy
- [ ] analytics
- [ ] storage — document tokenization
- [ ] compression
- [ ] context (storage)
