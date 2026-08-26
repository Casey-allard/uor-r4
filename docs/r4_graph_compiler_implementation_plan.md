# R⁴ Holographic Graph Compiler — Implementation Plan

> **Status banner (2026-08-25, #948): historical engineering reference; not
> current sequencing.** This document remains the original graph-compiler plan
> and preserves its phases, work items, proof obligations, and acceptance
> gates. The active architecture and work order are in the
> [Geometric Causal Decoder Roadmap](geometric_causal_decoder_plan.md), mirrored
> by GitHub #820/#949. Graph work resumes only through a post-viability trigger
> recorded there.

Status: historical engineering plan (2026-07-21); not an active commitment
Source of record: `docs/r4_holographic_graph_compiler_implementation_plan.pdf` (sections §1–§33 referenced throughout as "PDF §n")
Supersedes/extends: `docs/transformerless/TRANSFORMERLESS.md`, `docs/transformerless/PROOF.md`, `ROADMAP.md` p0

This plan converts the PDF's research direction — compiling a pinned Hugging Face teacher into a
transformer-free, multiresolution, overlapping **semantic graph** with an allocation-free,
integer-only runtime — into concrete phases, work items, proof obligations, and acceptance gates
grounded in the code that exists in this repository today.

---

## 1. Purpose and scope

Build an unsupervised behavioral cross-compiler with three strictly separated phases (PDF §3):

- **Compiler** (offline, may use floats/matmul/GPU): discovers multiresolution semantic regions,
  overlapping memberships, transition edges, and token-emission residuals from the teacher's
  representations and outputs. Every output is canonicalized and content-addressed (κ-pinned).
- **Runtime** (deployed, normative): per-token inference over immutable packed artifacts using only
  XOR/AND/OR/shift/rotate/popcount/integer add/compare/table reads. No transformer layers, no
  matrix multiplication, no floating point, no heap allocation in the hot path, no unbounded
  search, no mutable global state (PDF §16–§17).
- **Certifier** (offline instrumentation): measures teacher fidelity, semantic-neighbor
  preservation, stability, operation counts, allocation behavior, and artifact integrity. Never
  participates in inference.

Non-goals (PDF §2) are adopted verbatim: no exact-equivalence claim to the teacher, no semantic
atoms, no general-purpose parser/query engine/allocator in the runtime, and content CIDs (κ) stay
separate from semantic route codes.

---

## 2. Pre-commitment decisions (D1–D8)

These eight decisions came out of plan review. Each must be answered — or explicitly deferred with
an owner and due phase — before this plan is treated as committed. Recommendations are the plan
author's; the maintainer decides. Estimated total effort if all phases run: **31–57 weeks**
(PDF §28 estimates), which is why D1 and D8 come first.

| ID | Decision | Recommendation | Owner | Due |
|---|---|---|---|---|
| D1 | Commit scope + go/no-go targets | Tranche M0–M5 with a numerical checkpoint, not all 12 phases | maintainer | before Phase 0 |
| D2 | Reproducibility semantics | Canonical deterministic compiler mode for byte equality | maintainer | Phase 0, recorded in R4G1 RFC |
| D3 | Evaluation distribution + corpora | Natural-corpus mix, pinned CIDs/licenses, declared splits | maintainer | Phase 0, before baseline |
| D4 | Runtime fallback semantics | EXCT exact-residual lookup, then abstain (manifest policy) | maintainer | Phase 5 (chat UX when manifest lands) |
| D5 | Phase 6/7 scheduling | Trigger-gated on region count / measured bytes read | maintainer | revisit at M.V.G. checkpoint |
| D6 | Deployment target priority | **Revised 2026-07-21**: another platform is primary; wasm demoted to optional | maintainer | Phase 1 design (name target before freezing constraints) |
| D7 | Process: issues, branching, CI runners | File §9 backlog as GitHub issues; keep feature branch; pinned bench runner | maintainer | Phase 0 |
| D8 | Research vs. product priority | Plan serves the research thesis; product gate before Phases 8–10 | maintainer | before committing |

### D1 — Commit scope and go/no-go targets

Question: commit to all twelve phases, or to the evidence-producing tranche?
Options: (a) full plan now; (b) tranche M0–M5 (the first end-to-end generative graph) with a
minimum-viable-graph (M.V.G.) checkpoint at the end of Phase 5 and numerical targets agreed in
advance — teacher agreement, bits/token, artifact bytes, all on the D3 evaluation set — then
continue, redesign, or stop.
Recommendation: (b). Gate C only proves "not worse than the 31.7% baseline"; the checkpoint forces
the "useful enough to keep funding" question. Targets are written into
`docs/transformerless/BASELINE.md` during Phase 0 so they cannot drift.

### D2 — Reproducibility semantics

Question: what does Gate E's "byte-identical" mean when compiler-side floating point uses platform
SIMD matmul (`teacher.rs:159-356`, Accelerate/NEON/AVX2 variants) and κ-reproduction is
macOS-pinned today?
Options: (a) a canonical deterministic compiler mode (normative scalar FP path, pinned seeds, no
platform BLAS) — slower, but byte-reproducible across platforms; (b) per-platform byte pins;
(c) drop cross-platform byte equality and certify behavioral graph equivalence (PDF §15) instead.
Recommendation: (a) for all certificate-bearing artifacts, with (b) tolerated for local iteration.
Recorded in the R4G1 RFC in Phase 0; implemented as a compiler mode in Phase 2.

Implementation update (2026-08-15): #655-B2 removed the historical
Accelerate/NEON/AVX2 split from Llama/shared teacher projections, which now use
the pinned portable exact `uor-matmul` owner. GPT-2 Conv1D/MLP and tied
`lm_head` now use the separately versioned dense/2 certified-native/exact-
fallback owner; current-v2 QK/value attention retains its own execution record
and owner. Canonical mode continues to govern the remaining libm and
ordered-reduction choices.

### D3 — Evaluation distribution and corpora

Question: which distribution do all certificates certify on? The current baseline is
same-distribution on teacher-generated stories at ~10⁵ entries — a self-referential claim.
Decide: natural corpora (names, versions, licenses — these matter if artifacts are ever
redistributed, and they feed the PROV section), teacher-generated corpus mix, held-out split
rules, and corpus CIDs. Gates C, G, and K all inherit this declaration.
Recommendation: decide in Phase 0 before the baseline report; store corpus manifests under
`.uor-models/` and reference their CIDs in HEAD/PROV.

### D4 — Runtime fallback semantics

Question: concretely, what happens on Novel/Contradictory input when no teacher exists at runtime?
Options: (a) consult EXCT exact-residual evidence, then abstain with an explicit status
(recommended default); (b) fall back to the legacy geometric-Markov generator; (c) abstain only.
Recommendation: (a), manifest-declared per status (PDF §9). The `r4 chat` UX question — silent
abstention vs. surfacing status to the user — is decided when Phase 5 lands the manifest policy.

### D5 — Phase 6/7 trigger conditions

Question: Boolean routing synthesis and hardware-aware packing only pay off at large region
counts; today's exhaustive scan (4×256 classes) is trivially cheap.
Recommendation: make both phases trigger-gated, not calendar-scheduled. Phase 6 starts when the
region count exceeds a HEAD-declared threshold or shortlist-scan cost exceeds a measured fraction
of the per-token step budget; Phase 7 starts when measured bytes-read / cache-miss counters
violate the performance certificate. Triggers are re-evaluated at the M.V.G. checkpoint.

### D6 — Deployment target priority

Original question: is wasm a primary deployment target (the root crate ships `cdylib` and the
browser dashboard is a product surface)?
**Revised by the maintainer (2026-07-21):** the primary deployment target is a different platform,
to be named; wasm is lesser priority.
Consequence: caller-owned-bytes-first remains the design rule (it is target-neutral and keeps a
no_std core possible), but mmap/std adapters are acceptable in the primary target, and the wasm32
CI check is demoted from gate to optional. Before Phase 1 freezes R4G1 widths and the runtime
feature set, the maintainer names the target and its constraints (word size, endianness, alignment,
std vs no_std, mmap availability) are recorded here and in `BASELINE.md`.

### D7 — Process: tracking, branching, CI runners

Facts on the ground: origin is `github.com:UOR-Foundation/uor-r4`, `gh` is authenticated, and the
current branch `feature/proof-carrying-semantic-routing` already matches the PDF's working branch.
Recommendation: file the §9 backlog as GitHub issues mapped to phase milestones (done: issues
#11–#34); keep landing on the feature branch with the graph path cfg-gated rather than opening a
second long-lived branch; accept that reproducibility and benchmark gates need a pinned runner
(self-hosted or a fixed runner image) — otherwise they run as smoke-only checks and Gate D/E
evidence is produced manually on the pinned dev machine.

### D8 — Research vs. product priority

Question: even at Gate C success, the artifact is a distilled 135M-class graph LM — a strong
research result, possibly a mediocre chat product.
Recommendation: state explicitly that this plan serves the research thesis (proof-carrying graph
compilation). If product usefulness is also a goal, define a separate product gate (chat-usefulness
evaluation; the instruction-eval tooling already flagged missing in README.md:203-208) before
funding Phases 8–10.

---

## 3. Current-state assessment

The existing **transformerless** system in `crates/uor-r4-core` is a working first approximation of
this plan: it already compiles SmolLM2-135M into a mul-free table-native artifact with a witnessed
runtime. The graph compiler generalizes it from *flat quantized classes + graded store* to a
*multiresolution overlapping region graph with transitions and residual emission*.

### 3.1 What exists today (reuse inventory)

| Capability | Location | Notes |
|---|---|---|
| Teacher oracle (two-surface: embedding + next-token) | `crates/uor-r4-core/src/transformerless/teacher.rs:655` (`TeacherOracle`), `:772` (`HuggingFaceLlamaOracle`) | BF16 safetensors → f32 forward; pinned HF revision download |
| Observation corpus (resumable, content-addressed records) | `transformerless/compiler.rs:51` (`Corpus`), `:120` (`generate_to`) | 12-byte records: story, next, teacher argmax, teacher logprob (f32) |
| Multiresolution quantization (4 stages × 256 classes, D=288) | `compiler.rs:26-31`, `kmeans_rvq`/`hashed_rvq` `:281/:358` | Compiler-side only; output κ-pinned |
| Boolean context codes (36-byte sign-bit signatures) | `compiler.rs:248` (`SIG_BYTES`), `runtime.rs:108` (`sign_signature`) | The seed of the PDF's "compiled Boolean semantic code" H(x) |
| Hamming class assignment | `runtime.rs:95` (`hamming`), `:330` (`assign_plain`) | Exhaustive over 4×256 classes today; graph verifier will be masked-Hamming over shortlisted regions |
| Graded evidence store (code-prefix → next-token counts) | `runtime.rs:130` (`Store`), `:539` (`build_store`), TLS1 container `:559/:578` | Becomes the EXCT exact-context residual store + root priors |
| Hierarchical lexical context rows (unigram/bigram/trigram) | Root priors plus a compiler-side bigram weighting heuristic; no explicit trigram rows or lexical backoff | **Build new** (issue #362): packed context rows with trigram → bigram → unigram precedence |
| Mul-free integer kernel with op census | `runtime.rs:28` (`OpKernel`) | No multiply method exists; source-scan witness P-4 enforces this in CI (`transformerless/mod.rs:75-121`) |
| Allocation-free generation loop | `runtime.rs:497` (`generate_greedy_into`), `scenarios.rs:179/247` (`encode_into`/`decode_into`) | Caller-owned buffers; pattern to extend to `step()`; zero-alloc now machine-asserted by `tests/allocation_census.rs` |
| Packed containers | TLA3/TLA4 (`compiler.rs:585`, parse `:642`), TLS1 (`runtime.rs:559`) | Fixed-width LE; R4G1 succeeds them |
| Content addressing (κ = blake3 labels, UOR CIDs) | `compiler.rs:614`, `runtime.rs:625`, `src/tless_uor.rs:215-262`, `src/model.rs:207-248` | Deletion-attested store entries already exist |
| Proof/witness layer | Witness tests P-1…P-4 (`transformerless/mod.rs:28-137`), `Grounded` certificates + replay (`src/tless_uor.rs:563-617`), `uor-foundation-verify` | Extends to graph witness replay (Theorem 6) |
| Certification vs teacher | `transformerless/certify.rs:66-178`, `compare.rs` | Legacy baseline: **28.9% top-1 / 31.7% teacher-argmax agreement, 6.54 bits/token, 89,200 store keys** vs stories15M (PROOF.md P2); HF-path certificate missing (issue #34) |
| Reproducibility | `crates/uor-r4-core/tests/kappa_reproduction.rs` + `tests/fixtures/baseline_kappa.json` | Byte-identical recompilation, currently macOS-pinned |
| Allocation baseline | `crates/uor-r4-core/tests/allocation_census.rs` | Fresh (2026-07-21): 0 alloc/token hot path; 144,498 avg ops/token; store parse = 57k allocs (Phase 1 target) |

### 3.2 Gap analysis: PDF concept → current code → disposition

| PDF concept | Current state | Disposition |
|---|---|---|
| R4G1 sectioned artifact format (§18) | TLA3/TLS1 ad-hoc containers | **Build new** (`uor-r4-graph-format`); keep TLA3/TLS1 parsers read-only for fixtures |
| Fixed-width newtypes (`NodeId`, `ScoreQ`, …) (§20–21) | Raw `u16` tokens, `[u8;4]` codes, `usize` in `Prediction.depth` (`runtime.rs:369`) | **Refactor** — introduce newtypes; eliminate semantic `usize` across serialization boundaries |
| Region graph N, E_r, E_o, E_f, E_b (§5) | Flat 4-stage classes; no graph type | **Build new** |
| Masked-Hamming membership `popcount((H(x)^p_n) & m_n) ≤ r_n` (§5) | Unmasked Hamming to class sigs, nearest-wins | **Extend** — add masks, calibrated radii, top-M multi-membership; keep nearest-class fallback |
| Overlapping multiresolution cover induction (§6 st.3) | Single-scale RVQ stages | **Build new** (compiler) |
| Semantic transitions + residual emission scoring S(v) (§23) | Store counts + deepest-populated-class backoff | **Generalize** — root prior = level-0 backoff; ΔE/ΔT/ΔX residuals; ScoreQ fixed-point replaces f32 |
| Boolean routing synthesis, shortlist + exact verifier (§10) | Exhaustive 1024-class scan (cheap enough today) | **Build new** — trigger-gated (D5); exact verifier stays normative |
| Multi-timescale state (§8) | 8-token rolling window (`WINDOW = 8` Dyadic-Recency context truncation with zero-allocation slice `[t-7..t]`) | **Landed** — 8-token window bounds enforced; multi-timescale extension (local/segment/session) scheduled for Phase 8 |
| Resolution status (Supported/Boundary/BackedOff/Novel/Contradictory) (§9) | `ResolutionStatus` + `FallbackRouter` policy (`src/fallback.rs` cascade from `r4g1-graph` to `transformerless-tla5`) | **Landed** (PR #211) — explicit `ResolutionStatus` and `FallbackRouter` pipeline active |
| Proof-carrying witness, independent replay (§24) | Per-prediction `Grounded` + UOR attestation envelopes (`/api/uor/verify`) | **Landed** (PR #214) — UOR attestation envelopes with BLAKE3 CIDs |
| Epochs/patches/tombstones/route translation (§13) | Online `add_evidence`/`remove_entry` with κ attestation | **Formalize** — immutable patch epochs, bounded layers, compaction |
| Allocation-free `RuntimeState` (§16, §21) | Heap `Store`/`Compiled`; allocation-free *read* path exists and is now test-asserted; `add_evidence` allocates | **Refactor** — packed borrowed views + fixed-capacity state |
| no_std runtime | `std` throughout; wasm32 cfg-gates exist | **Refactor** — `graph-format`/`graph-runtime` cores no_std, caller-owned-bytes-first (target-neutral, D6); std adapters isolated |
| Anti-degeneracy / semantic validity (§7) | None | **Build new** (Phase 3) |
| Hardware-aware packing (§11) | None (byte layout is compile order) | **Build new** (Phase 7, trigger-gated D5) |
| Formal verification (§25, §27) | Witness tests only | **Build new** — Kani tranche, proof-status matrix (Phase 10) |
| CI for quality gates | Only GitHub Pages deploy workflow exists | **Build new** — fmt/clippy/nextest/doc/audit/reproducibility workflow (D7 runner policy) |

### 3.3 What is explicitly out of the migration path

`crates/uor-r4-router` (f64 geometric router, web dashboard, Markov fallback generator) and the
wasm facade stay as-is. The graph compiler succeeds the **transformerless** path, not the router.
During migration both paths coexist; the legacy transformerless module remains the Gate C reference
baseline until the graph runtime beats it on declared held-out metrics.

---

## 4. Target workspace layout (PDF §19, localized)

Seven new crates join the existing three. Dependency direction is strict:
`format → runtime → certify/cli`; `model-source → compiler → certify/cli`; `proof-model` depends on
`format`+`runtime` only.

| Crate | Contents | Migrated from / notes |
|---|---|---|
| `crates/uor-r4-graph-format` | no_std-compatible packed R4G1 types, parser, canonical serializer, checksums, versioning, two-stage validation, `GraphView` borrowed slices (caller-owned bytes primary — target-neutral, D6) | Generalizes TLA3/TLS1 (`compiler.rs:585`, `runtime.rs:559`) |
| `crates/uor-r4-graph-runtime` | Allocation-free runtime: fixed-capacity `RuntimeState`, normative scalar kernel, `ResolutionStatus`, witness generation, validated dispatch for arch kernels; primary deployment target per D6 | Generalizes `runtime.rs` (`OpKernel`, `assign`, `predict`, `generate_greedy_into`) |
| `crates/uor-r4-graph-compiler` | Observation orchestration, cover induction, overlap calibration, graph induction, residualization, Boolean synthesis, packing, checkpoint/resume, deterministic compile mode (D2) | Generalizes `compiler.rs` (Corpus, RVQ, train_cut, κ pinning); parallelism + memory budget per §4.1 |
| `crates/uor-r4-graph-certify` | Teacher comparison, held-out evaluation, allocation tests, op census, cache/perf benchmarks, witness replay, certificate emission | Absorbs `certify.rs`, `compare.rs`; same §4.1 rules |
| `crates/uor-r4-model-source` | HF source adapters (pinned download, safetensors load, tokenizer export); preserves the two-surface `TeacherOracle` and adds optional hidden-state/logit trace surface | Absorbs `teacher.rs` adapter, `scenarios.rs` tokenizer export, `src/model.rs` downloader |
| `crates/uor-r4-graph-cli` | `download`, `observe`, `compile`, `inspect`, `certify`, `compare`, `bench` | Absorbs `command.rs`; root `r4` binary delegates |
| `crates/uor-r4-proof-model` | Executable specification types + property tests; later Kani/Verus models without coupling production code to a proof tool | Extends witness tests P-1…P-4 (`transformerless/mod.rs:28-137`) |

Engineering standard (PDF §20) applies to all new crates: newtypes for `NodeId`, `SectionOffset`,
`TokenId`, `ScoreQ`, `Depth`, `Radius`, CIDs; no `unwrap`/panic on recoverable paths; focused error
enums at library boundaries; private fields except packed wire structs; **no unsafe** in the
portable runtime (later SIMD/mmap unsafe isolated in adapter modules with SAFETY comments + Miri);
borrowing over cloning; no wildcard imports outside tests.

### 4.1 Compiler-wide requirements — parallelism, determinism, memory budget

Applies to every `uor-r4-graph-compiler` and `uor-r4-graph-certify` stage (Phases 2–4, 6, 8+).
The runtime contract (§6) is unaffected: parallelism never reaches the deployed kernel.

**Parallelism model.** A work-stealing pool (e.g. `rayon`) lives in the compiler/certify crates
only; it must never become a dependency of format/runtime/proof-model. Per-stage shape:

| Stage | Parallel unit | Ordering rule |
|---|---|---|
| Observation generation / extraction | across sequences/contexts; each worker owns a decoding state, weights shared read-only | partition by content-addressed sample ID **before** any parallelism (PDF §22); per-shard outputs merged in shard-ID order |
| Cover induction (k-means family) | per-point assignment | centroid update = ordered reduction (per-shard partials combined in shard-ID order, f64 or fixed-point accumulators); iterations stay sequential |
| Graph construction | per-node edge building | dedup/canonicalization after a deterministic sort |
| Boolean synthesis (Phase 6) | per-candidate search | deterministic candidate ordering |
| Residuals / packing / certification | per-region streaming; per-eval-point | ordered aggregation for certificate statistics |

**Determinism, sharding, audit, and scaling rules (Gate E / D2).** See [parallel_observation_shards.md](file:///Users/casey.allard/uor-r4/docs/parallel_observation_shards.md) (Issue #170) for normative observation shard architecture, [compiler_concurrency_config.md](file:///Users/casey.allard/uor-r4/docs/compiler_concurrency_config.md) (Issue #168) for normative concurrency controls, [reproducibility.md](file:///Users/casey.allard/uor-r4/docs/reproducibility.md) (Issue #167) for normative parallel byte equality, [compiler_dependency_audit.md](file:///Users/casey.allard/uor-r4/docs/compiler_dependency_audit.md) (Issue #174) for the CPU-only compiler dependency and feature denylist audit, and [compiler_scaling_certificate.md](file:///Users/casey.allard/uor-r4/docs/compiler_scaling_certificate.md) (Issue #175) for the compiler parallelism benchmark harness and empirical scaling certificate schema.

1. Thread count is a pure performance knob: a compile at T=1 and at T=N must produce byte-identical
   κ. CI asserts this on a pinned mini-corpus (T=1 vs T=4 → identical artifact bytes).
2. No naive parallel FP reductions: FP addition is non-associative, so all reductions are ordered
   (shard-ID order) with f64 or fixed-point accumulators.
3. Llama/shared weight projections use pinned exact `uor-matmul`. Certificate-bearing compiles
   additionally select the canonical libm and ordered-reduction family; GPT-2's certified-exact
   dense/2 and current attention/2 sites are recorded separately rather than being relabeled by
   this historical D2 switch.
4. Seeds pinned; any shuffling derives from a seeded PRNG over sample IDs, never iteration order.

**Memory budget.** See [compiler_memory_budget.md](file:///Users/casey.allard/uor-r4/docs/compiler_memory_budget.md) (Issue #169) for the normative concurrency-aware memory budget and backpressure model. A `--memory-budget` flag derives shard sizes from
`peak ≈ M_weights + T × (S_shard + M_state) + M_cluster ≤ budget`:

- Weights: safetensors mmap'd. Default mode converts to f32 resident (~540 MB at 135M) for speed;
  low-memory mode converts per row from mapped BF16 (257 MB, touched pages only).
- Observations: streamed in shards, never a full in-RAM observation matrix; quantized to i8/sign
  codes at extraction time (f32 transient within a shard) — ~0.3 KB vs ~3 KB per observation.
- Clustering: streaming algorithms only (mini-batch spherical k-means, CLARA-style sampled
  medoids); no full n×n distance matrices, no naive HDBSCAN; distance computation always blocked.
- Cluster state: centroids only (k×d×4 B; e.g. 4096×288×4 ≈ 4.7 MB).
- Checkpoints: every stage spills content-addressed shards to disk and resumes (extends the
  existing corpus append-only resumability, `compiler.rs:120`).
- Worked example: 2 GB budget, 8 workers, mmap'd weights ⇒ ~190 MB/shard ⇒ ~700k quantized
  observations in flight per worker.
- Note: generation within one sequence is autoregressive and never parallelizes; parallelism comes
  from many sequences.

---

## 5. Phase plan (PDF milestones M0–M11, grounded)

Durations are the PDF's estimates (PDF §28). Each phase lists: objective, work items, exit
criteria. Gate letters reference §8 of this plan. Phases 6 and 7 are trigger-gated (D5), not
calendar-scheduled. Phase 5 ends with the M.V.G. go/no-go checkpoint (D1); Phases 8–10 are funded
only after it passes (D8).

### Phase 0 — Baseline and contracts (1–2 weeks)

Objective: freeze terminology, pin the baseline the graph must beat, stand up measurement harnesses.

- Write `docs/transformerless/GLOSSARY.md`: teacher, observation, region, membership, graph,
  runtime, certificate, novelty, fallback — aligned with PDF §5/§9 and existing doc vocabulary
  (signature, graded code, backoff, κ). **Done.**
- Claim language and mathematical notation are governed by `docs/formal_vocabulary.md`
  (issue #123): every equation is labeled Definition / Objective / Guarantee / Assumption /
  Empirical Criterion, guarantees carry a proof-matrix status, and
  `scripts/check_claim_wording.py` enforces the wording rule in CI. **Done (v0.1.0).**
- Baseline report `docs/transformerless/BASELINE.md`: current certified numbers (top-1 28.9%,
  agreement 31.7%, 6.54 bits/token, 89,200 keys — PROOF.md P2, legacy path), op census per token
  (144,498 avg ops/token fresh on the SmolLM2 path via `tests/allocation_census.rs`; ~1.8×10⁵
  cited legacy), artifact sizes/rate-distortion (P5 + fresh file sizes), allocation census
  (fresh: 0/token hot path; 57k allocs store parse), bytes read / cache misses and latency
  (pending pinned-runner bench setup). **Drafted.**
- **Declare the evaluation distribution (D3)** before any baseline number is recorded: natural
  corpora + teacher-generated mix, held-out split rules, corpus CIDs and licenses; store corpus
  manifests under `.uor-models/`. (Working assumption recorded; natural-corpus pick still open.)
- **Record the Gate E semantics choice (D2)** and the **M.V.G. checkpoint targets (D1)** in
  `BASELINE.md`. **Done (targets draft, awaiting maintainer confirmation).**
- Draft the R4G1 wire-format RFC (`docs/transformerless/R4G1.md`). **Done.**
- Decide and record region/membership limits to be manifest-declared (A, C, W, E, K, D from
  Theorem 4/9) as HEAD fields.
- Process (D7): file the §9 backlog as GitHub issues mapped to phase milestones; confirm the
  branching and CI-runner policy. **Done: issues #11–#34.**
- Threat model (`docs/transformerless/THREAT_MODEL.md`). **Done.**
- Exit: baseline committed; RFC draft reviewed; D1–D4, D7, D8 answered; gate thresholds table (§8)
  agreed.

### Phase 1 — Packed graph format and trusted parser (2–4 weeks) → Gates A, E (partial)

Objective: `uor-r4-graph-format` crate with R4G1 sections and two-stage validation.

- Fixed-width newtypes + `#[repr(C)]` packed records (`PackedNode`, `PackedEdge`, `RoutingOp`,
  `EmissionEntry` per PDF §21; exact widths chosen from measured graph sizes and recorded in the
  format spec). No `usize` in serialized structures. TokenId width must not bake in 16-bit
  assumptions (model-neutral emission adapters later, PDF §12).
- Caller-owned byte slices are the primary `GraphView` source (target-neutral, D6); mmap lives in
  a std-only adapter module (the only place `unsafe` may later appear, documented + Miri-covered).
  Before widths freeze, the D6 deployment target is named and its constraints (word size,
  endianness, std/mmap availability) recorded; a `cargo check` for that target stays green from
  the first commit (wasm32 check optional).
- Stage-1 structural validation: magic/version, section table, checked offset arithmetic, unknown
  mandatory-section rejection, canonical ordering, feature bits. Stage-2 semantic validation:
  edge targets in range, bytecode instruction validity, reverse-index consistency (Theorem 7),
  bound declarations vs. actual ranges (Theorem 8).
- `GraphView<'a>` borrowing validated immutable slices; never deserializes to heap objects.
- Canonical serializer + byte-reproducibility property tests; round-trip tests mirroring
  `tests/window_paths.rs::container_roundtrip_byte_identical`.
- Fuzz targets (cargo-fuzz): artifact parser, section validator, witness parser, malformed
  offsets, integer boundaries, unsupported versions.
- TLA3/TLS1 → R4G1 migration converter (compiler-side tool) so existing fixtures and the
  κ-reproduction test keep working; legacy parsers become read-only.
- Exit: malformed-artifact rejection suite green; canonical round-trips green; fuzz seeded in CI;
  converter produces a valid R4G1 from the pinned fixture set.

### Phase 2 — Overlapping multiresolution region prototype (3–5 weeks) → Gate C harness

Objective: replace flat 4-stage classes with a multiresolution overlapping region cover, measured
against the reference classifier.

- Observation pipeline v2 in `uor-r4-graph-compiler`: reuse `Corpus`/`generate_to` resumability;
  partition work by content-addressed sample IDs (PDF §22). Extend `TeacherOracle` (in
  `uor-r4-model-source`) with an optional trace surface: final hidden states, top-k logits,
  perturbation responses — compiler-only, cfg-gated.
- Compiler parallelism, determinism, and memory budget follow §4.1: sharded observation pipeline,
  ordered reductions, mmap'd weights, streaming clustering, checkpointed spills, T-invariance.
- Implement the deterministic compile mode chosen in D2 (normative scalar FP path / pinned
  backends) so certificate-bearing outputs can satisfy Gate E across platforms.
- Cover induction: recursive/spherical k-means or medoid/density variants over compiler-side
  representations + teacher distributions; candidate partitions scored by predictive entropy
  reduction, neighbor preservation, bootstrap stability, probe cost, artifact bytes (PDF §22).
- Calibrated overlapping membership: per-region masks + radii (masked-Hamming predicate of PDF §5)
  or top-M assignments; bounded multi-membership at each depth; **retain nearest-class fallback**
  (current behavior) as the backoff floor.
- Graph construction: parent/child refinement edges (E_r), lateral neighbor edges (E_o) from
  co-activation, explicit overlap nodes only where held-out gain justifies bytes (PDF §6 st.4, §22).
- Instrumentation: per-token operation and candidate-scan counts (extends `OpKernel` census),
  per-class Hamming distance distributions, calibrated radii dumps, frontier-width statistics.
- Reference classifier (exact compiler-side membership) frozen as the normative semantics; routing
  recall measured against it.
- Exit: compiled prototype graph for the pinned teacher; routing recall + frontier width reported;
  Gate C comparison harness runs graph vs. TLA3 baseline end-to-end (passing not yet required).

### Phase 3 — Semantic anti-degeneracy (3–6 weeks) → Gate G

Objective: regions must earn the name "semantic" (PDF §7).

- Perturbation corpus generation (unsupervised): token masking, span substitution, context
  truncation, word-order change, local counterfactuals (PDF §6 st.1).
- Multi-view objective J(C) with MDL penalty (PDF §7 formula) wired into cover induction and
  region retention/split decisions.
- Evaluation suites: paraphrase agreement, lexical substitution with preserved meaning, identical
  surface forms in unrelated contexts (polysemy separation), equivalent meaning across surface
  forms, truncation, irrelevant reordering, cross-document recurrence, tokenizer-boundary
  variation.
- Predictive-sufficiency measurement: divergence T(·|c) vs. T(·|R_d(c)) at broad/intermediate/
  full-cloud/residual-augmented depths → rate-distortion curve (bytes+ops vs. teacher information
  retained).
- Semantic-coherence certificate (separate from teacher-fidelity certificate): region reuse,
  invariance, boundary behavior, rare-context retention, anti-memorization evidence.
- Exit: semantic-coherence certificate emitted for the Phase-2 graph; regions failing
  reuse/stability thresholds are pruned or demoted to exact-context evidence.

### Phase 4 — Semantic transition and residual emission (3–5 weeks) → Gates A, C

*Normative Compiler Stage DAG & Concurrency Matrix: see [compiler_stage_dag.md](file:///Users/casey.allard/uor-r4/docs/compiler_stage_dag.md).*

Objective: the scoring model S(v) = B(v) + ΣΔE(n,v) + ΣΔT(m,v) + ΔX(X,v) (PDF §23).

- Forward transition edges E_f (active cloud → bounded next semantic cloud) and reverse indexes
  E_b built by sorting stable canonical edge IDs — Theorem 7 consistency by construction.
- Residual quantization: teacher log-probs → quantized log-domain ScoreQ residuals; root stores
  base priors B(v) (generalizes the level-0 backoff counts in today's store); children store
  corrections relative to parents; overlap nodes store interaction residuals only (Theorem 10
  non-duplication); exact-context store (EXCT, generalizes TLS1) captures remaining local evidence.
- Hierarchical lexical context evidence is compiled alongside the residual tables: unigram/root
  rows carry the context-free prior, bigram rows are keyed by the immediately preceding token,
  and trigram rows by the preceding two tokens. A prediction consults the most specific
  supported row first (trigram, then bigram, then unigram); sparse rows must fall back rather
  than contribute a misleading partial boost. Rows are story-boundary-safe, canonically sorted,
  and quantized into the same ScoreQ language as the root prior. This is a
  context-conditioned emission layer, not a replacement for masked-Hamming region similarity.
- Replace f32 semantic route scores in deployed paths with ScoreQ fixed point; remove `ctx_cb`
  f32 tables from deployed artifacts (certifier-only data moves to PROV/CERT or side files).
- Fixed-capacity top-K candidate structure with canonical tie-breaking (highest score, then
  lowest token ID — matches current `predict` tie rule).
- Emission blocks shared across regions where profiling shows duplication.
- Optional FMM1 translation rows: compiler-side low-rank factors are folded into
  a canonical fixed-point table; the borrowed runtime reader performs only
  sign-selected table reads and saturating add/sub, with artifact footprint and
  zero-allocation coverage recorded for the #290/#359 cost decision.
- Exit: witness replays S(v) exactly; graph prediction fixtures match reference runtime; Gate C
  measurement on declared held-out sets vs. 31.7%-agreement baseline.

### Phase 5 — Allocation-free graph runtime (2–4 weeks) → Gate B + M.V.G. checkpoint

Objective: `uor-r4-graph-runtime` implementing the PDF §16/§21 step contract.

- `RuntimeState<const ACTIVE: usize, const TOP_K: usize>` with fixed arrays (frontier,
  next_frontier, signature words, candidates); `step(graph, state, token, output, witness)`
  returning `Result<(), RuntimeError>`; no owned collections constructed anywhere in the API.
- Rolling context code updated incrementally (shift/XOR/add recurrence from prior state + entering
  + expiring token — generalizes the current window bundle, `runtime.rs:261-306`); never
  reconstruct the full context representation.
- Borrowed context-table lookup over the packed unigram/bigram/trigram rows, with fixed-capacity
  candidate output and exact trigram → bigram → unigram fallback. The deployed lookup uses only
  integer comparisons, table reads, and saturating add/sub; no allocation, multiplication,
  division, or float is permitted.
- Normative scalar kernel: extend the `OpKernel` discipline — complete operation set
  {xor,and,or,shl,shr,rot,popcount,add,sub,cmp,table read}, saturating where declared, **no
  multiply/divide/float**, census retained; source-scan witness test ported (P-4 pattern).
- Routing: shallow decision program → bounded shortlist → exact masked-Hamming verification of
  shortlisted regions only; deterministic widening and bounded exhaustive fallback per manifest.
- `ResolutionStatus` enum {Supported, Boundary, BackedOff, Novel, Contradictory} from calibrated
  distances/margins/support/disagreement/depth/residual availability (PDF §9, Theorem 12) +
  manifest-declared per-status behavior implementing D4 (default: consult EXCT, then abstain).
- Witness generation: bounded witness buffer (route, margins, edges, emission contributions, exact
  entry, selected token, op census) + independent replay verifier (Theorem 6) that needs no
  teacher.
- Multi-timescale state skeleton: token state wired now; local/segment/session states reserved
  with fixed capacities and update hooks for Phase 8.
- Counting-allocator tests: zero allocations per token step across every prediction API and
  witness mode (pattern already proven by `tests/allocation_census.rs`); no locks; recursion-free;
  Miri-compatible.
- Exit: Gate B in full; witness replay green on fixtures; op census within declared bounds;
  **M.V.G. checkpoint review (D1)**: compare against the targets recorded in Phase 0 — continue to
  Phases 6–10, redesign, or stop.

### Phase 6 — Boolean routing synthesis (4–8 weeks, trigger-gated — D5) → Gate H

Starts only when its D5 trigger fires (region count above the HEAD-declared threshold, or
shortlist-scan cost above the measured step-budget fraction). Objective: learned broad-to-fine
routing that shortlists, never decides (PDF §10).

- Compiler search over bounded sparse GF(2) XOR-polynomial and decision-test candidates:
  broad/stable/high-support tests first, refinement tests scoped within active regions (PDF §22).
- Emit deterministic decision DAG (ROUT section) + fallback exact verifier; decision program
  bytecode validated in Phase-1 parser.
- Certification: top-1/top-M region recall, false-negative rate, fallback rate, conditional
  fidelity without fallback, worst observed routing error — against the reference classifier.
- Runtime: confidence-triggered deterministic widening; bounded exhaustive fallback within
  manifest limits.
- Exit: Gate H recall/fallback thresholds met on held-out; exhaustive fallback exercised by
  adversarial suites.

### Phase 7 — Hardware-aware packing (3–6 weeks, trigger-gated — D5) → Gate D

Starts only when measured bytes-read / cache-miss counters violate the performance certificate
(D5). Objective: optimize bytes read and cache behavior, not arithmetic count (PDF §11).

- Profile-guided node/edge ordering from compiler co-activation statistics; hot edges/emission
  blocks on contiguous cache-line-aligned ranges; cold metadata separated.
- Evaluate SoA vs AoS, local offset widths, delta encoding, shared emission blocks, branchless
  acceptance, prefetch, mmap/huge-page/NUMA behavior — all deterministic for identical inputs.
- Byte-read cost model in the compiler objective (already in J(C) as runtime_cost term).
- Performance certificate: instructions, bytes read, cache misses, branch misses, latency per
  token with hardware metadata, stable fixtures, regression thresholds (criterion or
  iai-callgrind style; scalar reference path preserved as norm).
- Exit: Gate D — measured improvement without fidelity regression; layout certificate emitted.

### Phase 8 — Long-context multi-timescale state (4–8 weeks)

Objective: bounded persistent state beyond the 8-token window (PDF §8).

- Fixed-capacity local/segment/session semantic states with compiler-generated update programs
  (CODE section); caller-owned bounded storage; no level grows dynamically.
- Compiler learns state-compression operators retaining information useful to future teacher
  behavior, minimizing bytes read and ops.
- Optional external-memory references addressed by graph regions.
- Long-context certification: fidelity by dependency distance, entity reactivation accuracy,
  unresolved-reference retention, topic persistence, saturation behavior.
- Exit: long-context certificate; saturation behavior deterministic and documented.

### Phase 9 — Immutable epochs and patches (3–6 weeks) → Gate J

Objective: formalize today's online evidence/deletion into content-addressed patch epochs (PDF §13).

- Patch artifact: parent graph CID, added nodes/edges, score residuals, tombstones, compatibility
  limits, certificate. Lookup consults a manifest-bounded number of layers; periodic compaction
  emits a new canonical base.
- Route-translation evidence: retained/split/merged/removed region mapping between epochs.
- Provenance-deletion path: evidence at observation and membership-edge granularity (extends the
  existing κ-attested `delete_store_entry`, `src/tless_uor.rs:254-262`).
- Chain validation: deterministic newest-valid precedence; fork rejection unless compacted
  (Theorem 11).
- Exit: Gate J; patch-chain fuzzing and boundedness tests green.

### Phase 10 — Formal verification tranche (4–8 weeks) → Gate F

Objective: machine-checked evidence for the proof obligations (PDF §25, §27).

- `uor-r4-proof-model`: executable specification of step semantics, deterministic top-K,
  reverse-index construction, allocation-freedom assumptions, bounded ranges; differential tests
  runtime vs. spec.
- Kani proofs: fixed-capacity container invariants, checked arithmetic, validator range
  predicates. Evaluate Verus/Creusot for step-level properties; Lean/Coq only for stable math.
- Proof-status matrix (docs + CI-checked): every theorem/assumption → implementation link, test,
  formal result, or explicitly "unproven". No "machine-verified" wording without a CI-checked
  proof artifact.
- Exit: Gate F.

### Phase 11 — Architecture acceleration (ongoing) → Gate D profiles

Objective: AVX2/AVX-512/NEON kernels behind validated runtime dispatch (PDF §17, A5).

- Precedent exists: `assign_plain` bulk-popcount fast path witnessed equal to kernel path
  (PROOF.md P1). Historical teacher NEON/AVX2 backends were retired when #655-B2 made pinned exact
  `uor-matmul` the sole teacher projection owner; they are not a current acceleration precedent.
- Each specialized kernel: property-based equivalence over finite primitive domains + differential
  tests over runtime fixtures; optional disable flag; scalar safe Rust remains normative semantics.
- Unsafe confined to adapter modules with SAFETY comments + Miri.
- Exit: per-arch performance certificate profiles; equivalence evidence in CI.

---

## 6. Runtime contract and R4G1 summary (normative targets the phases implement)

Normative contract definition: `docs/transformerless/INFERENCE_OPERATION_CONTRACT.md` (v0.2.0;
0.1.0 is the initial historical version),
with machine-readable binding in `uor-r4-graph-format::inference_contract`.

Adopted from PDF §16–§18, §21 without dilution:

- `step()` is synchronous, deterministic, allocation-free, recursion-free, lock-free; parallelism
  only in compiler/certifier unless a deployment profile proves otherwise.
- Allowed ops: word XOR/AND/OR, shifts, rotates, popcount, int add/sub, compares, declared
  saturating arithmetic, indexed/sequential table reads. Multiplication by powers of two = shifts;
  other weights = fixed-point residuals or precomputed tables.
- Frontier entries: region ID, ScoreQ, margin, depth; max frontier width is an artifact-declared
  constant. Per-step work bound O(D + A·C·W + A·E·K) (Theorem 4).
- R4G1 container: versioned; explicit endianness/widths/alignment/checksums; unknown mandatory
  sections reject; offsets section-relative and checked; domain values fixed-width, never usize;
  deployed view borrows slices, never deserializes to heap objects.
- Scoring: S(v) = B(v) + Σ_{n∈A} ΔE(n,v) + Σ_{m∈F} ΔT(m,v) + ΔX(X,v) over sparse residual tables;
  candidate tokens from the union of bounded emission lists; full-vocabulary scoring only in a
  certified fallback mode. Canonical tie-breaking; greedy first; sampling only via caller-supplied
  deterministic RNG + separately specified integer routine.

---

## 7. Proof obligations → implementation mapping (PDF §25)

| Obligation | Mechanism | Evidence artifact | Phase |
|---|---|---|---|
| A1 validated-only GraphView | two-stage validator, typed borrowed views | validator property tests + fuzz | 1 |
| A2 preallocated state/buffers | `RuntimeState` fixed arrays, caller-owned outputs | API review + counting-allocator tests | 5 |
| A3 no allocator reachable | call-graph audit, no growth containers | allocation tests + source scan | 5 |
| A4 declared integer semantics | wrapping/checked/saturating declared per op | kernel unit + property tests | 5 |
| A5 scalar norm, kernels equivalent | differential + property equivalence | equivalence suite in CI | 11 |
| T1 allocation freedom | A2+A3 | counting-allocator harness | 5 |
| T2 operation-set conformance | kernel-only arithmetic | census + source scan + disassembly audit (release check) | 5 |
| T3 determinism | immutable bytes, total orderings, no clocks/races | replay fixtures, repeated-seed runs | 5 |
| T4 bounded work | manifest constants A,C,W,E,K,D | bounds asserted in validator + step | 1,5 |
| T5 memory safety (safe Rust) | borrow checker, safe indexing | Miri; no unsafe in portable runtime | 5 |
| T6 witness replay soundness | independent recomputation | witness replay verifier + fixtures | 5 |
| T7 forward/reverse consistency | reverse index built by sorting canonical edge IDs | construction + validator check | 4 |
| T8 artifact validation soundness | checked ranges → typed views | validator proofs (Kani) | 1,10 |
| T9 frontier/candidate boundedness | fixed-capacity containers, bounded loops | Kani container invariants | 10 |
| T10 residual non-duplication | canonical contribution IDs, acyclic ancestry | verifier rejection tests | 4,10 |
| T11 epoch traceability | immutable ordered layers, evidence refs | chain validation tests | 9 |
| T12 resolution-status determinism | integer features + manifest thresholds | property tests | 5 |
| Empirical fidelity (§26) | pinned eval sets, declared metrics/thresholds | certificate with CIDs, CIs, slices | 3,4,8 |
| Compile-time determinism (D2) | §4.1 ordered reductions, T-invariance | CI: T=1 vs T=4 compile → identical κ | 2 |

Existing P1–P5 propositions (PROOF.md) map onto T2 (P1), §26 (P2), Gate E (P3), A5/P4
(TeacherOracle discipline), and Gate D/rate-distortion (P5). PROOF.md gets a successor section
once the graph path lands; the old propositions remain valid for the legacy baseline.

---

## 8. Acceptance gates (PDF §29) with concrete checks

| Gate | Requirement | Concrete verification |
|---|---|---|
| A Correctness | graph output == reference runtime on all fixtures; malformed rejected; witnesses replay | `cargo nextest run -p uor-r4-graph-runtime -p uor-r4-graph-format`; fuzz corpus green |
| B Runtime contract | 0 alloc/token; no float/multiply in kernel; no locks; bounded frontier/probes; Miri clean | allocation-test suite (pattern: `tests/allocation_census.rs`); kernel source scan + disassembly audit; `cargo miri test` |
| C Fidelity | graph ≥ TLA3 baseline on declared held-out metrics before replacing default | `r4 certify` graph vs. baseline report (top-1, agreement, bits/token, declared CIs); HF-path certificate tooling per issue #34; M.V.G. checkpoint targets (D1) reviewed at end of Phase 5 |
| D Performance | fewer instructions/bytes/latency per token at equal fidelity | criterion/iai benchmarks with hardware metadata + regression thresholds (pinned runner per D7) |
| E Reproducibility | identical pinned inputs → byte-identical graph + certificate, per the D2 semantics (byte equality on the canonical deterministic compiler; behavioral equivalence across platforms otherwise); T-invariant compile per §4.1 | deterministic rebuild test in CI (extends `kappa_reproduction.rs`); T=1 vs T=4 mini-corpus compile → identical κ |
| F Proof status | every theorem/assumption linked to test/formal result or marked unproven | proof-status matrix CI check |
| G Semantic validity | reuse, perturbation stability, sufficiency, anti-memorization shown | semantic-coherence certificate thresholds |
| H Routing safety | shortlist recall + fallback limits met; verifier normative | recall certificate vs. reference classifier |
| I Novelty & safety | unsupported inputs detected/handled; adversarial inputs within bounds | OOD + collision + exhaustion suites |
| J Lifecycle | patches immutable, traceable, deletable, translatable | patch-chain tests + route-translation evidence |
| K Statistical rigor | distribution, n, CIs, slices, protocol on every empirical claim | certificate schema validation (distribution per D3) |

---

## 9. Immediate issue backlog (PDF §31), mapped to phases and code

Filed as GitHub issues #11–#33 (+ #34) on `UOR-Foundation/uor-r4`, mapped to phase milestones (D7).
The table is the source content.

1. R4G1 wire-format RFC + GraphView validation invariants → Phase 0/1 (issue #11; draft at `docs/transformerless/R4G1.md`).
2. Fixed-width newtypes; eliminate semantic `usize` (`Prediction.depth` at `runtime.rs:369`, router result structs) → Phase 1 (#12).
3. Counting-allocator tests around existing `Runtime` APIs → Phase 0/5 (#13; done: `crates/uor-r4-core/tests/allocation_census.rs`).
4. Per-token op + candidate-scan instrumentation → Phase 2 (#14).
5. Per-class Hamming distance distributions + calibrated radii compiler output → Phase 2 (#15).
6. Bounded multi-membership assignment, keep nearest-class fallback → Phase 2 (#16).
7. Reference `ActiveFrontier` + packed edge ranges → Phase 2/5 (#17).
8. Forward region transitions + reverse indexes from existing teacher corpus → Phase 4 (#18).
9. ScoreQ fixed-point residuals replace float route scores (`ctx_cb` f32 in TLA3/4, corpus f32 logprobs) → Phase 4 (#19).
10. Certificate schema with source/corpus/graph/metric/op/benchmark CIDs → Phase 0, emitted Phase 4+ (#20).
11. Deterministic artifact rebuild test in CI → Phase 1 (#21).
12. Executable proof model: allocation freedom, bounded ranges, deterministic top-K, reverse index → Phase 10 (#22).
13. Anti-degeneracy corpus transformations + evaluation harness → Phase 3 (#23).
14. Predictive-sufficiency + rate-distortion reports by depth → Phase 3 (#24).
15. Explicit `ResolutionStatus` + manifest fallback policy → Phase 5 (#25, policy per D4).
16. Shortlist top-M recall + fallback measurement vs. reference classifier → Phase 6 (#26).
17. Explicit unigram/bigram/trigram context rows with deterministic most-specific backoff →
    Phase 4/5 (#362).
18. Bytes-read / cache-miss / branch-miss counters in performance certificates → Phase 7 (#27).
19. Multi-timescale fixed-capacity `RuntimeState` design (local/segment/session) → Phase 5 skeleton, Phase 8 full (#28).
20. Tokenizer-neutral span and byte anchors in observation schema → Phase 2 (#29).
21. Immutable graph patch + route-translation RFC → Phase 9 (#30).
22. Source-bias amplification, rare-group erasure, provenance-deletion tests → Phase 3/9 (#31).
23. Threat model: semantic collisions, frontier exhaustion, candidate explosion, integer saturation → Phase 0 doc (#32; `docs/transformerless/THREAT_MODEL.md`), suites in 5/6/9 (Gate I).
24. Behavioral graph-equivalence + confidence-bounded empirical claims spec →
   `docs/transformerless/EQUIVALENCE_AND_EMPIRICAL_PROTOCOL.md` (Phase 0/3, #33, feeds D2).
25. Evaluation-report tooling for HF-compiled (SmolLM2) models → Phase 0/2 (#34; blocks Gate C harness).
26. Compiler-folded FMM translation table with allocation-free packed evaluation → Phase 4/5 (#359; accuracy and cost decision remains under #290).

---

## 10. Risk register (PDF §30, localized)

| Risk | Mitigation in this plan |
|---|---|
| Semantic collapse (geometry kept, prediction lost) | joint objective + held-out residual certification (Phase 2–3); Gate C |
| Graph explosion from overlaps | support/gain thresholds, bounded memberships, sparse materialization, byte penalty in J(C) |
| Double counting across memberships | **CONFIRMED 2026-07-22, not hypothetical**: literal Σ-over-cloud S(v) stacked correlated sibling-subtree residuals (one token won 98% of 2,000 probes at +91.7 nats; Gate C 0.3% vs 31.7% baseline). Root+residual decomposition alone does NOT prevent cross-sibling correlation when M>1 overlapping memberships share a refinement parent. **REDESIGN LANDED 2026-07-22 (issue #64)**: Rule 1 chain-telescoped residual application (ΔE only along the covered refinement chain root → deepest covered ancestor of the deepest-chained active region; ties to margin, then lowest region id — sibling stacking impossible by construction) + Rule 2 D4 EXCT precedence (support gate `EXCT_SUPPORT_MIN` = 5; status ExactContext/Graph/Novel witness-recorded, replay-verified; old formula retained as Gate C comparison column). Gate C (fixture corpus, 30,036 held-out): old Σ-cloud 0.3% / 76.96 bits → Rule 1 (no EXCT) 1.2% / 71.52 → Rule 1+2 31.7% / 9.86 vs TLA3 baseline 31.7% / 11.88. Win/loss: 1+2 vs old +9,497/−59; 1+2 vs baseline ±0 (identical argmax everywhere; bits differ by distribution shape). Rule 1 alone stays far below baseline — the 31.7% recovery is Rule 2 restoring the exact-context signal; the chain machinery governs contexts EXCT cannot answer (none on this corpus: status ExactContext 30,036/30,036) |
| Poor long-range behavior | Phase 8 multi-timescale states; long-context suites |
| Boolean approximation error | shortlist + exact verifier + fallback; recall certification (Gate H) |
| SIMD unsafety/nondeterminism | safe scalar norm; isolated adapters; equivalence tests; disable flag |
| Cross-platform compiler FP drift | D2 canonical deterministic compile mode for certificate-bearing artifacts |
| Parallel-compile nondeterminism | §4.1: content-addressed sharding, ordered reductions, T-invariance CI assert, scalar canonical mode |
| Compiler memory blowup | §4.1: mmap weights, streamed quantized shards, mini-batch clustering, budget-derived shard sizes, disk checkpoints |
| Overclaiming | proof-status matrix (Gate F); structural vs. empirical claims separated in every certificate |
| Semantic memorization (phrase tables) | multi-view invariance, MDL penalty, reuse thresholds (Gate G) |
| Novelty overconfidence | calibrated radii, support/disagreement signals, explicit status, fallback/abstain (D4) |
| Candidate false negatives | normative verifier, top-M recall certification, widening, bounded exhaustive fallback |
| Memory-bandwidth bottleneck | Phase 7 packing + byte-read objectives + cache-counter gates (triggered per D5) |
| Tokenizer artifacts mistaken for concepts | span/byte anchors, tokenizer-variation tests, emission adapters later |
| Patch-chain complexity | bounded immutable layers, compaction, route translation, chain validation |
| Bias/unsafe amplification | amplification/erasure metrics, per-slice certs, provenance granularity, deletion |
| Adversarial collisions | strict bounds, parser/bytecode fuzzing, checked arithmetic, no auth use of routes |
| Research success, product failure | D8 product gate before Phases 8–10; M.V.G. checkpoint (D1) caps sunk cost |

---

## 11. Decision records (PDF §32) — adopted

Adopted wholesale, with these repository-specific bindings:

- Semantics discovered from teacher behavior, not supervised ontologies (compiler is unsupervised
  w.r.t. external labels; the two-surface `TeacherOracle` discipline of P4 is preserved).
- Contextual occurrences clustered before lexical identities; multiple context-dependent
  memberships per token.
- Regions are multiresolution, overlapping, proof-addressable; not semantic atoms.
- κ content CIDs ≠ semantic route codes (existing blake3 κ infrastructure = the CID layer).
- Runtime = packed immutable tables + fixed-capacity state; no object graphs/dynamic containers.
- Prediction = semantic transition + sparse lexical emission + exact residual evidence.
- Scalar safe Rust is normative; optimized kernels are replaceable, equivalence-tested.
- Mathematical runtime guarantees and empirical fidelity certified separately (mirrors the
  existing "by construction / by witness / by measurement" split in PROOF.md).
- Predictive coherence alone ≠ semantic; multi-view invariance + reuse + sufficiency required.
- Boolean routing only shortlists; masked-Hamming verification normative until equivalence proof.
- Explicit resolution status on every step.
- Bounded multi-timescale states instead of unbounded history.
- Layout optimized for bytes read/cache, not arithmetic count.
- Source token IDs are lexicalization details, not permanent identities.
- Updates via immutable content-addressed epochs/patches with bounded lookup + route translation.
- Certificates include statistical uncertainty, slices, amplification/erasure analysis.

---

## 12. Migration and compatibility policy

1. New crates land alongside `uor-r4-core`; the legacy transformerless module keeps compiling and
   passing its existing tests (`window_paths`, `kappa_reproduction`, witness P-1…P-4) until Gate C
   replacement is certified.
2. TLA3/TLS1 artifacts remain readable forever (read-only parsers); a converter emits R4G1 from
   them for fixture continuity.
3. The `r4` CLI gains graph subcommands through `uor-r4-graph-cli`; existing
   `download/compile/store/certify/compare` UX is preserved as aliases during migration.
4. `uor-r4-router`, the wasm surface, and the web dashboard are untouched by this plan.
5. `QualityAttestation` gating in `src/model.rs:40` extends to graph certificates: a graph model
   manifest must carry a Gate-C-passing certificate before `ask`/`chat` accept it (mirrors the
   current instruction-chat evaluation gate).
6. Every phase lands behind its gate on the existing `feature/proof-carrying-semantic-routing`
   branch with the graph path cfg-gated (D7); a phase that misses its gate does not merge to the
   default path.

---

## 13. CI and engineering standard (PDF §20, §27)

New `.github/workflows/ci.yml` (the repo currently has no test CI — only Pages deploy):

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo nextest run --workspace` (+ `cargo test --doc`)
- `cargo check` for the D6 deployment target (wasm32 optional, not a gate)
- allocation-freedom suite (counting allocator; pattern in `tests/allocation_census.rs`)
- kernel source-scan + op-census invariants (ported P-4 pattern)
- deterministic rebuild / κ-reproduction (pinned fixtures; semantics per D2)
- compiler T-invariance: pinned mini-corpus compiled at T=1 vs T=4 → byte-identical κ (§4.1, Gate E)
- compiler memory-budget smoke: pinned mini-corpus compile under a small `--memory-budget` completes
  via shard spill, κ identical to unbudgeted run (§4.1)
- fuzz smoke runs (parser/validator/witness targets)
- dependency + license audit (`cargo deny` or `cargo audit`)
- proof-status matrix consistency check (Phase 10)
- benchmark regression thresholds on a pinned runner (Phase 7); without pinned hardware these run
  smoke-only and Gate D/E evidence is produced manually on the pinned dev machine (D7)

Testing strategy per PDF §27: unit (packed parsing, checked ranges, score arithmetic, tie-breaking,
rolling state, masked Hamming, top-K, witness serialization), property (round trips, canonical
bytes, kernel equivalence, membership invariants, no duplicate residuals, deterministic packing),
fuzz, differential (compiler vs. slow reference, runtime vs. executable spec, arch kernels vs.
scalar), allocation, formal (Kani first), performance, anti-degeneracy, adversarial, and
statistical certification suites.

---

## 14. Terminology bridge (old → new)

| Existing (TRANSFORMERLESS.md) | Graph-compiler term (PDF) |
|---|---|
| class / context class (stage codebook) | semantic region at a resolution depth |
| graded code `[u8;4]` | bounded membership set across depths |
| sign-bit signature (36 B) | compiled Boolean semantic code H(x) |
| Hamming-to-nearest-class | masked-Hamming region predicate + shortlist |
| store level-0 backoff counts | root token prior B(v) |
| deeper store levels / evidence counts | residual emission ΔE + exact-context store EXCT |
| code-prefix store levels | refinement edges E_r (parent/child) |
| (none) | overlap edges E_o, transitions E_f/E_b |
| `Prediction{token, depth, count}` | step output + `ResolutionStatus` + witness |
| κ (blake3 label) / UOR CID | content CID (unchanged) |
| TLA3/TLA4, TLS1 | R4G1 sections NODE/EDGE/ROUT/EMIT/EXCT/… |
| op census | op census (unchanged, extended with scan counts) |
| certificate (agreement, bits/token) | teacher-fidelity certificate + semantic-coherence certificate |
