# Roadmap

Capability checklist. This is the **product** roadmap; the **research**
programme — what has been measured, refuted and left open — lives in
[docs/RESEARCH.md](docs/RESEARCH.md), and the two are tracked separately on
purpose. Research direction is set by measurement and changes faster than this
list does.

_Last reviewed: 2026-08-18 (baseline audit; issue states reconciled to GitHub)._

## Next up (recommended sequencing)

Open issues are now exactly **#655** (serving epic; A–E and F0 complete, the
F flip blocked on #741 + a fresh quality read) and **#741** (distribution,
maintainer-deferred). The previously listed items #744, #745, #740, and #653
have all **closed** — see `docs/RESEARCH.md` and
`docs/project_baseline_audit_2026_08_18.md`. Recommended order now:

1. **Recompile the canonical local bundles with the #755 fix and take a fresh
   quality read** (no open issue yet — the audit's E-1 run contract). The
   compiler fix is merged and regression-tested, but every canonical bundle
   still predates it and remains degenerate at baseline (reproduced
   2026-08-18). This is the cheapest action that can move #655-F
   precondition 2, and it decides whether the #460-lineage codebook-collision
   work needs its own new issue.
2. **[#655](https://github.com/UOR-Foundation/uor-r4/issues/655)** — the
   remaining epic itself: C1e (loader reconciliation) and the F identity flip
   stay blocked per `docs/serving_default_flip_655_f.md`.
3. **[#741](https://github.com/UOR-Foundation/uor-r4/issues/741)** —
   explicitly deferred until the model-quality work lands; unchanged.
4. **Untracked remnants worth new issues when picked up:** the #653 phase-2
   "defer-open" items (GNAF cost/witness seam, xtask proof-integrity ports,
   Lean CI policy) and the #759→#460 codebook-collision root cause, both of
   whose parent issues are closed.

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
  2 (integration matrix, PR #765) landed and #653 closed 2026-08-17; its
  "defer-open" items are currently untracked (see Next up item 4).

## Capabilities

- [~] Text-based AI — compiles and serves; **coherent end-to-end generation
  remains the central research problem** (#745 closed 2026-08-17 with the root
  cause fixed in the compiler, #755; the canonical local bundles still predate
  the fix and answer real questions in non-grammatical word-salad at baseline,
  reproduced 2026-08-18, even though offline per-position metrics show real
  signal). See
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
