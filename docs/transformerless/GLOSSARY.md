# Glossary — R⁴ Geometric Decoder and Historical Graph Runtime

Originally the Phase 0 graph-compiler glossary; expanded by #948 for the now-
historical [geometric causal decoder](../geometric_causal_decoder_plan.md).
Current sequencing lives in the
[Geometric Intelligence Programme](../geometric_intelligence_programme.md),
and the active #973 mechanism is [ADR-0005](../adr/0005-predictive-geometric-connection-memory.md).
Decoder terms and graph terms name different execution lanes and must not be
substituted for one another.

## Active geometric-decoder terms

### Canonical geometric-intelligence vocabulary

- **Typed zero/identity bridge** — an explicit domain tag selecting either
  `continuous-null` (`0^0 -> 0`) or `discrete-empty-product` (`0^0 -> 1`). The
  project seam is written `exp(i*pi) + pi^0 =_bridge 0^0`. In
  `continuous-null`, the bridge preserves the complex cancellation as `0`; in
  `discrete-empty-product`, it phase-shifts/retypes that boundary into the
  discrete identity `1`. Here `=_bridge` is a domain-transition operator, not
  ordinary numerical equality. The tag is part of canonical identity and
  supplies a deliberate choice among the `-1 / 0 / +1` landmarks without
  claiming that untyped arithmetic assigns both values simultaneously.
- **Lexical codec** — the deterministic, tokenizer-bound conversion between
  input bytes, token or lexical-unit IDs, registered route atoms, and output
  bytes. Its tokenizer/version identity, normalization rules, unknown-unit
  behavior, and prime registry are kappa-bound. A codec supplies spelling and
  route identity; it does not by itself supply meaning, grammar, attention, or
  factual knowledge.
- **Prime atom (`p`)** — one registered prime assigned canonically to one
  lexical or route identity. The prime participates in factorable locality;
  the bound payload CID supplies exact payload identity. A raw prime, MAC
  address, hexadecimal spelling, or IPv6 presentation is not an authorization
  identity and does not establish semantic similarity.
- **Semiprime expert (`e`)** — the canonical unordered two-factor multiset for
  one adjacent ordered transition. For different factors it is the square-free
  semiprime `p*q`; an adjacent repeated atom is retained as the prime square
  `p^2`. The expert records factor overlap, not direction. Ordered route state,
  spin/torsion, and span identity retain direction.
- **Ordered n-let (`N`)** — a bounded ordered sequence of prime atoms together
  with its multiplicity-preserving sorted factor multiset and ordered kappa.
  The sequence is normative; a potentially overflowing numeric product is
  only a diagnostic convenience and never replaces it.
- **Fixed zeta grid** — a finite, ordered, revisioned set of non-trivial
  zeta-zero ordinates used as immutable log-polar phase channels. The grid CID,
  channel order, quantization, and table compiler are artifact-bound. Using the
  critical-line form is a coordinate-design assumption and is not a proof of
  the Riemann hypothesis or evidence of language capability.
- **R4/S3 spin state** — a non-zero local vector in `R4` normalized to the unit
  three-sphere `S3`, used as the full local spin/compute state. `R4` is the
  ambient coordinate space and `S3` its unit manifold; neither name denotes a
  text embedding, E8, or a correctness result without a declared mapping.
- **S2/R3 Hopf observation** — the `S2` point embedded in `R3` produced by the
  Hopf map from `S3`. It is a many-to-one observable or heatmap coordinate.
  The observation cannot reconstruct the full `S3` state without retained
  fiber information.
- **Exact spin class** — the existing `shared_class_kappa` exact-class
  lookup/grouping key that may key future operator sharing. It binds the full
  signed canonical S3 spin, checked Hopf observation, fiber, and torsion.
  Hopf/S2 is not a standalone class identity and the current
  Hopf-octant/torsion-bin `SpinSector` is only a coarse lookup bucket. Antipodal
  `q` and `-q` states remain distinct. A direction-sensitive operator requires
  a new exact `SpinTorsionState` relative relation or an explicitly bound
  spin-to-H4 map; the existing relative H4 witness is prime-derived route state.
- **Torsion** — the retained, quantized fiber/transport phase associated with
  a route state and its transition. It distinguishes states that share a Hopf
  observation. Here “torsion” is architectural transport metadata, not a claim
  of quantum entanglement, spacetime torsion, or physical energy.
- **S3/R3 trigonometric transition** — the typed angular adapter with
  `theta = atan2(sin(theta),cos(theta))`, activation `sin(theta)^2`, chirality
  `sign(sin(theta))`, and retained cosine polarity when antipodes must remain
  distinct. `(sin=+/-1,cos=0)` is active `1`; `(sin=0,cos=+1)` is continuous
  null `0`. Tangent is only a local chart. At a tangent pole or declared null
  boundary, routing switches to the angle/cotangent chart and records an
  explicit signed quarter-turn phase plus torsion shift; it never divides by
  zero or terminates the route. This is a Definition/architectural assumption,
  not evidence of semantic value.
- **Least-cost cross-domain chart anchors** — typed adapter markers, not
  equations between mathematical domains: `sqrt(2)` is the Euclidean chord of
  orthogonal unit directions; `2i` is the complex/discrete antipodal
  displacement marker; `[0,2]` is the declared normalized Riemannian/chord
  score interval. A versioned cost profile may choose the cheapest faithful
  chart while binding units, orientation, quantization, error bounds, and
  conversion witness.
- **Golden radial shell (`Z[phi]`)** — the exact coefficient pair `(a,b)` for
  `a + b*phi`. Multiplication by `phi` maps `(a,b)` to `(b,a+b)` and the inverse
  maps `(a,b)` to `(b-a,a)`. Fibonacci growth follows algebraically from
  repeated shell steps; semantic value does not.
- **Paired-H4/E8 bridge** — the declared load-bearing icosian coordinate
  bridge for the target hierarchy. **Project shorthand:** `E8 = H4 × H4`.
  The concrete serialized construction realizes that conceptual identity by
  representing the chosen icosian/E8 lattice presentation, as a `Z`-module,
  with a golden/Galois-coupled pair of R4 points and the declared 600-cell
  folding `H4 ⊕ φH4`. This is more specific than storing a bare direct-product
  assertion: basis, glue/parity rule, conjugation, scale, orientation, root
  ordering, and inverse witness are kappa-bound. Load-bearing is an
  architectural requirement; held-out advantage remains an empirical question.
- **Kappa (`kappa`) identity** — the canonical content address of a declared
  byte envelope, including its schema and provenance. Equal kappas identify
  equal canonical envelopes under the named hash contract; nearby kappas have
  no geometric meaning. Kappa identifies a route artifact but does not replace
  its factorable route coordinates.
- **Route hierarchy** — five identity-scoped causal accumulators, defined in
  [ADR-0004](../adr/0004-geometric-intelligence-route-hierarchy.md):
  **local** (current/previous bounded route), **sentence** (ordered lexical
  routes in one sentence), **paragraph** (ordered sentence-route identities),
  **conversation** (ordered turns/paragraphs for one session), and **global**
  (a versioned, bounded project or knowledge snapshot). Higher scopes commit to
  ordered child identities and incrementally transport overlapping trajectory/
  harmonic summaries; they do not authorize a full-prefix or corpus scan.
- **Transported route trajectory** — the ordered, incrementally updated path of
  route states across a scope, not merely its first or final coordinate. Its
  bounded summary retains the session hypersphere vector, winding/window state,
  projection energy, shared-prime factors, cosine resonance, and accumulated
  Hopf phase. Ancestor evidence indicates that masking or replacing the full
  trajectory with the last state destroys much of the routed signal; current
  held-out reproduction is still required.
- **Harmonic/trajectory locality** — overlap available when an exact hierarchy
  kappa misses: shared-prime multiplicity supplies discrete locality; projection
  energy and cosine resonance compare transported hypersphere summaries;
  winding/window and accumulated Hopf phase retain path/phase context. These
  summaries must be bounded and kappa-bound, but kappa equality itself is not
  the locality metric.
- **Causal admission versus harmonic influence** — two typed stages of a
  query. Admission uses declared causal/index rows to decide which routes are
  lawful candidates. Harmonic/neighbor influence transforms or ranks the state
  of those candidates and never admits one. In the current #953 repair,
  I1/I2/ordered-sentence/divisor are the primary tier; the existing coarse
  adjacent-spin retrieval row may admit only when that tier is empty.
- **Exact-spin global operator overlay** — a prospective harmonic mechanism:
  one deterministic result bound by global
  root/epoch kappa, operator kappa, chart/table identities, and exact spin
  class, reused by every immutable address reference in that class. The overlay
  may influence candidate-relative state only after admission; it does not
  mutate route/payload bytes or scan the corpus. The final candidate score
  remains query-specific. The operator is called harmonic only after its
  identity binds basis, mode order, coefficients, quantization, and transition
  law.
- **Bounded angular neighbor** — a finite, artifact-declared relation between
  non-identical exact spin classes using their exact relative orientation and
  transport state. It is not equivalence by Hopf octant, an all-to-all
  broadcast, or authority to inject candidates. It is built independently from
  the existing coarse adjacent-spin retrieval rows and remains unqualified
  until a matched real/disabled/permuted intervention changes selection and
  output.
- **Predictive connection memory** — the ADR-0005 mechanism family that keeps
  fixed-capacity causal state in R4/spin charts, moves it into the current frame
  through a declared connection transport, forgets or overwrites associations
  through bounded gates and delta writes, and reads only candidates admitted
  by the frozen lower path. It is a research hypothesis until held-out
  predictive value disappears under matched state/order/transport ablations.
- **`DirectCausalGeometricAttentionR4V1`** — #973's offline gold-standard
  reference for the literal attention function: learned Q/K/V/O roles, local
  S3 tangent projection, causal H4-frame transport of every prior key and
  value, stable softmax, transported value aggregation, and candidate-relative
  output scoring over unchanged admitted support. Its dense O(T^2) work and
  softmax are permitted only in compiler-side research; they are not the target
  runtime. It is one bounded attention kernel, not multi-head attention or a
  full block with residual, normalization, FFN/MLP, or stacking. V2 is
  non-promotable because of a raw-manifold-parameter mismatch. Equal-manifold-budget V3
  returned full H4 3/12 versus plain 12/12 and an inference-time coherent
  alternative-connection swap 10/12, so the current H4 representation/
  optimizer combination is not a qualified oracle. V4 later preserved
  construction covariance but failed held-out functional binding at 13/24 for
  every main arm. These are retained historical comparators.
- **`HELM-D-R4` / `R4SoftmaxReferenceGeneratorV1`** — #973's positive bounded
  attention reference and qualified provider-free native generator path. It pins the
  official MIT HELM-D source at
  `7501deca8f413848bfef804be64ce874b72a3cd7`, audits its full-decoder geometry,
  and credits it as an architectural reference only. It does not port HELM's
  decoder stack: UOR's existing pinned SmolLM2
  `HuggingFaceLlamaOracle` supplies embeddings, RoPE, residual/RMSNorm, MLP,
  final normalization, and the language-model head. No HELM checkpoint or code
  executed in the UOR generation or bridge gates, and no upstream result is
  inherited. Released HELM generation/cache is
  incomplete. Its checkpoint and full geometric decoder
  remain an optional external baseline behind a separate tokenizer and license
  gate, and are not directly an R4-block runtime. The reference then freezes an ordinary
  decoder donor and preserves its learned Q/K/V,
  stable causal softmax, linear value
  aggregation, and `W_o` while splitting heads into R4 blocks, binding exact
  cumulative Spin/H4 local frames, transporting K/V into the query frame, and
  mapping the aggregate back before `W_o`. Its bounded numerical and
  real-language behavioral parity, replay, destructive control, and causal
  audit pass; this is not geometric advantage. The first separately trained
  intrinsic R4 distance/centroid attempt reached construction validation but
  stopped unavailable before D3 on covariance. Source-faithful learned-manifold
  V2 then completed a valid non-D3 construction-validation run but failed donor
  retention and matched Euclidean parity; destructive-control separation proves
  intervention sensitivity only. The 8/8-contract localization attempt stopped
  at its two-document preflight and rejected tangent readout. Intrinsic
  attention remains unestablished and parked.
  Ordinary dot-product/stable-softmax attention in coherent R4/Spin frames is
  the accepted baseline. Provider-free autonomous
  `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) native CLI generation passes at
  4/5 frozen quality in both passes, 5/5 exact replay after deleting timing,
  exact all-layer audits with zero future reads, and donor reproduction. Its
  dedicated opt-in, loopback-only native HTTP endpoint also passes the frozen
  eight-token canary with exact CLI token, decision-CID, and state-CID parity,
  all 30 layers audited exactly, and zero future reads. Dashboard wiring/static
  native-readiness and WASM-isolation checks pass; browser
  interaction/E2E is `NOT_RUN`. The default engine is unchanged. It remains Transformer-compatible
  and source-weight backed. Upstream checkpoint parity, resonance replacement,
  and recurrence/lowering evidence remain `NOT_RUN`.
  See `helm_d_r4_softmax_decoder_973.md`,
  `r4_softmax_reference_generation_973.md`, and
  `r4_softmax_reference_generation_attempt_01_result_973.json`, plus
  `r4_softmax_reference_http_bridge_973.md` and
  `helm_d_learned_manifold_r4_construction_973.md`.
- **`R4SoftmaxTeacherTraceV1` / `R4SoftmaxTraceCompilerV1`** — #973's completed
  construction-only trace and first source-free compilation rung. The trace
  captures layerwise token, Q/K/V, top causal attention support, weighted
  aggregate, decoded output, and logit state from the exact qualified
  reference. The first compiler emits matched Q16 teacher-distilled,
  observed-count, and document-permuted suffix arms. It passed its bounded
  loss/top-1/control/replay/source-call gate, but the decoded continuation
  collapsed into a `, Scotland` cycle and the student does not consume
  geometric trace state. It therefore establishes deterministic bounded
  source-free distillation, not geometry advantage, coherent generation,
  reasoning, release, or WASM capability. See
  `docs/r4_softmax_trace_student_973.md`.
- **`R4SoftmaxTraceStateStudentV1` (proposed)** — the active #973 successor:
  compile a bounded causal state transition and readout from the captured
  query-gauge Q, transported K/V support, weighted aggregate, and decoded
  model-frame output. It must beat the established suffix artifact and an
  equal-budget non-geometric recurrent cell while losing its advantage under
  transport/state permutation. It is `NOT_IMPLEMENTED` and `NOT_RUN`.
- **H4 frame connection** — the exact object is the relative H4 group element;
  the current compiler-side f64 matrix represents its left-quaternion action
  and is numerically orthogonal under the tested finite-group law. It does not
  carry the paired-E8 companion or an independent fiber/torsion phase. It is not
  called Levi-Civita or shortest-geodesic parallel transport without a separate
  proof. Equal-manifold-budget direct-attention V3 rejected this particular connection on
  its frozen synthetic population; its finite-group correctness remains intact.
- **Multi-resonance attention sieve** — a parked post-reference replacement for
  softmax. It expands query/key compatibility in a finite artifact-bound set of
  spherical or hyperspherical modes so value-weighted numerator modes and
  normalization-denominator modes can be updated recurrently. The approximation
  must bind pointwise positivity, an exact normalization law, a weight or
  denominator floor, and kernel/decision error; adding epsilon only after the
  denominator is summed is not exact normalization. S2/R3 modes must retain the
  bound S3 fiber/torsion phase, or use an S3/SU(2) basis; a Hopf direction alone
  is not the full R4 spin state. Tan is chart-local with a pole-switch contract,
  not a global harmonic basis.
- **`GeometricGatedDeltaRetentionR4V1`** — the bounded recurrent-factorization
  candidate with separate K/V/Q roles, H4 frame transport, multirate retained
  banks, delta overwrite, and candidate-relative readout. Its first sealed
  synthetic smoke passed structural checks but showed no advantage over plain
  delta; corpus qualification, decoding, and exact lowering remain `NOT_RUN`.
- **Predictive key/value/query placement** — three separate compiler-learned
  roles layered over an immutable route/payload address. A key addresses a
  retained association, a value is written as predictive content, and a query
  reads one already-admitted candidate. They receive versioned artifact and
  provenance identities and never silently reinterpret a prime, digest, token
  rank, spin leaf, or payload CID as learned meaning.
- **`PredictiveConnectionRetentionGate0V1`** — a parked construction-validation
  fixed-route-feature diagnostic. It asks whether current, previous, ordered
  last-two, and complete-prefix exact-route relations support a deterministic
  candidate-discriminative readout under unchanged #953 support/work. It is not
  the full recurrent/gated-delta cell. Positive retains that channel as an
  input, residual, or initializer; negative drops it; unavailable makes no
  fixed-route claim. Its ignored corpus run remains `NOT_RUN`; it is not in the
  active dependency chain because it cannot establish the literal attention
  operation. The protected D3 join remains blocked.
- **Matched plain recurrence** — an equal-information, equal-support, equal-
  budget non-geometric recurrent/delta comparator. A gain shared with this arm
  is predictive recurrence evidence, not geometry-specific evidence. Geometry
  is credited only when connection transport and geometric placement add value
  beyond it and their permutations lose that value.
- **Corpus-induced geometric attention** — the programme hypothesis that a
  previously qualified causal, candidate-relative receptive mechanism can be
  parameterized offline from larger construction corpora into bounded
  transition support, multiscale summaries, versioned placement overlays that
  preserve immutable route/payload identity, and operator tables. The operator
  family/schema and induction rule freeze before scaling; every rung receives a
  new artifact/operator identity, and structural or placement changes rerun the
  bounded qualification. More rows, exact/backoff hits, or trace activity
  establish only coverage/capacity/recall. Attention is credited only when a held-out
  anti-recall choice depends on the induced geometric term under matched causal
  controls. The serving path loads the compiled artifact; it does not scan the
  corpus or load source tensors.
- **Corpus-induced semantic placement** — a versioned compiler-side coordinate
  overlay learned only from past construction observations while preserving
  immutable lexical-route, prime, CID, and payload identities. #986 proposed one
  deterministic causal co-occurrence/PPMI rank-8 placement but stopped before
  construction because its population/frame prerequisites were unavailable. A hash, prime index,
  hexadecimal spelling, exact route key, or historical `%500` hive cell is an
  address, not semantic placement.
- **HarmonicLinkState7** — #986's unconstructed static per-route link-state proposal: the route
  itself plus at most six construction-derived positive PPMI peers, with exact
  operator/phase identities and null slots. Six fixed half-restart Jacobi steps
  produce a candidate-rooted screened-harmonic reachability field used by the
  transport arm. #986 emitted no such rows. Seven is a bounded
  center-plus-six stencil, not an OSPF implementation and not seven Euclidean
  nearest neighbors. The record is semantic substrate, not attention; #986's
  frozen contract would have compared it with link-disabled and distance-only controls.
- **Signed transport attention candidate** — #986's proposed but unexecuted local readout.
  Exact ordered Cl(0,6)/SpiralCore state transports a bounded causal history;
  each already-admitted candidate receives one transported value per retained
  history slot. Its radial/angular alignments are centered across those slots
  and balanced to exact zero-sum `{-1,0,+1}` weights before aggregation in the
  candidate frame. Only a unique maximum above its score-family calibration-
  derived frozen margin selects. It becomes geometric attention only if the
  full arm causally beats matched placement/link/distance/transport/order/state
  controls and
  the table-native value comparator.
- **Source-free table-native lexical baseline (B0/#989)** — the established
  deterministic non-geometric engine compiled only from construction-corpus
  lexical observations, using integer context/count tables to predict held-out
  next units, invert the selected payload exactly, and produce a bounded
  continuation. #989 measured 22.261404% held-out top-1 versus 5.413561%
  unigram on 446,342 known targets and reproduced its complete report and
  artifact byte identically. It loads no teacher, provider, source weights, H4,
  SpiralCore, harmonic, placement, transport, or attention mechanism. This is
  a statistical lexical baseline, not semantics, attention, geometry,
  correctness, reasoning, chat, or release evidence. Its frozen artifact is
  the fixed comparator for one later matched #953 geometric intervention.
- **Table-native semantic-value comparator** — a non-geometric candidate-value
  engine compiled from the same source-free construction observations and
  evaluated with the same natural candidates, information, and work as the
  geometric arm. It is a decision comparator, not attention. If it transfers
  without geometric uplift, the honest branch retains geometry for addressing
  and transport and qualifies the table engine separately.
- **Hypothetical next-route state** — the deterministic provisional state
  `Update(S_t,c)` formed from observed state and one already-admitted candidate
  `c`. It may be scored before committing the unique winner and is not an
  observed future read. Actual future events, target continuations, evaluation
  answers, teacher outputs, and provider text are forbidden inputs. One-step
  projection is selection/inference; deeper bounded branch rollout belongs to
  reasoning, where the accepted consumer admits candidates anew from each
  cloned branch-node state.
- **Coverage witness** — a bounded replay record showing which lexical units
  had registered addresses; which hierarchy rows were consulted and physically
  present; which entries were examined, admitted, or used only as influence;
  fallback activation; candidate support before/after admission; exact spin
  class and operator/epoch identities where active; the selected route or
  abstention; and all artifact/control identities. Coverage proves only that a
  mechanism was reached; it does not prove attention, correctness, or
  reasoning.
- **Geometric recall** — exact or backed-off retrieval of a stored continuation
  from a known route key. Recall can be useful storage behavior but does not
  establish that geometry chooses among novel continuations.
- **Geometric attention** — bounded, causal support selection or ordering in
  which declared factor, phase, spin/Hopf, torsion, radial, or hierarchy terms
  — including the paired-H4/E8 coordinate and transported trajectory summaries
  — are load-bearing against matched disabled, count-only, or permuted controls.
  Least-energy language applies only to the explicitly admitted support. An
  exact-row hit alone is recall, not attention.
- **Inference** — one or more causal steps that convert an observed prefix and
  bounded state into token probabilities or a selected next token, update
  state, and eventually decode bytes. Reachable attention is one input to
  inference; it is not coherent generation by itself.
- **Geometric-intelligence sequence** — lexical ingestion, canonical
  serialization, and address membership are prerequisite plumbing. The current
  delivery order is the established source-free table-native lexical baseline
  (B0/#989), then exactly one #953 geometric intervention under the same
  corpus, support, decode, and work budget; paragraph, conversation, and
  global exact-spin operator qualification through that loop, followed by the
  ADR-0005 historical direct Q/K/V/O/V4 evidence, pinned HELM-D architecture,
  frozen ordinary donor,
  gauge-equivalent full-decoder ordinary softmax in transported R4/Spin frames,
  real next-token and decoded-output parity, a source-faithful learned-manifold
  intrinsic R4 construction qualifier after V1 stopped unavailable before D3,
  the two-document tangent-readout preflight rejection within the 8/8
  localization contract, qualified provider-free autonomous
  `R4SoftmaxReferenceGeneratorV1` (`HELM-D-R4`) generation, and its explicit
  opt-in, loopback-only native HTTP endpoint plus passing dashboard
  wiring/static native-readiness and WASM-isolation checks (browser
  interaction/E2E `NOT_RUN`), followed by the completed
  `R4SoftmaxTeacherTraceV1`/source-free suffix-student comparison and its
  decoded-cycle boundary. The active successor is
  `R4SoftmaxTraceStateStudentV1`. Intrinsic/readout,
  fiber-preserving multi-resonance replacement,
  whole-decoder recurrent factorization, and final requalification are parked
  (#973);
  correctness with abstention (#954); then bounded hypothetical-branch reasoning
  (#955). #969, #983, and #986 remain evidence provenance rather than a
  parallel implementation queue.
  Evidence may not skip a stage or count prerequisite plumbing as inference.
- **Correctness** — agreement with an independent task oracle, executable
  constraint, cited source, or other predeclared ground truth, reported with
  coverage and abstention. Teacher agreement, grammatical output, and exact
  recall are not general correctness.
- **Geometric reasoning** — a measured capability in which typed intermediate
  route states compose across multiple causal steps, preserve declared
  constraints, distinguish alternatives or counterfactuals, and reach a
  checkable conclusion on anti-recall inputs. Fluent continuation, chain-like
  prose, or a non-zero geometry trace is insufficient evidence.
- **Provider-free serving** — the evaluated serving process makes no runtime
  call to Ollama, a cloud model, a teacher endpoint, or another generative
  provider; required tokenizer, artifacts, state, and decoding are local and
  pinned. Provider-free does not imply transformerless, geometry-only,
  multiplication-free, correct, private, or production-ready.

- **Source control** — the pinned local source-model tokenizer and causal
  forward path, including embeddings, KV state, residuals, normalization,
  MLP/SwiGLU, and LM head. Its learned dense projections use `uor-matmul`.
  These components are present; #950 must establish their coherent
  free-running composition through the product-facing path. It is not the final
  transformerless topology.
- **GeometryContext** — bounded, identity-scoped decoder input containing
  session/route state, ordered memory spans as real tokenizer IDs, provenance,
  and per-position geometric keys or affinities. It binds the exact source
  tokenizer CID and deterministic memory-to-layer adapter/checkpoint identity.
- **Token/node** — one causal prefix position represented by its tokenizer ID,
  hidden state, position, and learned geometric coordinates. It is not an
  R4G1 semantic region or graph node.
- **R⁴ causal mixer** — learned operator that maps the current hidden state to
  geometric query coordinates, maps prior token/memory states to geometric
  keys and values, selects a bounded causal neighborhood by declared
  angular/geodesic compatibility, and aggregates its values into the residual
  before token selection.
- **Geometric query/key/value** — learned R⁴/quaternion coordinates and value
  contributions used by the causal mixer. They are not the historical
  288-bit semantic code, a UOR CID, a word-prime address, or the dormant
  XOR/popcount route-attention code.
- **Phase transport** — the declared update relating compatible geometric
  frames or fiber phase across token/memory states. The specific equation and
  chart assumptions belong to the operator issue; the name alone establishes
  no semantic or language property.
- **Student prefix** — a causal prefix containing tokens emitted by the
  candidate decoder. Student-prefix evaluation exposes rollout states that
  teacher-forced rows do not.
- **Transformerless decoder** — a promoted decoder with zero calls to the
  source-attention operator and no dense full-prefix Q·K matrix/softmax kernel.
  Its mixer selects bounded geometric support and is load-bearing under
  disabled/permuted interventions. The geometry may approximate teacher
  attention during distillation. It may retain
  embeddings, residuals, normalization, MLP/SwiGLU, LM head, and
  `uor-matmul` projections.
- **Multiplication-free runtime** — a separate operation-set claim belonging
  to an exact execution path. It is not implied by “transformerless.”

## Core roles

- **Teacher / source model (T)** — a deterministic evaluation procedure
  `T: C → Δ(V)` for a pinned Hugging Face revision and execution mode. The
  historical TLA compiler accesses it through `TeacherOracle`'s embedding and
  next-token surfaces. The active decoder may additionally use versioned
  hidden-state/Q/K/V/attention/logit traces for bounded mixer supervision; that
  trace use is offline and never makes the teacher a serving dependency.
- **Observation (o)** — the primary compilation sample: a bounded token context `c`, one or more
  teacher-derived representation vectors `h(c)`, the teacher distribution `T(c)`, and optional
  perturbation traces. Never an isolated token.
- **Observation corpus (O)** — the content-addressed set of observations, split into construction
  and held-out certification partitions. Today: the `Corpus` record stream (`compiler.rs`).
- **Certifier** — offline instrumentation that measures fidelity, stability, operation counts,
  allocation behavior, and artifact integrity. Never participates in inference.

## Graph structure

- **Semantic region (n)** — a predictively coherent area of teacher behavior, represented at
  runtime by a packed prototype `p_n`, comparison mask `m_n`, calibrated acceptance radius `r_n`,
  and packed edge ranges. Successor of the transformerless "context class". Regions are
  multiresolution and overlapping; they are not semantic atoms.
- **Membership** — region activation at a given resolution depth, defined by the masked-Hamming
  predicate: `active(n, x) iff popcount((H(x) XOR p_n) AND m_n) ≤ r_n`. An observation may hold a
  bounded set of memberships at each depth (successor of the graded code `[u8;4]`).
- **Semantic code H(x)** — the compiled Boolean encoding of a context (today: the 36-byte sign-bit
  signature). Locality-preserving by design; **not** a cryptographic digest.
- **Refinement edges (E_r)** — parent/child edges implementing zoom-out/zoom-in across resolutions
  (successor of store prefix levels). Multiple parents allowed: the graph is a DAG with
  cross-links, not a tree.
- **Overlap / neighbor edges (E_o)** — lateral edges between co-active or adjacent regions.
  Explicit overlap (intersection) nodes are sparse and must be justified by held-out gain.
- **Forward transition edges (E_f)** — likely semantic successors of the active cloud.
- **Reverse indexes (E_b)** — teacher-supported predecessor evidence; built by sorting the same
  canonical edge IDs used by E_f (Theorem 7). Evidence lookup, not mathematical inversion.
- **Semantic cloud (A)** — the set of currently active regions (the "frontier"). `F` denotes the
  predicted next cloud.

## Runtime

- **Runtime state** — fixed-capacity, caller-owned: active frontier, rolling context code, bounded
  token shortlist, optional witness buffer. Successor of the 8-token window + `Runtime` struct.
- **Frontier** — bounded set of active regions; each entry stores region ID, fixed-point score
  (`ScoreQ`), membership margin, and depth. Max width is an artifact-declared constant.
- **Step (R_G)** — one token of inference: consume state + newest token, produce new state +
  prediction, with zero allocation, no float, no multiply, no locks, no unbounded search.
- **Normative kernel** — the only arithmetic the runtime may use: word XOR/AND/OR, shifts,
  rotates, popcount, integer add/sub, compares, declared saturating arithmetic, table reads
  (today: `OpKernel` + census). Scalar safe Rust defines normative semantics; accelerated kernels
  are replaceable and equivalence-tested. The boundary and positive/negative operation lists are
  normatively frozen in `docs/transformerless/INFERENCE_OPERATION_CONTRACT.md`.
- **Bounded-work constants** — manifest-declared limits: `A` frontier width, `C` candidate regions
  per active node, `W` signature words per region, `E` emission entries per region, `K` token
  shortlist size, `D` decision-program depth. Per-step work is O(D + A·C·W + A·E·K).
- **ScoreQ** — quantized fixed-point log-domain score. Replaces all floating-point scores in
  deployed paths.
- **Scoring model** — after the issue-#64 redesign, two per-context rules. Rule 1
  (chain-telescoped): `S_graph(v) = B(v) + Σ_{n∈chain} ΔE(n,v) + ΔT-offset`, where `chain` is
  the covered refinement chain (root → deepest covered ancestor) of the active region with the
  deepest covered chain — emission corrections compose along one ancestry path instead of
  stacking across sibling subtrees. Rule 2 (D4 EXCT precedence): when the exact-context probe
  resolves at the FULL graded code with enough evidence (total ≥ `EXCT_SUPPORT_MIN` = 5),
  `S(v) = B(v) + ΔX(X,v)` and graph residuals are skipped entirely; a probe that resolves below
  full depth is prefix backoff, admits nothing, and falls through to Rule 1 (#234, maintainer
  decision 2026-07-29). An explicit supported NGRAM context row (trigram → bigram backoff, #380)
  takes the same most-specific precedence before the EXCT probe; the root prior `B(v)` remains
  the unigram backoff. Each table is sparse; no
  contribution is counted twice (Theorem 10). Supersedes the literal Σ-over-cloud form
  (`B + ΣΔE + ΣΔT + ΔX`), which double-counted correlated sibling residuals (Gate C: 0.3%
  vs 31.7% baseline).
- **Root prior B(v)** — base token distribution stored at the graph root (successor of store
  level-0 backoff counts).
- **Emission residual ΔE** — per-region correction to token scores relative to its parent.
- **Transition residual ΔT** — per-predicted-region correction for likely next tokens.
- **Exact-context residual ΔX / EXCT store** — residual evidence keyed by exact context, capturing
  behavior the compressed graph does not explain (successor of the TLS1 graded store).
- **Resolution status** — every step returns exactly one of:
  **Supported** (strong interior evidence), **Boundary** (several plausible overlapping regions),
  **BackedOff** (only a broader region met support), **Novel** (no calibrated region covers the
  input), **Contradictory** (active regions make materially incompatible predictions).
  Deterministic (Theorem 12); the manifest declares per-status behavior (continue, widen, consult
  EXCT, certified fallback, or abstain — default policy per decision D4). The deployed R4G1
  adapter (`src/r4g1.rs`, issue #78) wires the D4 policy over the scorer's
  `ScoreStatus` (`exact_context` → serve, `graph` → serve, `novel` → widen-once then abstain,
  `contradictory` → abstain, reserved) with an optional `config.status_policy` override in
  `score_report.json`; abstention is a typed, server-surfaced outcome, and widening is bounded
  by a fixed-capacity memory of confirmed-Novel signatures.
- **Multi-timescale state** — hierarchy of fixed-capacity states: token, local phrase/event,
  segment, document/session; none grows dynamically.

## Artifacts and identity

- **R4G1** — the versioned packed artifact container (mandatory sections HEAD/CODE/NODE/EDGE/
  ROUT/EMIT/PROV plus optional EXCT/CERT/PTCH/SECT/RTNX/FMM/NGRAM/FWDA). Succeeds
  TLA3/TLA4/TLS1. See `docs/transformerless/R4G1.md`.
- **κ (kappa) / content CID** — content address (blake3 label or UOR CID) preserving identity and
  provenance of bytes. CIDs are **not** semantic hashes and are never used as routing codes.
- **Semantic route code** — a compiled, versioned, intentionally locality-preserving code used for
  region routing. Separate lineage from CIDs; never an authorization or security identity.
- **Witness** — a bounded, replayable record of one prediction: graph CID, input code, active
  regions + margins, traversed decisions, applied edges, contributing emission entries, exact
  entry, selected token, op census. An independent verifier replays it without the teacher
  (Theorem 6).
- **Epoch / patch** — immutable base graphs are amended only by content-addressed patch epochs
  (parent CID, additions, score residuals, tombstones, compatibility limits, certificate). Lookup
  consults a manifest-bounded number of layers; compaction emits a new canonical base.
- **Route translation** — evidence mapping regions of one epoch to retained/split/merged/removed
  regions of the next.

## Certification

- **Teacher-fidelity certificate** — measured agreement of graph and teacher on a pinned
  evaluation set: top-1 agreement, top-k recall, bits/token, divergences, with CIDs, confidence
  intervals, slices, and protocol. Valid only on the declared distribution (decision D3).
- **Bits/token (canonical definition, issue #76)** — the mean cross-entropy of the true next
  token under a scorer's predicted distribution: for held-out positions `c_i` with true next
  token `v_i`, `bits = (1/N) Σ_i −log2 P_scorer(v_i | c_i)`, where `P_scorer` includes the
  scorer's floor mass for out-of-candidate tokens. One definition, one unit (bits, base-2 log);
  implemented in `score.rs::outcome_bits` (Gate C harness) and in the certificate path.
  **Comparability rule**: values are comparable only within the same scorer AND the same
  evaluation distribution. The historical "families" are scorer/distribution differences, not
  metric differences: 6.54 = P2 certificate (Witten-Bell store on its legacy corpus), 11.88 =
  the same Witten-Bell helper on the fixture corpus (Gate C baseline row), 9.86 = the Rule 1+2
  graph scorer on the fixture corpus. Reports MUST name the scorer and distribution alongside
  the value.
- **Semantic-coherence certificate** — separate evidence that regions generalize: cross-context
  reuse, perturbation stability, boundary behavior, rare-context retention, anti-memorization.
  Predictive coherence alone does not make a region "semantic".
- **Rate-distortion curve** — artifact bytes + runtime ops vs. teacher information retained,
  measured at broad/intermediate/full-cloud/residual-augmented depths.
- **Reference classifier** — the exact compiler-side region-membership procedure; the normative
  semantics every optimized router is measured against (shortlist recall, Gate H).
- **M.V.G. checkpoint** — the minimum-viable-graph go/no-go review at the end of Phase 5
  (decision D1), comparing the graph against pre-agreed targets recorded in
  `docs/transformerless/BASELINE.md`.
- **Baseline** — the certified transformerless artifact and its measured fidelity. The figures
  here are the original 150k-era record (TLA3/TLS1: 28.9% top-1, 31.7% teacher-argmax agreement,
  6.54 bits/token, 89,200 store keys — PROOF.md P2); the current-era pins live in the PROOF.md
  P2/P3 era notes and `baseline_kappa.json`. Gate C compares the graph against this baseline
  before replacement.

## Target operators and dormant lanes (Epic #602)

- **TARGET operator** — a versioned, registered `(id, version)` selection/attention mechanism
  candidate for the deployed R4G1 runtime, following the two-stage template established by #604
  and reused by #643: compiler/certify-side reference semantics + a replayable operation-count
  witness land first, a P-4-scanned packed R4G1 lowering (table read / integer compare /
  saturating add only, no runtime multiply/divide/modulo/float) lands second. Registered in
  `uor-r4-model-source::attention::operator_spec`.
- **Dormant lane (#515 "preserve-and-gate")** — an operator implementation that is
  constructible, differentially tested (reference vs. packed kernel agree bit-for-bit), and
  registered as an `open`-level claim in `model/ledger.toml`, but referenced by no serving path.
  Each claim's `statement` names an explicit activation gate (e.g. a pre-registered A/B exit
  rule) that must clear before the operator can be wired into serving.
- **`r4-route-attention/1`** (#604) — masked XOR+popcount route-code distance, bounded top-M
  selection, saturating `ScoreQ` aggregation. Dormant (`r4-route-attention-dormant`); an offline
  `route-fit/1` method (#605, `uor-r4-graph-compiler::route_fit`) fits route codes from teacher
  query/key vectors via the `bucket-average/1` projection for evaluation on a synthetic held-out
  corpus.
- **`msa-structured-selector/1`** (#643) — Modular Structural Arithmetic role-class + cascade-orbit
  selection: candidates classify by `candidate_id mod 11` into a paper-proven 3-anchor role table
  (MSA7's "11-Theorem") plus this project's own cascade-position-mod-3 extension for the
  remaining residues; selection and aggregation are plug-compatible with `r4-route-attention/1`
  (same top-M/tie-break/`ScoreQ`-fold shape) for a shared A/B harness. Classification is
  position-only (no fitting step). Measured NEGATIVE against `r4-route-attention/1` on the #605
  synthetic corpus (`msa-structured-selector-dormant`, `uor-r4-graph-certify::msa_ab_harness`):
  clears the unigram floor by a wide margin but falls well short of the fitted, content-aware
  route-attention arm.
- **PROV/1** (#637 phase 1, `crates/uor-r4-graph-format/src/prov.rs`) — the canonical bounded
  provenance-identity envelope for R4G1's PROV section: a presence-bitmapped set of digest slots
  (source-manifest κ, geometry, tokenizer-adapter, attention-operator, dense-operator, license)
  plus a strictly-ascending evidence-root list. Frozen as a format (parser + builder + RFC row);
  wired into the real producer path as an additive, opt-in `cover --emit-provenance` flag (phase
  2a, #733) — no existing artifact's bytes or κ moved. Not yet the default; the default-flip and
  consumer/era adoption are phases 2b/3, open pending format-governance review.
- **Release-bundle manifest** (#655-C0+, `crates/uor-r4-api/src/release_bundle.rs`) — the versioned
  `ReleaseBundleManifest` schema a packaged serving bundle declares: schema version, public model
  id, instruction-chat capability, ABI/contract version, pinned `uor-matmul` provenance,
  component digests, and tokenizer identity. Current production discovery and
  admission read and content-bind the schema-2 envelope; historical v0.1 remains
  explicit research-only input.

## Recent Architecture & Engine Additions (Epic #201)

- **`tokenizer_cid`** — BLAKE3 hash of the loaded `tokenizer.bin` checked against `R4G1Header::tokenizer_cid` in `uor-r4-graph-format` (`verify_tokenizer_cid`), guaranteeing that loaded tokenizers match compiled graph artifacts and preventing silent index shifts.
- **`parse_store_strict_u32`** — The normative 32-bit integer token ID store parser in `uor-r4-core::runtime`. Replaces deprecated `u16` legacy store loading.
- **`FallbackRouter`** — a retained legacy cascade type in `uor-r4-router`.
  Current production serving has no callers and no silent TLA fallback; the
  active geometric decoder does not route through it.
- **`EngineStatus`** — Typed status enum (`Success`, `UnmappedRegion`, `Pathological`, `Failed`) classifying engine inference outcomes.
- **`UorAttestationResult`** — Envelope wrapper for synthesis outputs containing verified BLAKE3 content-addressed CIDs (`uor_address`, `artifact_cid`, `store_cid`, `attestation_cid`), validated by `POST /api/uor/verify`.
- **`W(3,3) Phase Field`** — The 96-vertex $S^3$ graph canvas visualization in `index.html` rendering real-time Markov trajectories from WebSocket telemetry streams.
- **`ChatML Prompt Wrapper`** — Canonical ChatML format (`<|im_start|>system...\n<|im_start|>user...\n<|im_start|>assistant\n`) implemented in `scenarios.rs` (`encode_chat_prompt`) to format instruction-tuned teacher observations (`SmolLM2-135M-Instruct`).
