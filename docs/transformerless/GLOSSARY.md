# Glossary — R⁴ Holographic Graph Compiler

Phase 0 deliverable of `docs/r4_graph_compiler_implementation_plan.md` (§5).
This document freezes shared terminology. Where the graph-compiler plan (PDF §5, §9) and the
existing transformerless docs use different words for the same thing, both are listed and the
graph term is normative for new work. See the terminology bridge in the plan (§14).

## Core roles

- **Teacher (T)** — the pinned source model used only during offline compilation: a deterministic
  evaluation procedure `T: C → Δ(V)` for a pinned Hugging Face revision and deterministic execution
  mode. Accessed exclusively through the two-surface `TeacherOracle` (embedding table + next-token
  oracle), plus an optional compiler-only trace surface (hidden states, top-k logits).
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
- **Release-bundle manifest** (#655-C0, `crates/uor-r4-api/src/release_bundle.rs`) — the versioned
  `ReleaseBundleManifest` schema a packaged serving bundle declares: schema version, public model
  id, instruction-chat capability, ABI/contract version, pinned `uor-matmul` provenance,
  component digests, tokenizer identity. Schema and structural validation only as of #655-C0; no
  discovery/loading/serving code reads it yet (that is #655-C1).

## Recent Architecture & Engine Additions (Epic #201)

- **`tokenizer_cid`** — BLAKE3 hash of the loaded `tokenizer.bin` checked against `R4G1Header::tokenizer_cid` in `uor-r4-graph-format` (`verify_tokenizer_cid`), guaranteeing that loaded tokenizers match compiled graph artifacts and preventing silent index shifts.
- **`parse_store_strict_u32`** — The normative 32-bit integer token ID store parser in `uor-r4-core::runtime`. Replaces deprecated `u16` legacy store loading.
- **`FallbackRouter`** — The dynamic engine fallback router in `uor-r4-router` that wraps primary (`r4g1-graph`) and secondary (`transformerless-tla5`) engines, cleanly cascading on `UnmappedRegion` or `Pathological` statuses without dropping HTTP/WS payloads.
- **`EngineStatus`** — Typed status enum (`Success`, `UnmappedRegion`, `Pathological`, `Failed`) classifying engine inference outcomes.
- **`UorAttestationResult`** — Envelope wrapper for synthesis outputs containing verified BLAKE3 content-addressed CIDs (`uor_address`, `artifact_cid`, `store_cid`, `attestation_cid`), validated by `POST /api/uor/verify`.
- **`W(3,3) Phase Field`** — The 96-vertex $S^3$ graph canvas visualization in `index.html` rendering real-time Markov trajectories from WebSocket telemetry streams.
- **`ChatML Prompt Wrapper`** — Canonical ChatML format (`<|im_start|>system...\n<|im_start|>user...\n<|im_start|>assistant\n`) implemented in `scenarios.rs` (`encode_chat_prompt`) to format instruction-tuned teacher observations (`SmolLM2-135M-Instruct`).

