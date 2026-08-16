# Roadmap

Capability checklist. This is the **product** roadmap; the **research**
programme — what has been measured, refuted and left open — lives in
[docs/RESEARCH.md](docs/RESEARCH.md), and the two are tracked separately on
purpose. Research direction is set by measurement and changes faster than this
list does.

_Last reviewed: 2026-08-16._

## Next up (recommended sequencing)

Order for the currently open backlog, based on what actually unblocks
progress toward a working transformerless LLM — not effort or familiarity:

1. **[#744](https://github.com/UOR-Foundation/uor-r4/issues/744)** —
   reconcile the `instruction_eval_passed` quality gate with real generation
   quality (two manifests over identical compiled bytes currently report
   opposite numbers and both pass). Cheap and bounded, and everything
   downstream — bundle selection, knowing whether a #745 fix actually helped,
   trusting future compiles — depends on this gate being honest.
2. **[#745](https://github.com/UOR-Foundation/uor-r4/issues/745)** —
   root-cause the degenerate word-salad / empty-output generation on the
   best-available compiled bundle. This is the single open question that
   decides whether the transformerless/R4G1 track can produce coherent text
   at all — see
   [Which track can actually produce coherent text](docs/RESEARCH.md#which-track-can-actually-produce-coherent-text--the-honest-current-answer).
   Worth evaluating the dormant `#604` route-attention kernel
   (`R4RouteAttentionV1`, P-4-legal, already built and differentially tested
   but unused at serving time) as one candidate lever.
3. **[#653](https://github.com/UOR-Foundation/uor-r4/issues/653) phase 2**
   (GNAF adapter wiring) — lower priority; strengthens honesty/certification
   infrastructure but does not itself move generation quality. Phase 1
   (vendoring, #742) is landed.
4. **[#740](https://github.com/UOR-Foundation/uor-r4/issues/740)**
   (OpenAI-compatible API surface gaps) /
   **[#741](https://github.com/UOR-Foundation/uor-r4/issues/741)**
   (shipping/distribution) — explicitly deferred until #744/#745 land, per
   the maintainer's own prior redirect on #655. Shipping a product that does
   not yet generate coherent text is premature; revisit once it does.

## Landed

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
  2 (adapter wiring) is open, see Next up above.

## Capabilities

- [~] Text-based AI — compiles and serves; **coherent end-to-end generation is
  the open research problem** (#745), not just "quality is weak" — the best
  locally compiled bundle currently answers real questions in non-grammatical
  word-salad, even though offline per-position metrics show real signal. See
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
