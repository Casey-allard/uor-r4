# Roadmap

This is the product view of the authoritative
[Geometric Intelligence Programme](docs/geometric_intelligence_programme.md).
The earlier
[Geometric Causal Decoder Roadmap](docs/geometric_causal_decoder_plan.md) is a
preserved #948–#958 sequencing record. Measured, refuted, and frozen research
remains in [docs/RESEARCH.md](docs/RESEARCH.md).

_Last reviewed: 2026-08-26 (post-#958 architecture authority; GitHub programme
root #820 and the #961 → #952–#955 → #962–#965 dependency chain are
reconciled)._

> **Project priority:** build source-free geometric intelligence in which the
> route is the data location. A pinned lexical codec supplies text boundaries;
> prime/n-let routes, fixed-zeta state, R4/S3 transport, the load-bearing
> project bridge `E8 = H4 x H4` serialized as the golden/Galois-coupled
> `H4 ⊕ phi H4` icosian pair, and recursive sentence/paragraph/conversation/
> global context choose output. Source weights are offline teachers only. The
> final serving path contains no transformer, dense matrix intelligence kernel,
> MoE, or sparse learned router.

> The goal is frontier-like useful capability on ordinary local hardware, but
> that remains an unproven research target. Spherical harmonics are the working
> picture for overlapping spin-state storage and transport; R4/S3 compute and
> Hopf/S2 observation are bounded charts of that larger routed field.

> **Quality-baseline reconciliation (#933/#934, 2026-08-25).** The legacy
> pinned-report tolerance rounds to 29.7%; it is not a universal 30% product
> floor. The exact canonical broad `R4G1Runtime` census is now 29.5203% versus
> same-position TLA 28.1214% (+13.988‰, 95% CI [11.057, 16.919]) and 26.0723%
> with SKMX/PSIB absent (+34.479‰ [31.681, 37.277]). #933 records RATIFY for
> that CID-bound bundle/population/decode and strict schema-2 admission. #908's
> 29.702% remains separate `R4Engine` reference/off-serving evidence. See the
> [#933 record](docs/normative_r4g1_quality_933.md) and
> [#934 genealogy](docs/canonical_quality_baseline_934.md).

## Next up (recommended sequencing)

Native GitHub relationships are the source of truth:

1. **GI-0 — retain #958 foundation:** keep the schema-2 manifest, bounded route
   lookup, worker evidence, controls, and optional operator fixtures at their
   exact claim scope.
2. **GI-1 / #961 S0 — lexical geometry:** pin the lexical codec without weights;
   add prompt-to-route and route-to-payload inversion, canonical hierarchy
   serialization, independent attention-artifact identity, incremental
   API-neutral state, and the fixed concrete `H4 ⊕ phi H4` realization of the
   project shorthand `E8 = H4 x H4`. Do not generate.
3. **GI-2 / #952 — recursive attention:** combine exact route identity with the
   full transported trajectory, hypersphere/window summaries, shared-factor
   retrieval, resonance, and Hopf accumulation across sentence, paragraph,
   conversation, and global route levels. Emit a coverage witness and establish
   anti-recall causal value before producing text.
4. **GI-3 / #953 — source-free grammatical inference/generation:** compile
   grammar and syntax into the attention-qualified route engine, then connect
   the library, CLI, HTTP, and chat surfaces without weights or dense models.
5. **GI-4 / #954 — correctness and abstention:** test held-out answer
   correctness, relevance, abstention, and causal use of required context.
   Teacher weights may label or compare offline only after the source-free
   report freezes.
6. **GI-5 / #955 — reasoning:** add bounded goal-directed route composition, branch
   comparison, intermediate constraints, and closure/contradiction controls.
7. **GI-6 / #962–#965 — product, cost, formal closure, and release:** integrate
   hive-memory chat (#962), optimize only the measured route-native bottleneck
   (#963), freeze the serving contract (#964), then explicitly activate only
   the release QA needed to qualify the product (#965).

The live issue bodies and native dependencies now mirror this sequence. #961
is the sole active newly assigned implementation stage; every later stage is
unassigned and blocked in order. Legacy tracker #949 is closed as superseded;
#958 is retained directly under programme root #820 as GI-0 foundation.

The old S5–S7 and F0 issues are closed not planned/not triggered. R4G1/TLA,
XOR/popcount, W(3,3), proof, conformance, and release work remains preserved as
historical evidence and comparators.

## Landed

- [x] **#958 fixed-zeta prime-route foundation** —
  *`RETAIN_STORAGE_RECALL_ONLY`, 2026-08-26*. Source-free algebra, the complete
  schema-2 manifest/rebuild witness, bounded candidate mechanics, matched
  controls, and worker compilation landed. The product probe and teacher
  comparison were correctly `NOT_RUN`; no lexical generation loop or API/chat
  caller existed. This is the retained foundation for GI-1/#961, not evidence
  against geometric intelligence. See the
  [qualification](docs/prime_route_attention_qualification_958.md).

- [x] **#932 exact-parallel live-teacher parity instrument** — *negative
  prerequisite outcome, 2026-08-25*. Deterministic scheduling, durable
  observability, planted negatives, and fail-closed preflight are implemented.
  The selected canonical 135M bundle predates #933's schema-2 production
  envelope, so the preflight refused before opening teacher weights and the
  live tuner/parity marathon did not run. See the
  [append-only record](docs/teacher_parity_parallelism_932.md).

- [x] **#933 normative R4G1 serving and deployed-quality reconciliation** —
  *RATIFY, 2026-08-25*. One `R4G1Runtime` candidate/token owner now reaches the
  production adapters; the canonical schema-2 bundle clears same-position TLA
  and the frozen +20‰ sections-absent gate with zero surface/binding/witness
  failures and strict empty-store admission. The result is scoped to its exact
  measured envelope.

- [x] **Wrap transformerless into r4** — *done 2026-07-18*. The full
  transformerless program is integrated into
  [`uor-r4-core::transformerless`](crates/uor-r4-core/src/transformerless),
  rebased onto the UOR substrate (`src/tless_uor.rs`: uor-addr addressing,
  `TlessAxis`, per-prediction `Grounded` witnesses), and exposed at
  `/api/tless/{predict,index,generate}`. The old repository is superseded.
- [x] **`up`/`run` locally** — `./uor-r4-cli` orchestrates download → compile →
  score → serve → interactive client in one command.
- [x] **API `/v1/chat`** — OpenAI-compatible `/v1/chat/completions`, plus
  `/v1/models`, `/v1/status`, `/v1/reload`, `/v1/corpus`.
- [x] **Web-based chatbot** — the browser dashboard (`index.html`) with engine
  selector, telemetry, tokens/sec speed metric, semantic map and dev-mode
  toggle.
- [x] **Attestable / audit logging / provenance** — UOR attestation envelopes
  (`uor_address`, `artifact_cid`, `store_cid`, `attestation_cid`),
  `POST /api/uor/verify`, per-turn audit log rendered by `r4 audit`.
- [x] **GNAF vendored (#653 phase 1, PR #742)** — the pinned
  `WASM-GEMM-GNAF` Lean4 proof project is vendored at
  `proofs/wasm-gemm-gnaf/` with full provenance
  (`docs/gnaf_import_provenance.md`). It is a proof that a WASM GEMM kernel
  is cost-optimal, unrelated to text generation; source and provenance only
  — not wired into any pipeline, not in the deployed dependency graph. Phase
  2 (integration matrix, PR #765) landed and #653 closed 2026-08-17; its
  "defer-open" items are not part of the active geometric-decoder programme.
- [x] **Ready-by-default serving epic (#655, closed 2026-08-19)** — A–F
  complete: uor-matmul-owned compile arithmetic; shared bundle loader +
  sidecar manifest/verifier; Production/Experimental engine profiles
  (Production default, r4g1-only admission, typed declines); **`r4` as the
  canonical served identity** on every surface (`uor-r4` a deprecated request
  alias); **seeded sampling as the decode default** (pinned seed, greedy
  opt-out; 15/15 valid on the declared in-domain canary vs 0/15 greedy); the
  CLI ask path applying the **same deployed D4 abstention policy** as the
  server tier (#811). Execution records:
  `docs/serving_default_flip_655_f.md` + the #655 close comment.
- [x] **Release v0.1 + pipeline (#741, closed 2026-08-19)** — tag-triggered
  workflow (draft release binding code SHA + contract version; native CLI for
  linux x86_64 + macOS arm64; wasm frontend), the packaged model bundle as an
  attested GitHub Release asset, and the explicit hard-verified
  `r4 install-release --tag v0.1` fetch, proven end to end against the live
  release. `docs/RELEASE_PIPELINE.md`.

## Capabilities

- [~] Text-based AI — the repository has the retained #958 route/storage
  foundation, working geometric memory/retrieval, and historical compiled
  runtimes, but no source-free geometric decoder has established coherent
  product behavior. GI-1/#961 owns the missing lexical route loop. Historical
  canaries and pointwise results remain scoped evidence, not a claim that the
  current product works. See
  [Which track can actually produce coherent text](docs/RESEARCH.md#which-track-can-actually-produce-coherent-text--the-honest-current-answer).
- [ ] Image
- [ ] Audio

Stretch:

- [ ] Video
- [ ] vLLM

## Tooling

- [ ] agentic harness
- [ ] agentic tooling (loop-based calls)
- [ ] `SKILL.md`
- [ ] collaborative prompting
- [ ] Tauri desktop shell

## Cloud service

- [ ] `deploy` (to cloud) — command-based; stack
- [ ] analytics
- [ ] persistent and transient VMs
- [ ] egress/ingress replacement

## Self-host

- [ ] deployment to your own cloud — tooling

## Hologram

- [ ] app SDK

## AI features

- [ ] Identity
- [ ] RBAC / policy
- [ ] analytics
- [ ] storage — document tokenization
- [ ] compression
- [ ] context (storage)
