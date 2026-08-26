# Roadmap

This is the product view of the authoritative
[Geometric Causal Decoder Roadmap](docs/geometric_causal_decoder_plan.md).
Measured, refuted, and frozen research remains in
[docs/RESEARCH.md](docs/RESEARCH.md).

_Last reviewed: 2026-08-25 (#948 architecture reset; GitHub programme root
#820, execution tracker #949)._

> **Project priority:** make the local geometric AI produce coherent
> free-running text. Reuse the working router memory, source causal runtime,
> tokenizer, trace taps, and `uor-matmul`. Replace standard self-attention
> progressively. Do not scale, prove, or lower the old graph product first.

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

1. **Now — #948:** land the repository-wide architecture reset.
2. **G0 — #950:** establish the coherent local `uor-matmul` control, bind the
   tokenizer-to-manifold memory adapter, and execute one trainable R⁴ mixer
   layer on every generated token.
3. **G1 — #951:** fit and qualify that layer and the bound memory adapter on
   teacher and student prefixes against disabled and permuted geometry/memory.
4. **G2 — #952:** progressively replace every source causal self-attention
   block while bounded free-running behavior survives.
5. **G3 — #953:** make the all-layer decoder the shared CLI/HTTP product path
   and feed persistent identity-scoped memory directly into geometric support.
6. **G4 — #954:** profile and optimize only the dominant measured bottleneck.
7. **G5 — #955:** freeze the bounded capability and decide whether to retain
   `uor-matmul` or open a separately justified lowering successor.

#950 is the sole immediate engineering issue after #948. Downstream work stays
unassigned until its blocker closes.

The old S5–S7 and F0 issues are closed not planned/not triggered. R4G1/TLA,
XOR/popcount, W(3,3), proof, conformance, and release work remains preserved as
historical evidence and comparators.

## Landed

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

- [~] Text-based AI — the repository has the local source-model components
  needed to restore a coherent control, working geometric memory/retrieval, and
  historical compiled runtimes, but no native geometric decoder has established
  coherent free-running product behavior. #950–#953 own that gap. Historical
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
