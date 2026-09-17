> **UOR-R4 Project Knowledge Base — source report C (mathematics audit).** Produced 2026-09-16 by a Claude research session at Casey's request, from the GitHub clone (main @ a5655e93, 2026-09-14) and live sources. Read-only audit; nothing in the repo was modified. Treat "[Inference]"/"[ASSESS]" as reviewer judgement and everything else as quoted/derived from the cited files. Index: `00-project-brain-index.md` (this folder) / `claude/00-project-brain-index.md` (Claude Project).

# UOR-R4: Mathematical foundations review (report C)

Reviewer role: mathematician / ML theorist. Repository: `github.com/UOR-Foundation/uor-r4`, partial clone at `/home/claude/work/uor-r4` (main @ a5655e93, 2026-09-14). Nothing in the repository was modified. All quotations give the file path; line numbers refer to the clone.

Documents read completely: `README.md`; `docs/integration/geometric-attention-research-2026-09/{README,mathematical-foundations,repository-mathematics,attention-and-learning,local-research}.md` and the four JSON source lists; `docs/integration/architecture-2026-09/{README,mathematics,engines,imports}.md` (source-inventory.json skimmed); `docs/integration/shared-geometric-core-2026-09.md`; `docs/integration/{external-research-audit,nemesis-w33-relevance,uor-source-audit}.md` (afflom-ecosystem-review.json skimmed); all six ADRs; `docs/formal_vocabulary.md`; `docs/hologram_formal_analysis_direction.md`; `docs/fmm_290_novel_context_protocol.md`; `docs/hopf_projection_remediation_306.md`; `docs/native_geometric_independent_neighbor_973.md`; head of `docs/integration/current-state.md`. Source read: `crates/uor-r4-core/src/prime_route_attention.rs` (lines 1–1290), `native_geometric/ordered_state/runtime.rs` (all), `native_geometric/hamming_refinement/metric.rs` (all), `spiralcore_operator.rs` (header, lines 1–200), `native_geometric/addressed_attention/artifact.rs` (`BoundGeometry`), `native_geometric/training.rs` (`geometry()`), `native_geometric/hamming_policy/policy.rs` (constants), `native_geometric/shared_core.rs` and `shared_core/training.rs` (headers), `canonical_lexical_ingestion.rs` (`validate_h4_binary_icosahedral_closure`), `corpus_induced_spin_placement.rs` (`first_primes`).

Convention used throughout: **[REPO]** marks what the repository states; **[ASSESS]** marks my judgement; **[COMPUTED]** marks something I derived or computed for this review (appendix A). Statements about the literature drawn from my own knowledge are labelled **[MY KNOWLEDGE]**.

A general remark before the inventory. The repository's own synthesis documents are unusually candid: they repeatedly say that geometry has not been shown to help, that "ExactIdentity has the same outcomes" as the geometric path (`README.md:192`), that the initialized Hamming policy produced "0/24 authored answers ... all 1,536 Full generation steps had a maximum-score tie" (`docs/integration/geometric-attention-research-2026-09/repository-mathematics.md:24`), and that "Adding more mathematical names, coordinates, or expert labels will not by itself complete that route" (`repository-mathematics.md:9`). Much of my critique therefore agrees with the repository's own caveats; where I go further, I say so. The main additions of this review are (i) several exact facts about the implemented objects that the documents do not state (Section 1 and Appendix A), (ii) an information-capacity accounting, and (iii) an assessment of whether the program's mathematical commitments are load-bearing at all.

---

## 1. Mechanism inventory

Summary table; details follow.

| # | Mechanism | Implemented? | On a serving path? | Positive evidence of benefit? | My classification |
|---|---|---|---|---|---|
| 1 | Prime identity / ordered n-lets | Yes (`prime_route_attention.rs`) | Retained model uses prime-token ids; ADR-0003 route indexes are `RETAIN_STORAGE_RECALL_ONLY` | None beyond what any injective token id gives | Sound as identifiers; "prime" content is vacuous (it is an enumeration) |
| 2 | Zeta phases γ_j·log p | Yes (8 u16 channels compiled; 4 exposed to policies) | Present in artifacts; `ZetaDisabled` control defined; no positive result | None; no phase-vs-scrambled-frequency control has been run | Deterministic re-encoding of token rank; RH irrelevant (correctly stated by repo); no evidence |
| 3 | R4 / S3 unit quaternions, Hopf, fiber, torsion | Yes (Q1.30 types, `SpinTorsionState`) | Substrate types; Hopf sector addressing only in exploratory f64 router | Occupancy 7→43/512 sectors (#306) is not a quality result; INC0171 Hopf-MoE PPL worse | Correct mathematics; role in an LM undefined |
| 4 | H4 = binary icosahedral group 2I, 120 roots, product/inverse tables | Yes, exact and verified (`validate_h4_binary_icosahedral_closure`) | Yes: retained model, ordered-state, shared core | Negative or null: paired-H4 capacity FAIL; scrambled transport 0.033 nats *better* (ADR-0005 §"CPU recovery"); Full = ExactIdentity 2304/2304 | Sound finite group; 6.9 bits/element; used as a hash of bytes (Appendix A.2) |
| 5 | Exact Z[φ] arithmetic | Yes (`ZPhi`) | Compiler/anchor construction | n/a (construction tool) | Correct; decorative for the LM |
| 6 | Paired H4 / icosian / "E8 = H4 × H4" | Yes as Galois-companion coordinates + inverse witness | Storage/canonical identity only | `PAIRED_H4_PROMPT_CAPACITY_FAIL` | Standard lattice fact (icosians ≅ E8 as Z-module); companion is a deterministic function, zero extra capacity, as repo says |
| 7 | Hamming signatures (120 hemisphere predicates) | Yes (`metric.rs`) | Ordered-state exact-match test; Hamming policy (initialized, no fit) | Census: agrees with angular rank; no LM benefit | **[COMPUTED]** distance is a class function of the quaternion angle with exactly 9 values; adds nothing over the group element |
| 8 | Directed relative transforms r(i,j)=g_i⁻¹g_j | Proposed (tables exist) | No | None | Correct algebra; flat (pure gauge); no pairwise information beyond node states |
| 9 | SpiralCore / Cl(0,6) signed permutations, 64-state group | Yes (`spiralcore_operator.rs`) | No (`OPERATOR_SEMANTIC_STATUS = OPTIONAL_CONTROL_PENDING`) | None | Correct finite algebra; no defined action on model state |
| 10 | Vector bundles / connections / gauge transport | Dense f32 frame transport exists offline (HELM-D-R4) | No (violates serving contract) | Parity only ("not geometric advantage"); V4 held-out 13/24 all arms | Correct but a pure gauge re-description of ordinary attention |
| 11 | Tropical max-plus composition | Historical | No | Negative: top-1 .0044 vs .0620 null (#626) | Retired |
| 12 | FMM / low-rank far field | Historical certifier candidate | Removed (#425) | Negative: kernel rank ≤288 by construction | Retired; correct diagnosis |
| 13 | LUT4 Hamming policy (4096→1024→256→1796) | Yes | Experimental | Initialized only; 0/24; ties everywhere | Structural bottleneck correctly diagnosed by repo |
| 14 | Shared geometric core (H4 recurrence + byte emitter) | Yes (`shared_core.rs`) | No | "UNQUALIFIED_INITIALIZED_NO_FIT" | ~2,800 discrete parameters, zeroth-order search |

### 1.1 Prime identity and ordered n-lets

**[REPO] Definition.** `PrimeAtom(u32)` validated by trial division; `SemiprimeExpert{low,high}` sorts two atoms and computes `gcd` handoff; `OrderedPrimeRoute{ordered, factors}` keeps the sequence and a sorted multiset (`prime_route_attention.rs:180–327`). Tokens are bound to primes by `PrimeRegistry::compile`, which sorts atoms by id and assigns "the canonical sequential assignment from 5" (`prime_route_attention.rs:756–773, 832–836`). In the native geometry, `geometry(258,256)` assigns byte token `b` the `(b+3)`-th prime via a sieve (`training.rs:110–116`, `corpus_induced_spin_placement.rs:2510`).

**[REPO] Role.** "Prime atoms and ordered n-lets give records and compositions explicit structural identities" (`README.md:107`); "a commutative product is a locality and factor-overlap key, not a complete ordered identity" (ADR-0003 §2).

**[REPO] Status.** Implemented. ADR-0003's route indexes are `RETAIN_STORAGE_RECALL_ONLY`. The synthesis says explicitly: "Prime assignment is not learned semantic distance" (`repository-mathematics.md:44`) and "No prime-label metric is semantic by definition" (`mathematical-foundations.md:81`).

**[ASSESS].** The primes are an enumeration: token → n-th prime, where n is alphabetical rank (registry) or byte value (native). Every arithmetic property later derived from p (log p, p mod 120, gcd of semiprimes, divisor count) is therefore a fixed function of the rank. The only genuine property used is unique factorization, which gives a commutative, order-erasing multiset key (`pq = qp`); the repository knows this and stores the ordered vector separately. Once the order is stored separately, the multiset key is exactly a bag-of-tokens key and the product is redundant. There is no category error in the code, but the word "prime" is doing no mathematical work that an integer id would not do: gcd-based "handoff" is set intersection, "semiprime expert" is an unordered pair, `d(N)=2^k` "mostly records context length" (ADR-0003 §1, correct). The `J(6,2)` correspondence between 15 semiprimes of a sextet and 15 `Cl(0,6)` bivectors is a bijection between two 15-element sets with the same intersection graph; it is combinatorially correct and semantically empty until an action is defined (the repository says so: "The exact finite operator and the proposed semantic map remain separate", synthesis README line 100). Verdict: sound as identity plumbing; decorative as mathematics.

### 1.2 Zeta-zero phases

**[REPO] Definition.** `phase_j(p) = γ_j·log p mod 2π`, `zeta_phase_delta(channel, from, to) = γ_j (log to − log from)` (`prime_route_attention.rs:909–919`); the native compiler quantizes 8 channels to 16-bit turns relative to origin prime 2 (`training.rs:117–124`); a pinned table of 512 ordinates (`zeta_zeros.rs`). ADR-0003 motivates the form from `x^ρ = √x·e^{iγ log x}` under RH and states RH "may [be] assume[d] as a coordinate-design axiom" (ADR-0003 §Context).

**[REPO] Role.** "structured logarithmic phase observations ... not a pointwise identification of a prime with a zero, and using the finite constants does not depend on proving the classical Riemann Hypothesis" (`README.md:116`). The synthesis correctly rejects the "primes = zeros" slogan (`mathematical-foundations.md §16`), correctly states the explicit formula (§15), and correctly flags that "A phase benefit and a specifically zeta-derived benefit are distinct results" (synthesis README line 106).

**[REPO] Evidence.** None positive. `ZetaDisabled` is a defined control (`shared-geometric-core-2026-09.md:35`) but the shared core is "UNQUALIFIED_INITIALIZED_NO_FIT". No equal-cost scrambled-frequency control has been run.

**[ASSESS].** The mathematics stated is correct; the RH connection is correctly disclaimed. But the mechanism is a category error at the level of *features*: because `p` is the rank-th prime, `γ_j log p_rank` is a fixed sinusoidal encoding of token rank with 8 incommensurate frequencies — structurally a sinusoidal positional-style embedding applied to *token identity* rather than to position, using frequencies that happen to be zeta ordinates (14.13, 21.02, 25.01, ...). Since token rank is arbitrary (alphabetical or byte order), the phases carry no information beyond the 8-bit token id, and the *specific* choice of γ_j can only matter through aliasing/quantization patterns of `log p` sampled at those frequencies, which is a numerological property of the enumeration, not of language. The explicit formula (ψ(x) − x = −Σ x^ρ/ρ + ...) relates *counts of primes up to x* to the zeros; it says nothing about a single prime label. The repository's own "many-to-many spectral relation" argument (`mathematical-foundations.md:177`) would apply if the model summed phases over the primes *occurring in a context* (a sum Σ_{p in context} e^{iγ log p}), which is a windowed Chebyshev-type sum. That is exactly what `zeta_projection.rs` did (an offline projection) and what ADR-0003 §1 describes; the native path does not do that — it keeps per-token phases and (per `attention-and-learning.md:56`) commutative accumulations that "cannot distinguish AB from BA". Verdict: correct number theory attached to an arbitrary enumeration; unfounded as a language feature until a matched-frequency control shows otherwise.

### 1.3 R4 / S3 quaternions, Hopf, fiber, torsion

**[REPO] Definition.** `UnitS3Q30` (Q1.30 unit quaternion with sign retained), `hopf()` = (2(ac+bd), 2(bc−ad), a²+b²−c²−d²), `rotate_common_fiber` (the U(1) action), `SpinTorsionState{s3, hopf, fiber, torsion}` (`prime_route_attention.rs:486–675`). Architecture audit gives the exploratory Hopf chart `(ρ1,ρ2,χ,θ1,θ2,δ,α)` and transport law `α + (λ/2)cos(2χ)δ` (`architecture-2026-09/mathematics.md:104–122`).

**[REPO] Role.** "S3-to-S2 observation provides a lower-dimensional view. Retain the fiber/orientation information" (`README.md:128`). "Hopf/base neighborhoods can organize candidate pages" (`mathematical-foundations.md:55`).

**[REPO] Evidence.** #306: signed projection raised distinct-sector occupancy from 7/512 to 43/512 (`hopf_projection_remediation_306.md`), explicitly "not ... retrieval quality". INC0171 Hopf-partition MoE: PPL 164.5 vs 152.3 baseline (`engines.md:19`).

**[ASSESS].** The formulas are correct (Hopf map, U(1) fiber, `q` and `−q` collide). The repository correctly refuses the "reversible S3→S2→S1 chain". The transport law `α + (λ/2)cos(2χ)δ` is asserted, not derived; the audit says so. Nothing here is wrong, but nothing connects to language: the S3 state of a token is derived from its prime-indexed root (§1.4), so "Hopf sector" is a coarse hash of token identity. As a page-addressing scheme, Hopf sectors are one arbitrary partition of 120 points into ≤512 bins; there is no argument that this partition groups linguistically related items (it cannot, since placement is not learned). Verdict: sound geometry, decorative role.

### 1.4 H4-derived finite group state (2I)

**[REPO] Definition.** 120 scaled Z[φ] quaternion roots, closure under exact multiplication verified, identity and inverse tables (`canonical_lexical_ingestion.rs:2396–2440`); `BoundGeometry.product/inverse` (`artifact.rs:138–158`). Ordered state: two lanes of prefix products, `Right: old·leaf`, `Left: leaf·old`, at every position (`ordered_state/runtime.rs:23–59`). Byte → root: `leaf = prime % 120` (`training.rs:127–131`).

**[REPO] Role.** "Ordered left/right prefix products retain sequence information that a final product alone loses" (`README.md:125`). "A single 120-state root carries fewer than seven bits of state" (`architecture-2026-09/README.md:57`). "Four roots have at most log₂(120⁴) ≈ 27.63 bits" (`repository-mathematics.md:34`).

**[REPO] Evidence.** ADR-0005 CPU-recovery: "Scrambled transport was 0.033049 nats better, so no H4-specific advantage follows." Paired-H4 prompt capacity: FAIL (282/512 wins vs required 308). Independent-neighbor panel: "Full and ExactIdentity agree on 2,304/2,304, including failures" (`README.md:191`). A1R/#967: "shortest Cayley distance collapsed both candidates to energy 2 and tied on 6/6" (ADR-0003 §"#952 A1.0").

**[ASSESS].** The group is real and exactly implemented (I rebuilt 2I independently and confirmed order 120 and closure; Appendix A.1). Three facts the documents do not state:

1. **[COMPUTED]** `leaf = prime % 120` maps the 256 byte tokens onto only **33** of the 120 roots (primes > 5 are units mod 120, of which there are φ(120)=32, plus 2, 3, 5 ⇒ 35 leaves for 258 tokens, 33 for bytes). Up to 11 bytes share a root. So the "geometric placement" of a byte is a 5-bit hash (log₂ 33 ≈ 5.04) into an arbitrarily ordered table (roots are sorted by Z[φ] coordinate tuple; the index has no geometric meaning). Nothing in the docs' capacity accounting reflects this; "120 roots" overstates the reachable alphabet by ~3.6×.
2. The two-lane prefix fold is a deterministic function of the byte sequence, and the `Query` already stores the exact occurrences (`occurrences: Vec<u32>`, `ordered_state/runtime.rs:31`). The prefix products therefore carry **zero** information beyond the bytes; the "distance" of `ordered_state::distance` is exactly 0 iff the two sequences have identical *leaf* sequences (a 33-way-hashed byte string), and the router only accepts distance 0 (`runtime.rs:165–179`). "Geometric contextual read" in the active experimental core is exact-match retrieval over four records keyed by a lossy hash of the key bytes. This is why Full = ExactIdentity on every case.
3. The 120-element group is a finite automaton alphabet; prefix products are its transition monoid acting on itself. The information retained at position t about the prefix is at most log₂120 per lane (6.9 bits), i.e., the current state of a 120-state DFA. The docs' "order sensitivity" claim is true (noncommutative) but weak: a 120-state automaton distinguishes prefixes only up to its 120 classes. Retaining *all* prefix states (as `Query.prefixes` does) is retaining the input.

Verdict: sound algebra; capacity 6.9 bits/element; its use in the code is as a hash, not as geometry; no evidence of benefit; several matched controls (scrambled transport, ExactIdentity) show equal or better performance.

### 1.5 Exact Z[φ]

**[REPO].** `ZPhi{a,b}` with checked add/sub/mul using φ²=φ+1, Galois conjugation φ→1−φ, `times_phi: (a,b)→(b,a+b)` (`prime_route_attention.rs:379–481`). Used to build the 120 roots exactly and to state a "golden radial bridge" `R_φ(z)=φ·E(z)` (ADR-0003 §5).

**[ASSESS].** Correct. Necessary for an exact icosian table (the roots have coordinates in ½Z[φ]). The "golden radial bridge" is multiplication of a vector by a scalar φ; "preserves direction" is trivially true of any positive scalar. The "sqrt(2), 2i, [0,2]" landmark discussion (ADR-0003, ADR-0004, synthesis §11) is a correct but elementary exercise in chord vs geodesic distance and a typed-adapter contract; it contains no theorem and no mechanism. Verdict: sound tool, decorative as architecture.

### 1.6 Paired H4 / icosian / "E8 = H4 × H4"

**[REPO].** "conceptual identity `E8 = H4 × H4` ... realized through ... `Λ_E8 ≅_Z I` (E8 lattice and icosian ring as Z-modules); `B_ico(I) = (x, x') ∈ R4 ⊕ R4`; declared 600-cell folding `Φ_E8 = H4 ⊕ φH4`" (ADR-0004 §4; `formal_vocabulary.md` row `B_ico`). The companion `x'` is the Galois conjugate. The synthesis: "The companion is derived, not an independent second learned state ... It does not multiply independent semantic capacity" (`mathematical-foundations.md:51`). Cites Dechant.

**[ASSESS].** The lattice statement is standard **[MY KNOWLEDGE]**: the icosian ring (the Z[φ]-span of 2I) is a rank-8 Z-module, and with the quadratic form `Tr_{Q(√5)/Q}(|q|²)` (equivalently the reduced/"folded" inner product) it is isometric to E8 (Conway–Sloane SPLAG §8.2; Moody–Patera folding; Dechant's Clifford construction). So "E8 = H4 × H4" is a defensible shorthand for "E8 is the 8-dimensional Z-module underlying the icosians, seen as two Galois-conjugate R4 copies". The repository handles this correctly and explicitly denies extra capacity. The `PAIRED_H4_PROMPT_CAPACITY_FAIL` result is consistent with this: a deterministic companion cannot add information. Verdict: correct mathematics, correctly scoped, no computational role beyond exact identity/witness.

### 1.7 Hamming signatures

**[REPO].** For root `r` and landmark `l`, bit set iff rank(Re(r·l⁻¹)) > 0, i.e. "120 fixed H4 landmark predicates" (`artifact.rs:101–109`; `mathematical-foundations.md:17`). `root_distance` = XOR/popcount; two-lane `distance` = sum (`metric.rs:57–73`). Census: 120 unique signatures, all of weight 45, "no single-root ranking reversals relative to the current angular-rank table" (`mathematical-foundations.md:19`). The synthesis argues the full signature "retains more information than that count" and that Charikar/Plan–Vershynin bounds do not apply to a deterministic bank (`mathematical-foundations.md §1`).

**[COMPUTED] (Appendix A.1).** Because Re(ab)=Re(ba) for quaternions and the landmark set is the whole group, the signature Hamming distance is bi-invariant: d(r,s) = d(e, r⁻¹s) and d is a class function on 2I. 2I has exactly 9 conjugacy classes, indexed by Re(g) ∈ {1, φ/2, 1/2, 1/(2φ), 0, −1/(2φ), −1/2, −φ/2, −1}. I recomputed the signatures and found the Hamming distance takes exactly the 9 values {0, 22, 38, 44, 58, 66, 76, 88, 90} at angles {0°, 36°, 60°, 72°, 90°, 108°, 120°, 144°, 180°}, monotone in angle. Weight 45 = #{g ∈ 2I : Re g > 0} = 1+20+12+12.

**[ASSESS].** Consequences: (i) the census result "no ranking reversals relative to angular rank" is not an empirical finding about a codebook; it is a theorem — the Hamming distance *is* a quantized angle. (ii) The 120-bit signature is a 120-way code for a 120-element set: it "retains" exactly the root id (6.9 bits), not more. (iii) A 120×120 table of angle classes (or the product table already present) computes the same quantity with no XOR/popcount. (iv) Charikar-style random-hyperplane arguments are irrelevant here (the repo says so); the relevant object is the bi-invariant metric on a finite group, which has 9 levels. The synthesis's counterexample about lane ties (`(0,a)` vs `(a,0)`) is correct and generalizes: the two-lane sum is a coarse 17-level statistic. Verdict: correct implementation of a metric that is provably a re-encoding of the group angle; no information-theoretic advantage exists to be found.

### 1.8 Directed relative transformations r(i,j)=g_i⁻¹g_j

**[REPO].** "r(i,j)·r(j,k)=r(i,k) ... invariant under a common left frame change ... These relative-frame edges are a flat construction: their products telescope around loops. They must not be relabeled as nontrivial curvature or holonomy" (synthesis README lines 54–60; `README.md:130`).

**[ASSESS].** Entirely correct, and the repository draws the right conclusion. In gauge-theory language, an edge connection of the form g_i⁻¹g_j is pure gauge: its holonomy around every loop is the identity, so it is gauge-equivalent to the trivial connection. Storing all N(N−1)/2 of them is storing N node states redundantly (the synthesis says exactly this, `repository-mathematics.md:128`). What learned attention provides that this construction cannot is a *non-factorizable* pairwise interaction: softmax(q_iᵀW k_j) is a bilinear form whose matrix W is not determined by node states. A finite-group analogue would be a learned function G×G→scores that is *not* a class function of g_i⁻¹g_j; the repository has not proposed one. Verdict: sound, and correctly identified as not load-bearing.

### 1.9 SpiralCore / Clifford Cl(0,6)

**[REPO].** Exact oriented Fano octonion basis products, associator, signed 8×8 permutation matrices, six left/right generators, 15 bivectors, 64-element finite group with composition/inverse tables; `CHART_TRANSPORT_STATUS = "NOT_ESTABLISHED"`, `OPERATOR_SEMANTIC_STATUS = "OPTIONAL_CONTROL_PENDING"` (`spiralcore_operator.rs:1–53`). The synthesis notes the legacy `cayley_dickson.rs` and 16-d `endomorphism.rs` are *not* faithful implementations (cyclic surrogates; identity spectator block breaks anticommutation) (`mathematical-foundations.md §8`, `repository-mathematics.md:92–98`).

**[ASSESS].** The v63 adapter is correct finite algebra (the 64-state group generated by six anticommuting signed permutations with L_i² = −1 is the "Clifford group" of Cl(0,6) modulo signs; order 2·2⁵ = 64 is right for the group generated by the L_i up to the ±1 quotient). The repository's audit of the two legacy modules is a genuine finding: naming a function `clifford_generator` while embedding an identity block on half the space breaks γ_iγ_j+γ_jγ_i = 0 there. There is no defined action of Cl(0,6) on the model's H4 state, so the mechanism has no computational role. The docs say so. Verdict: sound algebra, no role.

### 1.10 Vector bundles, connections, gauge covariance

**[REPO].** HELM-D-R4 encodes each R4 block locally by Fᵢᵀ and transports by Pⱼ→ᵢ = FᵢᵀFⱼ before ordinary Q·K softmax; result "PASS ... PARITY", explicitly "Parity establishes only that the exact R4/Spin gauge representation can carry the donor's ordinary attention function. It is not geometric predictive advantage" (ADR-0005). `ConnectionGaugeCovarianceV4`: held-out 13/24 for H4-frame, alternative-frame and plain arms alike. Intrinsic Lorentz/R4-distance attention: `UNAVAILABLE`/`FAIL`. The synthesis correctly distinguishes principal S1 bundle from associated vector bundles (`architecture-2026-09/mathematics.md:132–144`; `imports.md:182`).

**[ASSESS].** Everything stated is correct. FᵢᵀFⱼ with orthogonal Fᵢ is a change of basis; attention computed in transported coordinates equals attention computed in the model basis, so parity is a tautology (the repo says "expected positive is numerical ... parity"). The non-trivial experiments (learned-manifold, intrinsic distance) failed to match Euclidean controls. Verdict: correct gauge bookkeeping with no predictive content; the only components that "pass" are ordinary transformer attention with matmul, which the serving contract forbids.

### 1.11 Tropical composition, FMM, W33 mod-9, Hopf MoE (historical)

**[REPO].** Tropical max-plus route composition #626: top-1 .0044 vs .0620 null (`engines.md:67`). FMM #290: "the interaction kernel is exactly rank ≤ 288 by construction, so there is no O(n²) for an FMM to remove" (`fmm_290_novel_context_protocol.md:7–11`); the certifier candidate improved top-8 from 0.052 to 0.260 but top-1 stayed 0.0104 on a 96-position fixture. W33 mod-9 mapping: all correctness cells fail; permuted controls match or exceed (`imports.md:112`). INC0171: Hopf-partition MoE PPL 164.5 vs baseline 152.3.

**[ASSESS].** These are correctly recorded negatives. The FMM diagnosis (Eckart–Young bound; low intrinsic rank) is right. Nothing to add except that the repository's discipline in preserving negatives is a strength.

### 1.12 The LUT4 Hamming policy and the shared core (the actual "learners")

**[REPO].** Policy: 4,096 input bits → 1,024 → 256 → 1,796 output bits, 3,076 LUT4 gates, 49,216 truth-table bits, randomly seeded (`hamming_policy/policy.rs:11–15, 47–53`). Derived: "one final output bit sees at most 4³ = 64 of the 4,096 input ports" and "Every output is a function of the same 256-bit intermediate representation" (`attention-and-learning.md:91–96`). Credit: finite parameter contrasts, "not ... a derivative" (`repository-mathematics.md:36`). Shared core: parameters are root indices, families EMBED(4×256)+TRANSITION(4×120)+QUERY(120)+KEY(120)+READ(4×120)+PHASE(64)+OUTPUT(512)+OUTPUT_MIX(9)+NULL(1) (`shared_core.rs:39–49`), fitted by random single-coordinate proposals accepted on full-forward NLL, ≤100,000 proposals (`shared_core/training.rs:164–230`).

**[COMPUTED].** Shared-core parameter count = 1024+480+120+120+480+64+512+9+1 = **2,810** parameters, each in {0..119}: ≈ 2,810 × 6.9 ≈ 19.4 kbit ≈ **2.4 KB** of learnable state. LUT4 policy: 49,216 bits ≈ **6 KB**.

**[ASSESS].** These are the only components that could be called learners on the geometric path, and both are tiny discrete circuits trained by zeroth-order search. The docs' receptive-field analysis is correct and the conclusion ("should not be presumed to be the eventual learner") is right. More fundamentally: a hill-climb over 2,810 categorical parameters on a hard forward path has no gradient signal; the expected number of accepted proposals per useful bit learned scales with the number of categories (120) times the number of parameters, and the docs' own record shows the "failed stochastic per-invocation recipe". No known result suggests a 2–6 KB discrete controller trained this way approaches byte-level language modeling.

---

## 2. Cited literature and what the synthesis takes from it

All entries below are from `attention-sources.json` (A01–A25), `math-sources.json` (M01–M36), the four `.md` syntheses, and ADR-0005's "Research basis". I state what is taken and whether the reading is accurate **[MY KNOWLEDGE]**. I found no material misreadings; a few items deserve nuance (marked ⚠).

**Attention and sequence models (A01–A25)**
- A01 Vaswani et al., *Attention Is All You Need*, 2017 — taken: decomposition into learned compatibility, normalization, value mixing across depth. Accurate.
- A02 Ramsauer et al., *Hopfield Networks Is All You Need*, 2020 — taken: softmax attention as associative energy update; caveat that convergence results don't transfer to hard finite transitions. Accurate.
- A03 Bricken & Pehlevan, *Attention Approximates Sparse Distributed Memory*, 2021 — taken: Hamming-neighborhood overlaps approximate exp-compatibility under random-pattern assumptions. Accurate; the caveat that a deterministic 120-codebook is not covered is correct (and, per §1.7, the codebook metric has only 9 levels, far from SDM's regime).
- A04/M07 Charikar, *Similarity Estimation Techniques from Rounding Algorithms*, 2002 — taken: P[sign disagreement] = θ/π for random hyperplanes. Accurate; correctly not applied to the deterministic bank.
- M08 Plan & Vershynin, *Dimension reduction by random hyperplane tessellations*, 2011 — taken: uniform additive distortion of normalized geodesic distance. Accurate, including the note that the theorem uses geodesic, not Euclidean, distance.
- A05/A25 Fuchs et al., *SE(3)-Transformers*, 2020 — taken: invariant scores vs equivariant values. Accurate.
- A06 Brehmer et al., *Geometric Algebra Transformer*, 2023 — taken: typed multivector representations; still dense. Accurate.
- A07 Loshchilov et al., *nGPT*, 2024 — hypersphere normalization; still dense. Accurate.
- A08 *HELM: Hyperbolic LLMs via Mixture-of-Curvature Experts*, 2025 — dense Lorentz attention; ⚠ ADR-0005 notes the released code computes a Lorentz inner-product surrogate, not arcosh² distance; that is a careful source-level reading, not a misreading.
- A09 Gu & Dao, *Mamba*, 2023; A10 RWKV Eagle/Finch, 2024; A11 RetNet, 2023; A12 GLA, 2023; A13 Gated DeltaNet, 2024; A15 Reformer, 2020; A24 RWKV-4 source — taken: lessons about selectivity/forgetting/recurrence; all excluded for containing matrix products. Accurate.
- A14 Zhu et al., *Scalable MatMul-free Language Modeling*, 2024 — taken: ternary accumulation is still a mathematical matrix product. Accurate and a fair, if strict, reading.
- A16 Petersen et al., *Deep Differentiable Logic Gate Networks*, 2022; A17 *Convolutional Differentiable Logic Gate Networks*, 2024 — taken: relaxation/discretization, structured connectivity, residual init. Accurate; correctly noted as classification-only evidence.
- A18 van den Oord et al., *VQ-VAE*, 2017; A19 Jang et al., *Gumbel-Softmax*, 2016 — straight-through estimators as licensed approximations. Accurate.
- A20 *VL-JEPA*, 2025; A21 Baevski et al., *data2vec*, 2022 — latent-target prediction as auxiliary objective; components excluded. Accurate; the note that data2vec shows language *understanding*, not generation, is fair.
- A22 Kleyko et al., *VSA as a Computing Framework*, 2022; Kanerva 2009 *Hyperdimensional computing* — binding/permutation/cleanup and capacity limits. Accurate.
- A23 Graves et al., *Neural Turing Machines*, 2014 — learned content/location addressing. Accurate.

**Number theory and zeta (M01–M06, M18–M20, M23, M26–M30, M33–M36)**
- Clay problem statement; DLMF §§25.2, 25.4, 25.10, 25.16, 27.2, 27.4 — used for definitions (pole at s=1 only, ξ completion, Hardy Z, divisor and von Mangoldt series). Accurate.
- M04 Bombieri, *The Riemann Hypothesis* (Clay) — explicit formula with test functions. Accurate.
- M28 Goldston, *Notes on pair correlation of zeros and prime numbers* — explicit formula, HL prime-pair conjecture, Montgomery-type equivalence under RH. Accurate.
- M27 Conrey & Keating, *Pair correlation and twin primes revisited*, 2016 — conditional ratios/correlation results. Accurate; the synthesis correctly refuses to promote them to a pointwise prime–zero identity.
- M33 Bondarenko–Radchenko–Seip, *Fourier interpolation with zeros of zeta and L-functions*, 2020 — Guinand–Weil duality with explicit convention. Accurate.
- M36 Dixit–Maji–Vatwani, Voronoi summation for divisor functions — divisor-sum transform distinct from zeros. Accurate.
- M18 Tao, *Heuristic limitations of the circle method* (blog, 2012) — losing oscillation destroys estimates; used only as an analogy. Accurate and appropriately hedged.
- M19/M20 Hardy–Littlewood 1921 / 1923 — bibliographic only; the synthesis correctly separates three different "Hardy–Littlewood" topics.
- M29 Guinand 1948 — record only. M30 Bombieri–Lagarias (Li's criterion) — access failed; nothing relied on.

**Geometry, harmonics, codes (M09–M17, M21–M22, M24–M25, M31–M32)**
- M09 Cohen et al., *Gauge Equivariant CNNs*, 2019; M11 Crane, DDG notes; M10 Singer & Wu, *Vector Diffusion Maps*, 2011 — frame alignment vs scalar affinity, holonomy. Accurate; correctly concluded that node-derived edges are flat.
- M12 DLMF §14.30; M13 Zhao & Song, *Exact heat kernel on a hypersphere*, 2017 — S³ heat kernel K_t = (1/2π²)Σ(l+1)e^{−l(l+2)t}C_l¹(x·y). **[MY KNOWLEDGE]** Correct (eigenvalues l(l+2), zonal Gegenbauer C_l¹, vol(S³)=2π²).
- M14 Dechant, *The E8 geometry from a Clifford perspective*, 2016 — 240 = two copies of 120 H4 roots with a reduced inner product. Accurate. M15/M16 Baez, *The Octonions* and *Integral Octonions* — Cayley–Dickson properties, zero divisors at sedenions. Accurate.
- M17 Furey, *Standard Model symmetries and nested embeddings R⊂C⊂H⊂O*, 2026 — used to show `cayley_dickson.rs` does not implement the cited algebra. The audit is a valid source-level finding.
- M21 Musin, *Multivariate positive definite functions on spheres*, 2007 — multi-reference kernels. Accurate; correctly not over-claimed. M22 Delsarte–Goethals–Seidel — record only.
- M24 Baez, *Klein quartic*; M25 designtheory.org — used to correct NEMESIS's "PGL(2,7)" for the Fano plane collineation group to PGL(3,2)≅PSL(2,7) of order 168 (|PGL(2,7)|=336). **[MY KNOWLEDGE]** Correct.
- M31 Conway & Sloane, *Voronoi regions of lattices*, 1982; M32 Kurkoski, *E8 lattice and error correction in flash*, 2010 — nearest-codeword radius √2/2 for standard E8. Accurate.

**ADR-0005 research basis** — HELM (2505.24722), Gated DeltaNet (2412.06464), RetNet (2307.08621), Mamba-2/SSD (2405.21060), Zoology (2312.04927), "From Self-Attention to Connection Laplacian" (2607.10677), RiemannFormer (2506.07405), Bronstein et al. *Geometric Deep Learning* (2104.13478), Katharopoulos et al. *Transformers are RNNs* (2006.16236), Choromanski et al. *Performers* (2009.14794), Shalizi & Crutchfield *Computational mechanics* (cond-mat/9907176), MatMul-free LM (2406.02528). The ADR states "No cited work establishes a geometry-native, transformer-free, causal local language model with the UOR runtime contract." Accurate.

**Local/contributor sources** (NEMESIS, W33, SpiralCore v63/v68, FBS, owner PDFs, GoldSnnail) — the syntheses record specific arithmetic errors in these (Hamming distances mis-tabulated in a NEMESIS PDF; "9 is a unit mod 256" ≠ primality; Fano group order; Z/256Z zero divisors; `||q·conj(k)|| = ||q||·||k||` making a quaternion attention score orientation-blind; `tanh(v)/|v|·tanh(|v|)` mislabeled). **[ASSESS]** Each of these corrections that I checked is right. The two owner RH manuscripts are shown to have a non-sequitur (fixed boundary phase ⇒ zero normal derivative) and a non-discriminating torque formula (−2a/(1/4+a²) ≠ 0 for a ≠ 0 regardless of δ); both derivations are correct.

⚠ One flag on framing rather than misreading: `attention-and-learning.md:36–38` and `mathematical-foundations.md:19` treat the census "no ranking reversals" as evidence about a codebook that "does not establish ... angle reconstruction". In fact (§1.7) it is a theorem that this distance *is* the angle class; the synthesis under-states what is known and thereby leaves open a "richer Hamming" research direction that is closed.

---

## 3. The "role separability" obstruction

**[REPO] Statement.** "If two occurrences have the same observation O(x) = O(y) but require different roles, no deterministic classifier using only that observation can assign both intended roles. Adding more training to the unchanged observation does not restore the missing information" (`README.md:136`). Concretely: the source-role key is a triple (center word id, left neighbour id, right neighbour id) with `64` = unknown neighbour; "The source role key for its interior will is (center 13, left 64, right 64) ... The corresponding expected query gives the same key a learned content role ... A deterministic assignment to this one key cannot retain both intended labels" (`native_geometric_independent_neighbor_973.md:28–32`). A retained auxiliary use ("navor will trust tavin") has the same key with the opposite role.

**Precise form.** Let O: X → K be the observation (feature) map and ℓ: X → L the intended label. For any function f: K → L, the error set contains every x such that ∃y with O(x)=O(y), ℓ(x)≠ℓ(y) and f(O(x))≠ℓ(x). Hence min_f Pr[f(O(X)) ≠ ℓ(X)] ≥ Σ_k [P(k) − max_l P(ℓ=l | O=k)], the Bayes error of the funnel. Equivalently, H(ℓ | O) > 0 whenever a key is shared by two labels with positive probability. This is the data-processing inequality applied to the hand-built key; it is trivially correct.

**[ASSESS] Is it information-theoretic about the architecture?** Only about *their* funnel. The obstruction is a property of the chosen feature map (three exact word ids with an OOV sentinel) — not of the data, not of finite geometry in general, and not of deterministic classifiers in general. The full context in the failing example ("today izpkgzr will call zlrkawr will ikbjwdx tomorrow.") *does* separate the two uses of "will": the first "will" is followed by a known verb "call", the second sits between two unknown seven-letter tokens after that verb; sentence position, distance from "call", and the count of prior "will"s all distinguish them. The information exists in X; the map O throws it away. So the correct reading is: **the feature funnel is insufficient**, which the repository also says ("a collision in the finite role observation, not in canonical identity"). It is not evidence that finite/hard classification is inadequate; a wider or hierarchical key (e.g., (center, left, right, left-left, position-in-clause, verb-relative offset)) would separate this case — at the cost of combinatorial key growth, which is the real issue with exact-key tables: every new distinction multiplies the key space and shrinks per-key training counts.

**What standard ML does.** (a) Learns O rather than declaring it: a contextual encoder (even a small causal RNN/CNN over characters) produces a representation of "will" that depends on the whole prefix, so aliasing at the identifier level is not fatal; (b) uses *soft*, *distributed* features so that unseen names ("zlrkawr") still contribute shape/position information instead of a single `unknown=64` sentinel; (c) makes the role decision probabilistic and lets downstream decoding marginalize or defer, rather than requiring a deterministic assignment before the first read; (d) trains the feature extractor and the decision jointly against the output loss, so the extractor is pushed to keep exactly the distinctions the task needs. The repository's own H1 proposal ("structured shared operator ... cross-field blocks") is a step toward (a) and (d), but it retains hard reads, exact keys and no gradient, so (b) and (c) remain unaddressed. The deeper problem is that the repository's architecture *requires* a deterministic role assignment at a hand-designed interface before any read is permitted ("A better ranking cannot recover a correct source that was never admitted", `README.md:75`); that design choice, not finite arithmetic, is what makes an unavoidable, ubiquitous phenomenon (lexical ambiguity of "will") into a hard failure of 288/288 cases.

---

## 4. Information capacity

All numbers **[COMPUTED]** from repository constants unless attributed.

**Per-step geometric state.**
- One 2I element: log₂120 = 6.907 bits.
- Two lanes (ordered-state Right/Left prefixes): 13.8 bits. Four slots (shared core `LANES=4`): 27.6 bits (the repo's 27.63, `repository-mathematics.md:34`, is correct).
- Hamming signature: 120 bits stored, 6.9 bits of information (bijective with roots; §1.7). Two-lane Hamming distance: ≤17 distinct values (≈4 bits).
- Byte leaf via `prime % 120`: 33 reachable roots ⇒ 5.04 bits per byte, i.e. ~3 bits of each byte are discarded before any geometry acts (Appendix A.2).
- Zeta phases: 8 × 16 = 128 bits stored per token, 8 bits of information (deterministic function of byte id). With 512 channels: 8,192 bits stored, still 8 bits of information.
- Ordered-state `Query` for a 128-byte prefix: 128 × (2×16 + 32) = 8,192 bits stored; information content = the 1,024 bits of the bytes (the prefixes are a function of them; the leaf sequence is a lossy 645-bit image).
- Hamming-policy intermediate layer: 256 bits; each output bit a function of ≤64 input bits per call.
- Recurrent control table: 2,048 rows × {Emit, Read, Stop} ≈ 3.2 kbit of policy.

**Learnable parameters.**
- Shared core: 2,810 root-valued parameters ≈ 19.4 kbit (2.4 KB).
- LUT4 policy: 49,216 bits (6 KB).
- Retained native artifact d590: 11.3 MB JSON; "131,373 learned associations", "611 token geometry entries" (`architecture-2026-09/README.md:51`) — mostly count tables, i.e. exact memory rather than a generalizing parameterization.
- ADR-0005's positive results come from dense models: `R4RetainedLanguagePathV1` 252,160 f32 parameters (≈8 Mbit) and the #1017 7.15M-parameter transformer-compatible model (≈229 Mbit) — both use matmul and are excluded from serving.

**Comparison.** A GPT-2-small residual stream is 768 × 16 bits ≈ 12.3 kbit *per position*, ≈ 12.6 Mbit over a 1,024-token context; parameters 124M × 16 ≈ 2 Gbit. A 1–3B-parameter local model (the smallest class anyone would call "frontier-adjacent" on an M1) holds 16–48 Gbit of parameters. The native geometric state is 3–4 orders of magnitude smaller per position than a small transformer's residual stream, and the learnable geometric controller is 5–6 orders of magnitude smaller than the smallest useful LMs. Exact memory can hold the *text*, but the *conditional distribution* P(next byte | context) must be represented somewhere; in the native design it lives in count tables keyed by exact/hashed contexts plus a ~2–6 KB decision circuit.

**What theory says about "finite geometric state + exact memory" [MY KNOWLEDGE].**
- A fixed finite controller with no external memory is a DFA: it recognizes exactly the regular languages (Kleene). The 120⁴-state recurrence is a DFA with ≤ 2×10⁸ states. That is not the binding constraint; a DFA with a 6-bit alphabet and 10⁸ states can track many things. Adding an unbounded random-access exact memory (as UOR-R4 does) yields, in principle, a RAM machine — Turing-complete. So *expressivity* is not the obstacle. Hard-attention transformers, by contrast, are provably limited: unique-hard-attention transformers recognize only languages in AC⁰ (Hao, Angluin & Frank 2022), saturated/softmax transformers are in TC⁰ (Merrill & Sabharwal 2023; survey Strobl et al. 2024) and cannot compute PARITY under standard assumptions, whereas finite-state recurrences can. So the repository's premise that finite recurrence + memory is not *formally* weaker than transformers is defensible.
- The obstacle is *statistical*: language modelling is density estimation, not language recognition. Shannon's estimate (1951) is ~1 bit/character for English; modern LMs reach ≈0.6–0.8 bits/byte, requiring a predictor that distinguishes on the order of 10⁹–10¹¹ context classes with calibrated probabilities. The classical exact-memory-plus-small-controller approach is context-mixing compression (PPM, CTW, PAQ/cmix): those achieve ≈1.1–1.2 bits/char on enwik8 with hundreds of MB of context statistics and *learned logistic mixing with multiplications*; they do not generate coherent long text, do not reason, and their state is the corpus statistics, not a 2 KB circuit. The repository's `source_free_table.rs` (22.26% top-1 held-out vs 5.41% unigram, `engines.md:77`) is exactly this family.
- Memory-augmented neural models (NTM/DNC, Graves et al. 2014/2016; kNN-LM, Khandelwal et al. 2020; RETRO, Borgeaud et al. 2022; Memorizing Transformers, Wu et al. 2022) show that exact retrieval helps, but in every case on top of a dense controller of 10⁷–10¹⁰ parameters. There is no published result in which a controller of ≪1 Mbit plus exact memory produces general prose or reasoning. The repository's own strongest results are consistent with this: everything that "passed" (#1014, #1017, `R4RetainedLanguagePathV1`) is a dense matmul model.
- Learnability: with no gradient (hard forward path, integer tables), the repository relies on finite-difference contrasts and coordinate hill-climbing. For a discrete parameter space of size 120^2810 the sample and query complexity of zeroth-order search is prohibitive; the difflogic literature the repo cites trains *continuous relaxations* with gradients and then discretizes. The synthesis's H1 proposes "a hard-forward surrogate with an explicitly declared backward approximation" — that is the right direction, but the serving contract permits offline matmul, so there is no reason to avoid a fully differentiable surrogate; the residual risk is the train/serve discretization gap, which is large for 120-way categorical parameters.

**Verdict on plausibility.** Exact memory + finite geometric state is a plausible route to a *retrieval/copy machine with bounded programmatic operations* (what the retained model already is: copy, add, dependent lookup over 4 records). It is not a plausible route to general language at the observed parameter scale: the conditional-distribution capacity (a few KB of learnable circuit) is 5–6 orders of magnitude below what is known to be necessary, the placement of tokens into the 120-element geometry is an unlearned 5-bit hash, and the only evidence of learned language on this codebase comes from models that violate the serving contract.

---

## 5. Alternatives proposed by the synthesis

**H1 — Structured geometric recurrent learner with relation-preserving reads** (`attention-and-learning.md §H1`, synthesis README). Replace the random LUT4 funnel with "deliberately connected shared nonlinear blocks", explicit carry paths, cross-field blocks; train by hard-forward surrogate; first task: one contextual relation → one byte + EOS. **[ASSESS]** Sensible engineering response to the receptive-field bottleneck, and the staged acceptance (learn → compose → prose → replace) is good methodology. But H1 keeps every capacity constraint of §4: state in 2I⁴, 5-bit token placement, hard reads. The first gate (copy one byte from a selected record) is a task that the existing exact-identity machinery already solves without learning, so passing it will not discriminate geometry from hashing; the synthesis anticipates this ("if learned recurrence legitimately retains the answer, a weak read ablation limits the explicit-attention claim"). The decisive missing element is a *learned placement* of tokens into the group (currently `prime % 120`) and a *learned, non-class-function* compatibility on G×G; without these, "relation-preserving reads" preserve only relations the hash happens to induce.

**H2 — Learned bounded traversal over a canonical relation graph** (typed occurrence/result nodes, ordered edges with r(i,j), causally recomputed queries). **[ASSESS]** This is essentially a learned graph-walk / pointer-network memory (NTM location addressing; graph neural memory). It is coherent and the identity/lineage discipline is a genuine strength of the codebase. Two risks the synthesis names are the right ones: the graph must be induced from text (hard), and the edge relations r(i,j) carry no information beyond node states (§1.8), so the "relations" that matter must be learned edge types, which reintroduces a learned pairwise scorer — i.e. attention. H2 without a learned pairwise function is a symbolic database; with one, it is attention over a graph. The all-pairs 256-window comparator (255 KiB) is cheap and worthwhile as a *diagnostic*, exactly as proposed.

**H3 — JEPA-like predictive latent objective on top of an autoregressive native model.** **[ASSESS]** Reasonable only after H1 learns anything; the synthesis says so. Collapse controls (EMA target, stop-gradient) require a continuous latent and gradients — available offline. The latent would need to live in something richer than 2I⁴ (27.6 bits) to be a useful prediction target; predicting a 120-way categorical from a 120-way categorical is a small classification problem, not a representation-learning objective.

**All-pairs relations.** **[ASSESS]** Mathematically redundant within a common frame (repo agrees); useful as an audit of what the bounded reader discards; not a mechanism.

**Chart bridge / typed adapter contracts (sqrt2, 2i, [0,2]; E_target∘T_source = T_target∘E_target).** **[ASSESS]** Correct as engineering hygiene (state metric, orientation, quantization error, commuting diagram). Contains no mechanism and should not be scheduled ahead of learning.

**OSPF analogy.** **[ASSESS]** Correctly demoted by the synthesis: a shortest path on a known graph presupposes learned edge costs, which is the missing part.

---

## 6. Verdict

**Load-bearing and sound.**
1. Exact identity, content addressing, versioned exact memory, causal commit, replayable witnesses, preserved negatives. This is the repository's real asset; it is engineering, not geometry.
2. The 120-element binary icosahedral group with exact Z[φ] tables — a correct, verified finite noncommutative monoid usable as a bounded recurrent state (6.9 bits/element). Sound as a component; currently used as a hash.
3. The n-gram / exact-prefix candidate tables (I1/I2/IS; `source_free_table.rs`) — the only source-free language signal (22.3% vs 5.4% unigram). Standard, honest, limited.
4. The repository's self-diagnoses: r(i,j) telescopes; companion coordinates add no capacity; LUT4 has a 64-input cone and 256-bit bottleneck; scalar Hamming discards lane identity; ExactIdentity = Full; scrambled transport ≥ H4. All correct.

**Decorative (correct mathematics with no computational role).**
- Zeta-zero phases (a sinusoidal encoding of token *rank*; RH correctly irrelevant, but so is the choice of γ_j).
- Hopf/fiber/torsion bookkeeping; "sqrt2 / 2i / [0,2]" landmarks; golden radial bridge; `0⁰` typed bridge; sin²/sign(sin) "activation/chirality".
- "E8 = H4 × H4" (true as a lattice statement; zero extra state).
- SpiralCore Cl(0,6) tables (correct; no action defined).
- Hamming signatures (provably a 9-level quantization of the quaternion angle; nothing beyond the root id).
- Gauge/connection transport of ordinary attention (parity by construction).

**Unfounded or retired.**
- Any claim that prime/zeta/H4 coordinates supply semantic proximity (the repo makes none, but the architecture is organized as if they will).
- Semiprime "experts" (unordered pairs, no parameters).
- Tropical composition, FMM far field, W33 mod-9, Hopf-partition MoE, Cayley–Dickson prototype, 16-d Clifford prototype — correctly recorded negatives or non-implementations.
- The implicit premise that a ≤10 KB discrete controller plus exact memory, trained by hill-climbing on a hard forward path, can reach general prose. No theory or precedent supports it; the repo's own dense-model results argue against it.

**Category errors to name plainly.**
- "Prime identity" is hashing by enumeration; "geometric placement" (`prime % 120`) is a 33-bucket hash; "Hamming relevance" is a 9-level angle table; "geometric contextual read" in the active core is exact match on hashed keys. None of these is wrong code; all are mislabelled as geometry.
- "Zeta phases as features vs RH": the docs handle RH correctly, but the residual claim — that zero ordinates are a *better* fixed frequency basis than any other — is untested and has no mechanism behind it.
- Finite-group capacity: quoted as "120 roots"/"27.63 bits" but the byte→root map reaches 33 roots (5.04 bits) — the docs never state this.

**Mathematics the project is missing and would need.**
1. **A learned embedding.** Token placement in the geometry must be a trained map (byte/word → state), not `prime % 120`. Everything downstream assumes geometrically close states are related; nothing makes it so.
2. **A learned, non-factorizable pairwise compatibility** f(g_i, g_j) that is *not* a class function of g_i⁻¹g_j — i.e. the finite-group analogue of a bilinear Q·K form. Without it, "relations" are the hash's coincidences. (If one insists on lookup-only serving, a 120×120 learned table is 14,400 entries and is legal under the contract; the repo has not tried this obvious object.)
3. **A capacity budget derived from the target.** State should be sized to the conditional entropy of the task: for byte-level prose at ~1 bit/byte with 10⁴-token contexts, the state must distinguish on the order of hundreds to thousands of bits per position, not 28. Either the group state must be a product of many independent finite factors (e.g. 2I^k with k in the dozens to hundreds, with learned couplings), or the serving contract must admit bounded integer linear maps.
4. **A differentiable training surrogate with a controlled discretization gap** (the contract permits offline matmul). Coordinate hill-climbing over 120-way categoricals has no path to scale; the difflogic/VQ literature the repo cites uses gradients throughout.
5. **Probabilistic role handling.** Replace deterministic admission over hand-designed keys with a learned soft assignment that downstream decoding can marginalize; the "role separability obstruction" is an artifact of the funnel.
6. **A statistical learning argument**, even informal, that the chosen hypothesis class can approximate P(next | context) to within a target bits-per-byte at the intended parameter count. None of the documents attempts this; the formal vocabulary's "Empirical Criterion" discipline is excellent but presupposes a mechanism whose capacity has never been budgeted.
7. **Matched-frequency controls** for zeta phases and **learned-vs-hash controls** for placement before any further geometric extension (the synthesis proposes these; they should be first, not "in response to measured collisions").

Overall: the repository's mathematics is almost entirely *correct* and almost entirely *inert*. Its strongest intellectual content is negative — careful audits showing which named structures do not help — and its strongest engineering is exact memory and provenance. The geometric program as currently instantiated (hash to 33 group elements, 9-level angle distance, ~2–6 KB of learnable circuit, zeroth-order search) has no realistic path to general language; the serving contract's prohibition on any linear contraction removes the one operation known to carry learned pairwise structure at scale, and no finite-group replacement with comparable capacity has been proposed.

---

## Appendix A — Computations performed for this review

**A.1 Hamming distance over the 120-landmark signature is a 9-level class function of the quaternion angle.** I constructed 2I as unit quaternions (24 Hurwitz units ∪ 96 even permutations of ½(±φ, ±1, ±φ⁻¹, 0)), verified |G|=120 and closure under Hamilton multiplication, formed sig(r)[l] = [Re(r·l̄) > 0] for all l ∈ G, and tabulated Hamming distance against Re(r⁻¹s):

| Re(r⁻¹s) | angle | Hamming |
|---:|---:|---:|
| +1 | 0° | 0 |
| +φ/2 = 0.809 | 36° | 22 |
| +1/2 | 60° | 38 |
| +1/(2φ) = 0.309 | 72° | 44 |
| 0 | 90° | 58 |
| −0.309 | 108° | 66 |
| −1/2 | 120° | 76 |
| −0.809 | 144° | 88 |
| −1 | 180° | 90 |

All 120 signatures distinct, all of weight 45 = #{g : Re g > 0} = 1+20+12+12. Proof sketch: Re(ab)=Re(ba) gives left-invariance; l ranging over all of G gives right-invariance; a bi-invariant function on G is a class function of r⁻¹s; 2I has 9 conjugacy classes determined by Re. This matches the repository's census (120 distinct, weight 45, agreement with angular rank) and explains it.

**A.2 Byte-to-root placement.** `training.rs:127–131` sets `leaf = prime % 120` with `prime` the (b+3)-th prime for byte b (first 258 primes 2..1627). Residues coprime to 120 number φ(120)=32; with 2, 3, 5 that gives 35 leaves for 258 tokens and 33 for the 256 bytes; the most populated leaf holds 11 bytes. Information per byte after placement: log₂33 ≈ 5.04 bits.

**A.3 Parameter counts.** Shared core (`shared_core.rs:39–49`): 1024+480+120+120+480+64+512+9+1 = 2,810 root-valued parameters ≈ 19.4 kbit. LUT4 policy (`hamming_policy/policy.rs:11–15`): 3,076 gates × 16 bits = 49,216 bits. Recurrent control (`recurrent_text/runtime.rs:13`): 2,048 rows × 3 actions.

**A.4 Sanity checks on repository arithmetic.** log₂(120⁴) = 27.63 ✓; |PGL(3,2)| = 168, |PGL(2,7)| = 336 ✓; 9·57 = 513 ≡ 1 (mod 256) ✓; centres of twin primes > 3 are multiples of 6 ✓; d(18)=6, d(12)=6, d(72)=12, d(102)=8 ✓; −a/((½−δ)²+a²) − a/((½+δ)²+a²) at δ=0 equals −2a/(¼+a²) ✓; N=256 ⇒ 32,640 pairs, 255 KiB at 8 B ✓; 120×120 u16 table = 28,800 B ✓.
