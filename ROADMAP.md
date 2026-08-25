# Roadmap

Capability checklist. This is the **product** roadmap; the **research**
programme — what has been measured, refuted and left open — lives in
[docs/RESEARCH.md](docs/RESEARCH.md), and the two are tracked separately on
purpose. Research direction is set by measurement and changes faster than this
list does.

_Last reviewed: 2026-08-25 (#933 completed the exact normative-runtime evidence
branch with RATIFY and strict schema-2 admission). Prior: 2026-08-24 (#933
reopened S0 after the scorer-scope audit)._

> **Post-v0.1 intelligence sequencing lives in the
> [R4 Intelligence Completion Plan](docs/r4_intelligence_completion_plan.md)** —
> the readable mirror of GitHub programme root #820 (stages S0–S7 plus the
> cross-cutting F0 formal lane). This file stays the **product** capability
> checklist and defers the ordering of intelligence work to that plan.

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

The foundational #933 truth-boundary correction has completed its empirical
branch with RATIFY: ADR-0001's sole selector now consumes SKMX/PSIB across the
shared production adapters, and the exact canonical full census is bound into a
hardened schema-2 envelope that admits from an empty model store. This restores
the corrected S0 evidence boundary without relabeling #908's reference result or
claiming live-teacher parity.

Current order:

1. **Continue through the native dependency chain #932 → #931 → #839.** #932
   owns the exact-parallel, observable live-teacher BDD harness. #933 ran no
   teacher forward or parity marathon; its full-census quality evidence is
   teacher-free over already-recorded canonical labels. The repository BDD run
   was 124 / 124, but live-teacher fixtures were absent and those scenarios
   vacuously skipped.
2. **Keep the claim boundary fixed.** The #933 RATIFY is exact-artifact and
   teacher-forced-position evidence, not instruction following, reasoning, or
   free-running coherence.
3. **Deferred research and product backlog remains unchanged:** the #784
   context-code-convergence family, the #653 phase-2 formal remnants, the
   #759→#460 codebook-collision root cause, a future vacuity-robust
   route-attention contract, and eventual `uor-r4` alias removal.

## Landed

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
  "defer-open" items are currently untracked (see Next up item 3).
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

- [~] Text-based AI — compiles, serves, and ships as a verified release;
  **output quality remains the central research problem**. The #755 corpus-
  ordering fix plus the seeded-sampling decode default moved baseline answers
  from word-salad/digit-attractors to valid English sentences (15/15 on the
  declared canary, 2026-08-19), but distinctness is weak (#784: distinct
  prompts converge onto similar completions) and factual quality is
  research-grade; the canonical local bundles still predate the #755
  recompile. See
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
