# Historical GitHub issue #820 before native recovery

Source: https://github.com/UOR-Foundation/uor-r4/issues/820

This preserves the complete issue body retrieved on 2026-09-04. Its old current/next instructions are historical. New work follows [the canonical plan](../project-track.md) and [current implementation](../current-state.md).

---

## Current authority — architectural-alpha track (2026-09-04)

Protected [PR #1122](https://github.com/UOR-Foundation/uor-r4/pull/1122) advances [`docs/integration/project-track.md`](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md) after the fixed recurrent checkpoint in #1120. This is the canonical build sequence; older next-action prose remains historical evidence.

1. Fixed recurrent geometric memory — mechanical checkpoint delivered in #1120.
2. Sparse geometric attention — mechanical checkpoint delivered in #1122.
3. Nonlinear geometric block — **current stage**.
4. Scale, data, and instruction behavior.
5. Retrieval and tools.
6. Representative product alpha.
7. Rust/table lowering and optimization.
8. Release proof, evidence, and QA.

The architectural-alpha target is useful prompt-dependent text through bounded recurrent geometric memory and bounded geometric selection, without complete-prefix retention/scan or runtime teacher, provider, or source-model access. Transitional f32 learned Q/K/V/O, bounded softmax, RMSNorm, SwiGLU/MLP, vocabulary projection, and allocation may remain until their named replacement/lowering stages. Product alpha, release candidate, and release are separate later checkpoints.

SpiralCore, HELM, W33, NEMESIS, UOR, and H4/zeta are on-demand donor reservoirs. Consult one only when the active implementation has a concrete unresolved question; map any adopted mechanism into UOR and measure it directly. They do not form a serial proof or compliance gate. Preserve negative results at the exact artifact/population/operator/control/budget/decision scope; a materially versioned successor may re-enter with a named change and causal rationale. `UNAVAILABLE` is an execution boundary, not model evidence.

The #1120 fixed recurrent path is measured mechanical behavior: eight live K/V records plus four H4 summary banks, constant 2,304-f32 / 9,216-byte K/V state, at most 13 read sources, real evictions and summary reads, no new fit, and zero teacher/provider/future/forbidden reads in the frozen two-prompt comparison.

The #1122 sparse path keeps that state and the accepted learned Q/K softmax reader, but uses exact H4 metadata to admit at most eight persistent records plus current before gathering K/V. Across the same two no-fit prompts, peak read sources fell from 13 to 9 and materialized attention scores fell from 3,824 to 3,240 (15.27%); common generated prefixes were 12 and 3 tokens. This establishes a bounded executed mechanism. It does not establish language quality, useful retrieval, long-context retention, geometric advantage, architectural alpha, or table-native serving. The #967 shortest-Cayley and #970 q0/q1 heatmap negatives retain their exact tested scope.

#973 remains open and assigned. The concrete next action is one versioned finite R4 nonlinear operator block, retaining the sparse selector and dense SwiGLU as comparators, followed by one direct no-fit prompt comparison before scale work.

---

## Adopted current map — 2026-09-03

Active research: #973 retains #1079's bounded two-stage R4 preservation and weak token-control terminal. #1082 completed the construction-only exposure/displacement diagnostic with exact replay in PR #1093. #1085 delivered the clause-segmentation specification through PR #1095. Parked #1094 next implements the fixed text adapter and binds independently prepared comparison inputs/runtime before observation, preserving reader/core, known lexicon/query form and four facts. No parser, population, fit or evaluation ran in #1085. #953 and #986 are closed historical references. #954 remains blocked; the main chain is #973 → #954 → #955 → #962 → #963 → #964 → #965.

#1081 delivered the integrated workflow and source dossier in PR #1092. Parallel planned homes: #1083 typed UOR identity/arithmetic, #1084 native API/browser shell, #1086 native learned-artifact integration, #1087 final-kernel lowering, #1088 coding/workspace capability, #1089 research publication, #1090 capability/resource scorecard and #1091 NEMESIS/W33 mappings. Native parents/blockers control eligibility; only assigned issues are active. No frontier, general-language, coding or final-kernel capability is inferred from these planning changes.

[Current #973 scope and consumer boundary](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5520112973). The original root and all earlier checkpoints below are retained history; any earlier text called 'current' is dated to its own checkpoint. #940 stays dormant administrator work for release governance. The [current map](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/current-state.md) contains the #1082 result, completed #1085 contract and #1094 successor. [Latest research handoff](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5520332894).

---

# Authoritative programme root

This issue and milestone 21, **Geometric Intelligence — route-native local engine**, are the authority for post-#958 sequencing. Native GitHub parent/subissue and blocker relations control eligibility; this body mirrors them but does not replace them.

## Objective

Build a local CPU-first language engine in which canonical lexical identity, exact finite geometry, bounded table lookup, and incremental route composition replace serving-time transformers, MoE networks, learned routers, softmax attention, dense matrix intelligence, and provider-authored output.

## Current evidence and claim boundary

- #958 and #961 are closed foundations for the fixed schema-2 candidate substrate, reversible lexical codec, exact payload inversion, and hierarchy serialization.
- #952 closed with `REDESIGN_ORDERED_ROUTE_SUMMARY`: natural candidate/value plumbing worked, but the exposed summaries erased earlier order.
- #967 closed with `RETAIN_STATE_ONLY`: exact associative H4 ordered state worked, while shortest Cayley readout tied 6/6 queries.
- #970 closed through protected PR #972 with `RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q`: its exhaustive paired-H4 heatmap classes aliased outcomes and its construction readout transferred on 0/6 validation cases. This rules out that readout, not H4 or typed geometry.
- #969 closed through protected PR #976 at `bed6a94a1bc4c0597046f8a7020b4e3c78b1dfa4` with `PROCEED_TO_I1_WITH_CAUSAL_R4_PATH_ATTENTION`. Its fixed matched smoke decoded `rr ll` versus `ll rr`; both first winners used non-identity retained path memory while last-only abstained and state-disabled was prompt-inert.
- PR #977 merged the reusable provider-free decode, render, append, termination, and replay plumbing for #953.
- PR #978 froze the natural-agreement support hard stop under the former flat-union admission.
- PR #981 merged through the protected queue at `3520ae33529b2300ce923a6476a74fa0648fa816` and completed `PrimaryThenAdjacentSpinFallbackV1`. Exact `{still}` followed by `{run,runs}` support was restored under equal work. The one permitted four-arm run produced `still run` for both full-path prompts and `still runs` for both state-disabled prompts, with exact inversion, append, bounded termination, source/provider closure, and deterministic replay.
- #953 remains open, unassigned, parked behind #986 at `REVISE_I1_GENERATOR_IN_PLACE`. `LocalSameObjectContextPlacementV1` reproduced all seven construction prototypes with zero cross-candidate class collisions and zero padded/populated identity aliases. Its raw label-free, selection-blind census froze before minima or labels; real placement then resolved `run/runs` (0/2 intended) while the same-artifact placement-permuted control resolved `runs/run` (2/2). The preflight stopped before decoded generation and replay. Protected PR #982 merged the record at `d6862ccebb29a04a15d46c6931ad39b3690d3fc3`; no second representation is authorized by that contract.
- #973 remains the later paragraph/conversation/global qualification and corpus-scale induction lane. It remains blocked by #953 and blocks #954.
Exact recall, distinct state, serialization, candidate reachability, load-bearing local attention, coherent inference, higher-scope context, correctness, and reasoning remain separate claims.

## Accepted #969 local mechanism

The existing schema-2 natural union is hard sparse adjacency `A(i,j)`. Observed lexical routes compose an ordered unit-quaternion path `P(0)=1`, `P(k+1)=P(k) composed with route(x_k)` on S3. Retained earlier prefixes form causal attention memory. For admitted candidate `c`, `Q_t(c)=P(t) composed with route(c)`. Selection minimizes the exact `(round-S3 closure shell, causal lease age)` over retained prefixes; exact candidate-cost ties abstain.

The 120-root H4 table lowers these S3 states exactly as a finite codebook. The golden-coupled `H4 + phi H4` / E8 state remains structural storage and control, not the attention score.

The accepted smoke used identical natural support and candidate/key group-comparison budgets across full-path, last-only, and state-disabled arms. Its canonical record kappa is `blake3:60360a9e22a56ea4af363e43f7103bb8104d015d58feb582d921fc17afaf207f`.

This result establishes only a load-bearing identity-derived local mechanism. It does not establish semantics, coherent generation, correctness, reasoning, higher-scope attention, performance, chat quality, or release readiness.

## Admission and shared operator action

Candidate admission and operator/harmonic influence are separate contracts.

- I1/I2/ordered-sentence/divisor rows determine #953's primary lawful candidate set. The existing coarse adjacent-spin retrieval row is traced but may admit only as an explicit empty-primary fallback.
- #953 is parked and runs no selector while #986 is open. Its preserved regression contains the accepted #969 local causal-path selector, but only a full-positive #986 mechanism or a separately qualified table-value successor may enter #953 after a fresh label-free preflight. Paragraph, conversation, global state, and shared operator action remain selection-inert until #973.
- #973 reuses the existing exact signed-S3/Hopf/fiber/torsion `shared_class_kappa` to share or precompute one immutable global operator result per class. It may transform candidate-relative state after admission but cannot create candidates or rewrite route/payload identity.
- Direction is not inferred from the current prime-derived H4 witness. #973 must bind either an exact `SpinTorsionState` relative relation or an explicit spin-to-H4 map.
- Similar non-identical states require a separately frozen finite neighbor operator compiled independently from the coarse adjacent-spin retrieval rows. This is not an all-to-all corpus broadcast.

PR #981 repaired #953's measured support contamination. The first frozen same-frame candidate/context overlay then failed before generation because its deranged placement control outperformed real placement. #953 remains open, unassigned, and parked behind #986; #973 remains the later causal locus for higher-scope global influence and corpus-scale induction and stays blocked. Neither result establishes semantics, grammar, coherence, semantic placement, correctness, knowledge, or reasoning.


## Corpus-induced attention and hypothetical next routes

The working hypothesis is that corpus-induced structure can become useful only through a causal, candidate-relative receptive mechanism. More lookup rows, higher hit rate, prototype density, or trace activity alone are recall/capacity, not attention.

#953 executed the first tiny local test of that hypothesis using only bounded predecessor-history -> observed-next-candidate prototypes from the exact frozen construction provenance, one versioned noncommutative ordered frame, and candidates already admitted by `PrimaryThenAdjacentSpinFallbackV1`. Each decisive `still` route was observed from live singleton support, not injected. No evaluation continuation, expected answer, held-out label, actual future event, source tensor, teacher output, or provider text constructed or tuned the overlay. The raw census froze before minimum/tie semantics or labels. The frozen-contract real arm then scored 0/2 while the same-artifact placement-permuted control scored 2/2, stopping before generation. Complete held-out histories were absent, but retained suffixes exactly recalled shorter construction subhistories, so full-history disjointness did not establish operative-representation anti-recall.

This narrow #953 separation does not activate a broad construction/validation ladder. Corpus-scale placement, learned multiscale summaries, operator statistics, and paragraph/conversation/global qualification remain #973 scope after #953 is accepted and every required #973 scope qualifies, unless an explicit native revision changes that scope. Every structural or placement epoch must re-pass its bounded matched gate. Scale promotes only when the real arm changes a predeclared held-out anti-recall route and decoded output while matched disabled, placement-permuted, or order-shuffled controls do not reproduce the effect.

The committed hierarchy remains typed current/previous/last-two/sentence/paragraph/conversation/bounded-global state. #953 may change only the local comparison operand over already-admitted candidates. #973 may later add qualified higher-scope operator terms. Deeper hypothetical candidate branches, rollback, and comparison remain #955 work after #954. Actual future events, target continuations, evaluation answers, teacher outputs, and provider text never enter inference.
## Native programme order

All stages are native children of #820. Future stages remain unassigned until their open blockers clear.

1. Foundations — #958 and #961: closed.
2. Ordered-summary diagnosis — #952: closed.
3. Associative ordered state — #967: closed `RETAIN_STATE_ONLY`.
4. Paired-H4 readout identifiability — #970: closed `RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q`.
5. Local route-attention mechanism — #969: closed `PROCEED_TO_I1_WITH_CAUSAL_R4_PATH_ATTENTION` through PR #976.
6. Construction-return transfer — #983: closed `UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER` at 0/6 before selection.
7. Corpus-induced harmonic signed transport — #986: open, unassigned, and the only active local qualification; blocks #953.
8. Bounded decoded grammar/sentence loop — #953: open, unassigned, parked, and untouched at `REVISE_I1_GENERATOR_IN_PLACE`; decoded generation and replay remain `NOT_RUN`.
9. Higher-scope attention and corpus-scale induction — #973: blocked by #953; first qualifies the exact-spin global operator prototype under matched controls, then still owes paragraph and conversation qualification; blocks #954.
10. Correctness and abstention — #954: blocked by both #953 and #973.
11. Reasoning — #955.
12. Product chat/memory — #962.
13. Measured optimization — #963.
14. Formal serving closure — #964.
15. Release QA — #965.

The native chain is closed #970 -> closed #969 -> closed-negative #983 -> open #986 -> parked #953 -> #973 -> #954 -> #955 -> #962 -> #963 -> #964 -> #965. #940 remains dormant release-governance administration.

## Higher-scope and downstream boundaries

- #953 remains dormant behind #986. When its blocker clears, it begins only with the full-positive #986 selector or a separately qualified table-value successor after a fresh label-free preflight. Paragraph, conversation, and global state may remain serialized and incrementally updated but cannot influence selection.
- #973 qualifies those higher scopes through the accepted #953 decoded loop. A positive global subprobe cannot close #973; paragraph and conversation still require matched load-bearing qualification or an explicit native scope revision.
- #954 remains blocked by both #953 and #973.
- #955 invokes the full accepted local-selector/#953/#973/#954 path while retaining #969, #983, and #986 as evidence provenance.
- #962 integrates the accepted path into chat and identity-scoped memory.

## Serving boundary

- Serving loads no source weights and calls no transformer, MoE, learned router, dense-matmul intelligence path, Ollama, hosted model, or provider fallback.
- A provider cannot repair grammar, replace abstention, or author output.
- Exact stored continuations remain recall, not attention.
- Provider absence, deterministic bounds, and artifact identity remain observable and fail closed.

## Testing discipline

Build one mechanism, then activate only the smallest check that can change its next action. Routine workspace-wide testing, channel censuses, control matrices, broad corpus runs, teacher/model runs, and product QA remain dormant until their owning later stage. Missing or unexercised evidence is `NOT_RUN`, `UNAVAILABLE`, or `NOT_EXERCISED`, never PASS.

## Programme definition of done

A fresh local install must eventually ingest text into canonical route state and emit useful provider-free chat through the same lexical codec. Local attention, useful generation, higher-scope attention, correctness, reasoning, cost, formal closure, and release readiness are earned separately in the native order above.


## A1Q-L2 sequencing correction (2026-08-28)

The known #953 population is no longer an independent discovery population. Native child #983, **A1Q-L2: construction-transferred candidate-conditioned geometric attention**, froze `ConstructionCausalReturnV1` on a disjoint three-family, six-decision natural population, with closed #969 and #970 as direct evidence-bearing predecessors. Its usable construction classes were pure but covered 0/6 held-out decisions; after the separately sealed label join, the offline strict ceiling was also 0/6. The terminal is `UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER`.

The #983 hard gate stopped before any deployed selector, payload inversion, or #953 generation. #983 remains open negative evidence and owns the next representation decision. #953 remains parked and unassigned as an untouched integration regression; its fixture, selector, decoded generation, and replay remain `NOT_RUN`. No second representation runs under the failed #983 contract.

The corrected native sequence remains closed #967/#970/#969 -> open-negative #983 -> blocked #953 -> #973 -> #954 -> #955 -> #962 -> #963 -> #964 -> #965. #953 continues to block #973 and #954; #973 continues to block #954. Closed #974 remains dormant provenance. This correction supersedes this body's earlier immediate in-place #953 next-action language without rewriting the historical #953 terminal.

## A1Q-L3 sequencing correction (2026-08-28)

The completed #983 result remains `UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER`: its usable construction classes were pure, but construction-to-validation structural coverage and the sealed strict ceiling were both 0/6. The deployed selector, payload inversion, #953 generation, and replay were `NOT_RUN`. #983 is now closed as bounded negative evidence; its contract and evidence are not reinterpreted.

Fresh native child #986, **A1Q-L3: qualify corpus-induced harmonic signed transport attention**, owns exactly one successor contract. Construction-only corpus induction first has to produce transferable semantic placement/value and one `HarmonicLinkState7` row per route: self plus up to six directed PPMI peers. Six fixed half-restart Jacobi steps produce a candidate-rooted screened-harmonic reachability field. An immutable pre-geometry CID binds the canonically selected vertex-disjoint first 16/32 matched pairs, audit-only pair metadata, code/formulas, shared label-free induction scales, the equal-budget full/table margin rule, controls, pair-blocked randomization, and full-sweep work ledger with no #986 re-freeze. Only then may one candidate-relative signed zero-sum Cl(0,6)/SpiralCore transport arm claim causal value by beating separate placement/link, address, link-disabled, distance, direct-link, transport, order, last, state, operator, absolute-only, positive-only, and recall/count controls plus the table-native semantic-value comparator under equal information and work and exact pair-blocked tests. The rows/diffusion are substrate, not attention; OSPF is only the bounded link-state-propagation analogy.

Exact prime/semiprime, fixed-zeta, R4/S3, H4, `H4 + phi H4`, and SpiralCore structure remains immutable addressing, ordered state, transport, and control unless #986 measures additional causal semantic value. The fixed zeta list is reused rather than recalculated. Compiler-side float/allocation and the existing richer Rust algebra are permitted for this qualification; base-256/integer/LUT lowering follows only after causal transfer and must reproduce the frozen decisions.

The terminal branches are decision-bearing: a positive signed-transport result advances unchanged to #953 after a fresh label-free preflight; table-value transfer without geometric uplift retains geometry as address/transport and creates one table-value qualifier, which must independently qualify before entering #953; failed placement redesigns the induction objective; any frame/population/contamination failure stops. No generation or second representation runs in #986.

**Now:** #986 only, unassigned until execution starts. **Next:** #953 -> #973 -> #954. **Later:** #955 -> #962 -> #963 -> #964 -> #965. The native chain is closed-negative #983 -> open #986 -> parked #953 -> #973 -> #954 -> #955 -> #962 -> #963 -> #964 -> #965. This section supersedes every earlier present-tense #983/#953 ownership, assignment, selector, and sequencing statement in this body; dated results remain historical evidence.

## Capability-first B0 reset (2026-08-28)

The maintainer has explicitly overridden the attention-first successor direction after #986. The project now optimizes first for one observable capability: learn lexical next-route statistics from source-free text and emit decoded text.

**Now:** #989 only, **B0: source-free table-native lexical baseline from corpus to decoded text**. It compiles construction-only integer unigram/bigram/trigram tables from the pinned 3,000-document corpus, measures held-out next-route prediction against its own unigram baseline, and emits one deterministic decoded continuation. It contains no geometric ranking, attention claim, teacher, provider, or source-weight path.

**Next, only after B0 works:** retain the frozen table model as the reference engine. #953 may then test exactly one geometric intervention against the same corpus, candidate support, decoding path, and work. Geometry advances only if it improves held-out choices and decoded output beyond the working baseline.

**Later:** #973 and the remaining correctness, reasoning, chat, optimization, formal, and release stages. New H4, SpiralCore, harmonic-link, signed-transport, or algebraic qualifiers are frozen until the baseline exists. If B0 misses, revise the lexical representation or count objective in #989; do not add geometry or scale.

This section supersedes earlier present-tense sequencing that required another population/frame geometry successor. Historical issue results and records remain unchanged.


## B0 lexical baseline established (2026-08-28)

Protected PR #990 merged at `a5745a1d2e8b1de420b2dd7a3edaf699f612847a` and closed #989 at `ESTABLISH_TABLE_NATIVE_LEXICAL_BASELINE`. On the pinned 3,000-document D3 corpus, the construction-only integer table scored 99,362/446,342 (22.261404%) held-out known-target top-1 versus 24,163/446,342 (5.413561%) for unigram, a +16.847843 percentage-point uplift. Two complete executions produced byte-identical reports and 35,655,288-byte artifacts at `blake3:ccdc399731cb866a329be478467a434cda4e445813421e5d17c21ccc87288297`. The fixed prompt emitted 16 valid UTF-8 units without a period-1/2 cycle.

This establishes only a statistical lexical prediction and exact-decoding baseline. It does not establish semantics, attention, geometry, correctness, reasoning, chat, performance, or release readiness. The table artifact, corpus, support, decode, and work contract are frozen. Exactly one later #953 geometric intervention may compare against that reference; #973 remains blocked. No other H4, SpiralCore, harmonic, algebraic, placement, transport, higher-scope, or scale expansion is authorized by this result.

## Authoritative predictive-connection reset after PR #997 (2026-08-28)

This final section supersedes earlier present-tense `Now`/`Next` prose in this
body without rewriting the historical contracts or comments.

The programme goal remains a local CPU-first, geometry-native,
transformerless language engine: no serving-time Transformer, MoE, learned
sparse expert router, Ollama, hosted provider, source weights, or hidden
generative fallback. The immediate goal is geometric attention: past route
state must causally improve held-out next-route prediction and decoded output
under matched controls. GPT-2-scale local generation is a later engineering
target; GPT-3/ChatGPT-level quality remains aspirational.

Current evidence is separated:

- #989/PR #990: source-free table reference 99,362/446,342 = 22.261404% held-out top-1.
- #953/PR #991: one bounded R4 count-radius increment 103,604/446,342 = 23.211797%, +4,242 / +0.950392 percentage points with exact replay.
- #969: one bounded causal ordered-route mechanism.
- #973 PRs #992-#996: bounded synthetic prior-prefix, paragraph, conversation, and noncommuting-global witnesses; these do not establish natural semantic transfer.
- #973/PR #997: causal but harmful componentwise-Frechet placement—2,931/35,028 = 8.367592%, below unchanged #953 at 4,281/35,028 = 12.221651%, order-shuffled at 2,934, and operator-permuted at 2,966.

The binding diagnosis is not that geometry failed. Prime/zeta/R4/S3/H4 routes
remain identity, causal state, frame, transport, provenance, and compilation
infrastructure. What failed is treating a fixed marginal center of
identity-derived coordinates as predictive lexical semantics.

#973 now owns one stop-first successor,
`PredictiveConnectionRetentionGate0V1`, governed by repository ADR-0005. It
tests construction-transferred discriminative signal in current, previous,
ordered-last-two, and complete-prefix exact-route relations under frozen #953
support/work. A deterministic construction fit/validation split must beat
#953, a matched plain recurrence, state-disabled, last-only, order-shuffled,
and transport/readout-permuted controls before the protected D3 held-out targets
are opened. A positive authorizes the full
`PredictiveConnectionRetentionR4V1`: separate learned key/value/query roles,
connection-transported multiscale state, controlled forgetting and delta
writes, and candidate-relative readout. Compiler-side float/multiply/allocation
are allowed for the research pilot; exact table/integer lowering begins only
after causal held-out value is established.

**Now:** open, assigned #973 only. **Blocked:** #954. **Later:** #955 -> #962
-> #963 -> #964 -> #965. No duplicate issue or blocker-edge change is needed.
A bounded Gate-0 positive does not close #973 or unblock #954.


## Authoritative direct-attention checkpoint after PR #999 (2026-08-29)

This final section supersedes earlier present-tense `Now` and successor prose without rewriting historical evidence. The goal remains a local CPU-first geometry-native transformerless language engine; the immediate goal remains geometric attention.

The bounded `GeometricGatedDeltaRetentionR4V1` smoke is structurally valid but negative against its matched plain recurrence: geometric `16/28` next-token and `55/112` association versus plain `23/28` and `98/112`. Direct-attention V2 is `NON_PROMOTABLE_BUDGET_MISMATCH`. Fresh equal-manifold-budget V3 establishes that dense causal Q/K/V/O learning, softmax, and value aggregation work in the fixed-tangent plain arm (`12/12`), while the tested mixed-gauge H4 projection/connection/optimizer combination returns `3/12`. The H4 group action itself remains valid; the inference-time alternative-connection `10/12` result is diagnostic, not a separately trained winner.

**Now:** open, assigned #973 only, on `ConnectionGaugeCovarianceV4`. V4 uses explicit local coordinates and separately trained H4, alternative-tangent, and fixed-tangent arms, with pre-label frame/gradient/covariance checks and a fresh 24-case disjoint validation freeze. **Blocked:** #954. **Conditional sequence:** V4 -> paired-H4/E8 and fiber qualification -> corpus oracle -> positive multi-resonance replacement of softmax -> bounded recurrent factorization -> H4/Q29/integer lowering -> autonomous generation. Paired-E8, resonance attention, recurrent resonance state, exact lowering, and generation are not yet implemented evidence.

This is a mechanism-first gate. A V4 failure redirects the coordinate/connection/training seam; it does not authorize corpus expansion, additional route families, or a lookup-density substitute for attention. #973 remains open until a geometric arm beats matched non-geometric recurrence and geometry-destroying controls on fresh causal data with deterministic replay.

## Authoritative HELM-D-R4 reference result (2026-08-29)

This final section supersedes earlier present-tense V4 and HELM-D-R4 next-action prose without rewriting historical evidence.

#973 has established the bounded ordinary-attention reference in coherent R4/Spin frames. Pinned-source provenance, exact ordinary-donor replay, full-decoder R4 numerical and behavioral parity, an equal-work source-frame-permuted destructive control, and a zero-future-read work ledger passed. The canonical result CID is `blake3:05eaad210198fbe39a0645c25b0c890c55d5f3d3dd8a1710472e976a637e2a07`.

This positive proves only that ordinary causal softmax attention can be expressed through UOR R4/Spin frames in the complete local donor decoder. It does not prove an R4 predictive advantage, intrinsic geometric attention, softmax removal, transformerless serving, correctness, reasoning, or scale.

**Now:** open assigned #973 trains one intrinsic R4 distance plus geometric weighted-centroid attention arm against the frozen donor, gauge-equivalent R4 reference, equal-budget Euclidean/plain controls, and geometry-destroying controls. **Conditional:** only a positive intrinsic arm authorizes multi-resonance replacement; only a positive replacement authorizes bounded recurrence and exact lowering. **Blocked:** #954 and all downstream capability stages.

## Authoritative attention and native-reference generation checkpoint after PR #1008 (2026-08-30)

This final section supersedes earlier present-tense `Now`/`Next` prose without rewriting historical contracts or evidence. Protected PR #1008 merged as `184bdb939ec8b14ba16c87de749b51735fecd960`.

The programme has established ordinary learned-Q/K/V dot-product/stable-softmax causal attention in coherent R4/Spin frames, first by bounded full-decoder donor parity and now by autonomous selected-token feedback through all 30 decoder layers. `R4SoftmaxReferenceGeneratorV1` passed the frozen five-prompt smoke at 4/5 quality in both passes and replayed 5/5 exactly after deleting timing. Causal, projection, and R4 work records were exact with zero future reads. Its terminal is `PASS_R4_SOFTMAX_REFERENCE_GENERATION_ADVANCE_NATIVE_PRODUCT_BRIDGE`; record and aggregate BLAKE3 values are `34b6d7ddcdb858092ec7b192338fc236e5dfc3768fd8f4fb05dbe4ff2a23e82e` and `580360f44fe7163e697043ab8d6f801f578a5e7ed3d905209a166e7d303af6d0`.

HELM-D is credited as the MIT architectural reference at commit `7501deca8f413848bfef804be64ce874b72a3cd7`; no HELM checkpoint or HELM generation code executed. The running language model is UOR's pinned SmolLM2 `HuggingFaceLlamaOracle` around the R4/Spin transport seam.

The result is intentionally a source-weight-backed, floating-point/matmul, allocating, Transformer-compatible native reference. It is not the final source-free, table-native, multiplication-free, geometry-advantaged model; it does not establish correctness, reasoning, frontier quality, or release readiness. The static WASM page does not run it.

**Now:** #973 remains open on one explicit opt-in native HTTP/dashboard bridge of this exact reference policy, with no default-engine change and one bounded end-to-end prompt. **Blocked:** #954 and all downstream correctness/reasoning/product stages. Intrinsic score/readout, resonance, recurrence, scale, E8 expansion, and exact lowering remain parked until a new decision-bearing contract explicitly reactivates them. No release tag or hosted promotion is authorized.

## Programme checkpoint after PR #1009 (2026-08-30)

The accepted ordinary causal dot-product/stable-softmax attention reference in coherent R4/Spin frames now has a verified, explicit opt-in, loopback-only native HTTP endpoint. PR #1009 merged as 28acc8278c2c02b6f923b86c23eb5b728ab96bc0. Its frozen eight-token request matched the qualified CLI token sequence, decoded text, decision CID, and persistent-state CID; all 30 layer audits were exact and future reads were zero. Dashboard wiring/native-readiness and static/WASM-isolation checks pass; browser E2E is NOT_RUN.

This remains a source-backed SmolLM2 reference, not the source-free transformerless target. HELM-D is credited only as the MIT architectural reference at 7501deca8f413848bfef804be64ce874b72a3cd7; no HELM checkpoint or upstream generation code executed.

The primary programme direction is now the proposed R4SoftmaxTeacherTraceV1 plus R4SoftmaxTraceCompilerV1 rung under #973: construction-only causal layerwise traces from the exact oracle, then the first source-free student/attention-state comparison on decoded tokens and next-token loss. Both components are NOT_IMPLEMENTED and NOT_RUN. #954 and downstream correctness/reasoning/product stages remain blocked; intrinsic/readout, resonance, recurrence, E8 expansion, exact lowering, scale, hosted promotion, tag, and release remain parked.


## Programme checkpoint after the layerwise-normalized readout terminal (2026-09-01)

#973 completed the sole zero-parameter `R4LayerwiseNormalizedRetainedReadoutLanguagePathV1` trajectory and an independent fresh-process verification. The candidate improved fresh held-out NLL/top-1 to `3.7126411677` / `31.661826%` from qualified V1's `3.8850003883` / `29.728138%`; retained state was load-bearing by `1.3495375637` nats and 20,595 correct decisions. On independent V3 prompt swaps, gain improved to `0.0286980210` from V1's `0.0073316237`, with `339/512` wins.

The frozen absolute and incremental prompt-capacity floors still missed: `0.0286980210 < 0.0433216988` and `0.0213663973 < 0.0253415693`. Terminal: `LAYERWISE_NORMALIZED_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL`; result `blake3:35396bd6e64fc2c0bc7d86a84cc9e212ed913ce28e5353f5f2b8212b4cf2c532`; independent verification `blake3:3f316541dbab8061ed5ba891bf6a47ef22c55bca21fba01f6f97dbb3cb8497aa`.

Binding direction: end all parameter-free readout variants and make one freshly frozen learned associative binding/readout the sole #973 successor. Preserve qualified V1 as the bounded source-free attention baseline and the direct/layerwise candidates as evidence. No retry, scalar tuning, third normalization placement, generation, widening, or exact lowering is authorized from this result.

#973 stays open and assigned and continues to block #954. Correctness, reasoning, H4 superiority, exact/table-native runtime, browser productization, and release remain unestablished. C1-SB6 remains unauthorized; downstream sequence is unchanged.



## Programme checkpoint: predictive block-delta binding freeze (2026-09-01)

This final section supersedes earlier present-tense successor prose without rewriting historical evidence.

The learned candidate-leaf readout completed terminal `LEARNED_ASSOCIATIVE_READOUT_NO_CAPACITY` in protected PR #1037. It preserved load-bearing qualified V1 state and found a stronger pooled fresh-language signal, but neither geometric nor pooled prompt gain crossed the frozen capacity floors and geometry-destroying controls were not weaker. A complete write/address census then showed the true target leaf was occupied on only about 36% of construction and validation decisions. The failure is localized to candidate/address binding over V1's unchanged identity-write field.

**Now:** #973 alone implements the publicly frozen `R4PredictiveBlockDeltaBindingV1`: immutable qualified V1 plus a bounded four-bank R4 matrix memory that causally binds previous context to the subsequently observed token. It adds 9,228 trainables and 816 f32 state values and compares canonical H4 leaf/frame transport against an independently trained identity arm, fixed connection permutation, live additive no-delta-overwrite, and state-off controls. A five-minute revealed-V4 expressivity gate is binding before any new target is created. Only a pass authorizes one fresh jointly sealed V5/fresh-language run. Generation remains blocked until prompt capacity passes; geometry attribution is separate. #973 remains open/assigned and blocks #954. Correctness, reasoning, H4 superiority, exact/table runtime, productization, and release remain unestablished. CUDA remains out of scope.

## Programme checkpoint after V5 predictive binding (2026-09-01)

This final section supersedes earlier present-tense programme-direction prose without rewriting historical evidence. Protected PR #1038 merged as `6c7354c2b935a11f78daef9074c42f3442845685` and publishes the complete #973 V5 result and synchronized repository direction.

`R4PredictiveBlockDeltaBindingV1` completed validly at `PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY`, result CID `blake3:6c67544d675eafcb8eb9c0dabb93617e3f6c3295af812e8acbb687107c010a74`, independently verified at `blake3:567cf336eb05c3ec562aef7135f6fb35b580d02c758b0e79f2508ca21fba01f6f97dbb3cb8497aa`. Geometric prompt gain reached `0.03896945868086732` and `375/512` wins but missed the frozen absolute capacity floor; geometry attribution against both required comparators and delta-overwrite attribution also failed. Fresh-language and causal/integrity/replay gates passed.

The binding action is `STOP_WITHOUT_GENERATION`: retire this write/binding law, not ordinary attention or the whole programme. Qualified `R4RetainedLanguagePathV1` remains the bounded source-free attention baseline, and the coherent ordinary R4/Spin softmax reference remains established separately. The active product direction is the existing #1017 `r4 generate` path. No new #973 mechanism, generation rung, exact lowering, correctness, or reasoning experiment is implied without a separately frozen authorization. #973 stays open/assigned and blocks #954; #955 and the downstream sequence are unchanged. CUDA remains out of scope.

## Current programme checkpoint after #1059 (2026-09-02)

This section supersedes older present-tense next-action/status prose; frozen historical contracts and results remain evidence. Protected PR #1060 merged as `83e1e90d9fae8e92457e32f16223996f5389f4ed`.

**Completed:** `ZoologyCoherentR4InferenceV1` preserved #1050's learned recall through native coherent R4 attention. Plain and R4 both scored **11,900/12,000 = 99.1667%** with all 12,000 top-1 decisions identical. The deliberately inconsistent source-frame transport control scored **2,307/12,000 = 19.225%**, a **79.9417 percentage-point loss**, at matched causal support and work. Max logit difference was `2.002716064453125e-05`, attention difference `8.940696716308594e-07`, and NLL difference `8.900960288271698e-09`; all frozen criteria passed.

Unchanged learned tensors/tied head, full 8,192-token mapping, zero optimizer updates, and exact fresh-process replay passed. Inference reached 24 of 120 H4 frames. Combined run/replay was **27.49 seconds**, peak RSS **1.844 GiB**, on four CPU/Apple Accelerate threads. No #1057 model or checkpoint was opened. This is retained-open-population integration and transport sensitivity, not H4 superiority, new sealed generalization, softmax removal, source-free/table lowering, English context binding, or generation. Broad QA remained dormant; required protected-queue jobs acknowledged delivery.

Result CID: `blake3:bdf5a440562bf31a6c0d6d53cef0454270638b87508f0a758aaf9eb3a0031f7d`. Replay CID: `blake3:458f6f8817203e57089580d851971d7d32234c5d9e4edf96967984097bd7f181`. Complete implementation, preparation, raw result, replay and operator instructions are in the merged [#1059 record](https://github.com/UOR-Foundation/uor-r4/blob/83e1e90d9fae8e92457e32f16223996f5389f4ed/docs/r4_zoology_coherent_inference_1059.md).

**Next recommended action:** separately scope inference-only reuse of the same adapter on #1057's preserved final block-40 artifact and original exact-data population, with the retained **8,071/8,192 = 98.5229%** plain reference and its own newly frozen transport control. This checks transfer to UOR serialization/longer binding context before a language-context application. Preserve #1057's original near-target result and unrun historical control. No third training window or geometry expansion is implied. Additional coherent frame labels alone change coordinates, not learned capacity; a later expansion must name an actual representation, memory, update-law, or score change.

#973 remains open and assigned as the active programme lane and continues to block #954. Correctness, reasoning, chat, exact/table runtime, and release remain unestablished. Prior ordinary/R4 and qualified source-free retained-attention positives remain intact.

## Current programme checkpoint after #1061 (2026-09-02)

This section supersedes older present-tense next-action/status prose while retaining historical contracts and measurements. Protected PR #1062 merged as `62c64ccf2cba7674e27628775e78fb81c44baee0`; main's file tree matches the reviewed delivery.

**Completed:** `ZoologyExactDataCoherentR4InferenceV1` applied the unchanged #1059 adapter and native map to #1057's preserved final block-40 model and original exact-data population. Plain and R4 both scored **8,071/8,192 = 98.5229%**, with every prediction ID identical. The new inconsistent-transport control scored **1,009/8,192 = 12.3169%**, an **86.2061 percentage-point loss**, with identical causal support and work. Maximum logit/attention/NLL differences were `2.5272369384765625e-05`, `2.2351741790771484e-06`, and `7.450580596923828e-09`; all frozen preservation criteria passed.

Learned tensors and tied weights stayed unchanged. Exact fresh-process replay passed. Combined run/replay took **33.79 seconds** on four CPU/Apple Accelerate threads, with **2.411 GiB** peak RSS. No training, checkpoint/optimizer/evaluation-RNG loading, historical physical-binding-control read, native rebuild/export, or old QA ran. Vocabulary 4,096, sequence length 120, eight queries per row were covered by the unchanged 8,192-token native map; 24 H4 frames were reached. #1057's original near-target training terminal and unrun physical-binding control remain unchanged. This is preservation and transport sensitivity on previously observed development, not new sealed generalization or geometric superiority.

Result: `blake3:ac2ec4d533ac47d25f8eb9dfd7a41147147d73c0e2d9531352d9f9fb2eb84e58`. Replay: `blake3:af6c239ec2d0e11f26f50f74150c992dea345ec21257141fcec1096a573e708e`. Full preparation, raw result, replay and operator commands are linked in the [#1061 record](https://github.com/UOR-Foundation/uor-r4/blob/62c64ccf2cba7674e27628775e78fb81c44baee0/docs/r4_zoology_exact_coherent_inference_1061.md).

**Next recommendation:** freeze one small English supplied-context binding curriculum under #973 using the working attention architecture and R4 adapter. Hold the question fixed while supplied facts change, add distractors and swapped/missing-history controls, and bind lexical encoding, learning dose, and disjoint construction/development populations before fitting. These MQAR checkpoints are not English models; zero-shot English failure is not an attention-existence test. Preserve both learned artifacts. Further learning and geometry expansion are outside this completed inference-only delivery.

#973 remains open and assigned, blocking #954. Correctness, reasoning, chat, exact/table runtime and release remain unestablished. Prior ordinary/R4 and qualified source-free retained-attention positives remain intact.

## Current programme checkpoint after #1063 (2026-09-02)

This final section supersedes older present-tense next-action/status prose while preserving every historical contract and result.

Protected PR #1064 merged as 6df75837e55a4ffbbc760bb2bd9650ab27fea433 and closed #1063. The merged tree exactly matches reviewed head 311db033f566a7f094abd4f269ebf12dad7378ba (tree 3e4ae3f54999273d0d93c6568dc2c8a1fa559ff1).

The one frozen English curriculum completed all 3,920 updates and 2,007,040 answer presentations, but returned `ENGLISH_BINDING_CONSTRUCTION_MISS`. Final construction supported accuracy was 2,396/8,192 (29.2480%), held-out supported accuracy 218/1,024 (21.2891%), complete counterfactual groups 0/256 (0/128 for each question type), and missing-binding accuracy 37/256 (14.4531%). Changing the question with the facts fixed changed only 12/512 predictions; location swaps changed 106/512. These output counts do not establish an internal cause or a capacity ceiling.

Exact fresh-process replay passed. Combined fit, scoring and replay took 249.3656 seconds, maximum RSS 0.4856 GiB, on eight Apple Accelerate threads. The optimizer received zero development decisions. Model, checkpoint and exact data remain retained at the local issue-1063-zoology-english-binding evidence root.

- Final model: blake3:a4eb5ef76c387ca6ebe9f185b1a5ad023c81291ce4cc9000bb5d23248aaef282
- Result: blake3:aaca100c5c2b8abfb126937523c5cce44bb7e6ca2eb8d48260f42e9281606e0f
- Replay: blake3:dd5984c22d507faa1e2cea0f9b0c8051fbd3ec923cf53c896768e62708295e02

Conditional R4 inference and transport control are `NOT_RUN_ENGLISH_BINDING_MISS`. #1061 still preserves all 8,192 prediction IDs while scoring 8,071 correct, with its 86.2061-point inconsistent-transport loss. #1059 and #1057 artifacts and historical controls are unchanged.

Next: one separately frozen, construction-only diagnostic on the retained English model, with zero training. Classify target/same-owner/same-object/unrelated-location choices and question/location-swap responses by fact slot before choosing a narrowly targeted lexical/readout change. This diagnostic is proposed and unrun. The English surface requires gathering two query words into a constant colon readout, unlike MQAR's direct query-key readout; the missed fit does not establish that the architecture cannot learn those extra compositions. More geometry remains deferred.

#973 stays open and assigned; #954 remains blocked. General English understanding, H4 superiority, reasoning, chat readiness, softmax removal and integer/table lowering remain unestablished. Broad QA stays dormant; queue acknowledgements were transport only.

[Binding record and raw evidence](https://github.com/UOR-Foundation/uor-r4/blob/6df75837e55a4ffbbc760bb2bd9650ab27fea433/docs/r4_zoology_english_binding_1063.md).

## Current programme checkpoint after #1065 (2026-09-02)

This is the current checkpoint and next recommendation; earlier sections remain historical evidence. #1065 completed the construction-only diagnostic of #1063's retained final English model, delivered through #1066. Terminal: `CONSTRUCTION_DIAGNOSTIC_COMPLETE`; descriptive focus: `QUESTION_READOUT`.

The entire #1063 construction score reproduced exactly in all 13 fields: 2,396/8,192 correct (29.2480%), NLL, predictions, full selected logits and attention. Of 4,096 fixed-history question pairs, 3,974 (97.0215%) retained the same prediction and only 20 were both correct. Fixed-target question-logit contrasts were positive in 2,040 pairs and negative in 2,056. Supplied-location outputs were 6,905/8,192; the remaining 1,287 were UNKNOWN. No absent-location or other-vocabulary output occurred.

No overall displayed-slot or pooled q0 attribute-confound majority fired. Maximum slot share was 27.7625% with balanced 25% target exposure; q0 in-history errors were same-owner 841, same-object 834, unrelated 578. Smaller position and type-specific attribute effects remain visible, so this does not prove their absence or identify the constant colon as an internal cause. Location swaps changed 1,134/4,096 predictions, with 247 both correct. The two mean contrast summaries are algebraically linked, not independent evidence.

All 11 named focused checks and independent source/evidence/documentation review passed. Fresh-process replay reproduced complete evidence exactly. Combined execution was 3.433826875 s; peak RSS 0.774902344 GiB on eight CPU Apple Accelerate threads. Training updates, new development decisions, development/checkpoint/frame payload reads, new rows and geometry changes were zero. Queue statuses acknowledge transport only.

Result CID: `blake3:65b23631b10fe62b215411932cd9fe45f76b43d6b8503d0f2e74dc3d256c9b61`. Replay CID: `blake3:7222a680c300552ab097ce184500c90c0e44ede8248c4c3f752aa09f4232c0ca`. Record: `docs/r4_zoology_english_diagnostic_1065.md` and its preparation/result/replay JSON.

**Next recommendation:** one separately frozen fresh fit changing only the supervised answer readout from constant colon position 40 to query-object position 37, with matched attention cell, construction inputs/labels, seed, sampler, optimizer and 3,920-update dose. Report both question types; direct object access does not remove the need to distinguish owners. The literal next input token at 37 is `?`; this is an explicit answer-readout experiment. A new transfer claim requires previously unexamined frozen development data. No such fit ran in #1065.

Preserve #1063's negative and #1059/#1061's attention positives. More geometry, generation, reasoning, chat readiness, softmax removal and integer/table lowering remain deferred or unestablished. #973 remains open and #954 blocked.

Protected delivery: [PR #1066](https://github.com/UOR-Foundation/uor-r4/pull/1066), merged as `e1d3c431ce520a59572e9158828fecef50e4f793`. Current main tree `2cb0214692cf9da51b576b0f680bd96344362e47` matches the reviewed head exactly.

## Current programme checkpoint after #1067 (2026-09-02)

This checkpoint supersedes earlier next recommendations; earlier measurements remain historical evidence. #1067 completed one matched fresh fit moving the supervised answer readout from colon40 to query-object37, with all 41 input tokens, original construction/vocabulary bytes, cell, seed, sampler/dropout shapes, optimizer and 3,920-update dose preserved.

Terminal: QUERY_OBJECT_READOUT_CONSTRUCTION_MISS, with substantial partial progress. Construction rose from 2,396/8,192 (29.2480%) to 3,735/8,192 (45.5933%): +1,339 correct / +16.3452 percentage points. NLL fell by 0.229062326 nats. Object-changing question pairs were both correct in 447/2,048 cases (previously 6), while owner-changing pairs reached 47/2,048 (previously 14). Prediction changes were 1,413/2,048 for object changes and 193/2,048 for owner changes. Wrong-owner/same-object selections account for 1,439/2,131=67.5270% of q0 in-history errors; descriptive focus OWNER_DISAMBIGUATION. Position bias remains: slots 2+4 receive 74.2686% of in-history selections despite balanced targets. These localize remaining behavior, not its internal cause.

The 8,111/8,192 construction gate was missed, so fresh development has zero model decisions, NOT_RUN_CONSTRUCTION_MISS. R4 is NOT_RUN_SEPARATE_INFERENCE_STEP. The fit completed the exact 2,007,040-presentation ledger, including 1,846,452 supported and 160,588 unknown. Fit/evaluation/replay took 290.31 seconds, peak 0.7764 GiB. Eight named focused checks and independent source/preparation/evidence/documentation reviews passed; exact separate-process replay reproduced the complete evidence. Broad QA remains dormant; queue acknowledgements are transport only.

**Next recommendation:** one separately frozen fresh matched fit retaining readout 37 and adding the owner-token embedding from 35 to the object embedding at 37 before unchanged embedding dropout/attention: x37=E(token37)+P37+E(token35). No additional parameters or random draws; keep full construction rows/labels, seed, optimizer and 3,920-update dose. Owner-changing both-correct improvement is the primary decision, with preservation of overall accuracy and object-changing both-correct behavior; report position effects separately. Freeze exact criteria and a new unscored development population before fitting. The owner was already causal; this tests direct joint-query access. No additional fit ran in #1067. More geometry remains deferred.

Preserve #1063/#1065 history and #1059/#1061 attention positives. General English, reasoning, chat readiness, H4 superiority and softmax removal remain unestablished. #973 remains open and assigned; #954 remains blocked.

Result CID: blake3:c6dfcb3a856963ab4493c3d26bf729f6d9cad70147316ef2b9b62e87c3116369. Replay CID: blake3:98c799c7844e36d68b56c6948824c1dacb53fb36e9f38bc7c052ec6fe0873fac.

Protected delivery: [PR #1068](https://github.com/UOR-Foundation/uor-r4/pull/1068), merged as `385fa32dcaa33e1a34c253e96edea69ee7b93da4`. Main tree `01a085cf7002b0bb054fe7e8fc596c49990e7e9a` matches the reviewed head exactly. [Binding record and raw evidence](https://github.com/UOR-Foundation/uor-r4/blob/385fa32dcaa33e1a34c253e96edea69ee7b93da4/docs/r4_zoology_query_readout_1067.md).

## Current programme checkpoint after #1069 (2026-09-02)

This checkpoint supersedes earlier next recommendations; prior measurements remain historical evidence. #1069 completed one fresh matched owner-plus-object encoding fit: add E(owner35) to E(object37)+P37 before unchanged source dropout/attention. Full 41-token inputs, construction/vocabulary bytes, labels, parameters, seed and 3,920-update dose remain matched.

Terminal JOINT_QUERY_PRESERVATION_MISS. Construction rises 3,735→4,118/8,192 (45.5933%→50.2686%, +4.6753 points); NLL falls 0.1386159509 nats. Owner-changing both-correct pairs rise 47→338/2,048 (+291/+14.2090 points), passing 150. Object-changing both-correct pairs fall 447→376/2,048 (−71/−3.4668 points), failing preservation even though that family's individual correct answers rise 2,029→2,083/4,096. Preserve both gains and the pair regression; #1067 remains the accepted reference under the frozen decision.

Position dependence is substantial: target-slot accuracies 35.8887%,36.0352%,29.4922%,99.6582%. Slot 4 gains 690 correct answers; slots 1–3 collectively lose 307 (38.8021%→33.8053%). Neither DISTRIBUTED_BINDING nor a false single-slot majority establishes position independence. The restricted fourth-slot result is retained evidence, not general positional retrieval.

The exact 3,920-update/2,007,040-presentation ledger completed. Fit/evaluation/exact fresh-process replay took 286.891776 seconds, peak 0.785583 GiB. All 12 named focused methods plus independent source/preparation/evidence/documentation review passed. Construction missed 8,111/8,192; new development has zero model decisions, NOT_RUN_CONSTRUCTION_MISS. R4 is NOT_RUN_SEPARATE_INFERENCE_STEP. No extra dose, geometry change or generation. Broad QA remains dormant; queue statuses are transport acknowledgements.

**Next recommendation:** one separately frozen cyclic fact-order augmentation fit using plain #1067 readout 37, without the owner residual. Retain the four-fact task, all 41 tokens, labels, fresh seed/model/optimizer and 3,920-update budget. Rotate complete fact blocks through four offsets across successive traversals, keeping all variants of a world aligned and auditing actual exposures. Unique owner-object keys preserve label/absence semantics. Compare the final candidate and retained reference on canonical construction and all four cyclic rotations. Freeze both question-type improvement criteria plus an explicit worst-slot criterion before fitting; full binding and new frozen development remain separate. Four cyclic orders do not cover all 24 permutations. No augmentation fit occurred in #1069.

Preserve #1063/#1065/#1067 history and #1059/#1061 qualified attention. General English, fresh transfer, reasoning, chat readiness, H4 superiority and softmax removal remain unestablished. #973 remains open/assigned; #954 blocked. More geometry deferred.

Result CID: blake3:bc1066eb0e9bbf08304ab296ca0c1681b7e8af4b0ea9026945ebef83c7fb9d53. Replay CID: blake3:6a6ad3ce5ef9e9541fec4994006b39d7a15ce52432a1af3cff351f0b9d96fcf2.

Protected delivery: [PR #1070](https://github.com/UOR-Foundation/uor-r4/pull/1070), merged as `87c1610a7e58980e82eaf14d1ab1852bfc021fac`. Main tree `4cf574357512041de34d9522c70eacdb3d2d240f` matches reviewed head `b232cd59c1320f2bca4fe88b5f287c91beae87e4` exactly. [Binding record and raw evidence](https://github.com/UOR-Foundation/uor-r4/blob/87c1610a7e58980e82eaf14d1ab1852bfc021fac/docs/r4_zoology_joint_query_1069.md).


## Current checkpoint — #1071 (2026-09-02)

#1071 / #1072 completed `CYCLIC_FACTS_PRESERVATION_MISS`. Canonical 45.5933%→20.7764%; all four augmented orders 20.7764–21.0083% vs reference 40.8569–45.5933%. Both paired question types regress; every slot misses 50%; no complete candidate quartet in any order. Retain #1067. Exact 3,920-update dose and fresh replay; 315.59s/.78595GiB. Fresh development 0; R4 NOT_RUN. Fifteen focused checks/reviews pass; broad QA dormant, queue transport only.

Next: separately freeze a fact-level learned owner–object Q/K and location-V softmax prototype, shared across slots with learned null/full vocabulary head; fixed grammar roles, no equality matcher or target routing. Keep four-fact controls and bounded dose. Structured binding would not establish learned English parsing. More geometry deferred; #973 open, #954 blocked; prior attention positives preserved.

[Complete record and raw evidence](https://github.com/UOR-Foundation/uor-r4/blob/88b084450067aa770099072f85348b00adbc1e6b/docs/r4_zoology_cyclic_facts_1071.md). Protected merge `88b084450067aa770099072f85348b00adbc1e6b`; main tree matches reviewed head.


## Compound binding checkpoint (2026-09-02)

#1073/#1074: `COMPOUND_BINDING_FRESH_PASSED`. All four orders: construction 8192/8192 supported + 2048/2048 absent; fresh 1024/1024 + 256/256. Value cycling follows reassigned locations with attention unchanged; exact replay. Fixed grammar supplies roles. Next: unchanged-R4 preservation of the full four-fact-plus-null mixture; geometry deferred. #973 open; #954 blocked. [Evidence and delivery](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5518534791).

### #1075 completion (2026-09-02)

COMPOUND_R4_PRESERVED, delivered in #1076 at `45320652a03984cfe52046484d188cf07fb693a2`. All 46,080 coherent predictions and the full fact-plus-null mixture are preserved on observed #1073 populations; same-work broken transport loses 81.58–92.68 supported accuracy points across views. Exact full-evidence replay; 15.5921 seconds and 2.069 GiB. [Evidence and selected next action](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5518792331). Keep current geometry; separately freeze a learned language interface replacing fixed-position roles. No successor fit in #1075. #973 open; #954 blocked; broad QA dormant.

### #1077 completion (2026-09-02)

LANGUAGE_INTERFACE_HELDOUT_PASSED, delivered in #1078 at `6ebcf4bf48ed82addd0a38c7fe368b989815f772`. All 25,600 primary answers and 358,400 role decisions are correct across seen and withheld wording combinations; same-bag owner contrasts and value-cycle controls pass. The 141,571-parameter reader received one 512-update fit; the 286,976-parameter binding core stayed fixed. Exact replay, 49.9201s total, 1.457169 GiB. [Evidence and next action](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5519108070). Next: separately freeze unchanged-R4 qualification against this learned ordinary soft interface, keeping reader/core/geometry fixed. Given boundaries, known lexicon, seen query form and observed worlds limit the claim. No R4 run or additional fit in #1077. #973 remains open; #954 blocked; broad QA dormant.

### #1079 completion (2026-09-02)

PR #1080 at `e627252e525201815169ffd8364184953a46018d` delivers LANGUAGE_R4_PRESERVED_CONTROL_WEAK. All 25,600 predictions and 358,400 supervised roles are preserved; 156/156 primary criteria pass. Fact control is strong in 6/6 views; token control in 3/6. All controls are valid. Exact replay: 57.0422 s combined, 2.217148 GiB. [Evidence and next action](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5519385959). Next: separately freeze one construction-only token-stage exposure diagnostic on the two existing renderings. Keep the reader, core, data and current control fixed. Measure attention mass on changed frames, attention-weighted individual-value displacement and net pooled-role displacement, against recorded changed/retained answers. Distinguish limited exposure or cancellation from downstream tolerance; the cause is not established. No replacement control, retraining, new development evaluation, generation or geometry expansion follows from this issue. #973 stays open; #954 remains blocked. Broad QA remains dormant.



---

End of preserved issue body.
