# Roadmap

Capability checklist. This is the **product** roadmap; the **research**
programme — what has been measured, refuted and left open — lives in
[docs/RESEARCH.md](docs/RESEARCH.md), and the two are tracked separately on
purpose. Research direction is set by measurement and changes faster than this
list does.

_Last reviewed: 2026-08-19 (post-S1 verdict; issue states reconciled to GitHub)._

## Next up (recommended sequencing)

**The issue tracker is at zero open issues** — a first for this repository.
The serving epic (**#655**), distribution (**#741**, release **v0.1**
published), the ask-path abstention gate (**#811**), and the route-attention
decision (**#804**, S1 verdict: FAIL — instrument vacuous; operator retained
dormant with the record on #605) all closed 2026-08-19. What remains is
maintainer-decision work, each item needing its own new issue + contract
when picked up:

1. **Recompile the canonical local bundles with the #755 fix and take a
   fresh quality read** (the audit's E-1 run contract; the standing next
   action). The decode-default change made baseline output *valid* (15/15
   on the declared canary) but distinctness and factual quality remain
   research-grade, and the v0.1 bundle's own score report keeps it below
   the serving-admission quality bar (CLI-served today, refused by the
   server's r4g1 tier). A post-#755 recompile that clears its own bar is
   the path to a **server-admissible release bundle**, and it decides
   whether the #460-lineage codebook-collision work needs its own issue.
2. **The #784 family** — output distinctness (context-code convergence)
   plus the D4 semantic-OOD finding (#811): the sharpest open quality
   question now that route-attention is measured and closed. Needs a
   maintainer-approved pre-registered contract; the S1 verdict's
   temporal-smoothness finding (#605) is directly relevant input.
3. **Untracked remnants worth new issues when picked up:** the #653
   phase-2 "defer-open" items (GNAF cost/witness seam, xtask
   proof-integrity ports, Lean CI policy), the #759→#460
   codebook-collision root cause, a future vacuity-robust route-attention
   contract (excess-over-N2 primary, or restricted-forward parity — #605
   verdict), and the eventual `uor-r4` alias-removal cleanup (#655-F
   deprecation window).

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
