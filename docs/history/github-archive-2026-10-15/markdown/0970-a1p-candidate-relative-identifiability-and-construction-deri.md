# #970 — A1P: candidate-relative identifiability and construction-derived readout

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-27T06:30:55Z
- **Updated:** 2026-08-27T15:52:59Z
- **Closed:** 2026-08-27T15:49:54Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/970
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Corrected evidence result — delivered through protected PR #972 (2026-08-27)

The frozen paired-H4 contract completed with terminal verdict `RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q`.

- Exact construction: `X=C(H,c)`, `Y=C(P_c,c)`, `D=X*Y^-1`; binary translation is derived only from the exact signed `(1,i)` R4 heatmap in `Z[phi]`.
- Contract κ: `blake3:2daacf538c022fab9580d1e124af6c18d0b06da04604fbc962a01bda57f08a98`.
- Complete 120×120 census κ: `blake3:dca725c0ec6060166bcd0023df956e1ff029661b5fa7800ccb9f20808712b796`; 14,400/14,400 ordered pairs, 120/120 relative rows at multiplicity 120, 45 structural heatmap classes, and 480 typed-null pairs.
- Construction fixture κ: `blake3:fb5f27fc1107f527d616f32affa8eba1746a2f60cfdb95ddbb21a0e493299652`; validation fixture κ: `blake3:ecbe8b404e7542d801ff4b4e66c91a41f90158d84efa484dc4edb53aff38b602`.
- Across 36 candidate decisions: 14 exact heatmap classes; construction 12/12 covered and pure; validation 10/12 covered; no-class-splitting ceiling 10/12; strict construction transfer 0/6; eight incompatible classes.
- Existing-additive comparator: two classes, both construction-impure; validation 12/12 covered; oracle ceiling 6/12; strict transfer 0/6.
- Report κ: `blake3:5f9239150dea8c0c27c4dfa6ad2e4d0068bc3d18afc127b315c0ec358ceddb3f`, reproduced by byte-identical double execution.
- The identifiability hard gate stopped before scalar search, scorer compilation, validation selection, downstream controls, or placement. Every downstream row is `NOT_RUN_IDENTIFIABILITY_HARD_STOP`; `NOT_RUN` is not PASS.
- The negative is scoped only to this exact heatmap readout. Paired H4, fixed-zeta phases, ordered n-lets, exact golden radial transport, and typed geometry adapters remain structural/control state. No zeta/n-let-to-phi shell-exponent rule is claimed.

Protected PR #972 merged as `e422fa2a3e4cd31ae7edc56182316a545db3af21`, closing #970. Unassigned #969 is now the eligible multichannel A1Q stage; #953 remains blocked by #969.

---

## Authoritative paired-H4 / R4 heatmap correction — pre-evidence freeze (2026-08-27)

This section supersedes every conflicting single-H4, raw-`D`-class, or 120-state class-key instruction below. The earlier local single-H4 result is a superseded scope diagnostic only: it is not terminal evidence, it does not close #970, and it does not unblock #969. No replacement result may be collected until this public freeze is implemented exactly.

### Paired operand and relative-projection binding

For this A1P contract, the E8/H4×H4 witness is the ordered pair `X(H,c) = C(H,c)`, `Y(P_c,c) = C(P_c,c)`. Neither operand may be discarded. The exact relative projection is `D(H,c) = X(H,c) * Y(P_c,c)^-1`.

This quotient is the equality/opposition-preserving projection: `X=Y => D=+1` and `X=-Y => D=-1`. The ordered pair is serialized as the construction witness; the binary translation is not run on either operand or on an opaque table offset. It is run only against the R4 heatmap derived from `D`. The pair domain is the bounded `120 × 120` ordered H4 witness space; exact multiplication closes into the frozen 120-root relative image. This probe does not relabel the repository's separately retained, golden-coupled per-leaf `H4 ⊕ φH4` icosian serialization as an independent Cartesian root system.

### Exact R4 heatmap and binary translation

Write the exact relative root in the canonical oriented basis as `D = (q0 + q1 i + q2 j + q3 k) / 2`, with every `qk = ak + bk φ` in `Z[φ]`.

The signed chart is frozen as `sin=q0/2`, `cos=q1/2`, `activation=sin²=q0²/4`, `chirality=sign(sin)`, and `cosine_polarity=sign(cos)`. The ordered `(1,i)` plane and its signs are fixed by the canonical root-table orientation; this is the orientation needed to make signed `sin` deterministic in R4. If `q0=q1=0`, the chart is typed-null and must abstain rather than invent an angle.

The binary endpoints are:

| Exact heatmap | Binary | Retained orientation |
|---|---:|---|
| `sin=+1, cos=0` | 1 | positive chirality |
| `sin=-1, cos=0` | 1 | negative chirality |
| `sin=0, cos=+1` | 0 | positive cosine polarity |
| `sin=0, cos=-1` | 0 | negative cosine polarity |

`binary_landmarks[].output` is the geometric activation bit only; it is not `SELECT`/`REJECT` and carries no candidate label. Non-landmark roots remain exact `Z[φ]` heatmap classes. There is no `activation != 0` shortcut, fitted threshold, arbitrary sign collapse, or general Q1.30 projection. A class receives a binary candidate outcome only when the construction split makes that complete exact heatmap class pure; unseen, impure, or typed-null classes abstain. For this contract, “complete exact R4 heatmap class” means the complete signed `(1,i)`-plane projection `(sin_Zφ/2, cos_Zφ/2, sin²_Zφ/4, chirality, cosine polarity, chart status)`. The remaining `q2/q3` coordinates stay exact in the full `D` witness but are not scorer-key fields. `X` and `Y` remain reproducibility witnesses but are not extra label-bearing scorer channels.

### Fixed-zeta, combinatorial n-let, golden radial, and typed-geometry binding

The heatmap contract is carried with the immutable zeta-zero grid `uor-r4-core::zeta_zeros:v1`, κ `blake3:512243ed9e2c1deef0691515caf02ca25e3d5c7990184cd804f6d65c1cc8d94c`, under `theta_j(p)=wrap(gamma_j*log(p))`. Ordered, multiplicity-preserving combinatorial prime n-lets carry route growth. The complex-to-Riemannian lift is the exact golden shell step `R_phi(z)=φ E(z)`, so `(a,b)*φ=(b,a+b)`, `(a,b)*φ^-1=(b-a,a)`, normalized direction is preserved, and the nonzero radial-length ratio is φ.

No zeta/n-let-to-φ shell-exponent derivation has been supplied or established by the repository, so this probe binds these exact channels side-by-side as structural transport and records `STRUCTURAL_BINDING_ONLY_NO_ZETA_NLET_TO_PHI_EXPONENT_RULE`; it does not invent a coupling equation.

The declared cross-chart equivalence marker is `Euclidean sqrt(2) <-> complex 2i <-> Riemannian [0,2]`. These are typed representatives, so conversion must bind units, orientation, normalization, and error; raw untyped numeric equality is forbidden. The fixed zeta identity, raw n-let identity, and raw radial magnitude are structural transport/provenance and may not become target or label shortcuts in this preflight.

### Canonical contract identity

Using the existing `canonical_kappa(serde_json::to_vec(wire))` convention, the exact contract κ is `blake3:2daacf538c022fab9580d1e124af6c18d0b06da04604fbc962a01bda57f08a98`. Wire field order is the JSON order below; each binary-landmark field order is `sin_zphi_numerator`, `cos_zphi_numerator`, `activation_zphi_numerator`, `chirality`, `cosine_polarity`, `output`.

```json
{"schema":1,"domain":"uor-r4.a1p-paired-h4-r4-heatmap-contract/1","construction_artifact_kappa":"blake3:2b70588d654c8e8bb2d8ab063f41853d45a21487d742ff7567f93a42cfb9011b","manifest_kappa":"blake3:1c77c4103732964af6776f1dfcabc8b2a9191eea875a8ba205c36ebbf5618a99","frozen_partition_kappa":"blake3:d008b82eda9b16b102cf4c7ffa4a47a40ad514b30f0763ed3f46c0ebae3e277b","construction_fixture_kappa":"blake3:fb5f27fc1107f527d616f32affa8eba1746a2f60cfdb95ddbb21a0e493299652","validation_fixture_kappa":"blake3:ecbe8b404e7542d801ff4b4e66c91a41f90158d84efa484dc4edb53aff38b602","h4_root_table_kappa":"blake3:8d33d62a239fb8001fea2bd14a9a5ec7321d0f07d81c74a5715eaeb3df53aa76","h4_multiplication_table_kappa":"blake3:90ee73a27ee2e8ba5bccd1507d7fb37ed1f044b1640772c86752bc0bb2111759","zeta_grid_revision":"uor-r4-core::zeta_zeros:v1","zeta_grid_kappa":"blake3:512243ed9e2c1deef0691515caf02ca25e3d5c7990184cd804f6d65c1cc8d94c","paired_h4_operands":["X(H,c)=C(H,c)","Y(P_c,c)=C(P_c,c)"],"pair_space_rule":"ordered H4xH4 witness; neither operand may be discarded","relative_product":"D(H,c)=X(H,c)*Y(P_c,c)^-1","relative_image_rule":"exact closure through the frozen 120-state H4 multiplication table","r4_basis":["1","i","j","k"],"oriented_heatmap_plane":"ordered (1,i) coordinates with signs fixed by the canonical root-table orientation","coordinate_scale_denominator":2,"exact_ring":"Z[phi]","sin_coordinate_rule":"sin=q0/2","cos_coordinate_rule":"cos=q1/2","activation_rule":"activation=sin^2=q0^2/4","activation_denominator":4,"chirality_rule":"sign(q0/2)","cosine_polarity_rule":"sign(q1/2)","typed_null_rule":"q0=0 AND q1=0 => TYPED_NULL_ABSTAIN","heatmap_class_key":["sin_zphi_numerator","cos_zphi_numerator","activation_zphi_numerator","chirality","cosine_polarity","chart_status"],"binary_landmarks":[{"sin_zphi_numerator":[2,0],"cos_zphi_numerator":[0,0],"activation_zphi_numerator":[4,0],"chirality":"POSITIVE","cosine_polarity":"ZERO","output":1},{"sin_zphi_numerator":[-2,0],"cos_zphi_numerator":[0,0],"activation_zphi_numerator":[4,0],"chirality":"NEGATIVE","cosine_polarity":"ZERO","output":1},{"sin_zphi_numerator":[0,0],"cos_zphi_numerator":[2,0],"activation_zphi_numerator":[0,0],"chirality":"ZERO","cosine_polarity":"POSITIVE","output":0},{"sin_zphi_numerator":[0,0],"cos_zphi_numerator":[-2,0],"activation_zphi_numerator":[0,0],"chirality":"ZERO","cosine_polarity":"NEGATIVE","output":0}],"intermediate_rule":"retain the complete exact heatmap class; compile only a pure construction outcome; unseen or impure classes abstain; no threshold","q30_rule":"exact rational landmarks only; no general Q1.30 projection without a separately bound quantizer and error contract","zeta_phase_rule":"theta_j(p)=wrap(gamma_j*log(p)) on the immutable fixed-zeta grid; phase identity is structural and not a target feature","ordered_nlet_rule":"ordered multiplicity-preserving combinatorial prime n-lets carry route growth; ordered primes and factor multiset are both retained","golden_radial_forward":"R_phi(z)=phi*E(z); (a,b)*phi=(b,a+b)","golden_radial_inverse":"(a,b)*phi^-1=(b-a,a)","normalized_direction_rule":"normalize(R_phi(z))=normalize(E(z)) for nonzero z","radial_ratio_rule":"norm(R_phi(z))/norm(E(z))=phi for nonzero z","radial_coupling_status":"STRUCTURAL_BINDING_ONLY_NO_ZETA_NLET_TO_PHI_EXPONENT_RULE","typed_geometry_equivalence":["EUCLIDEAN_ROOT_2","COMPLEX_2I","RIEMANNIAN_0_2"],"typed_geometry_adapter_rule":"Euclidean sqrt(2) <-> complex 2i <-> Riemannian [0,2] is a declared typed cross-chart equivalence; compare only through versioned unit, orientation, normalization, and error adapters, never as raw untyped scalars","semantic_exclusions":["validation labels","target token spelling","lexical/token IDs","prime/address identity","kappa/digest as geometry","raw zeta channel identity as scorer input","raw n-let identity as scorer input","raw radial magnitude as scorer input"]}
```

### Replacement hard gate

Enumerate the exact paired operand witnesses and derived heatmap classes for every inherited, construction, and validation candidate decision, keeping split denominators separate. Report the full bounded 120×120 structural heatmap universe independently from the 36 exercised candidate decisions. Construction purity, validation coverage, oracle ceiling, and construction-transfer ceiling are keyed only by the complete exact heatmap class. If a heatmap class aliases incompatible outcomes, is construction-impure, fails transfer, or is typed-null where selection is required, stop before scalar/readout/control execution with `RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q`. Do not try another threshold or add `X`, `Y`, zeta identity, n-let identity, or radial magnitude to split the class.

Validation histories and natural support must be prepared without labels, then joined with the sealed labels only for purity/ceiling evaluation. S4 parity must be derived from each exact history and the frozen role order rather than trusted from the fixture literal. The exact required downstream control-name set remains frozen and, after a hard stop, every row is `NOT_RUN_IDENTIFIABILITY_HARD_STOP`.

---
## Inherited A1R result

#967 executed its frozen six-query gate and reached `RETAIN_STATE_ONLY`.

Authoritative A1R report κ: `blake3:f0db7a5d5c81d51ebf3b4bf8a2715c4960ec16b14161e8bf7598d7b98c48c881`.

The associative noncommutative H4 fold repaired #952's ordered-state defect. Its exact group, regrouping, incremental, scope-isolation, candidate-support, and payload laws held on the frozen denominators. The full candidate-relative states

`D(H,c) = C(H,c) * C(P_c,c)^-1`

were distinct between `ll` and `rr` on 6/6 queries, and 5/6 matched same-candidate comparisons changed across left/right histories. The state and construction-only anchors therefore did not collapse.

The scalar shortest-Cayley-distance readout did collapse: every full-arm state had energy 2, producing 0/6 strict intended selections, 6/6 ties, and 6/6 abstentions. The existing additive-summary state was bound, but its ranking arm was `NOT_EXERCISED_NO_PREDECLARED_ADDITIVE_SCORER`; it is not a tied or weaker exercised control. Independently, `any_exercised_control_not_weaker = true`, so the terminal negative follows from exercised full/control evidence rather than from additive unavailability.

This retains the ordered fold, construction anchors, and candidate-relative states as structural representation only. It does not establish attention, inference, generation, correctness, reasoning, or product readiness.

## Eligibility and native graph

- Active immediate mechanism stage, assigned to Casey-allard after #967 closed through protected PR #971.
- Native child of programme root #820.
- Native successor of closed #967; the closed predecessor no longer blocks eligibility.
- Native blocker of A1Q/#969.
- #953 remains independently blocked by #969.

## Frozen inherited identities, support, and work

Preserve without tuning:

- S0 artifact κ `blake3:3f2043e15a32f6ef799c0073d0c714e3140449591b7d8a18069e39c5182662bd` (bound by A1R, not reproduced there);
- #952 partition κ `blake3:d008b82eda9b16b102cf4c7ffa4a47a40ad514b30f0763ed3f46c0ebae3e277b`;
- construction artifact κ `blake3:2b70588d654c8e8bb2d8ab063f41853d45a21487d742ff7567f93a42cfb9011b`;
- embedded schema-2 manifest κ `blake3:1c77c4103732964af6776f1dfcabc8b2a9191eea875a8ba205c36ebbf5618a99`;
- #952 A1.0 report κ `blake3:23e07a17e897abb701c09354d35a3113a1b075ba1d60ee634067fa3ed3fd1904`;
- A1R symmetric Cayley metric κ `blake3:5dfda068473f57afdf33f17ea6a53efea62fd13d9c98082032f00d5198bc96a2`;
- A1R report κ above;
- the exact associative fold, construction-predecessor anchors, frozen contrasts, and scope fixtures;
- 3 contrasts, 6 candidate queries, 12 payload inversions, 6 incremental checks, 7 row reads per query / 42 total, candidate-entry ceiling 56 per query, candidate ceiling 8, and maximum 2 observed candidates;
- the same candidate rows, support counts, admission, payloads, exact-row misses, and support-denominator κ `blake3:7b17bdc7f3686fd734c1770a4d6c5cc1a0590accbe8ceaa4e143ec1cd3369777`; and
- no future-route leakage, target injection, source weights, teacher, provider, external/corpus population scan, or generation.

The finite 120-state algebra is a fixed local fixture, not authority to expand into a corpus scan. Table-index distance, κ distance, identity/provenance bits, equal unit norm, candidate-independent invariants, and any form that cancels history remain forbidden as semantic geometry.

## Corrected objective

Determine, before metric design, whether the exact R4 heatmap derived from the ordered paired-H4 witness can identify a construction-derived candidate rule on independent histories. The six visible A1R labels remain byte-identical regression/root-cause evidence only. A new 6/6 score on those labels, or tuning against them, has no promotional force.

If exact paired-H4 heatmap classes are identifiable and construction evidence transfers, compile exactly one construction-only finite-table readout and compare it with an equally compiled additive-summary scorer. If exact heatmap classes alias incompatible outcomes or construction evidence does not transfer, stop before scalar search, retain paired H4 and its R4 heatmap as structural/control state, and advance #969 to its other already-declared channels.

A placement intervention is permitted only if the preflight proves placement is the isolated defect. Readout and placement may not both be tuned.

## Public pre-evidence fixture freeze

All fixture κ values below use the repository's existing `canonical_kappa(serde_json::to_vec(wire))` convention with schema 1. The compact wire binds the listed source partition, construction artifact, manifest, ordered role list, current token, natural candidate set, predecessor anchors, observation IDs, exact histories, observed next candidate, and permutation parity. Exact wire field order is `schema`, `domain`, `fixture_id`, `split`, `construction_artifact_kappa`, `manifest_kappa`, `frozen_partition_kappa`, `role_order`, `current_token`, `natural_candidates`, `construction_predecessors`, `observations`; predecessor fields are `candidate`, `predecessor`; observation fields are `id`, `history`, `observed_next`, `permutation_parity`. The exact `role_order` is `["aa","bb","cc","dd"]`, and parity wire literals are uppercase `EVEN` / `ODD` (the tables render them lowercase).

### Existing regression fixture — unchanged and non-promotional

| ID | History | Label |
|---|---|---|
| A1R-R01 | `aa bb dd cc qq` | `ll` |
| A1R-R02 | `bb aa dd cc qq` | `rr` |
| A1R-R03 | `aa dd bb cc qq` | `ll` |
| A1R-R04 | `dd aa bb cc qq` | `rr` |
| A1R-R05 | `bb dd aa cc qq` | `ll` |
| A1R-R06 | `dd bb aa cc qq` | `rr` |

Identity remains #952 partition κ `blake3:d008b82eda9b16b102cf4c7ffa4a47a40ad514b30f0763ed3f46c0ebae3e277b`; historical result identity remains #967 report κ `blake3:f0db7a5d5c81d51ebf3b4bf8a2715c4960ec16b14161e8bf7598d7b98c48c881`. Its denominator remains 3 contrasts / 6 queries. The frozen shortest-Cayley result remains 0/6 selections, 6/6 ties, and 6/6 abstentions. It is never training or promotion evidence.

### Construction fixture

Fixture ID `A1P-CONSTRUCTION-1`; κ `blake3:fb5f27fc1107f527d616f32affa8eba1746a2f60cfdb95ddbb21a0e493299652`.

| ID | History | Observed next | Parity |
|---|---|---|---|
| A1P-C01 | `aa bb cc dd qq` | `ll` | even |
| A1P-C02 | `aa cc bb dd qq` | `rr` | odd |
| A1P-C03 | `bb aa cc dd qq` | `rr` | odd |
| A1P-C04 | `bb cc aa dd qq` | `ll` | even |
| A1P-C05 | `cc aa bb dd qq` | `ll` | even |
| A1P-C06 | `cc bb aa dd qq` | `rr` | odd |

The predeclared rule family is the two-element abelianization of the permutation group on ordered roles `[aa, bb, cc, dd]`. The construction compiler admits only the trivial map or the sign homomorphism `S4 -> C2`; construction observations reject the trivial map, A1P-C01 determines the candidate orientation, and C02-C06 must confirm it. Thus construction evidence determines `even -> ll`, `odd -> rr`; it is not a set of labels attached to unrelated histories. The compiled scorer never receives role names, parity, token spelling, IDs, prime order, address order, or labels at validation time: it receives only exact construction-derived state classes.

### Sealed anti-recall validation fixture

Fixture ID `A1P-VALIDATION-1`; κ `blake3:ecbe8b404e7542d801ff4b4e66c91a41f90158d84efa484dc4edb53aff38b602`.

| ID | History | Sealed intended next | Parity |
|---|---|---|---|
| A1P-V01 | `aa cc dd bb qq` | `ll` | even |
| A1P-V02 | `aa dd cc bb qq` | `rr` | odd |
| A1P-V03 | `cc aa dd bb qq` | `rr` | odd |
| A1P-V04 | `cc dd aa bb qq` | `ll` | even |
| A1P-V05 | `dd aa cc bb qq` | `ll` | even |
| A1P-V06 | `dd cc aa bb qq` | `rr` | odd |

Validation histories are disjoint from construction and from the inherited regression: construction ends in `dd qq`, validation in `bb qq`, and regression in `cc qq`. All exact five-unit histories differ. Observation continuations are a separate κ-bound ledger and are not compiled into the schema-2 candidate manifest. Direct/exact or divisor continuation hits, target injection, future events, or any candidate union other than natural `{ll, rr}` invalidate the contract. Validation target labels are public for the identifiability ceiling, but validation scorer outputs remain unopened until any construction-to-readout compiler is frozen.

No #969 sentence, paragraph, conversation, or global fixture may be consumed, inspected, or exposed.

### Candidate rule, transformations, and anti-cheating controls

The candidate set is natural `{ll, rr}` with construction predecessors `P_ll = uu` and `P_rr = vv`.

- Candidate relabeling is `rho = (ll rr)(uu vv)`. Apply the same bijection to construction observations, predecessor roles, and validation labels, recompile from transformed construction evidence, and require outputs to follow the transformed evidence.
- The H4 prime-assignment permutation is `pi_p = (aa bb cc dd gg qq)(ll rr uu vv)`, defined by `leaf_pi(t) = leaf(pi_p(t))`. It changes only the H4 leaf binding after the unchanged natural candidate/support path; payload names, rows, admission, and support stay fixed. Recompile from the transformed construction states before validation.
- The deterministic geometry permutation conjugates every H4 state by the existing A1R first-noncentral conjugator under root-table κ `blake3:8d33d62a239fb8001fea2bd14a9a5ec7321d0f07d81c74a5715eaeb3df53aa76`. Apply it to construction and validation and recompile.
- These three are equivariance audits, not weaker-baseline claims. A valid positive must reproduce the appropriately transformed construction rule exactly. Current-only, additive, factor/count-only, hierarchy-disabled, exact-recall-only, and a conditional placement arm are matched comparator arms and must be strictly weaker than the full paired-H4-derived R4 heatmap arm.
- A result that depends on literal spelling, lexical/token IDs, sequential primes, canonical address ordering, diagnostic tie order, target orientation, or a manually selected sign is invalid.

### Denominators and ceilings

| Fixture | Histories | Candidate decisions | Natural candidates | Row ceiling | Candidate-entry ceiling | Paired witness domain / relative heatmap image |
|---|---:|---:|---:|---:|---:|---:|
| A1R regression | 6 | 12 | 2/query | 7/query, 42 total | 56/query | 120×120 / ≤120 |
| Construction | 6 | 12 | 2/query | 7/query, 42 total | 56/query | 120×120 / ≤120 |
| Sealed validation | 6 | 12 | 2/query | 7/query, 42 total | 56/query | 120×120 / ≤120 |

The preflight may inspect at most 18 fixed histories and 36 candidate decisions, plus one target-free exhaustive structural enumeration of the frozen 120×120 paired-H4 domain, its 120-state exact relative image, and the fixed heatmap chart. If readout is authorized, each full or control arm receives exactly 6 validation queries, 12 candidate decisions, 42 row reads, candidate-entry ceiling 56/query, candidate ceiling 8, and maximum two admitted candidates. There are one full arm plus eight required arms; a tenth placement arm exists only if preflight isolates placement. Execute the final authorized probe twice. No external/corpus population scan, source model, teacher, provider, generation, or #969 fixture is authorized.

## Identifiability hard gate

Before choosing any scalar or opening any validation scorer output:

1. Enumerate both exact H4 operands, their exact `D(H,c)` relative projection, and the complete exact R4 heatmap class for every fixed inherited-regression, construction, and validation candidate decision. Keep all three denominators separate; regression labels are diagnostic only and never train the compiler.
2. On construction, report complete exact heatmap-class coverage and per-class binary candidate-outcome purity (`observed_next == c` versus rejection).
3. Report validation candidate-class coverage and two ceilings:
   - the no-class-splitting oracle ceiling `sum_E max_y count_validation(E,y) / 12`; and
   - the construction-transfer selection ceiling: validation queries on which the construction class map makes the intended candidate uniquely positive and the alternative uniquely negative, divided by 6. Unseen classes abstain.
4. Repeat the same analysis for the additive comparator. Its exact class is `(A*(H), A*(c), A*(P_c))`, where `A*` is the existing additive non-digest summary with spelling, occurrence/position identifiers, lexical-unit ID, prime, address index, boundary/chain identity, κ/digests, and provenance excluded. No new geometric term is added.
5. If any retained exact heatmap class requires incompatible candidate outcomes, construction is impure, or construction observations define no rule that transfers to validation, terminate immediately as `RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q`.
6. After such a failure, do not search another metric, add scorer fields, tune a threshold, enlarge the corpus, or run selection/control arms.

This preflight is binding even if a different scalar could fit the six visible regression labels.

## Conditional implementation contract

Only if the hard gate permits a real experiment:

1. Compile exactly one deterministic bounded readout from construction observations only.
2. Use exact integer and finite-table operations.
3. Freeze the construction-to-readout algorithm and serialized artifact before opening validation scorer outputs.
4. Apply the same exact class-to-outcome compiler to the additive comparator.
5. Preserve candidate rows, payload inversion, support/admission, source partition, row and candidate ceilings, causality, and equal work across arms.
6. Expose only an API-neutral serializable `select_or_abstain` result and minimal trace for possible #969 consumption.
7. Reproduce the inherited old-six shortest-Cayley failure unchanged, but report it separately from construction and validation.
8. Execute twice and require byte-identical inputs, report, and κ.

Required arms/status rows are:

- full paired-H4 R4 heatmap;
- current-only;
- additive summary with compiled scorer;
- factor/count-only;
- deterministic geometry permutation;
- candidate relabeling;
- prime-assignment permutation;
- hierarchy-disabled;
- exact-recall-only; and
- one predeclared placement intervention only if identifiability proves placement is the isolated defect.

Every row reports selection, ties, abstentions, exact hits, support equality, work, and explicit status. `NOT_EXERCISED`, `NOT_RUN`, `UNAVAILABLE`, and `INVALID` are not passes. If the identifiability gate stops, all downstream selection/control rows are truthfully `NOT_RUN_IDENTIFIABILITY_HARD_STOP`; this is a valid bounded paired-heatmap readout-identifiability negative, not an unavailable contract.

## Terminal decisions

- `PROMOTE_H4_READOUT_CANDIDATE_TO_A1Q`: only after the frozen independent contract, an identifiable construction rule, strict selection on all 6/6 sealed validation queries, every required arm exercised, every matched baseline strictly weaker, exact candidate/prime/geometry equivariance, equal support/admission/payload/causality/work, and deterministic selector/trace bytes and κ. This promotes only the exact paired-H4-derived R4 heatmap readout candidate into #969; it establishes neither attention nor generation.
- `RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q`: use for inherited or construction class aliasing, no transferable construction rule, held-out miss/tie/abstention, inert readout, or failure to beat an exercised comparator. Close #970; keep both H4 operands and the derived R4 heatmap structural/provenance/diagnostic/control-only; make #969 eligible to test its other predeclared candidate-relative trajectory/harmonic channels. Do not create another paired-H4 heatmap scalar-readout chain. #953 remains blocked by #969.
- `UNAVAILABLE_NO_INDEPENDENT_HOLDOUT` or `INVALID_CONTRACT`: leave #970 open and #969 blocked; publish the exact blocker.

## Out of scope

No A1Q implementation, sentence/paragraph/conversation/global scope validation, open-ended text generation, teacher/model run, transformer, MoE, learned router, dense intelligence matrix, Ollama, hosted provider, broad corpus compile, release QA, product integration, or unrelated cleanup.

## Definition of done

- The issue and living programme text publicly bind the independent fixtures, rule compiler, transformations, denominators, ceilings, controls, exclusions, and decisions before readout implementation or result collection.
- Both fixture κ values and paired-H4/R4 heatmap contract κ `blake3:2daacf538c022fab9580d1e124af6c18d0b06da04604fbc962a01bda57f08a98` reproduce from exact canonical wires.
- The unchanged old-six shortest-Cayley regression remains separately reported.
- Paired-H4 R4 heatmap and additive identifiability coverage, purity, aliasing, and ceilings are report-bound.
- If authorized, exactly one construction-only readout and every required arm are reported twice with identical bytes and κ.
- The result has a new independent report κ and truthful terminal verdict.
- A valid negative closes #970 and exposes only #969; unavailable/invalid evidence leaves #970 open.




## Comments (4)

### comment 5439840538 — 2026-08-27T13:29:24Z — Casey-allard

Pre-evidence contract correction is now frozen publicly against live `main` `cabc922751d8f1bc3c1d3a291a0a1a18897fcdd0`. No replacement scalar/readout has been implemented and no new validation selection output has been collected.

- The inherited A1R six remain regression-only: #952 partition κ `blake3:d008b82eda9b16b102cf4c7ffa4a47a40ad514b30f0763ed3f46c0ebae3e277b`; #967 report κ `blake3:f0db7a5d5c81d51ebf3b4bf8a2715c4960ec16b14161e8bf7598d7b98c48c881`; frozen result 0/6 strict, 6/6 ties, 6/6 abstentions. They cannot promote a replacement readout.
- Construction is `A1P-CONSTRUCTION-1`, κ `blake3:fb5f27fc1107f527d616f32affa8eba1746a2f60cfdb95ddbb21a0e493299652`: six observed full sequences induce the nontrivial `S4 -> C2` sign rule and its candidate orientation.
- Sealed anti-recall validation is `A1P-VALIDATION-1`, κ `blake3:ecbe8b404e7542d801ff4b4e66c91a41f90158d84efa484dc4edb53aff38b602`: six disjoint histories, separately reported, never ingested into the candidate manifest.
- Candidate relabeling `rho = (ll rr)(uu vv)`, H4 prime-assignment permutation `pi_p = (aa bb cc dd gg qq)(ll rr uu vv)`, and deterministic H4 conjugation are frozen equivariance audits.
- The identifiability hard gate ranges over inherited regression, construction, and validation exact `D(H,c)` classes. The inherited six are falsification-only. Any incompatible retained class or no construction-to-validation transfer terminates before scalar search as `RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q`.
- A valid positive can promote only an H4 candidate term to #969. A valid negative closes #970 and advances #969 with H4 structural/control-only. Unavailable or invalid evidence leaves #970 open and #969 blocked.
- No #969 scope fixture, generation run, teacher/model run, broad corpus work, or downstream implementation is authorized.

The issue body now contains every history/label, exact denominator and ceiling, additive comparator, support/work invariant, required arm, exclusion, and terminal rule. The next action is the bounded identifiability preflight only.


### comment 5440469187 — 2026-08-27T14:18:14Z — Casey-allard

Contract-correction hold before any deliverable evidence:

The current local A1P probe evaluated one 120-state `OrderedH4FoldState` class. That is insufficient for the newly clarified load-bearing operation: `H4 x H4` must first produce the R4 quantum/trigonometric state, and any binary reduction must be derived from that R4 heatmap:

- aligned: `sin=0, cos=+1`;
- adjacent/oriented: `sin=+/-1, cos=0`;
- opposite: `sin=0, cos=-1`;
- binary activation: `sin^2`, so `(sin=+/-1, cos=0) -> 1` and `sin=0 -> 0`, while chirality/cosine polarity remain explicit rather than discarded.

Therefore the local single-H4 identifiability result is a superseded scope diagnostic only. It is not terminal #970 evidence, will not close this issue, and will not make #969 eligible. No PR exists and no paired-H4 result has been collected.

Before rerunning evidence, the public contract will bind the exact two H4 operands, fixed R4 basis/orientation, exact cosine/signed-sine construction, heatmap/class wire and kappa, construction/validation separation, and transformed controls. Until that freeze is complete, #970 remains open and #969 remains blocked.

### comment 5441643172 — 2026-08-27T15:48:45Z — Casey-allard

Corrected paired-H4 evidence is implemented in PR #972: https://github.com/UOR-Foundation/uor-r4/pull/972

The exact public contract, complete 120×120 census, independent fixtures, target-free preparation, derived S4 parity, class ceilings, additive comparator, hard-stop statuses, and scoped nonclaims are report-bound. Binding identities:

- contract: `blake3:2daacf538c022fab9580d1e124af6c18d0b06da04604fbc962a01bda57f08a98`
- pair census: `blake3:dca725c0ec6060166bcd0023df956e1ff029661b5fa7800ccb9f20808712b796`
- report: `blake3:5f9239150dea8c0c27c4dfa6ad2e4d0068bc3d18afc127b315c0ec358ceddb3f`

Terminal verdict: `RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q`.

The protected merge remains the eligibility boundary: #970 stays active and #969 blocked until PR #972 lands; #953 remains blocked by #969.

### comment 5441693669 — 2026-08-27T15:52:59Z — Casey-allard

Delivered through protected PR #972 as merge commit `e422fa2a3e4cd31ae7edc56182316a545db3af21` (live `main`).

Definition-of-done evidence:

- paired-H4/R4 heatmap contract κ `blake3:2daacf538c022fab9580d1e124af6c18d0b06da04604fbc962a01bda57f08a98`
- complete pair census κ `blake3:dca725c0ec6060166bcd0023df956e1ff029661b5fa7800ccb9f20808712b796`
- corrected report κ `blake3:5f9239150dea8c0c27c4dfa6ad2e4d0068bc3d18afc127b315c0ec358ceddb3f`
- byte-identical focused double execution: PASS
- target-free/parity unit invariants: PASS
- inherited #967 regression: PASS
- exact source closure, focused compilation, host CLI, WASM library, formatting, diff, and claim wording: PASS
- broad workspace/corpus/model/teacher/generation/product QA: NOT_RUN under the live dormant-QA policy

Terminal verdict: `RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q`.

#969 is open, eligible, and unassigned. Its native blocked-by relationship points only to closed #970. #953 remains blocked by open #969 (and closed historical #952). No #969 or #953 implementation began.


---
_Comment/review ids in this file: 5439840538, 5440469187, 5441643172, 5441693669_

