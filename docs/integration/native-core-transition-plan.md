# Native core transition plan — module-level design

Owner: Casey · Drafted by: Zed (agent) · Date: 2026-09-19 · Status: draft pending owner approval

Authority: [`DECISIONS.md`](DECISIONS.md) (D0-a, D1) supersedes conflicting wording in
`README.md` / `AGENTS.md` once owner-directed delivery lands. Sequence and acceptance
remain owned by [`project-track.md`](project-track.md); live results by
[`current-state.md`](current-state.md). This file adds implementation detail only.

Objective restated against the owner's actual goal: **beat existing local runtimes on
tok/s and resident memory, without a multiplier and without a GPU, at competitive
quality on real text.** Quality parity is the hard part; efficiency is where the
architecture is genuinely different.

---

## 0. Two findings from source inspection that change the design

**(i) Correction to my earlier characterisation.** The token→root assignment is **not**
`prime % 120` in the active model. `learner/jepa_trainer.rs:1119` assigns
`token_to_root[v] = embeddings.nearest_h4_root(v)` — a Voronoi partition over the 120
canonical H4 roots induced by the **learned** continuous embeddings. The
`prime % 120` hash is only the *initialisation* (`learner/embedding.rs:170`). So the
assignment is learned; what is limited is the **resolution** (120 buckets ≈ 6.9 bits),
not the learning.

**(ii) New: there are two inconsistent codebooks, and they do not talk to each other.**

| Codebook | Source | Used by |
|---|---|---|
| `token_to_root: Vec<u8>` (120 roots, learned) | `nearest_h4_root` over learned embeddings | lanes, lattice, hierarchical routing, hierarchical softmax |
| `Codebook<64>` (4096-bit hypervectors, **fixed random** from `vsa_seed` + token id) | `vsa/codebook.rs::generate_basis_vector` (SplitMix64) | the four VSA attention heads and the VSA score term |

The VSA layer therefore operates on a **fixed random hash of the token id that is
disconnected from the model's learned representation**. For a random codebook,
`E[d_H(E(a), E(b))] = D/2` for all `a ≠ b` (sd ≈ 32 at D = 4096), so the only
recoverable signal is `a == b`. This is why the heads behave as identity detectors and
why their contribution is expected to be ≈ 0 (see Card P7, experiment E1.1). It is also
the deepest single defect in the current serving graph: **the geometric "attention" is
not attention over the model's representation at all.**

---

## 1. Stage 0 — Contract and denominator

| Item | Artifact | Gate |
|---|---|---|
| Record D0-a / D1 | `docs/integration/DECISIONS.md` | signed |
| Ground truth on M1 | Card P1 as written (`cards/P1-ground-truth-m1.md`) | pinned table exists |
| One evidence table | `docs/integration/EVIDENCE.md` | one row per card |

Deliverable is the denominator: tok/s, J/token (`powermetrics`), BPB, resident
bytes/token for `bitnet.cpp` + BitNet b1.58 2B4T, `llama.cpp` + SmolLM3-3B and
Qwen3-4B Q4, the current native `.rgm`, and the TLA bundle — same machine, same session.

---

## 2. Stage 1 — Falsification sweep (do this first)

Rationale: it is the cheapest work that can change our minds, and D1 makes it a
precondition. Nearly all of it is **ablation without retraining**, because the artifact
format already gates optional mechanisms behind `flags` and one scalar:

- `binary_model.rs:63-72` — `FLAG_HAS_HIERARCHICAL_CODEBOOK` (1), `..._ENGRAM_TABLE` (2),
  `..._HIERARCHICAL_LATTICE` (4), `..._JEPA` (8)
- `binary_model.rs:1450` — `if self.header.vsa_scale_q15 != 0 && ctx_len > 0`
- `native_capability_api.rs:1151,1185` — flag guards already present in the scorer

### E1.1 Per-mechanism ablation BPB table

New binary `crates/uor-r4-core/src/bin/ablate-prose.rs` (or `--ablate <name>` on the
existing eval path). Load the `.rgm`, apply exactly one ablation, evaluate held-out BPB,
print a table of ΔBPB per mechanism.

| Ablation | Implementation |
|---|---|
| `none` | baseline |
| `vsa` | `vsa_scale_q15 = 0` |
| `engram` | clear flags bit 1 |
| `lattice` / `lattice_coarse` / `lattice_fine` | clear flags bit 2 / zero `coarse_trigram` / zero `fine_residual` |
| `jepa` | clear flags bit 3 |
| `lanes` | zero `discrete_tables` |
| `s2_readout` | zero `discrete_s2_readout` |
| `induction` | requires a small code flag on the `4096/k` term (`native_capability_api.rs:1050-1067`) |

**Gate (per [D2](DECISIONS.md)).** A near-zero delta retires nothing on its own. Report
`|ΔBPB|` against the equivalence margin `epsilon`, the declared mechanism class, and the
decision-flip rate. A mechanism is retired only if it is a `count-table` or a correctly
wired `primary-carrier` that is measurably net-negative, or it carries a large resource
cost for no measured return. An `enabler` or `selector` with a near-zero delta is a
**wiring/instrument** problem: repair it and re-measure. Deliverable: one table in
`EVIDENCE.md` with the band, class, wiring status and flip rate per mechanism.

### E1.2 Fix the metric/serving divergence

The reported BPB and the served output come from **two different scorers**:

- **BPB** uses `evaluate_bpb_with_engram` (`jepa_trainer.rs:2216`), a probability model:
  `p_geom = root_prob × leaf_prob` interpolated with engram probabilities at
  λ = 0.85/0.75/0.60/0.40/0.30 (`:2397`, `:2455-2469`).
- **Serving** uses the additive `i32` scorer in
  `native_capability_api.rs::score_and_select_candidate` (7 summed terms, argmax/sample).

So the headline BPB does **not** measure the product. Required: an evaluation path that
computes BPB under the serving scorer (softmax/log-loss over the 64-candidate shortlist),
and both numbers reported side by side. Until then, no claim may tie BPB to serving
quality.

### E1.3 The role-vector inversion — fix and test properly

Diagnosed correctly by the Antigravity pass, **not present in the code**:
`HeadRole::from_head_index` (`vsa/attention.rs:99-106`) still derives `r_query` from
`salt_base` and `r_key` from `salt_base + 1`; every head binds `q` with `r_query` and
`k_j` with `r_key` (`:230/:241`, `:294/:306`, `:360/:372`, `:436/:442`); and
`test_multi_head_roles_orthogonality` (`:522`) *asserts* `d(r_query, r_key) ∈ 1850..=2250`
— the exact configuration that forces matching tokens to `d_H ≈ 2048 ≥ threshold` and
therefore weight 0.

Required: a shared role basis for identity-matching heads (`r_key = r_query`, or the
standard unbind-the-bundle formulation), the orthogonality assertion removed, and
`test_head_1_induction_circuit` (`:559`) replaced with a controlled test —
several induction pairs, permuted target order, shuffled-context control, many seeds,
and a requirement that the true successor beats **all in-window distractors**. The
current version compares one candidate against an out-of-window token and is a coin flip
fixed by the seed.

### E1.4 Recall vs generation attribution

Cheap, decisive, and required by literature norms (memory-as-contamination).
New binary `crates/uor-r4-core/src/bin/attribute-recall.rs`: index every k-gram
(k = 5…13) of `tinystories_train.u16` with a rolling hash; for generated output report
longest exact match against the corpus and the fraction of tokens covered by ≥k-gram
matches. **Gate:** if the fluent spans are ≥13-gram matches, the output is recall and
must be described as such.

### E1.5 The two provable checks

Unit tests, no model execution:

- `hamming_equals_angle_table` — verify `Metric::distance` equals the 120×120 angle-class
  table for all pairs, confirming the metric is a 9-valued class function.
- `byte_to_root_reachable` — assert the reachable-root census (33 of 120 at current
  initialisation) and report it as measured, not assumed.

### E1.6 Geometry controls (Card P4)

As written in `cards/P4-geometry-controls.md`: random phases vs zeta-zero phases;
random token→root assignment vs the learned/initialised assignment; unstructured count
baseline. Each a matched refit.

---

## 3. Stage 2 — The efficiency frontier (make the real claim)

The architecture's genuine advantage is **bounded computation**, and it is currently
neither measured nor maximised.

- Sweep shortlist size 64 → 32 → 16 → 8 (`native_capability_api.rs::populate_shortlist`,
  `Shortlist<64>` in `vsa/hierarchical.rs`) against held-out BPB. Find the knee.
- Instrument and report per token: candidates scored (**I3**), learned-store bytes read
  (**I2**), fraction of the store touched (**I1**), ops by class (**I4**), addressed-memory
  reads (**I5**).
- Instrument the mechanism-class and equivalence-margin reporting from `DECISIONS.md D2`
  (implemented in `ablate-prose`).
- **Interaction-aware attribution is required before any removal.** Single-mechanism
  deltas do not sum; a pairwise factorial for `jepa` × `s2_readout`, or a Shapley
  attribution over the mechanism set, is the correct instrument once the set is stable.
- Compare the same quantities against `llama.cpp` / `bitnet.cpp` on the same M1.

**Gate:** a stated frontier — "at BPB ≤ X, this model scores N candidates and streams B
bytes per token, versus V and P·bits for the comparator." A dense transformer must score
V candidates per token; this is the measurable difference.

---

## 4. Stage 3 — Resolution and codebook consistency

Two coordinated changes; neither alters the serving paradigm (still integer table reads).

**(a) Raise discrete resolution.** 120 roots (6.9 bits) and 480 fine clusters (8.9 bits,
`lattice_table.rs:23`) are the binding capacity limit. Options in increasing cost:
1. increase `MAX_FINE_CLUSTERS` and let the learned assignment fill it;
2. product codes `H4^k` — `k` learned 120-state factors give `6.9k` bits with exact
   integer composition (`Z[φ]` tables already exist);
3. a learned product-quantisation codebook with multiple sub-codebooks.

New module: `native_geometric/learner/product_code.rs`. Trained offline with floats;
served as table reads.

**(b) Make the VSA codebook derive from the learned representation.** Replace the fixed
random SplitMix64 basis with per-token codes derived from the learned embeddings (e.g.
bind/bundle the learned root quaternion and its companion into a hypervector, with
per-token residual). Artifact change:

- add `SECTION_VSA_CODES = 7`, `FLAG_HAS_LEARNED_VSA_CODES = 1u16 << 4`,
  `NUM_SECTIONS = 7`, `RGM_VERSION = 2`
- `Codebook::from_codes(&[u64])` constructor beside `on_demand`/`new`
  (`vsa/codebook.rs:23,36`)
- keep a version-1 reader; `MmapGeometricModel` currently uses a fixed
  `[RgmSectionHeader; NUM_SECTIONS]` (`binary_model.rs:932`) and must be version-gated

**Gate:** held-out BPB improves, **and** the E1.1 `vsa` ablation stops being ≈ 0. If the
heads still contribute nothing with a learned codebook, delete them rather than carry
them.

---

## 5. Stage 4 — Nonlinearity (the log-linear ceiling)

The serving scorer is **purely additive** — seven summed terms, no learned feature
interaction (`native_capability_api.rs:898-1003`). It is a log-linear model over sparse
discrete features, so its ceiling is n-gram/backoff quality. That is the single deepest
algorithmic limit, and it is why LUT-4 (Milestone 3) is the right next mechanism.

New module `native_geometric/learner/lut4/`:

- learnable connectivity (not fixed random — the previous failure had a ≤64-input cone
  per output bit and a shared 256-bit bottleneck)
- residual / identity initialisation to avoid vanishing gradients
- `soft` forward–backward as a mixture over the 16 two-input gates; `hard` export to
  truth tables; **report the discretisation gap and gate utilisation**
- insert as a learned nonlinear term over the shortlist feature vector; it must be a
  *selected* read (D0-a), not a dense contraction

**Gate:** improvement on **real** held-out text (not TinyStories-shaped), with
discretisation gap and utilisation reported.

**Sequencing is mandatory:** LUT-4 needs a good input representation to have a chance;
that is Stage 3. Running LUT-4 before Stage 3 reproduces the `hamming_policy` failure
(0/24, all ties).

---

## 6. Stage 5 — Ternary multiplier-free core (contingency)

If Stages 3–4 plateau against the Kneser-Ney 5-gram gate, the precedent-backed path is a
low-bit core trained offline with matmul and served by bit-serial/LUT accumulation
(T-MAC class), benchmarked against BitNet b1.58 2B4T on `bitnet.cpp`. This is compliant
under D0-a because the multiplier circuit is absent. Kill criterion as Card P3.

---

## 7. Stage 6 — The two contributions worth publishing

1. **E8 / icosian lattice quantisation.** The icosian ring is E8 under the trace form,
   and E8 is the optimal 8-dimensional lattice quantiser. Use it to quantise the learned
   core's weights and activations; measure bits/weight vs BPB vs bytes/token. This is
   the project's mathematics doing provably real work.
   New module: `learner/quantize_e8.rs`.
2. **Exactly decodable codes for VSA clean-up.** Replace the 120-landmark Hamming bank
   (provably a 9-valued angle table, not a code) with a genuine error-correcting code so
   clean-up becomes syndrome decoding rather than nearest-neighbour. Measure items
   recovered vs code length against standard clean-up.
   New module: `vsa/ecc.rs`.

---

## 8. Retire from the critical path

Zeta-zero phases (control first; expect null), `prime % 120` as anything but
initialisation, the 120-landmark Hamming bank, `r(i,j)`, the `E8 = H4×H4` companion *as
capacity* (provably zero), SpiralCore `Cl(0,6)`, gauge/bundle transport, and all
previously retired tracks (tropical, FMM, W33, Cayley–Dickson, 16-d Clifford,
Hopf-sector transport).

---

## 9. Predictions, for falsification

1. `vsa` ablation ΔBPB ≈ 0 at current resolution and codebook. (E1.1/E1.2)
2. A large fraction of fluent generated text is ≥13-gram recall. (E1.4)
3. After the role-vector fix alone, the heads still contribute little, because a random
   codebook supports only identity discrimination. (Stage 1 → 3)
4. P4 controls are null; random placement occasionally beats `prime % 120`. (E1.6)
5. Raising resolution + consistent codes improves BPB materially without changing the
   serving paradigm. (Stage 3)
6. The log-linear scorer cannot beat a matched KN 5-gram by 0.30 BPB at any table size
   that fits in RAM; LUT-4 nonlinearity is the only in-paradigm escape. (Stage 4)

---

## 10. Resource note

Stages 0–2 are evaluation-only (no training) and must be charged to the shared ledger as
model time. Stages 3–5 require retraining. The ledger at
`.uor-models/native-joint-learning-2026-09-04/model-time.json` is the authority; the
2026-09-18 full-corpus run is not yet recorded in it and must be reconciled before the
next training projection.
