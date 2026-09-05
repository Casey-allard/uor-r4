# Formal Vocabulary, Notation, and Claim Classes

- **Version:** 0.1.35
- **Status:** Normative for all new specification, plan, proof-model, and certificate text.
- **Source:** `docs/hologram_formal_analysis_direction.pdf` §§1, 7, 13; tracker
  [#122](https://github.com/UOR-Foundation/uor-r4/issues/122); issue
  [#123](https://github.com/UOR-Foundation/uor-r4/issues/123).
- **Related:** terminology for existing graph-compiler concepts lives in
  `docs/transformerless/GLOSSARY.md`; this document governs *claim language* and
  mathematical notation. Where the two overlap, this document wins for claim
  classification and the glossary wins for structural terms. The route-scope
  decision is [ADR-0004](adr/0004-geometric-intelligence-route-hierarchy.md),
  and current execution follows the
  [native geometric AI policy](integration/agent-execution-policy.md).
  [Geometric Intelligence Evaluation](geometric_intelligence_evaluation.md)
  retains historical experiment protocols. A preserved #973 terminal contract is
  [`R4PredictiveBlockDeltaPromptCapacityV5`](r4_predictive_block_delta_binding_prompt_capacity_973.md).

**Current authority (2026-09-05).** This document governs claim language and
typed mathematical distinctions. The [project plan](integration/project-track.md)
owns the goal, and [current-state.md](integration/current-state.md) owns the
implemented model and next work. Dated results and table cells naming a
"current" reference, a parked operation or an unimplemented boundary retain
their original artifact scope; they do not select a new task or prohibit a
versioned native successor. Preserve their measurements without importing a
reference model's capability or runtime guarantees into the active model.

This spec separates the statement classes that the research notes previously mixed —
architectural definitions, compiler optimization objectives, empirical certification
claims, and structural runtime guarantees — and fixes the symbols that recur across
the compiler, artifact, certificate, and proof model.

## 1. Claim classes (role of a statement)

Every equation and every normative mathematical statement in repository documents
MUST carry exactly one of these role labels, written inline in bold
(`**Definition**`, `**Objective**`, `**Guarantee**`, `**Assumption**`,
`**Empirical Criterion**`):

| Label | Meaning | Where it may appear |
|---|---|---|
| **Definition** | Introduces an architectural object or mathematical term. True by convention; never "proven". | Anywhere |
| **Objective** | A quantity the offline compiler attempts to optimize. Never a runtime invariant, never a theorem. | Compiler-side docs/code only |
| **Guarantee** | A structural property of the compiled artifact or runtime. Requires a claim status from §2 and a proof artifact, witness, test, or explicit `Unproven` record. | Artifact spec, runtime docs, proof model |
| **Assumption** | A condition a proof or certificate requires but the implementation does not itself establish. | Proofs, certificates |
| **Empirical Criterion** | A measured property with a declared distribution, protocol, sample count, and uncertainty. Never stated as a proof. | Certificates, evaluation docs |

Design aspirations are not theorems: an optimization target (`arg max …`) is an
**Objective**, and a measured approximation (`P_θ(a|c) ≈ P_θ(a|Z(c))`) is an
**Empirical Criterion** until the protocol behind it is declared.

## 2. Claim status (evidence behind a statement)

Every **Guarantee** and every **Empirical Criterion** MUST also carry one of these
statuses. The statuses are the machine-readable vocabulary of the proof-status
matrix (`crates/uor-r4-proof-model/src/proof_matrix.rs`); the mapping is normative.

| Status | Meaning | `ProofStatus` mapping |
|---|---|---|
| **Structural** | Established by construction or by a named machine-checked test, fuzz target, or formal proof artifact that has actually run for the declared evidence. | `Verified` |
| **Witnessed** | Established per execution by a bounded, replayable witness an independent verifier replays without the teacher. | `Verified` (witness path) |
| **Empirical** | Measured on a pinned corpus with protocol, confidence intervals, and provenance identifiers, per `docs/transformerless/EQUIVALENCE_AND_EMPIRICAL_PROTOCOL.md`. | `DifferentialPass` |
| **Assumed** | Required by a proof or certificate but not established by the implementation. | recorded as an assumption entry |
| **Unproven** | Asserted as a goal but currently without evidence. | `Unverified` (blocks `verify_all`) |

`ExecutableSpec` in the proof matrix means the obligation has a runnable
specification but no activated binding check yet; documents must label such claims
**Unproven** (or **Assumed** where applicable), never **Structural**.

### 2.1 Wording rule (automated check, dormant by default)

The prohibited phrases are **"machine-verified"**, **"machine verified"**, **"exact equivalence"**, **"exact teacher equivalence"**, and **"provably equivalent"**. They are prohibited in
normative documents (`docs/**`, crate READMEs) unless the same line links a proof
artifact or certificate, or the phrase is explicitly disavowed on that line.
`scripts/check_claim_wording.py` enforces this when explicitly activated (see §6). The project does not
claim exact teacher equivalence (disallowed per this rule), does not claim
human-level reasoning, and does not treat plausible language output as evidence of
coherent internal state transitions.

## 3. Core notation

Symbols are listed with their claim role and their concrete Rust binding. Entries
marked *compiler/reference-only* have no deployed-runtime representation yet; the
runtime path for them lands in the issues noted.

| Symbol | Role | Meaning | Rust / artifact binding |
|---|---|---|---|
| `x_t^ℓ` | **Definition** | Hidden/residual state at token position `t` and decoder layer `ℓ`. | Active geometric-decoder/source-control state; exact binding lands under #950/#952. |
| `q_t^ℓ, k_i^ℓ, v_i^ℓ` | **Definition** | Learned query, causal prefix key, and value contribution. These predictive roles are distinct from immutable R4G1 route codes, addresses, and CIDs. | The #950/#951 G0/G1 mixer and `DirectCausalGeometricAttentionR4V1` are retained comparators. The current offline reference is `HELM-D-R4` under ADR-0005; its first parity arm leaves every learned Q/K/V and `W_o` unchanged. |
| `p_t, e_t, N_t` | **Definition** | Registered prime route atom; canonical unordered semiprime transition expert `e_t = p_(t-1)*p_t`; and bounded ordered n-let. If adjacent atoms repeat, `e_t = p_t^2` is retained. The square-free case is the subset `p_(t-1) != p_t`. `N_t` retains order and a multiplicity-preserving sorted factor multiset rather than relying on an overflowing numeric product. | Schema-2 source-free manifest records under #958. This binding establishes representation and deterministic recall substrate only. |
| `Gamma = (gamma_0,...,gamma_(m-1))`, `theta_j(p)`, `delta_theta_j` | **Definition** | `Gamma` is a finite ordered, revisioned zeta-ordinate grid. `theta_j(p) = wrap(gamma_j*log(p))`; `delta_theta_j` is the declared local phase difference. | Compiler/reference math and bounded compiled signatures under #958. Critical-line use is an **Assumption**, not a proof of RH; no serving or capability claim follows. |
| `s_t in S3 subset R4`, `h(s_t) in S2 subset R3`, `tau_t in S1` | **Definition** | `s_t` is the full normalized local spin state, `h` is its many-to-one Hopf observation, and `tau_t` is retained quantized fiber/transport phase (“torsion”). `h(s_t)` alone is not a rebuild identity. | Quantized source-free route records and experimental bounded lookup under #958; no physical or quantum interpretation is claimed. |
| `theta=atan2(sin(theta),cos(theta))`, `a(theta)=sin(theta)^2`, `chi(theta)=sign(sin(theta))`, `pol(theta)=sign(cos(theta))` | **Definition** | S3/R3 trigonometric transition contract. `(sin=+/-1,cos=0)` has activation `1`; `(sin=0,cos=+1)` is continuous-null activation `0`. Chirality and, where needed, cosine polarity preserve orientation/antipodes. `tan(theta)` is a local chart only. | At a tangent pole or declared null boundary, the architecture switches angle/cotangent chart and records a signed quarter-turn phase plus torsion shift instead of dividing by zero or terminating. Semantic value is **Unproven**. |
| `m_E=sqrt(2)`, `m_C=2i`, `m_R in [0,2]` | **Definition** | Typed least-cost adapter markers: Euclidean orthogonal-unit chord, complex/discrete antipodal displacement, and declared normalized Riemannian/chord score interval. They are not literal equalities of Euclidean, complex, discrete, and Riemannian domains. | A target-specific cost profile binds chart, units, orientation, quantization, error bounds, canonical tie-break, and conversion witness before choosing the cheapest faithful operation. |
| `r_t = a_t + b_t*phi in Z[phi]` | **Definition** | Direction-preserving golden radial shell. `phi:(a,b)->(b,a+b)` and `phi^-1:(a,b)->(b-a,a)`. Repeated steps have Fibonacci recurrence, which does not establish semantic value. | Exact coefficient updates and manifest chart profile under #958; product use remains separately qualified. |
| `I1, I2, IS` | **Definition** | Bounded causal indexes keyed by last route, ordered previous-plus-last route, and ordered sentence-route identity. | Incremental identity, exact rows, and an off-serving geometric-attention experiment exist under #958. Exact hits are recall; attention and product claims require the evaluation policy. |
| `e^(i*pi)+pi^0 =_bridge 0^0`, `b_0` | **Definition** | Typed zero/identity transition. `ContinuousNull` preserves the complex cancellation as `0`; `DiscreteEmptyProduct` phase-shifts/retypes the seam into the discrete identity `1`. `=_bridge` is a domain-transition operator, not ordinary numerical equality. The selected tag is part of route identity and deliberately exposes the `-1 / 0 / +1` landmarks without assigning two simultaneous values in one untyped calculation. | `ZeroPowerBridge` and schema-2 manifest binding under #958 select the boundary value; the full phase-shift transition remains an architectural contract for the route hierarchy. |
| `C_lex` | **Definition** | Deterministic lexical codec from pinned input bytes/tokenization to registered route atoms and back to output bytes where decoding is defined. Normalization, unknown-unit behavior, tokenizer CID, registry, and codec version are part of its identity. | Product binding remains **Unproven** until a provider-free serving path exercises the same codec end to end. |
| `B_ico : Lambda_E8 ~=_Z I -> (x,x') in R4 ⊕ R4`, `Phi_E8 = H4 ⊕ φH4` | **Assumption** | **Project shorthand:** `E8 = H4 × H4`. The load-bearing implementation contract realizes that conceptual identity through the chosen icosian presentation: the E8 lattice and icosian ring are identified as `Z`-modules, one state is represented by golden/Galois-coupled R4 points, and the declared 600-cell folding is `H4 ⊕ φH4`. This is a typed construction, not a claim that `R4 = E8`. | Basis, glue/parity rule, conjugation, scale, orientation, root order, and inverse witness must be kappa-bound. Architectural load-bearing is assumed; held-out advantage remains unproven. |
| `kappa(X)` | **Definition** | Canonical content identity of the schema/provenance-bound byte envelope `X`. Equality identifies equal canonical envelopes under the named digest contract; digest distance has no routing meaning. | Manifest/artifact CIDs. Kappa is integrity and provenance, not a semantic code. |
| `R_t^q`, `q in {local,sentence,paragraph,conversation,global}` | **Definition** | Identity-scoped, causal route accumulator at hierarchy scope `q`. A parent scope commits to ordered child identities and bounded geometric summaries; it does not rescan all descendant tokens at query time. | ADR-0004 contract; scopes beyond the current local/sentence substrate are **Unproven**. |
| `T_t^q = (v_session, w_window, E_proj, F_shared, c_res, alpha_Hopf, B_ico)` | **Definition** | Bounded transported trajectory/harmonic summary for scope `q`: session hypersphere vector, winding/window state, projection energy, shared-prime factors, cosine resonance, accumulated Hopf phase, and paired-H4/E8 coordinate. It is updated incrementally from the full transported path, not reconstructed from only the last route. | Ancestor routing evidence motivates these fields; current held-out locality when exact hierarchy keys miss is an **Empirical Criterion**, status **Unproven** until reproduced. |
| `W_cov` | **Definition** | Bounded coverage witness recording codec/address coverage, hierarchy rows read and hit/miss, candidates before/after admission, controls, selection or abstention, and artifact identities. | Coverage establishes reachability only; capability status comes from a separate empirical record. |
| `A_recall`, `A_geo` | **Definition** | `A_recall` retrieves a stored continuation by exact/backoff route identity. `A_geo` selects or orders admitted causal support using declared geometric terms and must be load-bearing against matched controls on anti-recall inputs. | Exact-row recall is implemented substrate. Geometric-attention advantage is an **Empirical Criterion**, currently not implied by implementation presence. |
| `F_i^b`, `P_(j -> i)^b` | **Definition** | `F_i^b` is the exact cumulative Spin/H4 orthogonal model-frame basis for R4 head block `b`; local encoding is `transpose(F_i^b) x`, and `P_(j -> i)^b = transpose(F_i^b) F_j^b` moves both locally encoded causal `K_j^b` and `V_j^b` into query frame `i`. The current vector action is a compiler-side f64/f32 oracle, not exact deployed arithmetic. | `HELM-D-R4` uses this declared orthogonal gauge transport before comparison and aggregation. It is not called Levi-Civita or shortest-geodesic transport without a separate proof. |
| `B_c(g)`, `theta_x^r` | **Definition** | `ConnectionGaugeCovarianceV4` represents each predictive role by the same three local coefficients `theta` in a declared tangent frame `B_c(g)` and transports them by `B_c(d) transpose(B_c(s))`. | V4 passed construction covariance, all 120 frames/14,400 ordered pairs, and finite-difference gradients, but its protected held-out reveal scored 13/24 for every main arm with insufficient destructive-control separation. It is frozen negative evidence. |
| `alpha_(i,j)`, `r_i` | **Definition** | In the `HELM-D-R4` parity arm, `alpha_(i,j)` is the unchanged stable causal-softmax normalization of the donor's ordinary compatibility score after K transport, and `r_i = F_i sum_(j<=i) alpha_(i,j) P_(j -> i) transpose(F_j) v_j` is mapped back before unchanged `W_o`. | This is the accepted gauge-equivalent reference-attention baseline. Its bounded held-out full-decoder parity result and native `R4SoftmaxReferenceGeneratorV1` gate are **Pass**. The dedicated opt-in, loopback-only HTTP endpoint additionally passes exact eight-token CLI token/decision-CID/state-CID parity with all 30 layers audited and zero future reads. Dashboard wiring/static native-readiness and WASM-isolation checks pass; browser interaction/E2E is `NOT_RUN`. These are numerical/behavioral source-donor/reference results, not geometric advantage or a deployed transformerless-runtime mechanism. The generator uses UOR's pinned SmolLM2 `HuggingFaceLlamaOracle` decoder path and remains Transformer-compatible and source-weight backed. HELM-D is an MIT architectural reference only at `7501deca8f413848bfef804be64ce874b72a3cd7`; no HELM checkpoint or code executed and no upstream result is inherited. See `helm_d_r4_softmax_decoder_973.md`, `r4_softmax_reference_generation_973.md`, `r4_softmax_reference_generation_attempt_01_result_973.json`, and `r4_softmax_reference_http_bridge_973.md`. |
| `R4SoftmaxTeacherTraceV1`, `R4SoftmaxTraceCompilerV1` | **Definition** | Construction-only trace schema and compiler. The trace binds causal layerwise token, Q/K/V, attention-support, value-aggregate, decoded-output, and logit states from the exact qualified reference. The first compiler emits matched teacher-distilled, observed-count, and document-permuted Q16 suffix arms without granting future or target inputs at inference. | **Pass** at the bounded source-free-distillation scope: context-bearing covered teacher CE was `2.660721` versus count `9.678894` and permuted `4.342019`; teacher top-1 was `3/9` versus `2/9` and `1/9`; actual-next top-1 was `2/9`, tied with count. Artifact/reveal CIDs and replay were exact, causal audits passed, and student inference made zero source calls. Decoded continuation collapsed into a `, Scotland` cycle, and the student does not use geometric trace state, so this is not geometry advantage, coherent generation, reasoning, or a deployed transformerless runtime. See `r4_softmax_trace_student_973.md`. |
| `R4SoftmaxTraceStateStudentV1` | **Definition** | Bounded source-free state transition and next-token readout over the captured query-gauge Q, transported K/V support, weighted aggregate, and decoded model-frame trace, compared with the established suffix student, an equal-budget non-geometric recurrent state, and a transport-permuted control. | **Completed negative** at the frozen #1011 gate. On the same nine context positions and `422,875` Q16 teacher mass, covered CE / teacher top-1 / actual-next top-1 were suffix `2.660721032`, `3/9`, `2/9`; plain `2.660770919`, `3/9`, `2/9`; geometric `2.660705367`, `3/9`, `2/9`; and transport-permuted `2.660729215`, `3/9`, `2/9`. The geometric gains of `0.000015665` nats over suffix and `0.000023848` over permutation were below the frozen `0.10` threshold; no top-1 decision changed, the control lost none, and all arms retained the period-two `, Scotland` cycle. Exact replay and causal/runtime audits passed. State/freeze/seal/result CIDs are respectively `blake3:b617fc38e7bef1cdea76991f6e5e7cc653118451d63bcbd595f8ffd7e247ae7b`, `blake3:67cf67bb46b94cf5644b8dde286e89adb7e49159b3749790dffb500d8047fedb`, `blake3:64587526f7883ab046e884a28b6af7e9e89818c9ead2039f8c995de7fb483060`, and `blake3:dc04a8a8b21750799db2d451c8237d1e62cf90ffa74561fb54272b1e9704c824`. Terminal: `STOP_R4_SOFTMAX_TRACE_STATE_STUDENT_REPAIR_OR_RETIRE_REPRESENTATION`. This falsifies the current 4D signed-reduction/token-derived state cell, not ordinary R4/Spin softmax attention. See `r4_softmax_trace_state_student_1011.md`. |
| `R4GroupAddressedRetentionLMV1`, `M_t^b(h)` | **Definition** | #973's fixed-size source-free state: four banks hold an R4-block field over 120 addresses, updated through one frozen group action and read candidate-relatively with separate learned query/value tables. Exact-H4 is compared with equal-size cyclic-120 and destructive scrambled-H4 actions; a state-off intervention tests whether retention changes logits. | **Construction terminal:** `UNAVAILABLE_FRAME_POPULATION_OR_LOCAL_BUDGET`. Geometry, population, reachability, gradients, memory, equal work, and held-out sealing passed; timing and disposable learning smoke failed. Main optimization and held-out model criteria are `NOT_RUN`; there is no attention or H4-advantage verdict. The exact cell is not retried or tuned. See `r4_group_addressed_retention_973.md`. |
| `R4GroupAddressedRetentionDecoderV1CpuRecovery` | **Empirical Criterion** | Independently frozen complete construction run of the group-addressed retained read/write law inside an exact 3.17M-parameter, two-block decoder recipe. A state-off intervention measures whether retained state is load-bearing; exact-H4 versus scrambled transport measures geometry-specific separation. | **Completed:** state-off on the disjoint construction-validation partition lost `0.967227` nats and 182 top-1 hits, qualifying a bounded causal retained-attention component. Aggregate validation CE moved `8.371911 -> 8.976155`, so the exact data/dose/parameter recipe did not satisfy its frozen full-decoder generalization criterion. Scrambled transport was `0.033049` nats better, so no H4-specific advantage follows. Formal H4 specificity is `NOT_EVALUATED`. Result CID `blake3:68355ad2f61d02dc73dbf22de4c24834815a23069ed5735630dc365081cf91db`; see `r4_group_addressed_retention_decoder_cpu_recovery_973.md`. |
| `R4RetainedLanguagePathV1` | **Definition** | A 252,160-parameter source-free language path using the frozen group-addressed retained-state law, compared under equal parameter budget with an ordinary causal-softmax decoder and under a state-off intervention. | **Qualified at its declared scope:** retained NLL `3.899862`, ordinary-control NLL `3.903394`, and state-off NLL `4.234849`; state removal lost 16,660 correct next-token decisions. A separate retained-only smoke produced five valid, exactly replayed local continuations without forbidden/provider/source reads, but every output drifted from its prompt subject or scene. This establishes load-bearing retained attention and autonomous local decoding, not prompt-conditioned coherence, H4 superiority, reasoning, or exact lowering. See `r4_retained_language_path_v1_973.md`. |
| `R4PairedH4LanguagePathV1` | **Definition** | The one frozen V1-capacity successor that holds training data, seed, schedule, parameter count, and work fixed while replacing the shared per-layer H4 token address with a deterministic paired-H4 address. | The address construction reduced repeated joint addresses by `97.5477%`. On fresh held-out language it scored NLL `3.8832293739` and top-1 `29.780171%`, versus V1 `3.8901151940` and `29.706357%`. This is construction/general-language evidence only; it does not establish prompt capacity or H4 superiority. |
| `g_d`, `G_prompt` | **Empirical Criterion** | For each of 512 bidirectional matched-prompt contrasts sharing the final four prompt tokens, `g_d = (log P(y_d | own prompt) - log P(y_d | paired prompt)) / 16`, and `G_prompt` is the mean. Candidate promotion requires `G_prompt >= 0.043321699`, at least `308/512` directional wins, and the separately frozen capacity-gain rules. | **V1 completed negative:** paired-H4 candidate `G_prompt = 0.0062477543` versus V1 `0.0063672952` (delta `-0.0001195409`), with `282/512` wins. Both state-off contrasts were exactly zero; causal, forbidden-read, artifact-before-reveal, CID-binding, and replay checks passed. Terminal `PAIRED_H4_PROMPT_CAPACITY_FAIL`; result CID `blake3:508a4ff352f1e533d669d9616f65b972b0f13e8efe35867b7b095281ad940274`. Preserve V1, reject the paired candidate, and do not run generation. See `r4_paired_h4_prompt_capacity_result_973_raw.json`. Later V2/V3 results are recorded by their named readout terms below. |
| `R4LayerwiseNormalizedRetainedReadoutLanguagePathV1` | **Definition** | #973's final zero-parameter readout variant: `E @ [N(h) + (g/sqrt(2))*(N(a1)+N(a2))]`, fixed candidate `g=1` versus matched V1 `g=0`, with recurrence, state, parameters, data/order, optimizer, 2,730-step dose, and tied vocabulary projection held fixed. | **Completed partial:** V3 candidate `G_prompt = 0.0286980210` versus V1 `0.0073316237`, incremental `0.0213663973`, with `339/512` wins. Every fresh-language and mechanics gate passed, but the candidate missed the absolute `0.0433216988` and incremental `0.0253415693` prompt-gain floors. Terminal `LAYERWISE_NORMALIZED_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL`; result CID `blake3:35396bd6e64fc2c0bc7d86a84cc9e212ed913ce28e5353f5f2b8212b4cf2c532`; independent verification CID `blake3:3f316541dbab8061ed5ba891bf6a47ef22c55bca21fba01f6f97dbb3cb8497aa`. Generation, reasoning, and exact/geometry-native lowering are `NOT_RUN`. |
| `R4LearnedCandidateLeafAssociativeReadoutV1`, `zG[t,c]` | **Definition** | #973's completed learned readout reads V1's strict-prior transported retained value at candidate `c`'s canonical exact-H4 leaf with an independently learned `[2,4096,12,4]` candidate-query table: `zG[t,c] = <E[c],N(h[t])> + (1/(2*sqrt(48)))*sum_l <qG[l,c],R(V[l,t,lambda(c)])>`. Qualified V1 is immutable. The equal-parameter address-blind arm has an unshared, byte-identically initialized query table and reads the occupied-address mean; a fixed-leaf derangement reuses the geometric table while destroying candidate/address binding. | **Completed negative at the frozen prompt-capacity scope:** geometric gain `0.00637679`, `299/512` wins, and own NLL `3.71038302` failed the capacity floors. Pooled gain `0.01026323`, `324/512` wins, and own NLL `3.68289051` was partial but also below both gain floors. Both fresh-language and state-load-bearing gates passed. Terminal `LEARNED_ASSOCIATIVE_READOUT_NO_CAPACITY`; result CID `blake3:cedba37738ee249457bb589f716ee75afb16a0c4937c2a22ae9f917dd3eb97c1`; verification CID `blake3:443d711ce9a228e26e2eb2eebb55c582848424e2677c3473d41deaf8afd69ec7`. See `r4_learned_associative_readout_prompt_capacity_973.md`. |
| `R4LearnedAssociativeReadoutPromptCapacityV1` | **Empirical Criterion** | Each learned arm must pass the frozen V4 absolute and incremental prompt-gain floors, `308/512` wins, own-NLL nonregression, fresh-language nonregression, load-bearing state, mechanics, replay, work, and causal audits. `GEOMETRY_ATTRIBUTED` separately requires the geometric arm to exceed both pooled and fixed-leaf-deranged controls by the frozen incremental effect and win/NLL rules. | **Empirical terminal:** neither arm passed capacity. Geometry attribution failed: geometric-minus-pooled gain `-0.00388645` with `209/512` paired improvements and geometric-minus-deranged gain `-0.00028887` with `251/512`, below `+0.02534157` and `308/512`. All ten mechanics gates and independent replay passed; post-reveal optimizer steps were zero. No retry or generation is authorized. Its then-next separately frozen architecture had to alter the retained value write/binding law and retain pooled plus geometry-destroying controls; `R4PredictiveBlockDeltaPromptCapacityV5` below records that successor. #973 remains open and #954 blocked. |
| `R4PredictiveBlockDeltaBindingV1`, `S_t^(b,l)`, `r_t` | **Definition** | #973's completed predictive write/binding law stores four bounded banks of 12 independent `4 x 4` R4 block memories. In the current frame, each bank transports prior state, applies either the full delta update `S_t = rho*P*S_(t-1)*P^-1 + eta*(v_t - rho*P*S_(t-1)*P^-1*k_bar) outer k_bar` or the declared additive control, and reads `r_t = sum_b softmax(alpha)_b*S_t*q_t`. The observed value and candidate scorer are anchored to immutable token H4 leaves. | Compiler-side f32 experiment only. The full-delta geometric, identity/plain, and additive arms each used `9,228` trainable values and identical `2,730`-step construction dose. This definition grants no runtime legality, geometry advantage, attention-capacity, generation, or lowering claim. See `r4_predictive_block_delta_binding_prompt_capacity_973.md`. |
| `R4PredictiveBlockDeltaPromptCapacityV5` | **Empirical Criterion** | On the sealed 512-direction V5 population, terminal capacity requires geometric gain at least `0.04332169878499658`, at least `308/512` wins, the frozen incremental and own-NLL rules against immutable V1 and pooled, fresh-language nonregression, load-bearing state, and complete integrity. Geometry and delta overwrite are separate attributions against independently fitted plain plus transport-permuted, and independently fitted additive, using the frozen `0.025341569256760274`, paired-win, and own-NLL rules. | **Empirical terminal:** `PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY`. Geometric gain `0.03896945868086732` with `375/512` wins missed the absolute floor. Geometric-minus-plain gain `0.023929811749894725` and worse own NLL failed geometry attribution even though the transport-permuted gates passed. Geometric-minus-additive was `-0.006512463228773413` with `234/512`, so delta superiority was not established. Fresh-language and integrity passed. Result CID `blake3:6c67544d675eafcb8eb9c0dabb93617e3f6c3295af812e8acbb687107c010a74`; scoring CID `blake3:44f8941d24a99fc230710fd700e7a7b13cee87587bfbe4e13bf7b095222e2ee6`; recovery CID `blake3:7b76e36e44798bebf184ece08fdd8a2065bdd370106b5d64d5fae4c59dc6d88b`; exact-replay verification CID `blake3:567cf336eb05c3ec562aef7135f6fb35b580d02c758b0e79f2508cae57065f5d`. Action `STOP_WITHOUT_GENERATION`; only this law is retired. |
| `O_trace^(0..3)` | **Objective** | Construction-only, leave-one-document-out observability ladder measuring the same teacher-relative candidate loss at four frozen boundaries: full ordered final-layer Q/K/V trace blocks; fixed 576-to-4 signed reduction; token-derived role maps plus recurrent state features; and fitted residual readout/logit scale. | **Completed** in [#1012](https://github.com/UOR-Foundation/uor-r4/issues/1012) at `INSUFFICIENT_SUPPORT_COVERAGE`: aggregate primary coverage was `0.6202622204224402`, but the minimum fold was `0.3469116829611222`, below the frozen 50% floor, so no boundary attribution is licensed. On the covered rows, full Q/K/V CE was `2.215410922655504` versus suffix `2.215064603216862` (`suffix - full = -0.0003463194386417179`, direction `0/4`); the fixed label control separated by `1.3807454322642605` nats in `4/4`. Exact replay and zero source/document-13 reads passed. The ladder will not be expanded or repeated. #1014 subsequently completed the direct-learning pivot; see `Delta_attn-off` below. |
| `d_R4`, `mu_R4` | **Objective** | If reactivated, a separately trained intrinsic arm may score transported manifold-valued Q/K pairs and aggregate transported values with an artifact-bound equivariant geometric centroid `mu_R4`. | V1 stopped `UNAVAILABLE` before D3; learned-manifold V2 failed donor retention and matched Euclidean parity; the 8/8-contract attempt stopped at its two-document preflight and rejected tangent readout with pooled normalized audit-MSE ratio `1.0643688804269025`. Intrinsic attention remains unestablished and this objective is **Parked**. |
| `A_M`, `K_hat`, `N_t`, `Z_t` | **Objective** | A finite fiber-aware resonance amplitude, its nonnegative normalized kernel `K_hat = floor + abs(A_M)^2`, and the recurrent transported value-numerator and normalization-denominator mode sums that replace dense all-pairs weighting. | ADR-0005 and the #973 multi-resonance reuse audit retain the conditional contract. The normalized sieve, recurrent factorization, and H4/Q29/integer lowering are **Parked**, `NOT_IMPLEMENTED`, and `NOT_RUN`. |
| `I_geo` | **Definition** | Causal inference step mapping observed prefix plus bounded hierarchy state to next-token scores/selection and updated state, followed by lexical decoding. | A support trace alone is not inference or coherent generation. |
| `Corr(D)`, `Abst(D)` | **Empirical Criterion** | Correctness and abstention on declared distribution `D`: correctness uses an independent oracle/constraint/source and is reported both conditional on answered cases and over all cases; abstention is a typed outcome, never silently scored correct. | Protocol and denominators are required by `geometric_intelligence_evaluation.md`. |
| `Reason(D)` | **Empirical Criterion** | On anti-recall tasks from `D`, typed intermediate route transitions preserve constraints, compare alternatives or counterfactuals, and reach an independently checkable conclusion. | Fluent text, recalled answers, teacher agreement, or non-zero geometric activity is insufficient evidence. |
| `Serve_pf` | **Definition** | Provider-free serving: the evaluated process performs no runtime call to Ollama, a cloud model, teacher endpoint, or other generative provider; all required artifacts and decoding are local and pinned. | Does not imply transformerless, geometry-only, multiplication-free, correct, or production-ready. |
| `M_R4^ℓ` | **Definition** | Declared bounded causal geometric mixing operator at layer `ℓ`; its metric, chart, support rule, transport, and aggregation must be frozen by the owning issue. | ADR-0005 accepts ordinary R4/Spin softmax attention as the reference baseline. Source-backed generation/native HTTP and the first source-free suffix-distillation compiler pass at their declared scopes, but the suffix student decoded a short repetition. `R4SoftmaxTraceStateStudentV1` then completed negative without selection-bearing separation from its transport-permuted control. The construction-only `O_trace^(0..3)` ladder completed with insufficient per-fold support and no boundary attribution. #1014 then established ordinary causal attention as load-bearing at the directly learned R4/Spin scope through `Delta_attn-off = 2.6773925609275944` nats and two-arm Rust parity; its complete quality DoD failed at enabled NLL `2.127407277216677` and subject/scene retention `3/5`. Intrinsic attention replacement, resonance, reasoning, WASM promotion, and recurrent/exact lowering remain parked. A coherent geometry-native deployed realization remains `NOT_YET_IMPLEMENTED`. |
| `Delta_attn-off` | **Empirical Criterion** | On one frozen checkpoint and one identical sealed population, enabled causal attention NLL is subtracted from NLL after zeroing every attention output after `W_o` and before residual addition. The owning issue must preserve all other weights, inputs, decoding, and work. | [#1014](https://github.com/UOR-Foundation/uor-r4/issues/1014) measured `4.804799838144271 - 2.127407277216677 = 2.6773925609275944` nats/token against a frozen `>= 0.10` rule. Both policies matched Python/Rust top-1 within `0.005`; all six layers had exact causal/R4/output-policy audits and zero future reads. This establishes load-bearing ordinary causal attention for that learned R4/Spin checkpoint, not geometry advantage, coherent quality, transformerlessness, or exact deployment. See `r4_softmax_end_to_end_attention_1014.md`. |
| `d_R4^ℓ` | **Definition** | Declared compatibility or least-energy score over factor overlap, prime-gap phase delta, R4/S3 transport, Hopf observation, and torsion. | Normative G1R objective under #958; the score and its selected-support trace are `NOT_YET_IMPLEMENTED`. The learned `d_R4^ℓ(q,k)` from #950/#951 remains a negative comparator. |
| `N_t^ℓ` | **Definition** | Bounded causal neighborhood of prior route and memory states selected before value aggregation. Future positions are excluded. | Exact-row reachability exists in the source-free substrate; the intervention-qualified geometric neighborhood and deployed trace/census are `NOT_YET_IMPLEMENTED`. |
| `P_mem^ℓ` | **Definition** | Deterministic, tokenizer-CID-bound projection from an identity-scoped memory span into ordered prime-route and spin/torsion state. | G0/G1 key/value adapter is historical; the factorable memory projection and product integration are `NOT_YET_IMPLEMENTED` under #958/#953. |
| `Δ_geo-null` | **Empirical Criterion** | Difference on the predeclared held-out metric between real geometry/memory and its equal-budget disabled or permuted control. A nonzero support trace alone is reachability, not advantage. | Bounded one-layer report under #951. |
| `S` | **Definition** | Semantic state space; every observation projects into a state `s ∈ S`. Regions are subsets `R_i ⊆ S`; beliefs are predicates over `S`; goals are desired subsets of `S`. | *Compiler/reference-only abstraction* (issue #124). The deployed runtime carries a fixed-capacity approximation: the runtime state (frontier, rolling context code, token shortlist) in `crates/uor-r4-core`. |
| `G = (V, E)` | **Definition** | Compiled semantic graph: `V` packed semantic states, `E` typed transitions. | `crates/uor-r4-graph-format` `GraphView` over the R4G1 NODE/EDGE sections (`docs/transformerless/R4G1.md`). |
| `H(x)` | **Definition** | Holographic encoding of observation `x`: a family of overlapping projections `{h_0, …, h_k}` with partial recoverability, distributed evidence, progressive fidelity. The deployed path today is still the single compiled Boolean semantic code; the certifier additionally defines and measures overlapping projection families for issue #126. | Runtime binding: sign-bit signature path in `crates/uor-r4-core` (see "Semantic code H(x)" in the glossary). Measurement contract binding: `crates/uor-r4-graph-certify/src/holographic_encoding.rs` and its deterministic fixture tests. |
| `T : S × A → S` | **Definition** | Typed graph dynamics: transition function over states and actions/semantic operators `A`. | *Compiler/reference-only* (issue #124). Deployed precursor: forward transition edges `E_f` / R4G1 ROUT section. |
| `R` | **Definition** | Reconstruction / behavioral-recovery operator; `R(H(x)) ≈ x` read behaviorally as the divergence condition below. | *Compiler/certifier-only*; exercised through the fidelity-certification harness (`score.rs`, Gate C). |
| `C : Θ → G` | **Definition** | The compiler as a map from teacher parameter space `Θ` to the space of compiled artifacts. Compilation is lossy semantic compression: parameters → behavioral probing → latent graph induction → Boolean synthesis → packed immutable artifact. | `crates/uor-r4-core` compiler pipeline; graph generalization in `crates/uor-r4-graph-compiler`. |
| `P_θ(·‖c)` | **Definition** | Teacher distribution over next tokens for context `c`, pinned HF revision, deterministic mode. | `TeacherOracle` next-token surface (`crates/uor-r4-core`). |
| `P_G(·‖H(x))` | **Definition** | Runtime/graph distribution produced by the compiled artifact. | Graph scorer (`crates/uor-r4-core` `score.rs`; R4G1 adapter `src/r4g1.rs`). |
| `D(·, ·)`, `ε` | **Definition** | Declared divergence measure and empirical tolerance for the behavioral reconstruction condition (below). | `docs/transformerless/EQUIVALENCE_AND_EMPIRICAL_PROTOCOL.md`. |

Behavioral reconstruction condition (the testable form of `R(H(x)) ≈ x`):

> **Empirical Criterion.** `D(P_θ(· | x), P_G(· | H(x))) ≤ ε` for a declared
> divergence `D` and tolerance `ε`, measured on a pinned held-out distribution with
> confidence intervals. Status: **Empirical**; never a structural claim.

## 4. Objectives versus runtime invariants

These are **Objectives** — quantities the offline compiler optimizes. They must
never be stated as runtime properties:

| Quantity | Role | Binding |
|---|---|---|
| `J = L_teacher + λ·C_runtime + μ·C_artifact` | **Objective** | Compiler cost model: teacher behavioral loss, inference cost, artifact size/complexity (issue #129). |
| `min_Z I(Z;X) − β·I(Z;Y_future)` | **Objective** | Information-bottleneck compression target: discard surface detail, keep future-relevant information (issue #127). |
| `H(A \| R)`, `H(S_future \| R)` | **Objective** | Predictive-entropy criteria for splitting, merging, or removing regions (issue #127). |
| `π* = arg max_π [ V(G \| T(B,π)) − P(C,π) − R(U,π) ]` | **Objective** | Plan-ranking target for bounded future-state optimization (issue #131). Not a theorem. |

These are **Guarantees** — structural properties of artifact and runtime, each with
a proof-model entry. Their current statuses are the proof-matrix records; a document
citing one MUST cite the same status:

| Guarantee | Status | Evidence |
|---|---|---|
| Allocation freedom on the prediction hot path | **Structural** | `allocation_proof` counting-allocator harness; `allocation_census.rs` (proof matrix: PDF §16) |
| Bounded packed ranges | **Structural** | `range_bounds_proof` (Theorem 8) |
| Deterministic top-K (canonical tie-break) | **Structural** | `deterministic_topk_proof` (PDF §23) |
| Forward/reverse index consistency | **Structural** | `theorem7_proof` (Theorem 7) |
| Score arithmetic safety (no overflow/panic) | **Structural** | Kani-1 harness (`kani_proofs.rs`) |
| Fixed-capacity container invariants | **Structural** | Kani-2 harness (`kani_proofs.rs`) |
| Inference operation-set conformance | **Witnessed** (Structural after machine-code audit) | `INFERENCE_OPERATION_CONTRACT.md` + P-4 source scans (`transformerless/mod.rs`) |
| Termination, bounded frontier width, valid references, canonical serialization, provenance completeness | per proof matrix | R4G1 two-stage validation + proof-model entries; anything lacking an executed evidence artifact is **Unproven** |

## 5. Term discipline (overloaded words)

| Avoid (unqualified) | Use instead | Rule |
|---|---|---|
| "intent" | **Future-state optimization**: belief = estimated current state, goal = desired future-state subset `G ⊆ S`, constraint = forbidden subset `F ⊆ S`, action = transition operator, plan = bounded trajectory `π = (a_0, …, a_n)` with `T^π(s_0) ∈ G` and `T^π_i(s_0) ∉ F` for all intermediate `i` (**Definition**, PDF §12). | Unqualified "intent" is informal prose; it must not appear in a labeled statement. |
| "semantic atom" | **Semantic region** — multiresolution, overlapping, proof-addressable; explicitly not an atom (glossary). | "Semantic atom" is prohibited in normative text. |
| "equivalence" | Qualified forms only: **byte reproducibility** (identical pinned inputs ⇒ identical artifact bytes) or **behavioral equivalence** (an **Empirical Criterion**, valid only on the declared distribution, per the equivalence protocol). | Unqualified "equivalence" is informal; "exact equivalence" hits the §2.1 wording rule. |
| "reasoning" | Precise mechanisms: **typed state transitions**, **graph navigation**, **bounded planning** (trajectory evaluation over `T`). | Bare "reasoning" is informal; plausible language output is never evidence of it (§2.1). |
| "transformerless" | **Zero source-attention calls and no dense full-prefix Q·K matrix/softmax kernel** on the exact promoted decoder path; the replacement uses bounded geometric support shown load-bearing by disabled/permuted interventions. | Does not imply multiplication-free, integer-only, allocation-free, coherent, or production-ready. Name retained source components and the checkpoint/scope. |
| "geometric intelligence" | Name the measured mechanism: **geometric support change**, **memory-causal logit effect**, **student-prefix rollout**, or another declared empirical result. | Geometry being present or visually structured is not evidence of intelligence. |

Existing documents predate this convention. Their already-qualified uses
("behavioral equivalence", "not semantic atoms", "equivalence-tested") conform;
unqualified uses are hereby marked informal and are migrated opportunistically, not
by wholesale rewrite.

## 6. Enforcement

- `scripts/check_claim_wording.py` scans `docs/**/*.md` and crate `README.md` files
  and fails on §2.1 violations. Run locally: `python3 scripts/check_claim_wording.py`.
- Current PR and merge-group CI run the focused native checks described by the
  [execution policy](integration/agent-execution-policy.md#verification-and-records).
  Four historical required statuses acknowledge compatibility only. Run the
  claim-wording script when changing claims; broader legacy release checks
  remain separately activated. Neither status names nor unrun jobs are evidence.
- The proof-status matrix (`proof_matrix.rs`) is the machine-readable registry for
  §2 statuses; `verify_all` fails on any `Unverified` entry.

## Changelog

- **0.1.35** (2026-09-01) — Recorded the independently verified
  `R4PredictiveBlockDeltaPromptCapacityV5` terminal
  `PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY`. Geometric gain
  `0.03896945868086732` with `375/512` wins missed the frozen absolute
  `0.04332169878499658` capacity floor. Geometry versus independently fitted
  plain missed the gain and own-NLL rules even though transport-permuted gates
  passed; full delta also failed attribution versus independently fitted
  additive at `-0.006512463228773413` and `234/512`. Fresh-language and
  integrity gates passed. A scoring-only recovery preserved the frozen arms,
  created no optimizer, and was independently replayed exactly. The binding
  action is `STOP_WITHOUT_GENERATION`; retire only this predictive block-delta
  law, without revising ordinary-softmax or retained-attention evidence.
  Result/scoring/recovery/verification CIDs are respectively
  `blake3:6c67544d675eafcb8eb9c0dabb93617e3f6c3295af812e8acbb687107c010a74`,
  `blake3:44f8941d24a99fc230710fd700e7a7b13cee87587bfbe4e13bf7b095222e2ee6`,
  `blake3:7b76e36e44798bebf184ece08fdd8a2065bdd370106b5d64d5fae4c59dc6d88b`,
  and `blake3:567cf336eb05c3ec562aef7135f6fb35b580d02c758b0e79f2508cae57065f5d`.
- **0.1.34** (2026-09-01) — Recorded the independently verified
  `R4LearnedAssociativeReadoutPromptCapacityV1` terminal
  `LEARNED_ASSOCIATIVE_READOUT_NO_CAPACITY`. Geometric gain `0.00637679` and
  pooled gain `0.01026323` both missed the frozen absolute `0.04332170` and
  incremental `0.02534157` capacity effects; geometry also lost the pooled and
  fixed-leaf-deranged attribution comparisons. Both learned arms passed every
  fresh-language and state-load-bearing gate, so the pooled improvement is
  retained as a non-geometric control signal. All mechanics and independent
  replay passed. No retry or generation is authorized; the next separately
  frozen #973 architecture must change the retained value write/binding law.
  Result CID
  `blake3:cedba37738ee249457bb589f716ee75afb16a0c4937c2a22ae9f917dd3eb97c1`;
  verification CID
  `blake3:443d711ce9a228e26e2eb2eebb55c582848424e2677c3473d41deaf8afd69ec7`.
- **0.1.33** (2026-09-01) — Froze
  `R4LearnedCandidateLeafAssociativeReadoutV1` and its independent
  `R4LearnedAssociativeReadoutPromptCapacityV1` campaign before implementation,
  V4 population creation, fitting, or outcome access. Bound the exact-leaf,
  address-blind pooled, fixed-leaf-deranged, head-off, and state-off arms plus
  the fresh-data, create-once reveal, capacity, geometry-attribution, compute,
  and divergent-action contracts. All empirical outcomes remain `NOT_RUN`;
  #973 remains open and #954 blocked.
- **0.1.32** (2026-09-01) — Recorded
  `R4LayerwiseNormalizedRetainedReadoutLanguagePathV1` as terminal
  `LAYERWISE_NORMALIZED_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL`. All
  fresh-language, state-off, causal, replay, and independent-verification gates
  passed, while prompt gain `0.0286980210` and incremental gain
  `0.0213663973` missed their frozen floors. Ended the parameter-free readout
  ladder and made a freshly frozen learned associative binding/readout the sole
  #973 successor. Candidate generation, reasoning, and exact/geometry-native
  lowering remain `NOT_RUN`; #954 remains blocked.
- **0.1.31** (2026-09-01) — Recorded
  `R4DirectRetainedReadoutLanguagePathV1` as terminal
  `DIRECT_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL`. The zero-parameter readout
  increased prompt gain and fresh-language quality while passing state-off,
  causal, replay, and independent-verification checks, but missed both frozen
  prompt-gain floors. Generation and lowering remain `NOT_RUN`. Froze exactly
  one layerwise-normalized, zero-parameter successor and a stop/pivot falsifier;
  no coherence, reasoning, H4-superiority, exact-runtime, browser, or release
  claim follows.
- **0.1.30** (2026-09-01) — Recorded qualified
  `R4RetainedLanguagePathV1` and its terminal paired-H4 capacity successor.
  Paired addressing reduced construction repeats `97.5477%` and slightly
  improved fresh-language NLL/top-1, but prompt gain fell from `0.0063672952`
  to `0.0062477543`, with `282/512` wins. Terminal:
  `PAIRED_H4_PROMPT_CAPACITY_FAIL`; result CID
  `blake3:508a4ff352f1e533d669d9616f65b972b0f13e8efe35867b7b095281ad940274`.
  Preserved V1, rejected candidate generation, and directed #973 to an
  independently frozen prompt-state-to-logit readout seam. The canonical
  prompt population CID is
  `blake3:c11a7c935139ca169460b90c01392d7c9e0929e4c10710e76e6c8f74cbdf0340`;
  it replaced a provisional pre-freeze scan that omitted whitespace
  normalization and overlapped training data.
- **0.1.29** (2026-09-01) — Recorded #973's completed CPU-recovery boundary.
  Disabling retained state on the disjoint construction-validation partition
  lost `0.967227` nats and
  182 top-1 hits, qualifying a bounded causal retained-attention component. The
  exact 3.17M-parameter, two-block, data/dose recipe did not satisfy the frozen
  full-decoder generalization criterion; scrambled transport was
  `0.033049` nats better, so no H4-specific advantage follows. Directed #973 to
  a data-supported language-path decoder with an ordinary matched non-geometric
  control. Also made representative-step backend/thread/worker calibration the
  operating rule for substantial offline jobs; deployed runtime remains
  CPU/table-native and CUDA remains explicit-scope only.
- **0.1.28** (2026-09-01) — Recorded the terminal #973 construction result
  `UNAVAILABLE_FRAME_POPULATION_OR_LOCAL_BUDGET`. Mechanical audits passed;
  timing and disposable learning smoke failed, so main and held-out model work
  stayed `NOT_RUN` with no attention or H4 verdict. Retired retries/tuning of the
  exact cell and directed #973 to select a fuller source-free decoder block.
- **0.1.27** (2026-09-01) — Defined the independently re-scoped #973
  `R4GroupAddressedRetentionLMV1` and recorded only its observed construction
  boundary: geometry and population `PASS`; training and held-out model evidence
  `NOT_RUN`. Retained the older gated-delta, trace-state, intrinsic/readout, and
  resonance lanes as negative or parked history, with #954 still blocked and no
  C1-SB6 authorization.
- **0.1.26** (2026-08-31) — Recorded the one bounded #1019 local fast-path
  result: an isolated exact-shape MPS test with 10 warmup plus 40 measured steps
  combined fused AdamW and deferred logging, measuring `4.485223 s/step` versus
  the signed `3.491307 s/step`. Because it was slower, `fused=True` was removed
  immediately. This is a bounded implementation negative, not a model or
  attention result. #1019 tuning/full-run work stops and remains optional/
  paused; product work and #973 no longer wait for it. The active next step is
  using and productizing the bounded #1017 generator through `r4 generate`.
- **0.1.25** (2026-08-31) — Corrected #1019's active execution scope without
  rewriting its frozen contract or measured preflight: UOR's deployed
  architecture/runtime remains CPU-native; Apple Accelerate/BLAS and MPS are
  permitted only for local offline training, compilation, and bounded tests;
  CUDA and external GPU execution are out of scope. The observed
  `UNAVAILABLE_HARDWARE_BUDGET` terminal applies only to the frozen eight-hour
  offline implementation. Reuse the passed population/smoke/parity artifacts
  and proceed prototype-first on the local M1: build, demonstrate, then harden
  one working efficiency mechanism without recurring broad research gates.
  #1019 is an optional quality-capacity improvement and no longer blocks use or
  productization of the bounded #1017 generator through `r4 generate`.
- **0.1.24** (2026-08-31) — Bound #1019 as the sole parameter-capacity
  successor: twelve layers, exactly 13,130,784 parameters, seed 1019, 16,800
  steps, and 275,251,200 tokens over the unchanged causal-softmax R4/Spin and
  Rust evidence path. Recorded every execution gate as `NOT_RUN`, required the
  cheap preflights and an eight-hour hardware projection, and prohibited paid
  external execution without explicit owner approval. This is a frozen
  Objective/Empirical Criterion, not a result claim.
- **0.1.23** (2026-08-31) — Recorded #1017's frozen continuation at
  `149,995,520` cumulative tokens: enabled Rust parity, all mechanical gates,
  prompt retention `5/5`, and normalized replay `5/5` passed; sealed NLL
  `1.5727521962806827` failed the strict `<1.50` gate. The result is negative
  solely on NLL. Preserved #1014's bounded attention conclusion, prohibited
  further 7.15M exposure/LR tuning, and advanced only one predeclared
  parameter-capacity increase.
- **0.1.22** (2026-08-30) — Added `Delta_attn-off` and recorded the completed
  #1014 split verdict: ordinary causal attention established at the learned
  R4/Spin intervention/parity scope; full language-quality DoD negative at
  enabled NLL `2.127407277216677` and prompt retention `3/5`. The exact
  campaign closes without rerun or diagnostic expansion. Its #1017 exposure
  continuation then closed negative solely on sealed NLL
  `1.5727521962806827`, with retention `5/5` and all other gates passing. One
  predeclared parameter-capacity increase over the unchanged mechanism is next.
- **0.1.21** (2026-08-30) — Recorded #1012
  `INSUFFICIENT_SUPPORT_COVERAGE`, forbade boundary attribution and further
  support/localization loops, and advanced [#1014](https://github.com/UOR-Foundation/uor-r4/issues/1014),
  direct end-to-end causal-softmax R4 attention training on a fresh untouched
  split with autonomous decoding. This stops the
  bounded current-step trace-distillation path, not all trace distillation or
  attention.
- **0.1.20** (2026-08-30) — Bound the completed negative
  `R4SoftmaxTraceStateStudentV1` result, exact metrics and artifact identities;
  narrowed its falsifier to the tested 4D signed-reduction/token-derived state
  cell; and defined the construction-only leave-one-document-out
  `O_trace^(0..3)` observability ladder in #1012, the native child/blocker of
  #973, as the next gate.
- **0.1.19** (2026-08-30) — Promoted `R4SoftmaxTeacherTraceV1` and
  `R4SoftmaxTraceCompilerV1` from proposed objectives to a bounded empirical
  source-free-distillation result, retained the incoherent decoded-cycle
  boundary, and defined `R4SoftmaxTraceStateStudentV1` as the next geometric
  state-compilation objective.
- **0.1.18** (2026-08-30) — Recorded the exact opt-in loopback HTTP/CLI
  reference parity boundary and defined the proposed, not-yet-implemented
  `R4SoftmaxTeacherTraceV1` / `R4SoftmaxTraceCompilerV1` source-free
  trace-compilation objective.
- **0.1.15** (2026-08-26) — Defined the geometric-intelligence route hierarchy
  and its claim boundaries: typed zero/identity and trigonometric chart bridges,
  lexical/prime/semiprime/n-let identities, zeta/Hopf/torsion/golden state,
  conceptual `E8 = H4 × H4` realized by the witnessed icosian
  `H4 ⊕ φH4` construction, hierarchical kappa and trajectory summaries,
  attention versus recall, inference, correctness, reasoning, and provider-free
  serving.
- **0.1.14** (2026-08-20) — Added the issue-#830 register execution-scope, serving-reachability, and empirical-verdict vocabulary: a per-row execution `scope` (`reference-only` / `offline-compiler` / `certifier-instrument` / `dormant-portable-runtime` / `normative-runtime` / `deployed-production`) and a serving `reachability` (`deployed-serving` / `off-serving-path` / `dormant-gated`), plus a three-value empirical status (`PASS` / `FAIL` / `UNAVAILABLE`) kept as a separate axis from the harness-built `build` level — an absent fixture is `UNAVAILABLE`, never `PASS` (`crates/repo-model/src/registry.rs`, `crates/repo-model/src/empirical.rs`; rendered into `CONFORMANCE.md`).
- **0.1.13** (2026-07-25) — Added issue-#175 compiler parallelism benchmarks and scaling certificate definitions (`Compiler Parallelism Scaling Report`, `Multicore Thread Sweep Matrix`, `Stage Scaling Classification Taxonomy`, `Byte-Equality Scaling Premise` in `docs/compiler_scaling_certificate.md` and `uor-r4-graph-certify::compiler_scaling`).
- **0.1.12** (2026-07-25) — Added issue-#174 CPU-only compiler dependency and feature audit definitions (`Compiler Dependency Denylist Gate`, `Default Feature Unification Rule`, `Teacher-Backend Isolation Invariant`, `CPU-Only Runner Compliance` in `docs/compiler_dependency_audit.md` and `uor-r4-graph-compiler::dependency_audit`).
- **0.1.11** (2026-07-24) — Added issue-#170 parallel observation, trace, and evaluation processing definitions (`Content-Addressed Shard Partitioning`, `Ordered Deterministic Reduction`, `Shard ID Determinism`, `Teacher-Backend Boundary` in `docs/parallel_observation_shards.md` and `uor-r4-graph-compiler::observation_shards`).
- **0.1.10** (2026-07-24) — Added issue-#169 compiler memory-budget and backpressure model definitions (`Concurrency-Aware Memory Formula`, `Per-Stage Memory Estimate`, `In-Flight Backpressure Limiter`, `Constrained-Memory Determinism` in `docs/compiler_memory_budget.md` and `uor-r4-graph-compiler::memory_budget`).
- **0.1.9** (2026-07-24) — Added issue-#168 compiler thread-pool and jobs configuration definitions (`Compiler Concurrency Control`, `Jobs Precedence Resolution`, `Dedicated Thread-Pool Ownership`, `Oversubscription Policy` in `docs/compiler_concurrency_config.md` and `uor-r4-graph-compiler::jobs_config`).
- **0.1.8** (2026-07-24) — Added issue-#167 normative reproducibility & canonical byte-equality definitions (`Normative Reproducibility Invariant`, `Parallel Reproducibility Harness`, `Thread-Count Invariance`, `Deterministic Reduction Policy` in `docs/reproducibility.md` and `uor-r4-graph-compiler::reproducibility`).
- **0.1.7** (2026-07-24) — Added issue-#166 compiler stage ownership and parallelization DAG definitions (`Compiler Stage DAG`, `Parallel-Safe Stage`, `Deterministic Merge Stage`, `Bounded Parallel Stage`, `Sequential Canonical Finalization Spine` in `docs/compiler_stage_dag.md` and `uor-r4-graph-compiler::stage_dag`).
- **0.1.6** (2026-07-24) — Added issue-#165 deterministic compiler executor definitions (`Compiler Executor Abstraction`, `Sequential Reference Executor`, `Rayon Parallel Executor` in `uor-r4-graph-compiler::executor`).
- **0.1.5** (2026-07-24) — Added issue-#161 runtime operation, allocation, and CPU portability certificate definitions (`Runtime Performance Certificate`, `Evidentiary Class Schema`, `Declared-Zero Evidence Link`, `CPU Portability Record` in `uor-r4-graph-certify::performance_certificate`).
- **0.1.4** (2026-07-24) — Added issue-#160 machine-code, allocator, and dependency CI audit definitions (`Machine-Code Disassembly Audit`, `Counting Allocator Witness`, `Dependency Denylist Gate` in `uor-r4-proof-model::inference_audit`).
- **0.1.3** (2026-07-24) — Added the issue-#158 normative scoring semantics definitions (`Fixed-Point Scoring Semantics`, `Residual Taxonomy`, `Overlap Residualization`, `Deterministic Tie-Breaking` in `docs/scoring_semantics.md` and `uor-r4-graph-format::scoring_semantics`).
- **0.1.2** (2026-07-24) — Added the issue-#157 normative inference contract definitions (`Normative Inference Contract`, `Permitted Operation Class`, `Zero-Allocation Steady State`, `CPU-Only Target Contract` in `docs/inference_contract.md` and `uor-r4-graph-format::inference_contract`).
- **0.1.1** (2026-07-24) — Added the issue-#126 measurement-contract binding for
  `H(x)` (projection family schema, ablation semantics, and deterministic partial
  recovery/progressive-fidelity fixture in `uor-r4-graph-certify`).
- **0.1.0** (2026-07-24) — Initial version. Claim classes, claim statuses, core
  notation (`S`, `G=(V,E)`, `H(x)`, `T`, `R`, `C`, `P_θ`, `P_G`), objectives vs.
  runtime invariants, term discipline, and the CI wording rule. (Issue #123.)
