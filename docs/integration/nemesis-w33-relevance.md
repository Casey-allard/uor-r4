# NEMESIS and W33: direct relevance and the missing bridges

Reviewed 2026-09-03. **Both repositories contain material directly relevant to this effort.** The earlier intake was representative source triage, not a complete reading of either corpus. This follow-up inspects the documents and executable definitions below and separates four useful transfers from the additional claims surrounding them.

**Primary audit verdict: conditional on an explicit carrier/operation mapping to the R4 computation.** There is a direct specification and diagnostic connection now, plus two concrete finite-algebra/runtime candidates for subsequent work. There is no evidence here that importing either repository improves the frozen model or removes its learned attention computation.

## What was actually read

Pins remain NEMESIS `0d106967843c2c96477cf3e57aeff213e7db1c97` and W33 `5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d`. The archived full inventories contain 205 NEMESIS files and 29,843 W33 files. Those counts describe discovery, not reading coverage.

| Source | This follow-up's inspection |
|---|---|
| NEMESIS, [Structure Carrying Substrates report](https://github.com/markrnd87-cmd/NEMESIS-Theory/blob/0d106967843c2c96477cf3e57aeff213e7db1c97/Technical%20Report_%20Integration%20of%20Hypercomplex%20Geometries%20as%20UOR%20Structure%20Carrying%20Substrates.pdf) | All three PDF pages, including the three carrying criteria on p. 1 and interpretation/complexity claims on pp. 2–3. Attributed in the document to Mark / NEMESIS 3D Studio. |
| NEMESIS, [Canonical Geometry and Gauge Structure](https://github.com/markrnd87-cmd/NEMESIS-Theory/blob/0d106967843c2c96477cf3e57aeff213e7db1c97/Canonical%20Geometry%20and%20Gauge%20Structure.pdf) | All three PDF pages: gauge parameterization, proposed E8 quotient, and the still-required action construction. The document labels itself Gemini A.I. / Research Core, addressed to Alex Flom, October 30, 2025; preserve that attribution. |
| NEMESIS, [Formalizing Cayley-Dickson Topology](https://github.com/markrnd87-cmd/NEMESIS-Theory/blob/0d106967843c2c96477cf3e57aeff213e7db1c97/Formalizing%20Cayley-Dickson%20Topology.pdf) | PDF pp. 2–3 and 6–7 in detail; page-opening coverage across all ten pages. Rendered and visually inspected pp. 2 and 6 to check formulas/table and Theorem 1. This is not a full proof audit of the ten-page document. |
| W33 [microVM runtime](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/analysis/w33_fractal_microvm_runtime.py) | Geometry constructor/chambers, routing, CAS, shallow resolution, path rewrite, execution/delivery, snapshots and uniform-tree sections. Read associated runtime documentation and selected tests for geometry, replay, copy-on-write, delivery and routing. |
| W33 [chamber GAP witness](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/analysis/w33_pass4324_4327_chamber_hecke_hashimoto.g) | Read explicit projective constructor, panel matrices, commutator/projector construction and exact check expressions (lines 1–289), plus its [frozen certificate](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/data/PART_W33_PASS4324_4327_CHAMBER_HECKE_HASHIMOTO.json). These two files were additionally fetched at the same pin. |
| W33 [routing GAP witness](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/analysis/w33_fractal_microvm_routing.g) and [audited chamber report](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/analysis/BT4324_BT4334_CHAMBER_HECKE_AND_AUDITED_CORRECTIONS.md) | Full small routing witness; chamber report §§1–2.1, including its explicit limits and point/line projector formulas. The report's later audit sections were not exhaustively reviewed. |
| W33 [HeisenbergQ3.lean](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/formal/W33/HeisenbergQ3.lean) | Full file. It explicitly proves facts about 3×3 matrices over F2, not a formal bridge to the 40×40 incidence operator. |
| Current R4 [#1079 record](../r4_zoology_language_r4_1079.md) and [#1075 coordinate contract](../r4_zoology_compound_r4_1075.md) | Both coordinate-stage definitions and #1079's outcome/next diagnostic. The #1079 implementation freeze is `91aecda179209041decacef9488d5e8ec2681299`; the result CID begins `dee107190172afcb`. |

No upstream Python/GAP/Lean source, model, or benchmark was executed. `PASS` in a supplied certificate is **upstream recorded finite-computation evidence**, not a new independent replay. PDF text extraction and page rendering were read operations only. NEMESIS's missing detected reuse license still limits copying code/documents; it does not prevent careful mathematical study and attributed discussion. W33 carries MIT licensing in the audited tree.

## Four useful transfers

### 1. NEMESIS: make the carrying criterion the actual compiler contract

The SCS report p. 1 asks for a bijection of states, faithful transitions, and an explicit primitive interpretation. Page 2 separates the target structure from its Rust/CPU interpretation. This is directly applicable to our current architecture.

For one R4 block, let the mathematical vector be `v ∈ R^4`, `F` an admitted orthogonal frame, `encode_F(v)=Fᵀv`, and `decode_F(u)=Fu`. For source/destination frames define `T_ci=F_cᵀF_i`. Then in exact real arithmetic:

```text
decode_c(T_ci encode_i(v)) = v
decode_c(sum_i a_i T_ci encode_i(v_i)) = sum_i a_i v_i
```

These are the object/operation mappings required by the carrying criterion. #1079 applies them to sixteen four-lane blocks per 64-vector, then separately measures f64/f32 preservation through learned pooling and the full output path. This explains what has been demonstrated without treating coordinates as a new learned computation.

**Disposition: apply now to the integration specification in [#1083](https://github.com/UOR-Foundation/uor-r4/issues/1083) and the unchanged diagnostic in [#1082](https://github.com/UOR-Foundation/uor-r4/issues/1082).** Record the state domain, encode/decode, operation law, admitted frame witness, rounding boundary and allowed runtime primitives together. The report's wider assertions of O(d) arbitrary-network queries, zero error, or physical energy optimality require separate algorithms and measurements; the carrying criteria themselves do not supply them. The gauge synthesis's proposed E8 quotient also leaves its generators/action to future work, so it does not justify expanding the current 120-frame atlas.

### 2. NEMESIS: explicit involution and four-lane algebra, with the base ring fixed

The Cayley-Dickson document p. 2 gives the concrete recursion

```text
(a,b)(c,d) = (ac − d* b, da + b c*)
(a,b)* = (a*, −b).
```

Pages 6–7 identify conjugation with a chosen real-axis-preserving sign action. At the real quaternion stage this gives `J=diag(1,−1,−1,−1)` on `R^4`, with `J²=I`. This is a useful minimal involution/sign-convention target for a four-lane implementation. It is a restriction of the proposed correspondence, not evidence for every dihedral or lifecycle claim in the document.

The missing bridge is substantive: the document starts over `Z/(2^n)Z` while its property table invokes real division algebras. Already in `Z/256Z`, `2·128=0`, so zero divisors do not first appear at the sedenion stage. Real unit-quaternion inverses and positive norms cannot be transferred unchanged. Also, `det(J)=−1`; quaternion conjugation is not itself one of the orientation-preserving Spin(4) transports. Norm preservation alone does not attest a valid semantic transition: another orthogonal map preserves norm while changing the vector.

**Disposition: retain as a bounded algebra/specification candidate under [#1091](https://github.com/UOR-Foundation/uor-r4/issues/1091), with the arithmetic contract owned by [#1083](https://github.com/UOR-Foundation/uor-r4/issues/1083).** First choose either a real/quaternion convention or a finite word-ring convention and write its product, involution and invertibility preconditions. Compare that to existing Spin/H4 types before copying an implementation. No dimensional doubling or replacement of learned Q/K/V follows from this identification.

### 3. W33: an explicit finite test object for ordered operations and observable subspaces

The runtime [constructor at line 100](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/analysis/w33_fractal_microvm_runtime.py#L100) constructs projective points of `F3^4`; adjacency is zero symplectic pairing. The chamber witness constructs 160 incident point-line pairs and acts on `Q^160`. The field used to construct the incidence geometry and the coefficient field of its operator representation are distinct.

Its two rational/integer panel operators `P,L` hold one incidence coordinate and change the other. The [actual checks](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/analysis/w33_pass4324_4327_chamber_hecke_hashimoto.g#L210) test:

```text
P²=2P+3I; L²=2L+3I; PLPL=LPLP
Ω=LP−PL; Ω³=−60Ω; Π=−Ω²/60
Π²=Π=Πᵀ; rank(Π)=48.
```

This supplies a concrete ordered-operation contrast: `LP` and `PL` have the same operation multiset but can act differently. It also distinguishes changing an operator from exposing a state to that change: a state in `ker(Ω)` is insensitive to this ordering. That distinction is directly useful when interpreting #1079's valid but weaker token intervention.

**Missing R4 bridge:** our learned role vectors are in `R^64`, with sixteen `R^4` blocks; they are not W33 chambers. A port needs a declared encoding `E`, decoding `D`, and operation correspondence, such as `DE=I` on the retained state subspace and `E A=ρ(A) E` for the relevant operations. Mapping arbitrary 64-vectors injectively into the rank-48 image is impossible without restricting the state or adding capacity. `P,L` are three-way adjacency sums, not orthogonal frame matrices; the deterministic `HP0..HL2` selectors are chart choices and do not individually inherit the aggregate Hecke identities. The current count of 24 reachable R4 frames is not W33's rank-24 eigencarrier. Finally, division by 60 in `Π` cannot simply be lowered into `Z/256Z`, where 60 has no inverse.

**Disposition: a strong finite ordered-state/reference candidate under [#1091](https://github.com/UOR-Foundation/uor-r4/issues/1091), after the current diagnostic.** A small independent specification can reproduce the constructor and commutator identities before considering any learned-state map. It is more concrete than a symmetry-name analogy, but supplies neither a replacement attention rule nor a physical-compute advantage by itself.

### 4. W33: persistent state and routing for a CPU-first artifact/runtime layer

The [CAS implementation](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/analysis/w33_fractal_microvm_runtime.py#L314) hashes canonical JSON; [state_at/_rewrite_at](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/analysis/w33_fractal_microvm_runtime.py#L746) resolve one radix-40 address and copy its ancestor path. [send_at](https://github.com/wilcompute/W33-Theory/blob/5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d/analysis/w33_fractal_microvm_runtime.py#L874) binds source, target, route, message and previous receipt into a persistent receipt chain. These are actual implementation seams for immutable execution evidence and deduplicated state snapshots.

The routing witness constructs the base graph and checks diameter two; changing one digit at a time gives at most `2n` **logical line-bus hops** for an n-digit address. It is not constant-time end-to-end execution or a hardware timing bound. The runtime stores base geometry data even though it stores no recursive next-hop table. Seven unique nodes describe a six-level uniform tree because subtrees are identical, not because arbitrary independent leaf states compress to seven blobs. A changed leaf can add up to `n+1` path-state blobs plus a delivery receipt; costs include serialization, hashing and retained versions.

**Disposition: later product-layer adapter, tracked for transfer in [#1091](https://github.com/UOR-Foundation/uor-r4/issues/1091) and dependent on [#1083](https://github.com/UOR-Foundation/uor-r4/issues/1083)'s identity contract.** Map a leaf to a typed immutable state/evidence object, use the existing UOR identity contract rather than silently equating this SHA-256 JSON digest with every κ realization, and define lifecycle/storage budgets. Keep Python's dynamic allocation outside the no-allocation prediction kernel. OCI-shaped layout is not OCI conformance, and this runtime is not a sandbox or hypervisor.

## Nearest useful action and one bounded research target

The native next action remains the **construction-only exposure/cancellation diagnostic already named by #1079 and now owned by [#1082](https://github.com/UOR-Foundation/uor-r4/issues/1082)**. Preservation passed; fact-source transport is strongly sensitive in all six views; the token-source intervention met its strong criterion in only three. The two existing construction renderings and all learned artifacts/weights/frames/control definitions stay fixed. This follow-up proposes no new model run or changed criterion.

NEMESIS's carrying discipline makes the diagnostic's quantities precise. In exact real arithmetic, decoded corrupted token value is `v'_i=F'_i F_iᵀv_i`; set `δ_i=v'_i−v_i`. For each of the 14 used roles with nonnegative soft weights summing to one, distinguish:

```text
M = Σ a_i 1[actual source matrix changed]      weighted exposure
μ = Σ a_i δ_i                               net pooled displacement
E = Σ a_i ||δ_i||²                          weighted displacement energy
C = Σ a_i ||δ_i−μ||² = E−||μ||² ≥ 0          cancellation/dispersion
```

This decomposition is an elementary weighted-variance identity, not a new NEMESIS/W33 theorem. Floating-point tolerances and any zero-energy ratio convention must be explicit. It distinguishes low attention exposure, changes acting weakly on the actual state, cancellation in pooling, and downstream robustness. W33's `ker(LP−PL)` gives an exact finite illustration of why changed structure need not produce changed output; it is not assumed to be the learned role space.

The nearest separate W33 research target in [#1091](https://github.com/UOR-Foundation/uor-r4/issues/1091) is a **constructor-to-operator correspondence note** for the 160 chambers: define `P,L`, connect them to the six deterministic selectors, and identify exactly which finite state directions expose the `LP` versus `PL` difference. Its first deliverable is the typed specification and independently replayable finite witness, with no neural training, geometry expansion or physical claims. Admit a model integration only after a concrete encoder/decoder and a falsifiable behavior-level purpose exist. If that map cannot be stated, keep W33 as an exact reference and use its persistent-state ideas in the product layer instead. [#1089](https://github.com/UOR-Foundation/uor-r4/issues/1089) owns publication claim/provenance treatment; this dossier's inspected-versus-executed distinctions must remain attached when cited.

No change to the main integration plan, dependency pins, production code, frozen artifacts or issue state was made in this review.
