# Geometric attention: mathematics, source and learning synthesis

September 13, 2026 · UOR-R4 Geometric Language Model · References [#973](https://github.com/UOR-Foundation/uor-r4/issues/973) and [#964](https://github.com/UOR-Foundation/uor-r4/issues/964)

**Research result: SELECTED_RELATIONAL_GEOMETRIC_LEARNING_HYPOTHESIS. Model status remains UNQUALIFIED_INITIALIZED_NO_FIT; retain `15baec48`.** This investigation changes the next design decision, not the retained model. It performs no fit, model evaluation, replay of V3–V7 or runtime change.

The recommended direction is a **learned geometric recurrent model that preserves directed relationships through contextual reads**. Use Hamming comparisons to find candidates, full signed relative transformations to interpret them, and shared learned nonlinear operators to update state and generate bytes. Prime identities and canonical memory preserve exactly what an object is. Zeta-derived phases and the H4/icosian construction supply explicit geometric coordinates and actions. Learning must determine how these representations predict language.

The current initialized Hamming policy is a useful execution reference, but its random three-layer LUT4 funnel should not be presumed to be the eventual learner. Source inspection identifies a concrete restriction: each output bit can combine at most 64 of its 4,096 inputs per call, and the whole output shares a 256-bit intermediate compression. The owner’s concern that richer relationships may never reach a decision is therefore actionable. Recurrence and comparisons broaden dependencies, so this is not a proof that the entire model sees only 64 bits or cannot learn.

The proposed next implementation is one complete structured learning path, combining the representation correction, offline update, deterministic export and a minimal contextual learning experiment. It should not become another sequence of isolated process probes. Accumulated memory repair is a later replacement gate, not the first learning threshold.

## Research map and reading boundary

This report is the decision synthesis. Its companion records preserve detail:

- [Mathematical foundations](mathematical-foundations.md): metrics, relative frames, H4/E8, Hopf/fiber, harmonics, zeta/prime formulas, Hardy–Littlewood, Cayley–Dickson, chart bridges and proof obligations.
- [Attention and learning](attention-and-learning.md): transformers, associative memory, geometric/nontransformer models, discrete training, JEPA and the three candidate architectures.
- [Actual repository mathematics](repository-mathematics.md): source locations, implemented domains, current wiring, exact operators and current versus historical imports.
- [Local research consultation](local-research.md): NEMESIS, SpiralCore/FBS, owner PDFs and HDC, with exact limits on reuse.
- [Coverage inventory](coverage.json), [mathematical sources](math-sources.json), [attention sources](attention-sources.json) and [repository source ranges](repo-math-sources.json): discovery versus reading versus measured evidence.

The complete tracked-path inventory has 14,629 paths. The existing knowledge index provides hundreds of repository records and source relationships; it is a discovery aid, not proof that their contents have been read or are current. Sixteen relevant repository heads were queried, fifteen accessible and GoldSnnail unavailable through the current endpoint. Specific current source was inspected where it could affect the conclusion. Seventeen matching Downloads paths resolve to twelve distinct PDFs; four are image-only and required visual inspection. Long archival PDFs, repository inventories, abstracts and unavailable originals are not labeled fully read. The combined source lists intentionally retain overlaps and do not inflate them into independent experiments.

This is a substantial bounded research pass across all requested mathematical families and the relevant available engines. It does not claim to have read every file, every paper or every proof in all available repositories. The sources that would change the next decision have been made explicit, and remaining access/reading gaps are recorded. Retrieved source instructions never replaced the owner's request or project policy.

## What transformers actually supply

The original transformer learns query/key projections, compares eligible context, normalizes competing scores, combines values, and applies further nonlinear updates. Its prediction loss trains those operations together; later layers can ask questions based on earlier retrieved information. Attention is an architectural path through which relevance can be learned, rather than a property that appears merely because training occurs. [Vaswani et al., §§3.1–3.3](https://papers.neurips.cc/paper/2017/file/3f5ee243547dee91fbd053c1c4a845aa-Paper.pdf).

For this project, preserve the functional responsibilities while changing the computational mechanism:

| Responsibility | Proposed native geometric operation | What must be learned |
|---|---|---|
| Represent the current question | Typed causal state and a finite geometric query | Which distinctions and relationships the query expresses |
| Find eligible context | Exact occurrence admission, geometric pages and Hamming comparison | Contextual key placement and compatibility |
| Compare alternatives | Bounded relation descriptors and a shared nonlinear comparator | Which candidate changes prediction usefully; when Null is appropriate |
| Bring information into state | Exact selected payload plus finite relative transport | How selected content modifies working state |
| Compose dependencies | Update state, then recompute the next query/read | Which earlier result the later decision needs |
| Produce language | Native learned byte/EOS decisions and observed-byte recurrence | Lexical choice, syntax, termination and longer-range consistency |

An invariant compatibility score and an orientation-bearing value have different jobs. This distinction is explicit in geometric attention research, including SE(3)-Transformer and gauge/connection methods. Their tensor contractions are not imported into serving. [SE(3)-Transformer](https://proceedings.neurips.cc/paper_files/paper/2020/file/15231a7ce4ba789d13b722cc5c955834-Paper.pdf), [gauge-equivariant methods](https://proceedings.mlr.press/v97/cohen19d/cohen19d.pdf).

Hard selection can provide attention/context access without runtime softmax. It also makes credit assignment harder: a small parameter change can switch an address and alter the entire later trajectory. Training must follow that changed trajectory. Replacing softmax by a minimum distance without learning the query, key, update and output leaves most of the problem unresolved.

## Hamming is relevant, but a scalar distance is not the full relationship

For bipolar encodings, `b·c = D − 2 d_H(b,c)` is an exact identity. For random hyperplane codes, disagreement probability is proportional to angle under the construction's assumptions. The current H4 signatures instead use a fixed 120-landmark bank. Their preserved finite census establishes unique signatures and single-root ranking agreement in that codebook; it does not establish general semantic similarity or equivalence of two-lane summed scores. [Charikar §3](https://www.cs.princeton.edu/courses/archive/spr04/cos598B/bib/CharikarEstim.pdf), [Plan–Vershynin](https://arxiv.org/pdf/1111.4452).

Sparse Distributed Memory supplies a direct attention precedent: neighborhood overlaps can approximate attention-like compatibility under specified data conditions. That makes Hamming neighborhoods a serious mechanism to investigate. It is not a proof that our current hard H4 reader inherits the same behavior. [Bricken and Pehlevan](https://arxiv.org/html/2111.05498).

The present signature weights are all 45. Therefore comparing only each signature's number of set bits would discriminate nothing. XOR/popcount between two signatures measures which landmark predicates disagree; keeping the full signature retains more information than that count. Similarly, two equal distances to a query can represent different directions, roles or histories.

The better candidate descriptor retains a directed relative element for actual finite group states:

`r(i,j) = inverse(g_i) · g_j`.

This gives `r(i,j) · r(j,k) = r(i,k)` and `r(j,i) = inverse(r(i,j))`. It is invariant under a common left frame change. The existing inverse/product tables can compute these finite actions without executing a matrix contraction. For two four-root states, a 4×4 block of relative elements preserves cross-lane relationships that two scalar distances cannot express. All sixteen need not become permanent storage or compulsory work in every decision; their value should be tested against a simpler relation representation.

These relative-frame edges are a flat construction: their products telescope around loops. They must not be relabeled as nontrivial curvature or holonomy. A genuinely path-dependent connection needs independently defined edge transport and corresponding consistency tests. Likewise, the group identities above cannot be copied unchanged to nonassociative octonion products.

Hamming is also useful for drift diagnostics: compare signatures, selected references, emitted bytes and operator traces separately. A large route change is not automatically a semantic error; a small Hamming change may still switch an exact value or version. Preserve payload and occurrence identity beside every approximate representation.

## The all-pairs, prime and semiprime proposal

The owner's proposal becomes concrete as a canonical graph of occurrences and directed relations. A node contains exact content/occurrence/version identity, learned geometric state, fixed phase coordinates and type. A relation records endpoints, the relative transformation, role/order, any retained connection/path data and lineage. A query supplies a context-dependent compatibility score.

All-pairs information is reasonable for a small reference window. It should not be dismissed solely because it scales quadratically:

| Occurrences | Unordered pairs | Eight-byte pair payload | Sixteen-byte pair payload |
|---:|---:|---:|---:|
| 256 | 32,640 | 255 KiB | 510 KiB |
| 4,096 | 8,386,560 | 63.984 MiB | 127.969 MiB |
| 65,536 | 2,147,450,880 | 15.9998 GiB | 31.9995 GiB |

These are arithmetic payload projections, before indices, versions, phases or sixteen-lane blocks; they are not runtime measurements. Within a common frame, all pairwise group relations can be reconstructed from node states or one anchor plus `N−1` relative states. Caching them adds access convenience and redundancy, not independent information. Pairwise angles/lengths identify a spanning configuration only up to an orthogonal transformation, including reflection; they do not recover chirality. Keep oriented state explicitly.

The proposed online path stores exact nodes and bounded selected links, computes missing relations on demand, and updates only links whose state/version changed. An offline all-pairs reference at the existing 256-occurrence bound can expose what a bounded candidate rule discards. It is a development comparator, not an oracle that supplies hidden correct semantic links at inference.

Canonicalization is particularly valuable here. Immutable geometry can be cached by the exact geometry/phase/encoding identity. A learned attention score additionally depends on the current query, model parameters, source version, candidate mask and metric. A score stored only under two content hashes can be stale even when the content is unchanged.

The repo already has a `SemiprimeExpert` type that sorts two factors and identifies shared-prime handoffs. It is a routing combinator, not a trained specialist. `pq = qp` loses direction; the existing ordered n-let representation supplies information that the product alone cannot. Prime-addressed shared typed operators are immediately compatible with the project direction. Separately trained experts and a learned gate remain a conditional future model choice requiring demonstrated need and laptop-cost evidence.

## Zeros, primes and the geometric bridge

There is an established analytic bridge between primes and zeta zeros. It is expressed by Euler products, logarithmic derivatives and explicit formulas, with prime powers and convergence conventions included. It does not pair the first prime with the first zero as equal geometric objects. Hardy–Littlewood prime-correlation conjectures also enter the study of zero correlations; those conditional statistical relationships must not be promoted to an unconditional pointwise identity. The mathematical companion states the explicit formula and distinguishes its assumptions. [Bombieri, RH chapter §5](https://www.claymath.org/wp-content/uploads/2022/02/MPPc.pdf), [DLMF §25.16](https://dlmf.nist.gov/25.16).

The implemented project coordinate is of the form

`phase_j(p) = gamma_j log(p) mod 2π`,

with relative phase `gamma_j log(p_to/p_from)`. The current native compiler exposes eight quantized channels and the addressed policy uses four; the fixed source table contains 512 ordinates. These are logarithmic oscillatory coordinates. Current Hamming distance is measured between geometry-derived root signatures, not between a zeta zero and a manifold vector.

More zeros can provide a richer fixed phase basis. This is a meaningful hypothesis when the learned state can use the added channels and a concrete aliasing or predictive limitation motivates them. At 512 `u16` channels, a raw phase vector occupies 1,024 bytes instead of eight bytes for four channels. Quantized high-frequency channels introduce resolution and cost questions. Their sums remain commutative: adding more such sums cannot distinguish AB from BA. Ordered finite actions and occurrence records must preserve sequence order.

**Owner clarification during research:** “L1” means a one-dimensional visual projection, not Manhattan distance or an L¹ function space. The n-lets mean combinatorial groups of up to six primes projected into the geometric construction. The proposed interpretation is a low-cost prime/divisor structure coupled to the zero-spectrum view, with exact E8/UOR structure correcting drift. This meaning supersedes the provisional consecutive-prime-span example in the mathematical discussion; that example is retained only as a distinct mathematical comparison.

For divisor counts `d(n)`, the precise Dirichlet-series relation is `zeta(s)^2 = sum_(n>=1) d(n)/n^s` for `Re(s)>1`. This follows by grouping pairs of factors in the product of two absolutely convergent zeta series. Its coefficients are not the ordinates of its zeros. Squaring zeta doubles zero multiplicities, while preserving their locations; it does not make a one-dimensional divisor-count plot equal to a plot of zero ordinates. The prime-power weights in `-zeta'(s)/zeta(s)` and the divisor counts in `zeta(s)^2` are different arithmetic signals. Both can inform a declared spectral representation. [Harvard Dirichlet-series lecture, p3, proposition 1](https://legacy-www.math.harvard.edu/archive/math124_2012/math124_fall_2012-13/Third.lecture.Dirichletseries.Dirichlet.Product.pdf).

The recalled six-prime design is already explicit in [ADR0003](../../adr/0003-fixed-zeta-prime-route-attention.md): an ordered sextet yields fifteen distinct semiprime pair labels, GCD adjacency supplies the Johnson graph `J(6,2)`, and the pair slots can correspond to fifteen `Cl(0,6)` bivectors under a specified basis and orientation. The exact finite operator and the proposed semantic map remain separate. A square-free k-prime product has `d(N)=2^k`; repeated factors instead give `product(a_i+1)`. This is exact combinatorial scaling, but every square-free product at the same k ties. It cannot by itself locate the correct language answer. The ordered identities, full relations and learned contextual use remain necessary.

Between twin primes greater than 3, the middle integer is a multiple of 6: both primes must be `6m−1` and `6m+1`. Such centers have at least four positive divisors. This is an exact modular fact, not a theorem that all centers are highly composite or that their divisor count increases monotonically; for example, the centers 72 and 102 have respectively twelve and eight divisors. Among integers greater than one, divisor count 2 characterizes primes, but a routing cost defined that way ties every prime destination. It is a structural prior, not a measured physical energy or a semantic selector. The current source variable `divisor_counts` in the historical manifest compiler counts observed transitions indexed by a prime atom; it is not the arithmetic function `d(n)`.

Exact `Z[φ]` operations and coupled inverse witnesses can enforce algebraic consistency and reconstruct trusted canonical objects. A separate nearest-codeword decoder can correct bounded corruption under a declared metric and noise radius. Neither automatically recovers the intended contextual state when the model selects a different valid codeword. Distinguish serialization corruption, quantization error and legitimate learned motion before applying correction. The combined candidate is an exact canonical object with arithmetic and spectral observations, directed finite transport and learned relevance—not a projection that assumes the answer is already encoded at the lowest-divisor location.

The proposed evaluation is causal and matched: compare the selected learned representation with phases disabled and, later, equal-cost alternative/scrambled frequency controls. A phase benefit and a specifically zeta-derived benefit are distinct results. This does not demote the prime/zeta architecture; it identifies how to show what it contributes.

The owner's `sqrt(2) = 2i = [0,2]` bridge has two possible readings. In the repo's existing ADR, `[0,2]` is the range of unit-sphere chord distances, while `sqrt(2)` is the orthogonal chord and `2i` an antipodal complex displacement of magnitude 2. If `[0,2]` instead denotes the coordinate pair `(0,2)`, it is exactly the ordinary real-coordinate representation of `2i`. It still has length 2 under the standard Euclidean metric, not `sqrt(2)`.

A constructive mathematical example is the displacement `z=1+i`, whose Euclidean length is `sqrt(2)`. The map `F(z)=(1+i)z` sends it to `2i`: a rotation by π/4 plus a scale of `sqrt(2)`. If the target metric is scaled by one half, the physical squared length remains 2. This is an explicit coordinate/metric correspondence; it is not equality of raw quantities in unchanged metrics. It is also not a proposed serving matrix action.

R4 and C² have a simpler exact coordinate identification: `(a,b,c,d) ↔ (a+bi,c+di)`. A genuinely different Riemannian metric needs a specified metric tensor or discrete metric. On unit spheres, chord and geodesic distances are related by `d_chord = 2 sin(d_geodesic/2)`. A faithful adapter must preserve the declared operation, composition order, signed orientation/fiber, reconstruction domain and attention score ordering, with any quantization error explicit. Geometry conversion can be useful without solving RH; proving a prime-zero formula alone would still not supply the token metric or learned relevance.

## What the other mathematics contributes

| Toolset | Concrete architectural role | Missing obligation |
|---|---|---|
| Exact H4/quaternion actions | Closed, associative finite state transport and relative frames | Learn useful placement and action selection; account for finite capacity |
| Coupled icosian/E8 representation | Exact `Z[φ]`, companion construction, basis/glue and inverse witness | Preserve the specified inner product; companion is not independent learned state |
| Hopf observation | Coarse base-space view with full state/fiber retained | Avoid an assumed globally invertible S3→S2→S1 chain |
| Spherical harmonics/heat kernels | Offline finite multiscale angular score tables | Truncation, sign/orientation loss, quantization and learned usefulness |
| Gauge/connection geometry | Separate frame alignment from scalar affinity | Define genuine path dependence where needed; lower to legal finite actions |
| Cayley–Dickson/Clifford | Rich typed noncommuting operator families | Faithful multiplication convention, associators, actual model action and legal runtime |
| UOR/Prism | Canonical identity, exact object operations and persistence | Do not confuse modular rings, Boolean rings, extension fields or semantic distance |
| W33 | Finite incidence/action and current immutable process-lineage mechanisms | A faithful R4-language map and predictive learning remain absent |
| NEMESIS | State/transition carrying contracts, Hamming code learning and context-search ideas | Correct specific arithmetic/claim errors; provide an executable learner |
| SpiralCore/FBS | Exact action/path/observable separation and typed composition | Lawful moves and Bell labels do not select useful language actions |

The source audit finds two especially relevant implementation gaps. The legacy `cayley_dickson.rs` uses cyclic coordinate products and a scalar contraction rather than the cited Cayley–Dickson multiplication algebra. The dormant 16-dimensional Clifford prototype has an identity spectator block that breaks its advertised full-space anticommutation relation. The exact finite SpiralCore adapter is a stronger algebraic donor, but has no established semantic H4 transport bridge. These findings concern particular modules; they are not conclusions that the mathematics is unusable.

The local RH manuscripts also contain specific unproved steps. Their coordinate regularization does not establish a zero normal derivative of `arg ξ` along an entire boundary, and the displayed isolated-pair torque remains nonzero even for the claimed critical-line case. A full proof campaign is unnecessary for using a pinned finite phase table. Preserve the manuscripts and their useful phase/chart ideas while leaving those proof obligations explicit.

## Architecture alternatives and decision

**H1 — Structured geometric recurrent learner with relation-preserving reads.** Keep the exact memory/refinement/trajectory scaffold; replace the presumed random funnel with deliberately connected shared nonlinear blocks. Preserve identity and selected bytes through explicit carry paths. Let cross-field blocks combine the geometry, phase, payload and role information needed by the current task. Learned query/key/state/emission decisions act through finite geometry and logic. This is the first recommendation because it addresses an observed source restriction and the missing learner without abandoning the project's primary representation.

**H2 — Learned bounded traversal over a canonical relation graph.** Extend the H1 interface with typed occurrence/result links, full relative transformations and causally recomputed queries. It directly expresses the owner's recursive relationships and permits an all-pairs small-window reference. The graph must be inferred from observed data or declared training supervision; a hidden semantic parser must not do the model's work.

**H3 — Predictive geometric hierarchy with a JEPA-like auxiliary objective.** Add prediction of future/missing latent states to a model that already learns exact lexical output. Text-oriented latent prediction has primary-source precedent in data2vec, and VL-JEPA separates embedding prediction from decoding. Their actual transformer/pretrained components are not admissible serving replacements. A native auxiliary objective may help long-range organization but needs anti-collapse controls and must improve generated language. This remains in the research plan, as the owner requested. [data2vec](https://proceedings.mlr.press/v162/baevski22a/baevski22a.pdf), [VL-JEPA](https://arxiv.org/html/2512.10942v2).

No reviewed prose-capable alternative supplies this entire runtime unchanged. Mamba/RWKV/RetNet/GLA/DeltaNet retain mathematical matrix products. HELM/nGPT/GATr retain dense contractions. Even the published “MatMul-free” model's ternary signed accumulation is a matrix product under this project's stronger rule. Their selection, memory-update and optimization lessons remain useful; their runtimes are not imported. Exact finite geometric actions are permitted; a tensor contraction disguised as a codec is not.

## One next implementation, followed by a fair learning progression

The next task should implement H1's structured shared operator and offline Rust update as one coherent change. The current hard trajectory remains the reference. The design must state each learned input/output, recurrent state, valid finite action, how a selected payload influences a byte/EOS margin, and the estimator's bias. Resolve the per-decision dependency restriction by construction rather than adding more disconnected inputs. Preserve all current artifacts as references.

Use a hard-forward surrogate with an explicitly declared backward approximation as the leading optimizer candidate, rather than restarting the failed stochastic per-invocation recipe or enumerating the entire truth-table space. Offline smooth computation is allowed; exported execution must match the deterministic hard path. Shared parameters require credit through every downstream changed read and publication. A tiny finite contrast can check the direction's causal scope, but is not itself a scalable training algorithm. Structured logic-tree research motivates connectivity and pass-through initialization; it does not guarantee language scaling. [Logic-tree networks](https://arxiv.org/html/2411.04732), [VQ-VAE](https://arxiv.org/pdf/1711.00937).

The progression is learning → dependent composition → natural prose → integrated replacement:

1. **Learn one contextual relation.** Train a small balanced source-value/position task and generate the selected single byte followed by EOS. Freeze a construction/development split and a finite dose before fitting. Freeze quantitative byte-plus-EOS success and improvement thresholds against initialization before fitting, with held-out source/value/position combinations. Require changed-source sensitivity and degradation with the same artifact's ReadDisabled control. Define whether that intervention disables ingestion reads, prediction reads or both, and balance counterfactuals against final-byte/position shortcuts. If learned recurrence legitimately retains the answer, a weak read ablation limits the explicit-attention claim rather than disproving contextual learning. The parameter update, not an authored query/output fixture, must produce the behavior. Record every response and actual winning margins. A first failure prompts bounded diagnosis within the recorded budget, not broad rejection of geometry.
2. **Compose learned reads.** Make the second address depend on the first selected value, and reuse a computed result. Test unseen compositions/relocations and update-disabled/restricted-context controls. Preserve exact order/version/occurrence independently of geometric similarity.
3. **Learn small natural prose.** Use licensed document-separated text in Rust preparation. Measure held-out byte loss and actual free-running continuations, context interventions and complete serving costs. Copy/arithmetic success alone cannot pass this stage. Include distributional modeling and termination, rather than treating retrieval as all of language.
4. **Qualify a replacement.** Only at replacement readiness apply accumulated retained repair/independent-turn/conflict/history controls, complete API/session checks and laptop costs. Existing acceptance remains intact. Never promote a regression because a new panel improved.

Do not launch all geometric ablations at once. Start with the least rich representation that retains the target distinction. Add cross-lane relations, more phase channels or a harmonic scale in response to measured collisions or predictive limits, preserving matched comparisons. An inexpensive all-pairs relation diagnostic is legitimate when it answers a specific admission question; it must not turn into another standalone campaign that postpones learning.

This research task used a zero-model-execution projection: 180,000 ms engineering commands, three-hour wall ceiling, 4 GiB RAM, 64 MiB new storage and the existing 128 MiB stop margin. The shared model ledger remains `123730724/132950000 ms`. The next implementation must refresh actual balances and record its own complete build/fit/control/correction projection before execution, using the standing local-extension authorization where necessary. No paid compute or destructive cleanup is authorized.

## What this result establishes

The investigation establishes a source-grounded research decision, a mathematical role for the owner's proposed relations, a concrete current topology concern and a staged way to test learning. It does not establish general prose, a successful optimizer, semantic advantage from more zeros, a classical RH proof, energy savings or model promotion. The canonical plan and current-state pointer adopt this next design responsibility; all old source, negative candidates, original research and sealed evidence retain their identities and paths.
