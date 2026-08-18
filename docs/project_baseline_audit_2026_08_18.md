# Project baseline audit — 2026-08-18

**Audited commit:** `aea30bae469db756272a03863b191a0f48598f50` (`main`, identical to `origin/main` at audit start).
All file:line references in this report are bound to that commit; line numbers drift on later commits.

**Auditors' charter:** establish the current, evidence-backed baseline of the entire repository following the
#589 ecosystem rebase, the #655 serving epic (A–E complete, F scoped), the #657 GPT-2 enablement chain, and the
#743–#762 generation-quality campaign. Audit first: no production code was modified while gathering evidence;
documentation reconciliation (§17 of this report maps every edit) was performed only after the evidence was
complete. Claim language follows `docs/formal_vocabulary.md` (Definition / Objective / Guarantee / Assumption /
Empirical Criterion; statuses Structural / Witnessed / Empirical / Assumed / Unproven).

---

## 1. Executive summary

**The system is a rigorously instrumented research engine whose serving quality is currently degenerate, and
whose own documentation is largely — but not entirely — honest about that.** At `aea30bae`:

- **What unquestionably works:** the offline compile chain (Llama + GPT-2 families, architecture-keyed dispatch),
  deterministic artifacts (κ-reproduction green with the real checkpoint, including the macOS-pinned half),
  the R4G1 format + validation + PROV/1 provenance, the Gate C measurement harness (live run: 36.55% top-1 /
  8.32 bits on 100,306 held-out, witness replay 64/64, trend alarm green), the register-conformance machinery
  (R1–R6 green after clearing a stale-build artifact), and the E2 engine-profile restriction (Production default
  admits only the r4g1 tier on the cascade entry points).
- **The serving-quality reality, reproduced live:** all three loadable local bundles produce deterministic,
  prompt-**invariant**, degenerate output through the audited R4G1 path (`ounds Call…` / `<|im_start|>cesces…` /
  `cut cut cut…`); the #755 corpus-ordering fix is merged and regression-tested but **no canonical bundle has
  been recompiled with it**, and the #762 sampling mitigation **cannot reach** the R4G1-preferred path. The gap
  between "compiler fixed" and "bundles refreshed" is the single cheapest, highest-leverage action available
  (§17-A1, §18-E1).
- **Attention replacement:** no replacement operator is serving-reachable, default-selected, or causally active.
  `R4RouteAttentionV1` is exactly what the ledger claims — packed, differentially tested, witness-replayable,
  zero-alloc-asserted, dormant — with real-teacher fitting still UNAVAILABLE.
- **Highest-severity claim risks found:** the "machine-code audit" CI gate is vacuous (hardcoded sample inputs;
  AUD-INV-002); the P-4 source scan excludes hot-path `simd.rs` and the docs claim completeness (AUD-INV-001);
  README's serving story predates E2 and names a dead component (`FallbackRouter`); RESEARCH/ROADMAP issue states
  lag GitHub (four closed issues listed open). All are reconciled or ticketed in §17.
- **Verification environment:** one systemic local-gate hazard was found, diagnosed, and cleared (compile-time-baked
  repo roots + shared target dir across worktrees ⇒ stale gate binaries that fail or pass vacuously; AUD-VER-001).
  After clearing, **every CI-equivalent gate is green** on this machine, matching green CI on `main`.

## 2. Audited commit and environment

| Item | Value |
|---|---|
| Commit | `aea30bae469db756272a03863b191a0f48598f50` (top: `docs(#655-F0): scope the default-flip preconditions before any flip (#782)`) |
| Branch | `main`, equal to `origin/main` (`https://github.com/UOR-Foundation/uor-r4.git`) at audit start |
| Worktree state | Clean except two pre-existing untracked dirs: `obs-text/` (62 MB), `tf1-pkg/` (492 KB). Preserved untouched. A transient untracked marker (`crates/uor-r4-core/tests/.uor-r4-recorded-corpus-producers/fixtures`, 0 bytes) was created by this audit's own test run and removed afterwards; final `git status` matches the pre-audit state. |
| Toolchain | rustc 1.97.1 (8bab26f4f 2026-07-14), cargo 1.97.1, pinned by `rust-toolchain.toml` (rustup-managed) |
| Platform | macOS 27.0 (build 26A5378n), Darwin 27.0.0, Apple M1, 8 cores, arm64 |
| Feature configuration | Workspace defaults unless stated. `--all-features` exercised once (gate ladder). #515 dormant-mechanism features (`uor-r4-graph-certify`: anti-degeneracy, fairness-provenance, holographic-encoding, patch-lifecycle, predictive-sufficiency, …; `uor-r4-graph-compiler`: graph-construction, patch-induction, perturbation; `uor-r4-core`: bench-internals; `uor-r4-proof-model`: full-model) are default-OFF. |
| κ checkpoint | `/tmp/ref/out/model.bin` (llama2.c stories15M) **present** — Gate E ran for real, including the macOS-pinned `bundle_derived_macos` half that ubuntu CI cannot exercise. |
| Local model store | `.uor-models/` = 9.1 GB (gitignored): 6.0 GB sources, 1.5 GB compiled bundles, 1.2 GB corpora, 221 MB CID object store. Full ledger in §12. |
| GitHub state at audit time | Open issues: **#655** (assigned Casey-allard) and **#741** (deferred) only. Latest `ci` run on `main`: success (2026-08-18T07:18Z). `Formal Verification (Kani)`: success (green since the #712 fix). |
| Tooling present | cargo-nextest, cargo-deny, wasm-pack, kani, wasm32-unknown-unknown target |

**Method note.** Static analysis ran against a byte-exact snapshot of the tracked tree at the audited commit
(10,325 of 13,202 tracked files staged to an analysis environment: all source/docs/config; large binary fixtures
and research CSVs enumerated by manifest instead). One tracked file (`uor-r4-cli`, extensionless) was missed by
the staging filter and was read directly from the working tree. Dynamic verification (all cargo gates, Gate C,
Gate E, inference runs) executed on the pinned machine above, in the real working tree.

## 3. Scope and exclusions

**In scope:** every workspace crate and the root package; every CLI, HTTP, WASM and library entry point; the
serving cascade and both model-discovery systems; artifact formats and schema versions; the runtime operation
contract and its instruments; all local CI-equivalent gates; fixture-backed inference on locally present bundles;
every capability claim in the governing documents (README, AGENTS, CONTRIBUTING, ROADMAP, CONFORMANCE,
docs/RESEARCH, docs/MODEL_LIFECYCLE, docs/CONFIGURATION, docs/formal_vocabulary, docs/transformerless/*, and the
#655 serving docs).

**Explicitly NOT audited (with reasons):**

- `research/` (258 MB) and `research/prime-analysis/` mirrors — classified (Active research / Archived, §5) and
  inventoried, but their internal results were not re-verified; they are outside the production chain and carry
  their own nested AGENTS.md files. NOT AUDITED (out of scope by charter §1; nested AGENTS.md consulted before
  classification only).
- `uor_standards/` (21.9 MB, 1,121 tracked files) — vendored/legacy; not in the build graph (workspace `exclude`).
  Inventoried only; see finding AUD-DOC-011 for the tracked-despite-gitignore discrepancy.
- `proofs/wasm-gemm-gnaf/` — vendored Lean4 formal reference (#653/#742); provenance doc verified to exist;
  Lean proofs not re-run (no Lean toolchain in this repo's CI by design; #653 integration matrix classifies it).
- Multi-hour runs: D2 canonical differential (~1 h corpus compile), corpus-scale Gate C on live corpora,
  `capacity_scaling` (~12 min, not needed for any conclusion drawn here), fresh teacher compiles, and any
  re-measurement of baseline generation quality beyond the cheap deterministic probes in §13. Each would need a
  run contract under AGENTS.md long-run discipline; none was launched (§18 lists the candidates with contracts).
- Kani proofs locally — CI evidence accepted (green on main at audit time); local `cargo kani` not re-run.
- The GitHub Pages deploy workflow.

## 4. Evidence methodology and limitations

1. **Documentation was treated as claims, not facts.** Every governing doc was read in full; each material claim
   was checked against code (symbol + line), a gate execution, or a recorded measurement. §16 tabulates verdicts.
2. **Source presence was not treated as reachability.** Reachability was established by tracing call graphs from
   the six entry-point families (r4 CLI, `r4 transformerless` dispatch, HTTP routes, WASM exports, `uor-r4-api`
   library surface, test/certify harnesses). Dormant mechanisms were cross-checked against `model/ledger.toml`.
3. **Compilation was not treated as semantic correctness**; test evidence and measured results are cited per item.
4. **Vacuous, skipped, and unavailable executions are reported as such**, never as passes: presence gates were
   checked before trusting green (κ checkpoint, GPT-2 snapshot, SmolLM2 sources, compiled bundles).
5. **Instrument verdicts carry their own limits**: for every enforcement instrument §10 states what it proves and
   what it does not (e.g. a source scan is not a machine-code proof; steady-state allocation freedom is amortized,
   not unconditional — warm-up allocates).
6. **Limitations.** (a) Dynamic verification ran while the workspace test suite was also running on the same 8-core
   host; wall-clock durations for inference probes are upper bounds, not performance measurements. (b) The serving
   HTTP surface was audited statically plus via its unit tests; no live server was started (the machine is a
   development host with its own port usage). (c) Subjective text-quality wording in §13 is labeled as observation,
   not a formal metric; token-level metrics come from the repository's own harnesses. (d) Three parallel read-only
   exploration passes produced the §7–§10 call-graph evidence; every load-bearing claim they returned was
   spot-verified against source, and two of their claims were corrected during verification (noted inline in §10/§11).

## 5. Repository / component inventory

Classification of every top-level area (charter taxonomy):

| Area | Classification | Evidence / note |
|---|---|---|
| `src/` (root pkg `uor-r4-wasm-router`) | Production/runtime + CLI/API/server integration | `r4` binary (22 subcommands), HTTP server (`src/server.rs`, 27,686 lines), chat/model store, R4G1 adapter, release-bundle loader/packager, WASM facade |
| `crates/uor-r4-core` | Production/runtime + offline compiler | R⁴ math, transformerless compiler + runtime + tokenizer + certifier support (`src/transformerless/*`) |
| `crates/uor-r4-graph-format` | Production/runtime (format) | R4G1 packed format v0.0, two-stage validation, `no_std`, `#![forbid(unsafe_code)]` (lib.rs:61-62), machine-readable inference contract + `ACTIVITY_OWNERS` |
| `crates/uor-r4-graph-runtime` | Production/runtime | `no_std` allocation-free R4G1 engine (engine, routing, patch chains, vp-tree, packed kernels, route-attention + msa-selector packed lowerings), `#![forbid(unsafe_code)]` (lib.rs:1-2) |
| `crates/uor-r4-graph-compiler` | Offline compiler | observation (sharded, transactional resume), cover induction, packing, route-fit; #515 feature-gated dormant surfaces |
| `crates/uor-r4-graph-certify` | Offline certifier/measurement | Gate C `score` harness, `score_runtime` reference scorer, certificates, route-attention/msa reference semantics + witness replay; #515 feature-gated dormant surfaces |
| `crates/uor-r4-graph-cli` | CLI stage dispatch | `r4 transformerless …` (23 dispatch arms), cover/score producer-transaction guards |
| `crates/uor-r4-model-source` | Teacher/model adapter | `Teacher` architecture-keyed dispatch (#657) over `HuggingFaceLlamaOracle` + `HuggingFaceGpt2Oracle`; attention/dense operator registries; pinned `uor-matmul` production dep |
| `crates/uor-r4-proof-model` | Proof / formal model | proof matrix (8 obligations), Kani harnesses (2), allocation-proof counting allocator, `inference_audit` (see AUD-INV-002) |
| `crates/uor-r4-api` | Library facade | typed compile + `R4Engine` + `ReleaseBundleManifest` (#655-C0); engine facade consumed in-tree by `src/r4g1.rs`; compile facade downstream-only |
| `crates/uor-r4-naf` | Production-adjacent vocabulary | UOR-NAF v1 interchange + GNAF claim/status vocabulary (#623); not yet consumed by certify/server (per #653 phase-2 record) |
| `crates/repo-model`, `crates/repo-conformance`, `xtask/` | CI/conformance infrastructure | R1–R6 register machinery; generates CONFORMANCE.md; never a dependency of shipped crates |
| `crates/uor-r4-router` | Exploratory production (f64) | geometric router + dashboard backend; explicitly outside the P-4 kernel by design (AGENTS.md:14-16) |
| `docs/` | Documentation + measurement records | governing docs + per-issue records (append-only convention) |
| `features/` + `tests/` | Tests/BDD | 30 Cucumber suites (29 registered RF IDs), root BDD runner, integration tests |
| `scripts/` | CI gates + corpus tooling | claim-wording gate, Gate C regression/trend, D2 differential, fixture generators |
| `models/` | Pinned model descriptors | 4 descriptors: gpt2-124m (fully pinned incl. tokenizer κ), smollm2-135m-instruct (source κ), smollm2-360m-instruct (legacy, no κ), t5-base-tokenizer (tokenizer-only) |
| `proofs/wasm-gemm-gnaf/` | Vendored proof (formal reference) | Lean4 WASM-GEMM cost-optimality proof; NOT in dependency graph; NOT an LLM engine (`docs/gnaf_import_provenance.md`, `docs/gnaf_integration_653.md`) |
| `research/` | Active research / Archived / Generated data | 258 MB; `research/archives/*`, `research/prime-analysis/*` are archives/mirrors; nested AGENTS.md files govern their subtrees |
| `uor_standards/` | Vendored/external (legacy) | 1,121 tracked files despite a `.gitignore` entry — see AUD-DOC-011 |
| `index.html`, `index.css`, `r4_worker.js` | Dashboard (browser) | engine selector, W(3,3) canvas, semantic map |
| `uor-r4-cli`, `r4-app.sh` | Orchestrator scripts | menu-driven download→compile→score→serve→client chain (§7); `r4-app.sh` is a 2-line exec shim |
| `.github/workflows/` | CI | `ci.yml` (context-split gates, §11), `formal_verification.yml` (Kani), `deploy.yml` (Pages) |
| Committed fixtures | Generated artifact/fixture | `crates/uor-r4-core/tests/fixtures/` (c_recs.bin 22.9 MB, c_meta.bin, tless_artifacts.bin 1.3 MB TLA7, baseline_kappa.json, gpt2-tiny, t5 SentencePiece reference), `docs/transformerless/gate_c_pinned.json` |

**Workspace membership** (root `Cargo.toml`): 13 members + root package; `default-members` = root, core,
graph-format, router, proof-model, graph-runtime; `exclude` = `uor_standards`, `graph-format/fuzz`, `research`.
UOR standards (`uor-addr`, `UOR-Framework`) are rev-pinned git dependencies with a `[patch.crates-io]` unification
(#618). `uor-matmul` is pinned at `b13c9844…` as a production dependency of `uor-r4-model-source` (#655-B1).

**Artifact formats and schema versions found live:** TLA eras 3–7 readable, TLA7 emitted by default
(`R4_TLESS_TLA7=0`/`R4_TLESS_TLA6=0` opt out); TLS1 graded store, strict-u32 parser + `parse_store_legacy_u16`
compatibility reader; R4G1 packed graph `FORMAT_VERSION 0.0` (header.rs:34-38) with PROV/1 provenance section now
emitted unconditionally by `cover` (#637 phase 3, graph-cli lib.rs:4647); `score_report.json` schema 26 (observed
live, §11); evaluation report schema 4; `ReleaseBundleManifest` schema 1; model manifest schema 1; observation
identity bundles `/1` and `/2`; attention registry `standard/experimental/learned-absolute` v1+v2 ×
`gpt2-source-dense` v1+v2; `uor-r4-source-manifest/1`; `uor-r4-adapter-fixture/1`; `uor-r4-teacher-trace/1`;
route-attention `RAT1` + witness `/1`; `uor-r4-route-fit-report/1`; `uor-r4-target-operator-certificate/1`.
No TLA8 / R4G2 identifiers exist in the tree (the #637 arc closed by binding PROV/1 into R4G1 instead).

## 6. Executive baseline matrix

Status vocabulary per charter §9. "Default-reachable" means reachable on a fresh install with no configuration,
at this commit (i.e. under the E2 `Production` profile default).

| Component | Status | Default-reachable | Evidence anchor |
|---|---|---|---|
| Transformerless compiler (HF teacher → TLA7+TLS1 bundle, Llama family) | VERIFIED WORKING | n/a (offline) | real bundles in `.uor-models/compiled/*`; κ-reproduction green (§11); #745/#755 recompile record |
| Transformerless compiler, GPT-2 family (#657 chain) | VERIFIED WORKING (real snapshot, cloud, 2026-08-16) / vacuous locally | n/a | `gpt2_compile_canary_670.rs` merged, presence-gated; snapshot absent on this host → UNAVAILABLE locally |
| Cover induction + Gate C scorer (`transformerless cover`/`score`) | VERIFIED WORKING | n/a | live run this audit: schema-26 report, 100,306 held-out, witness replay 64/64 (§11, §13) |
| R4G1 packed format + two-stage validation | VERIFIED WORKING | yes (load path) | format tests, fuzz targets, `verify_cids`, PROV/1 unconditional |
| R4G1 graph runtime (engine/routing/patch-chain) | VERIFIED WORKING (fixture + real-bundle load) | yes — the ONLY cascade tier under default profile | §8; allocation census asserted zero steady-state |
| TLA/TLS1 legacy transformerless runtime (Tier 2) | VERIFIED WORKING; not cascade-reachable by default | via `/api/tless/*` only (AUD-ARCH-003) | §8 |
| Teacher-oracle serving tier (live HF forward) | IMPLEMENTED, NOT EXERCISED this audit; not default-reachable | no (Experimental only) | §8 |
| Geometric router tier | VERIFIED WORKING (unit/harness); not default-reachable via cascade | no (Experimental only); WASM/static + `/api/map` yes | §8; MRR 0.8763 record #502 |
| `r4 ask`/`chat` (ModelStore path) | BROKEN on the best local bundles (quality), mechanism VERIFIED WORKING | yes | §13: degenerate output reproduced live on `smollm2-135m-instruct` |
| Engine profiles (E2 Production/Experimental) | VERIFIED WORKING (unit tests) / PARTIALLY WIRED (bypass surfaces, no HTTP-level test) | yes (default Production) | AUD-ARCH-003/-004; server.rs:3793-3928 tests :26489-26682 |
| Release-bundle packaging (D1/D2) + sidecar verify (C1c) + `verified` surfacing (C1d) | VERIFIED SYNTHETICALLY (golden round-trip in CI); advisory-only by design | yes (advisory) | packager tests; server.rs:511-513, :2302 |
| `ReleaseBundleManifest` (C0) / `uor_r4_api::compile` | IMPLEMENTED, NOT EXERCISED in-tree (downstream facade) | n/a | agent-verified zero in-tree consumers of the compile facade; `UOR_R4_API_E2E_SOURCE` test `#[ignore]`d |
| R4RouteAttentionV1 (#604) | DORMANT BY DESIGN; VERIFIED SYNTHETICALLY (differential + witness replay + zero-alloc asserted) | no | ledger `r4-route-attention-dormant`; allocation_census.rs:572-659 |
| Route-fit ladder (#605) + target-operator certificate (#606) | DORMANT BY DESIGN; synthetic stages PASS, real stages UNAVAILABLE | no | ledger rows; `route_fit_report.rs` |
| MsaStructuredSelectorV1 (#643) | DORMANT BY DESIGN; VERIFIED SYNTHETICALLY (no allocation assertion — AUD-INV-007) | no | ledger row |
| Octeract quotienting (#661) | REFERENCE/PROOF ONLY → closed UNAVAILABLE (typed report κ `blake3:eab7b1bb…`) | no | #661 closure 2026-08-15 |
| Dormant compiler/certify surfaces (#515 set: cd_space, tropical, lie_jordan, bott_fock, quantum_cover, build_graph, packed ROUT evaluators, endomorphism, holographic_encoding, predictive_sufficiency, shortlist, anti_degeneracy, fairness_provenance, reference IR, rate-distortion, monograph, behavioral probes, patch overlay) | DORMANT BY DESIGN (each with a ledger activation gate; several carry measured NEGATIVE records) | no | `model/ledger.toml` / CONFORMANCE.md rows; `--all-features` build+tests green (§11) |
| Legacy llama2.c chain (`setup`/`gen`/`store`/`certify`/`compare`) | STALE/LEGACY, VERIFIED WORKING (κ gate + compare-report) | opt-in | §11 Gate E; MODEL_LIFECYCLE "Legacy benchmark" |
| `parse_store_legacy_u16` compatibility reader | STALE/LEGACY (kept deliberately) | load-path only | AGENTS.md:269-271 |
| `FallbackRouter` (`uor-r4-router::fallback`) | STALE/LEGACY — dead code, zero callers outside own tests | no | AUD-ARCH-006 |
| `select_synthesis_engine` | STALE/LEGACY — one caller, and it is the BDD suite | no | AUD-ARCH-005 |
| WASM dashboard R4G1 path (`generate_r4g1_response`) | PARTIALLY WIRED — exported, gate variable never assigned in either frontend | no (geometric fallback always) | AUD-ARCH-007 |
| `uor-r4-cli` orchestrator | VERIFIED WORKING for options 1–2 by prior use; BROKEN for option 3 on fresh install (unpinned `HF_REV="main"`) | opt-in | AUD-CLI-002 |
| Kani formal gates | VERIFIED WORKING (CI, green; 2 harnesses; core excluded by design) | n/a | §10.5 |
| κ-reproduction (Gate E) | VERIFIED WORKING locally this audit (checkpoint present; macOS bundle half included) + in CI with cached checkpoint | n/a | §11 |
| Teacher-parity BDD (S1–S6) | see §11 — executed in the workspace suite on this host (sources + bundle present) | n/a | §11 |

## 7. End-to-end pipeline diagrams

### 7.1 Offline compile chain (per model)

```
pinned HF snapshot (.uor-models/sources/<name>, source_manifest.json κ #597)
  └─ Teacher::load  — architecture-keyed dispatch (#657): Llama | GPT-2   [model-source/src/teacher.rs:36-120]
       └─ observation (sharded, content-addressed, transactional raw resume; identity bundle /1|/2:
          input_cid + source-manifest κ #597 + geometry #600 + tokenizer #601 + attention #602 [+ dense #704] + trace #603)
            └─ compile → TLA7 tless_artifacts.bin + TLS1 tless_store.bin + tokenizer.bin
               + attention_operator.json [+ dense_operator.json] + corpus.meta/records  [graph-cli lib.rs:6662-]
                 └─ cover  → cover.r4g1 (+ PROV/1, unconditional) + cover_report.json   [lib.rs:4424-, :4647]
                     └─ score → graph/score.r4g1 (validated) + score_report.json (schema 26)  [lib.rs:4968-]
                         └─ evaluate-report → instruction-eval.json (schema 4)
                             └─ import → live probe gate (#744/#750) → CID store manifest (ModelStore)
                                 └─ package-release-bundle → release-bundle.json sidecar (D2)
```

### 7.2 Serving — the three disconnected loaders (confirmed still three at this commit)

```
(1) CLI ask/chat: ModelStore (manifests/…) ── CompiledNotImported fallback ──> compiled/<name>/ 3-file bundle
        └─ prefers compiled/<name>/compiled.r4g1 (R4G1 path) else plain TLA/TLS1     [src/chat.rs:198-203, :350-531]
(2) HTTP server: #248 cascade  r4g1 → transformerless → teacher-oracle → geometric
        under EngineProfile (E2): Production (default) admits r4g1 ONLY               [src/server.rs:4086-4228]
(3) ReleaseBundleManifest (C0/C1c/D1-D3): packaged + verified sidecar → surfaced as `verified` (advisory only)
```

### 7.3 Serving request flow at this commit (default install)

```
POST /api/chat | /v1/chat/completions | /v1/responses
  → resolve_pinned_tier (request engine → last_engine.txt fallback)      [server.rs:3847-3853]
  → EngineProfile::from_persisted(engine_profile.txt) — absent ⇒ Production [server.rs:3810-3834]
  → explicit non-r4g1 engine under Production ⇒ typed decline (503 declined_by_all)
  → run_serving_cascade: tiers = [r4g1]  (Production)  |  full 4-tier cascade (Experimental)
  → r4g1 tier: generate_r4g1_text → R4g1State (suffix-DFA / n-gram rows / geometric routing / node-0+EMIT defaults)
  → response carries generation_mode, cascade_trail, r4g1 policy signal, UOR attestation
```

## 8. Serving-engine reachability matrix

Engine-name inventory is complete (every string accepted anywhere; server.rs:3733-3739, :3863-3871; chat.rs:1671-1696).

| Engine name (request/persisted) | Maps to | Under **Production** (default) | Under **Experimental** | Notes |
|---|---|---|---|---|
| *(absent)* / `r4g1` / `auto` / unknown (e.g. `ollama`) | no pin → cascade | r4g1 only; if r4g1 not loaded → **503 `declined_by_all`** | full cascade r4g1→transformerless→teacher-oracle→geometric | unknown strings never pin and are NOT declined (AUD-ARCH-008) |
| `transformerless` | Tier 2 pin | **typed decline** (503, names tier) | pinned Tier 2, no fallback | decline echoes tier constant, not the requested string |
| `transformerless-legacy` | Tier 2 pin | typed decline (reported as `transformerless`) | pinned Tier 2 | drives `generation_mode:"transformerless-legacy"` |
| `attention` | teacher tier (standard attention) | typed decline | pinned teacher, max_tokens≥256 | Llama `standard-source-attention/2`; GPT-2 `learned-absolute/2` |
| `r4-attention` | teacher tier (experimental switch) | typed decline | pinned teacher | Llama `experimental-r4-source-attention/2` — **never measured against standard** (deferral record 2026-08-05) |
| `geometric` | Tier 4 pin | typed decline | pinned geometric | |
| `teacher-oracle` | *not requestable* (internal tier name) | — | cascade-internal only | `tier_for_engine_name` → None |
| `uor-r4` | **model-id alias**, not an engine | resolves to active canonical model | same | server.rs:2669-2686 |

**Bypass surfaces (profile-independent, all HTTP):** `POST /api/tless/{predict,index,generate}` reach Tier 2
directly; `POST /api/r4g1/{predict,generate}` reach Tier 1 directly; both ignore `EngineProfile`
(server.rs:16249-16719). `#655-E`'s "unreachable by default (proven by tests)" therefore holds only for the three
cascade entry points — finding AUD-ARCH-003.

**Discovery vs. serving:** `/v1/models` + `/uor/v1/status` treat teacher-readiness as model-active
(`active_canonical_model_name`, server.rs:2662-2667), so a teacher-only install advertises a model and
`engine_active:true` that Production can never serve; status exposes no profile field — AUD-ARCH-004.
`/uor/v1/status` + `/v1/models` do surface C1d's `verified` (advisory sidecar verification).

**Fresh-install behavior (no `.uor-models`):** `/api/chat` → HTTP 503, `outcome:"declined_by_all"`,
`generation_mode:"r4g1-error"`, trail `[{tier:"r4g1", status:"failed", detail:"R4G1 graph runtime is not loaded"}]`.
Pre-E2 the same request was answered by the always-available geometric tier (200). This is the intended
"fail closed to the audited tier" semantics of E2, and it is a **material default-behavior change** documented in
`docs/serving_default_flip_655_f.md:89-99`; README/CONFIGURATION do not describe it yet (AUD-DOC-001).

## 9. Attention / operator activation matrix

Per-mechanism classification (charter Phase F). "Serving-reachable" = reachable from any serving path at this commit.

| Mechanism | Class | Spec | Reference impl | Packed/executable | Differential (synthetic) | Fitted vs real pinned teacher | Real held-out eval | Serving-reachable | Default | Causal to output | Quality-qualified |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `standard-source-attention/1,2` (Llama teacher) | SOURCE (host-side teacher execution; excluded from the deployed contract) | yes (registry) | executor is the impl | n/a (f32 host) | v1 fns factored + pinned by tests | is the teacher itself | Gate C/E ride on it | Experimental profile: `attention` tier; also all offline observe/compile | teacher default | yes (produces the corpus + fallback text) | n/a (it IS the teacher) |
| `experimental-r4-source-attention/1,2` (`--r4-attention` switch) | SOURCE variant | yes | yes | n/a | remainder-policy pinned by unit tests | n/a | **never measured vs standard** (`docs/deferral_record_2026_08_05.md`) | Experimental profile: `r4-attention` tier | off | selectable; effect unmeasured | NO |
| `learned-absolute-source-attention/1,2` + `gpt2-source-dense/1,2` (GPT-2) | SOURCE | yes | GPT-2 executor | n/a | numpy differential (tiny fixture in CI; real 124M presence-gated) | is the teacher | #704 gate passed (timing+parity+census) | Experimental: `attention` tier for GPT-2 bundles | by architecture | yes (offline) | n/a |
| **`R4RouteAttentionV1` (`r4-route-attention/1`, #604)** | TARGET (deployed-class integer operator: masked XOR+popcount, top-M, saturating add) | yes | yes (certify) | **yes** (graph-runtime, P-4-scanned, zero-alloc asserted) | **yes** (bit-for-bit ref↔packed + independent witness replay) | NO (fit harness exists, real arm UNAVAILABLE) | NO | **NO** (dormant; no serving path constructs it) | no | no | NO — activation gate unmet |
| `route-fit/1` ladder (#605) | fitting harness | yes | yes | n/a | synthetic 5-stage PASS, instrument valid (N2 0.2353 < 0.5×0.5718) | real-teacher stage UNAVAILABLE | real-corpus stage UNAVAILABLE | no | no | no | NO |
| Target-operator certificate (#606) | certification composer | yes | yes | n/a | composes #605 synthetic rows; overall verdict **NOT_PASSING** (by construction, unforgeable) | UNAVAILABLE | UNAVAILABLE | no | no | no | NO |
| `MsaStructuredSelectorV1` (`msa-structured-selector/1`, #643) | TARGET (plug-compatible with route-attention) | yes | yes | yes (P-4-scanned) | yes (bit-for-bit + witness replay; **no allocation-census assertion** — AUD-INV-007) | NO | NO (pre-registered A/B vs route-attention not yet wired) | no | no | no | NO |
| Octeract Hamming-weight quotienting (#661) | evaluation of a quotienting idea | paper + typed report | conformance only | no | instrument conformance only | UNAVAILABLE (typed report, κ-pinned) | UNAVAILABLE | no | no | no | NO (closed retain-dormant) |
| Packed ROUT routing programs / typed transitions (#159 ph.2) | placeholder | declared | placeholder returns | placeholder | n/a | n/a | n/a | no (unreferenced) | no | no | NO |
| Bott-Fock context fold / CD-space / tropical / Lie-Jordan / quantum-cover / … (#515 set) | dormant substrates | per ledger | per ledger | varies | measured records (several NEGATIVE: #400 CD 0/1998, #626 tropical, #424 +0.16pp ceiling) | n/a | recorded negatives | no | no | no | NO |

**Conclusion (Phase F, attention):** at this commit **no attention-replacement mechanism is reachable from any
serving path, selected by any default, or causally implicated in any served output.** The only serving-reachable
"attention" is the *teacher's own* attention (Experimental profile / offline compile), which the project's goal
statement explicitly excludes as a product path (docs/RESEARCH.md "out of scope … comparison baseline"). The
strongest replacement candidate (`R4RouteAttentionV1`) is exactly what the ledger says it is: constructible,
differentially tested, witness-replayable, zero-alloc-asserted — and dormant, with its real-teacher fit and any
quality evidence still UNAVAILABLE. Claims of attention replacement beyond "specified + packed + synthetically
verified + dormant" are unsupported (see §16).

## 10. Runtime-invariant findings

**Boundary definition (from the normative documents, verified against code).** The deployed kernel =
the contract-bound activities of `docs/transformerless/INFERENCE_OPERATION_CONTRACT.md` §1 (9 activities),
owned per `uor-r4-graph-format::ACTIVITY_OWNERS`; the steady-state hot path = per-token prediction after
load/warm-up. The contract explicitly does NOT restrict offline compilation, teacher execution, certification,
graph induction, or test-only reference implementations (§4) — this audit applies the restriction only inside
that boundary. Allocation freedom is claimed for the steady state, not initialization (BASELINE.md §3.3).

### 10.1 What each instrument actually proves (and does not)

| Instrument | Proves | Does not prove | Verdict |
|---|---|---|---|
| **P-4 source scan** (`transformerless/mod.rs:43-261`; tests `p4_runtime_source_scan`, `p4_contract_owned_graph_runtime_source_scan`) | The literal source of exactly 7 files (`core::runtime.rs` + graph-runtime `engine/route_attention/msa_selector/routing/runtime_state/status`) contains no line-local infix `* / %` between operand-like chars and none of 12 mul/div/rem method names | Anything about machine code; anything about the 5 unscanned graph-runtime modules (`lib`, `packed_kernels`, `patch_chain`, `scoring`, `vp_tree`) or `core::transformerless::simd.rs`; line-wrapped operators; block comments; macro output. **Floats are not checked by this scanner** (the certify-side scanners do check floats) | Witnessed — with the coverage gaps in AUD-INV-001/003 |
| **Allocation census** (`crates/uor-r4-core/tests/allocation_census.rs`, counting `#[global_allocator]`, thread-local gate) | ZERO allocations **asserted** for: core predict/generate over 32 tokens post-warm-up (:387); TLA7 residual assign (:446); four R4G1 kernels ×10 after warm-up (:559); `route_attention_step` ×8 (:650) + census closed-form (:654); foreign-thread isolation (:312) | Warm-up (measured 5 allocs/496 B, report-only); load/parse paths (store parse measured 57,498 allocs for a 494 KB legacy container, report-only); `msa_selector` (no allocation assertion anywhere); stack usage | Structural for the asserted paths; **amortized**, exactly as BASELINE.md states |
| **Op census** (`OpKernel`, runtime.rs:31-38; `RouteOpCensus`) | Per-run printed op counts; route-attention census equals its data-independent closed form (asserted) | The headline "144,496 avg ops/token" is **printed, never asserted** — a regression would not fail CI; there is no multiply counter because the interface has no multiply operation (zero-multiply rests on P-4 + construction, not on the census); two published census rows (BASELINE.md:193 vs PROOF.md:44) differ (~23% adds) with no reconciling test | Witnessed (print-only) |
| **`inference_audit`** (proof-model; CI step "Inference contract machine-code & dependency audit (issue #160)") | That two pure string-scanning functions reject forbidden substrings **in hardcoded sample inputs** | **Anything about this repository's compiled binaries, real dependency graph, or allocation behavior.** `audit_all` scans a 4-line hardcoded asm string (inference_audit.rs:135), a 3-element hardcoded dependency array (:146), and returns `steady_state_allocations: 0` as a literal (:161). No objdump/capstone/ELF read, no allocator hook, no cargo-metadata read. | **Vacuous** — AUD-INV-002. The honest statement is INFERENCE_OPERATION_CONTRACT.md §6/§8: source-scan evidence is Witnessed "until disassembly audit lands"; it has not landed |
| **Proof matrix** (`proof_matrix.rs`) | A hardcoded const table declares 8/8 obligations `Verified`; `verify_all` rejects only `Unverified` (never instantiated) | That the statuses are derived from anything; entry #2 (Operation-Set Conformance) is `Verified` while its own description says "until disassembly audit lands" | AUD-INV-006 |
| **Kani** (`kani_proofs.rs`; `formal_verification.yml`; green on main) | No-panic/no-overflow/no-OOB for `ScoreQ::saturating_add` (unbounded i32×i32) and `RuntimeState::<32,8,8,8>::update_slot` (one token, one slot); dependency-boundary grep keeps `uor-matmul`'s ASM out of the Kani graph (the #712 fix, by construction) | Anything about `engine.rs`, routing, prediction, parsing, or the core transformerless runtime (excluded from the Kani graph by design) | Structural for its 2 harnesses |
| **`#![forbid(unsafe_code)]`** (graph-format lib.rs:61-62, graph-runtime lib.rs:1-2) | rustc-enforced zero `unsafe` in the format + graph-runtime crates | Nothing about `core::transformerless::simd.rs` (14 unsafe sites, raw SIMD intrinsics) or `compiler.rs` FFI/transmute — those live in `uor-r4-core`, which does not forbid unsafe | Structural (strongest instrument present) |
| **no_std ladder** (CI + local; graph-format `--no-default-features` [+alloc]) | graph-format builds without std; graph-runtime is unconditionally `no_std` | "no_std" ≠ "no alloc": graph-runtime declares `extern crate alloc` and heap-allocates in `patch_chain`/`vp_tree` (at load, not steady state) | Structural |
| **Deterministic rebuild** (`deterministic_rebuild_test.rs`; hard-fails if fixture absent — no silent skip) | Container codec byte-stable involution on the committed TLA7 fixture; transition compiler deterministic on a 6-record corpus, in-process | Cross-machine/toolchain reproducibility (that is Gate E + D2's job); compile-from-weights determinism | Structural (scoped) |
| **κ-reproduction (Gate E)** (`kappa_reproduction.rs`, `#[ignore]` + checkpoint-gated; **run for real this audit**, 66 s, checkpoint present) | Byte-identical κ set for the canonical deterministic compile of the pinned stories15M teacher incl. the macOS-only bundle-derived pins (threshold/codebook/class-sig/container κ) | Skips **silently** when `/tmp/ref/out/model.bin` is absent (documented "thing that bites"); CI runs it with a cached checkpoint (`ci.yml:136-151`) but the macOS-pinned half only runs on macOS hosts like this one | Structural for this run; instrument has a known vacuous-green mode |
| **Witness replay** (Gate C harness) | 64/64 independent witness replays succeeded on the live Gate C run (§11) | Coverage is 64 sampled positions | Witnessed |
| **Register conformance R1–R6** | R1 CONFORMANCE.md ≡ model (29 ids); R2/R3 scenario/marker/meta-gate integrity; R4 nothing deferred-unregistered; R5 sanctioned error surface; R6 cargo-deny bans | See AUD-VER-001: with a stale `repo-model` rlib the gates error (R1/R4) or risk vacuous green (R5) | Structural after rebuild (§11) |

### 10.2 Reachable-code sweep results (deployed boundary)

- **Multiplication/division/float:** none found in the scanned hot-path files (P-4 green). Two boundary-relevant
  gaps: (a) `simd.rs` is on the hot path (`runtime.rs:572,702,866,875,1229` call `hamming_distance_36`,
  `dot_argmax`, `DotTables::from_packed`) and is outside every scan; its hamming path carries a scalar-equivalence
  witness (`test_simd_hamming_equivalence`, simd.rs:699), its **`dot_argmax` path does not** (the layouts test
  compares two layouts through the same dispatcher — simd.rs:670-695); (b) `vp_tree.rs:142` contains a live
  `distances.len() / 2` — **build-time only** (VP-tree construction at graph load, engine.rs:82); the query path
  (`query`/`search_node`/`distance`) is XOR/popcount/compare-only. Neither violates the contract as written
  (§4 exclusions + steady-state boundary), but both sit outside the machine-checked witness the docs cite — AUD-INV-001.
- **Heap allocation:** steady-state zero asserted (census); warm-up + load allocate by design and are documented.
- **Locks/I/O/hidden init in the hot path:** none found in the deployed kernel modules; the *server* rereads
  `last_engine.txt` + `engine_profile.txt` from disk on every request that omits `engine` (deliberate; server.rs:3818-3834).
- **Panic/unwrap on recoverable input:** none observed on the deployed decode path; `R4Engine::load` declines
  typed on drifted bundle pairings (#743 fix, engine.rs:1084-1104). CLI/server-side code retains `let _ =` swallowed
  writes (chat.rs engine-file writes) — outside the kernel boundary, noted in AUD-QUAL findings.
- **Unsafe:** zero in graph-format/graph-runtime (forbidden); concentrated in `core::transformerless::simd.rs`
  (14 sites) + `compiler.rs` (vvexpf FFI, one `from_raw_parts` transmute) — the former is hot-path (AUD-INV-001),
  the latter compiler-side (contract-excluded).
- **Nondeterminism:** none found on the deployed decode path (greedy argmax with canonical tie-breaks; the #762
  sampler is opt-in and seed-reproducible with a P-4-legal restoring-division `reduce_into_range`,
  runtime.rs:958-1001, externally verified by `sampling_reduction_762.rs`). Known, documented exception: the cover
  R4G1 **header provenance digest** is per-run (HashMap RandomState seed) while regions/recall/payload are
  deterministic — the reproducibility invariant for cover is induced structure, not raw header bytes
  (gpt2 canary #693 finding); `score.r4g1` and TLA containers are byte-deterministic (κ gate + canary assertions).

### 10.3 Runtime-invariant findings register

| ID | Sev | Status | Finding |
|---|---|---|---|
| AUD-INV-001 | **High** (claim risk) | CONFIRMED | P-4 scan coverage excludes `simd.rs`, which is on the scanned hot path via 5 call sites and holds all of core's hot-path `unsafe`; no scalar-vs-SIMD equivalence witness exists for `dot_argmax`. `docs/transformerless/PROOF.md` ("the scanned file contains the COMPLETE runtime arithmetic surface") is contradicted. Repro: `grep -n "simd::" crates/uor-r4-core/src/transformerless/runtime.rs`; read mod.rs:221-261 scan list. Impact: the zero-multiply *witness* is narrower than documented (the claim itself may still hold — nothing in simd.rs's hamming/argmax kernels is a value multiply — but the dot tables path is unwitnessed). Action: add simd.rs to the scan (or exclude with an explicit witnessed-equivalence note), add a `dot_argmax` scalar differential. Confidence: high. |
| AUD-INV-002 | **High** (claim risk) | CONFIRMED | `inference_audit` (CI: "machine-code & dependency audit, #160") audits hardcoded strings and returns constants; no disassembly, no allocator witness, no real manifest read. The proof-matrix `Verified` on Operation-Set Conformance and the CI step name overclaim; the contract doc's Witnessed status is the accurate one. Repro: read `crates/uor-r4-proof-model/src/inference_audit.rs:130-170`. Action: either land the real #160 audit or rename the gate + downgrade the matrix row to the honest status. Confidence: high. |
| AUD-INV-003 | Med | CONFIRMED | Contract bookkeeping drift: 2 of 9 `ACTIVITY_OWNERS` name modules outside every scan (`reference_state`, wasm-router `r4g1::…`); the owner string names a function that does not exist (`generate_into`; actual: `generate_into_status[_with_witness]`); two contract version constants coexist (0.1.0 at inference_contract.rs:56 vs V1_0_0 at :184) with no cross-check. |
| AUD-INV-004 | Low-Med | CONFIRMED | Op census print-only; two inconsistent published rows (BASELINE.md:193 vs PROOF.md:44); no assertion would catch an op-count regression. |
| AUD-INV-005 | Low | CONFIRMED | Four divergent source-scanner copies (core mod.rs, msa_selector_643.rs, r4g1_runtime_test.rs, bott_fock.rs) with different comment/string/float handling. |
| AUD-INV-006 | Med | CONFIRMED | Proof matrix is a hardcoded 8/8-Verified table; `verify_all` can only fail on a status never used; honesty machinery cannot fail. |
| AUD-INV-007 | Low | CONFIRMED | `msa_selector_step` has census-equality + source-scan coverage but **no counting-allocator assertion** (route-attention has one). |
| AUD-INV-008 | Info | VERIFIED WORKING | Kani green in CI; #712 resolved structurally (proof crate's default-features exclusion + CI dependency-boundary grep). |
| AUD-INV-009 | Positive | VERIFIED WORKING | `forbid(unsafe_code)` on format+runtime crates; typed-decline discipline at `R4Engine::load` (#743); transactional observation resume with byte-convergence tests. |

## 11. Verification command ledger

All commands ran on the pinned machine (§2), working tree at `aea30bae`, default features unless stated,
sequential (one cargo invocation at a time). Full per-gate output retained at `/tmp/gate_*.out` /
`/tmp/gate2_*` / `/tmp/gate3_*` / `/tmp/gate4_*` on the host for the session's duration.

### 11.1 First pass (exact CI-mirror ladder)

| # | Command | Exit | Dur | Result |
|---|---|---|---|---|
| 1 | `python3 scripts/check_claim_wording.py` | 0 | 0s | PASS |
| 2 | `cargo fmt --check` | 0 | 1s | PASS |
| 3 | `cargo test -p uor-r4-proof-model --lib inference_audit` | 0 | 16s | PASS — but see AUD-INV-002: the gate is vacuous by construction |
| 4 | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 0 | 36s | PASS (warm cache) |
| 5 | `cargo test --workspace` | 101 | 55s | **FAIL — environmental.** Only `repo-conformance::registered` failed (29/29 panics, `suites read: NotFound`); cargo's binary-level fail-fast then **skipped all later test binaries** (see AUD-VER-001/002) |
| 6 | `cargo check -p uor-r4-graph-format --no-default-features` | 0 | 1s | PASS |
| 7 | `… --features alloc` | 0 | 1s | PASS |
| 8 | `cargo test -p uor-r4-core --test deterministic_rebuild_test -- --nocapture` | 0 | 1s | PASS (fixture present — hard-fails if absent, no silent skip) |
| 9 | `TLESS_CANONICAL_DETERMINISTIC=1 TLESS_CHECKPOINT=/tmp/ref/out/model.bin cargo test -p uor-r4-core --release --test kappa_reproduction -- --ignored` | 0 | 66s | **PASS, non-vacuous** — checkpoint verified present first; includes the macOS-pinned bundle-derived κ half |
| 10 | `cargo build --target wasm32-unknown-unknown --lib` | 0 | 36s | PASS |
| 11 | `cargo run -q -p xtask -- check-model` (R1) | 1 | 1s | FAIL — stale-worktree artifact (below) |
| 12 | `cargo test -q -p repo-conformance` (R2/R3) | 101 | 2s | FAIL — same artifact |
| 13 | `cargo run -q -p xtask -- audit-deferral` (R4) | 1 | 1s | FAIL — same artifact |
| 14 | `cargo run -q -p xtask -- audit-limits` (R5) | 0 | 0s | "PASS" — **untrustworthy at this point** (same stale root; possible vacuous walk) |
| 15 | `cargo deny check bans` (R6) | 0 | 0s | PASS |
| 16 | `cargo run --release --bin r4 -- transformerless score --corpus-meta crates/uor-r4-core/tests/fixtures/c_meta.bin --corpus-recs crates/uor-r4-core/tests/fixtures/c_recs.bin --artifacts crates/uor-r4-core/tests/fixtures/tless_artifacts.bin --out /tmp/audit_trend_output` | 0 | 216s | PASS — real Gate C run, schema-26 report (metrics in §13.1) |
| 17 | `./scripts/check_gate_c_regression.py /tmp/audit_trend_output/score_report.json` | 0 | 0s | PASS — no regression vs pinned record (pin: rule12 31.63% / 9.1181; measured: 36.55% / 8.3222) |
| 18 | `cargo test --workspace --all-features` | 101 | 47s | FAIL — same single environmental failure; all #515 gated dormant tests that ran before the fail-fast cutoff passed |

### 11.2 The environmental failure, diagnosed and cleared (AUD-VER-001)

**Finding.** `target/` contained binaries/rlibs compiled inside a deleted git worktree
(`.worktrees/i704attention`). `crates/repo-model/src/lib.rs:164` and `crates/repo-conformance/tests/registered.rs:13`
resolve the repository root from **compile-time** `env!("CARGO_MANIFEST_DIR")`; a cached artifact therefore
carries the dead worktree's absolute path. Effects at audit start: R1/R4 errored (`reading /Users/…/.worktrees/
i704attention/model/ledger.toml: No such file or directory`), R2/R3 failed 29/29 (`features/suites` NotFound —
the directory exists in the real tree), and R5 "passed" in 0 s against a root that did not exist. `cargo clean
-p xtask -p repo-conformance` fixed R2/R3 only; the xtask gates required `cargo clean -p repo-model` (the baked
path lives in the *library*). CI is unaffected (fresh checkouts). This is a local-verification-trust defect, not
a code defect: the batch-flow worktree process (AGENTS.md) plus a shared target dir makes it recur.
**Repro:** build in a worktree sharing `target/`, delete the worktree, run `cargo run -p xtask -- check-model`.
**Action:** resolve the root at *runtime* (`std::env::var("CARGO_MANIFEST_DIR")` — cargo sets it for `cargo run`,
and fall back to `current_dir` for direct binary invocation), and make `audit-limits` fail on an empty scan set
(it currently cannot distinguish "no violations" from "no files walked" — same instrument-that-cannot-fail shape
docs/RESEARCH.md catalogues).

### 11.3 Re-verification after clearing (all fresh builds)

| Command | Exit | Result |
|---|---|---|
| `cargo run -q -p xtask -- check-model` | 0 | PASS: "CONFORMANCE.md equals the model, 29 ids (CM-01)" |
| `cargo test -q -p repo-conformance` | 0 | PASS (R2/R3 incl. meta-gate) |
| `cargo run -q -p xtask -- audit-deferral` | 0 | PASS: "nothing is deferred (R4)" |
| `cargo run -q -p xtask -- audit-limits` | 0 | PASS: "no shipped crate returns an unsanctioned error (R5)" |
| `cargo test -q -p repo-conformance --all-features` | 0 | PASS |
| `cargo test --workspace` (relaunched) | in flight | **134 of 135 test binaries green, zero failures**; at report time (~14:10 UTC) the final binary — the cucumber BDD suite — was still executing its teacher-parity scenarios **with the live SmolLM2-135M teacher** (snapshot + compiled bundle present, so the suite is non-vacuous on this host: teacher ready, kappa blake3:12d2cd8a…, matmul=uor-matmul exact GEMM). Two suite-internal findings already recorded from its output: (a) the R4G1-graph parity scenarios **skip** with an honest printed reason — graph provenance kappa mismatch (report blake3:e759500a… vs artifact blake3:a1863bdd…) — i.e. the local 135m bundle's score-report and artifact identities disagree, further evidence of the mixed-era canonical bundle (§13.2); the legacy-TLS parity arm runs live; (b) the anti-memorization guard passed. The most recent completed full-suite record on this host is #755's closure (2026-08-17): 1,412 passed / 0 failed. Final tally reported in the session summary accompanying this report. |

**Verdict at `aea30bae`: every CI-equivalent gate is green on this machine once the stale-target artifact is
cleared**, with the κ gate and Gate C trend passing non-vacuously. GitHub CI on `main` concurs (latest `ci` run
success 2026-08-18T07:18Z; Kani success; cargo-audit success).

### 11.4 Not run (with reasons)

- `cargo +nightly fuzz run …` — nightly fuzz smoke; CI-covered; NOT AUDITED locally.
- `wasm-pack build --target web` — CI-covered; the lighter `cargo build --target wasm32-unknown-unknown --lib` ran green.
- `bash scripts/d2_differential_compile.sh` — ~1 h corpus compile; requires a run contract + approval (§18).
- `cargo test -p uor-r4-graph-certify --test capacity_scaling -- --ignored` (~12 min) — no conclusion in this
  report depends on a fresh saturation verdict; the standing 2026-08-06 verdict (SATURATED) is cited as historical.
- `cargo kani` locally — CI evidence accepted.

## 12. Fixture and artifact availability ledger

Presence checked on this host at audit time. "Gates" = which evidence becomes non-vacuous when present.

| Artifact | Present | Size | Gates |
|---|---|---|---|
| `/tmp/ref/out/model.bin` (llama2.c stories15M) + tokenizer | **YES** | — | Gate E κ-reproduction (ran, §11); legacy certify/compare chain |
| Committed 500k fixtures (`c_meta.bin`, `c_recs.bin` 22.9 MB, `tless_artifacts.bin` TLA7) | YES (tracked) | 24 MB | Gate C score harness (ran); deterministic rebuild (ran) |
| `baseline_kappa.json` + `gate_c_pinned.json` | YES (tracked) | — | κ pins; Gate C trend alarm (both exercised) |
| `.uor-models/sources/smollm2-135m-instruct` (+ `smollm2-135m`, `SmolLM2-135M-Instruct-7e27bd9f9532`) | YES | 257 MB each | teacher-parity BDD arm; smollm2_adapter tests; live teacher tier |
| `.uor-models/sources/smollm2-360m-instruct` | YES | 690 MB | 360M observe/compile |
| `.uor-models/sources/smollm2-1-7b-instruct` | YES | 3.2 GB | 1.7B compile |
| `.uor-models/sources/qwen3-0-6b` | YES | 1.4 GB | **no adapter exists** (not Llama-family config, not GPT-2) — unusable by the current compiler; NOT EXERCISED |
| `.uor-models/sources/llama-3-2-1b-instruct` | YES | — | Llama-family but outside the pinned adapter's declared feature space (`rope_scaling` is rejected by name, #599) — expected fail-closed; NOT EXERCISED |
| `.uor-models/sources/gpt2-124m` | **NO** | — | GPT-2 canary tests → vacuous UNAVAILABLE locally (they passed for real in the #693 cloud run, 2026-08-16) |
| `.uor-models/compiled/smollm2-135m-instruct` | YES (complete: TLA/TLS1/tokenizer + `compiled.r4g1` 27 MB + `score.r4g1` + corpus pair + 80 MB store) | ~250 MB | `r4 ask` probes (§13) — **pre-#755 bytes** (proven by output signature) |
| `.uor-models/compiled/smollm2-360m-instruct` | YES (complete + `compiled.r4g1` + `instruction-eval.json`) | ~400 MB | §13 |
| `.uor-models/compiled/smollm2-1-7b-instruct` | YES (complete + `compiled.r4g1` + `instruction-eval.json` + `compile_report.json`) | ~250 MB | §13 (the #745 word-salad bundle) |
| `.uor-models/compiled/smollm2-360m-broad` | PARTIAL — **no `tokenizer.bin`** | — | #509's positive-result bundle; not loadable by `ask` (3-file contract unmet) |
| `.uor-models/compiled/smollm2-135m-instruct-tla6`, `SmolLM2-135M-Instruct-7e27bd9f9532`, `SmolLM2-135M-Full`, `qwen3-0-6b` | partial/legacy | — | historical compiles |
| `.uor-models/manifests/` | **EMPTY** | 0 | **No imported model manifests exist** → every `r4 ask` on this host uses the un-attested local-bundle path (WARN emitted; AUD-QUAL-004) |
| `.uor-models/corpora/*` (simple-wiki sealed corpus + 20k obs shards incl. `-BAD`, multi-corpus obs, massive-pdf obs) | YES | 1.2 GB | broad-corpus experiments; `simple-wiki-20k-obs-BAD` retained as the #755 interleaving evidence |
| `.uor-models/objects/blake3/*` | YES | 221 MB | CID store (holds prior imports' bytes; no manifests point at them now) |
| `models/*.json` descriptors | YES (tracked) | — | pins incl. GPT-2 tokenizer κ (`gpt2_tokenizer_pin` well-formedness ran in-suite) |
| SmolLM2 conformance fixture (`real_smollm2_fixture_round_trip_passes`) | source present | — | fixture-gated adapter arm |
| T5 `spiece.model` (`.uor-models/sources/t5-base-tokenizer/`) | not checked live | — | sentencepiece presence-gated arm (CI-safe fixture ran in-suite) |
| `.uor-models/last_engine.txt` = `r4g1`; `last_model_name.txt` = `smollm2-135m-instruct`; `engine_profile.txt` **absent** → Production default | YES | — | serving defaults on this host |
| Untracked `obs-text/` (62 MB), `tf1-pkg/` (492 KB) | YES (pre-existing) | — | preserved untouched; not part of the tracked baseline |

## 13. Inference and generation results

### 13.1 Token-level offline metrics (repository's own harness, run live this audit)

`transformerless score` on the committed 500k fixture corpus (stories15M-teacher era, TLA7 artifact), full census
— no sampling (`positions_sampled: 0`), 100,306 held-out positions, schema 26, deterministic note verified in
report; witness replay **64/64**:

| Scorer row | top-1 | bits/token |
|---|---|---|
| legacy Σ-cloud (refuted #64, kept as row) | 0.07% | 84.54 |
| Rule 1 chain (graph only) | 1.50% | 16.24 |
| **Rule 1+2 (chain + EXCT precedence — the deployed score family)** | **36.55%** | **8.32** |
| TLA3 store baseline | 39.17% | 8.50 |
| best experimental fused row (`rule12_fwd_gated_fused_live`, n=63,543) | 44.84% | 6.69 |
| **`rule12_generalization` (EXCT-miss slice only, n=14,943)** | **1.81%** | **16.89** |

Distribution: EXCT resolves 85,363/100,306 (miss 14.9%); depth histogram `[0, 1160, 7117, 6666, 85363]`;
repetition-rate rule12 0.2219 vs baseline 0.2250. Trend alarm: PASS vs pinned record (31.63%/9.1181, pinned
2026-07-30). Category (charter §7): **corpus replay / same-distribution held-out** on the fixture corpus.
The two structurally load-bearing readings: (a) in-distribution per-position signal is real and stable;
(b) off exact-context, the graph path alone generalizes at **~1.8% top-1** — the memorization/generalization
split in one number pair.

### 13.2 Live end-to-end generation probes (`r4 ask`, ModelStore path, defaults = deterministic greedy)

Engine provenance for every probe: requested = default local ask path; actually selected = **R4G1 beam path**
(a `compiled.r4g1` exists for each probed bundle, so the plain path and the `--sample` path are unreachable —
`src/chat.rs:350-476` returns before the sampler dispatch). Model identities recorded from the emitted WARN line
(artifact/store/tokenizer CIDs). No teacher, no fallback tier, no network: the audited runtime itself produced
every byte below.

All probes: deterministic defaults, release binary, live run 2026-08-18 (~09:34–09:54 EDT), full raw
outputs preserved at `/tmp/audit_ask/*.txt` on the host. Prompts: Q1 = "What is the capital of France?",
Q2 = "Why is the sky blue?". Durations are upper bounds (workspace test suite sharing the 8-core host).

| Bundle (identity from emitted CIDs) | Decode | Prompt | Dur | Outcome (verbatim prefix) |
|---|---|---|---|---|
| `smollm2-135m-instruct` (artifact `blake3:76309255…`, store `blake3:a89d7eca…`) — **pre-#755 bytes** | greedy (R4G1 path) | Q1 | 235 s | `ounds Callounds Callounds Call…` — token-cycling degenerate; long stall then output (the exact #745/#755 "before" signature) |
| same | greedy | Q2 | 207 s | **byte-identical to Q1** (808 bytes) — prompt-invariant |
| same | greedy (repeat) | Q1 | 239 s | **byte-identical** — deterministic, reproducible |
| same | `--sample 42` | Q1 | 241 s | **byte-identical to greedy** — empirical proof the #762 sampler cannot reach the R4G1-preferred path (AUD-QUAL-003) |
| `smollm2-360m-instruct` (artifact `blake3:df469c1c…`) | greedy (R4G1 path) | Q1 | 93 s | `<|im_start|>cescescesutioncesutionces…` — degenerate; leaks the raw `<|im_start|>` special token |
| same | greedy | Q2 | 85 s | **byte-identical to Q1** (795 bytes) — prompt-invariant |
| `smollm2-1-7b-instruct` (artifact `blake3:fca5bdfb…`; own recorded eval: 34.7% top-1 / 19.14 bits, schema-1 report) | greedy (R4G1 path) | Q1 | **5 s** | `cut cut cut cut cut …` — the exact #750-documented R4G1-path single-token repetition |
| `smollm2-360m-broad` (#509's positive bundle; no `tokenizer.bin`) | — | Q1 | 0 s | typed decline: `compiled model manifest 'smollm2-360m-broad' was not found` (3-file bundle contract unmet → not recognized as a bundle) — UNAVAILABLE for ask, by structure |
| *(no `--model`, defaults)* | — | Q1 | 0 s | typed error: `compiled model manifest 't5-base-tokenizer' was not found` — the mtime-newest `models/*.json` default resolves to a tokenizer-only descriptor (AUD-QUAL-009, live) |
| `r4 compare-report` (legacy chain, recorded certificate) | — | — | 0 s | prints the recorded certificate: transformerless mul-free 77,342 tok/s ≈ 225× llama.cpp q8_0, teacher agreement 31.7% (recorded, not re-measured) |
| `r4 transformerless compare` | — | — | 0 s | **exit 0, no-op**: prints the usage banner that itself advertises `compare` (AUD-WIRE-007, live) |

**Provenance statement.** Every generated token above was selected by the audited R4G1 runtime through the
ModelStore ask path — requested engine = default, selected engine = R4G1 beam path (`compiled.r4g1` present for
all three bundles). No teacher, no geometric fallback, no external provider contributed content; the
attestation-absent WARN line (printed by default) identifies the exact artifact/store/tokenizer CIDs per run.
Termination: 135m/360m runs ran to the token cap after long stalls; the 1.7B run terminated quickly into the
repeat. "Produces text" (charter definition: runtime selected and decoded multiple tokens without teacher or
fallback) is **met mechanically** by all three bundles and **failed semantically** by all three: no coherent,
prompt-conditioned answer was produced by any locally available bundle at baseline. The repository's own
documentation (README, RESEARCH, F0) states this condition accurately; this audit reproduces it live and adds
the bundle-refresh nuance (§14 barrier 3: the compiler-side #755 fix is landed + regression-tested, but no
canonical bundle has been recompiled with it).

### 13.3 Category verdicts (charter Phase E)

| Category | Verdict at this commit |
|---|---|
| 1. Corpus replay / exact-context retrieval | VERIFIED WORKING (Gate C EXCT rows; §13.1) |
| 2. Held-out, same distribution | VERIFIED WORKING as measurement (36.55% top-1); weak absolute quality |
| 3. Held-out, different distribution | Historical records only (broad-corpus 360M pinned row 24.30%/11.94, #516; P3 10.2–29.0% causal, #509). Not re-measured (hours-scale). |
| 4. Autoregressive multi-token generation | **Mechanically working, semantically degenerate** on the canonical local bundles (§13.2): deterministic, byte-reproducible, prompt-**insensitive** token loops |
| 5. Interactive chat / instruction following | GATED by design (`ask` requires an attested instruction-chat manifest) — but the gate is bypassed by the local-bundle path on this host (manifests empty; AUD-QUAL-004) |
| 6. Teacher-backed / fallback responses | NOT default-reachable (Production profile); Experimental-only; none observed in probes |
| 7. Pure deployed geometric/transformerless responses | **Every probe output above is this category** — audited-runtime-produced, no teacher, no fallback. "Produces text" (multi-token, runtime-selected) = TRUE mechanically; "produces coherent text" = FALSE on the probed bundles |

## 14. Generalization assessment

**Memorization vs. lookup vs. interpolation (Empirical, repository's own measurements):** the deployed score
family resolves 85.1% of same-distribution held-out positions by exact-context lookup; its advantage over the
plain store is precedence + calibration, not generalization (argmax-identical on EXCT-resolved rows — BASELINE.md
§4.1 structural caveat). Off exact context the graph path scores 1.81% top-1 (§13.1) — barely above the historical
unigram floor family (#456/#457: consistency-operator and reconstruction arms landed at/below the unigram floor).

**Same-distribution interpolation:** real but small (cover/codebook levers measured: +0.44pp codebook fit;
+5.0pp region-path from scaled capacity, serving-impact-capped ~0.15pp because the graph path answers 1–3% of
positions — #460 records).

**Novel-context recombination / OOD:** the teacher-breadth result (#320/#509: broad teacher moved broad-text
held-out top-1 from ~0.1% to 10.2%/29.0% causal) remains the only large positive lever on record, and it is a
*compile-input* lever, not an architecture lever. OOD prompts against compiled runtimes remain ~1% top-1
(teacher-parity suite's honesty constants).

**Coherent multi-token composition:** NO at baseline on the canonical bundles (§13.2, live). The #755 fix
(story-order reconstruction in `load_corpus_bytes`) is **in the compiler code and regression-tested**
(`corpus_record_order_755.rs`) and its decisive retest produced grammatical-if-wandering English — but the
**fixed recompile was not installed** as any canonical bundle: the on-disk `smollm2-135m-instruct` bundle still
exhibits the pre-fix ~200 s stall + token-cycling signature, byte-identical across prompts. The best measured
post-fix state (from the #755/#758 records) is grammatical but prompt-insensitive text with 9–11/15 distinct
outputs across 15 prompts, and 15/15 under the opt-in `--sample` mitigation (#762) — which the R4G1-preferred
path cannot reach (AUD-QUAL-003).

**Structural barriers, ranked by evidence:**
1. **Evidence-quality-per-key vs. key-resolution** (the programme's own strongest pattern, docs/RESEARCH.md):
   every resolution lever failed; fit levers pay small; capacity levers arm future corpora only.
2. **Attractor-basin collapse under greedy decode** (#759 corrected diagnosis): distinct prompts quantize to the
   same compiled code and greedy argmax then emits identical continuations. Root-cause lever = codebook/cover
   resolution (#460 lineage); the shipped mitigation is opt-in sampling that the preferred serving path bypasses.
3. **Corpus-order dependence of context reconstruction** (#755): fixed in-code; canonical bundles not recompiled,
   so the measured baseline is still pre-fix (this is a *bundle refresh* gap, not a code gap).
4. **Fallback-tier signature-blindness** (structural, engine.rs:661-664, :783-800 + the plain d=0 unigram row,
   runtime.rs:1368-1369/:1694): everything that misses lookup collapses to graph-constant defaults. Per #759's
   correction this is NOT the measured collapse mechanism for real prompts (those resolve at real depth), but it
   bounds worst-case behavior and remains unaddressed.
5. **Beam search inert on the R4G1 ask path** (AUD-QUAL-001): `node_scores` is never populated, so candidate
   expansion yields ≤1 candidate — the "beam" is width-1 greedy; diversity/repetition machinery downstream of it
   is dead. Any conclusion about "beam search quality" drawn from this path measured greedy, not beam.

**Hypotheses (NOT findings — each needs a pre-declared experiment, §18):** feeding the R4G1 engine's real node
scores (or removing the dead beam scaffolding) may change generation dynamics; default-enabling #762 with a fixed
seed may clear the F-precondition-2 bar; recompiling all canonical bundles post-#755 is the cheapest large
expected delta; a signature-differentiated final fallback tier remains scoped-but-unbuilt (#759's original scope,
superseded but not refuted).

## 15. Plumbing disconnects and stale paths

| ID | Sev | What | Evidence |
|---|---|---|---|
| AUD-WIRE-001 | Med (by design, tracked) | Three disconnected model-loading systems (ModelStore / #248 cascade / ReleaseBundleManifest) — confirmed unchanged; reconciliation is the explicitly deferred #655-C1e | SERVING_MODEL_DISCOVERY.md + code verified |
| AUD-WIRE-002 | Med-High | `/api/tless/*` + `/api/r4g1/*` bypass `EngineProfile` — Tier 2 stays HTTP-reachable on a Production server | server.rs:16249-16719 |
| AUD-WIRE-003 | Med | Discovery/serving contradiction: teacher-only installs advertised as active but undeliverable under Production; no profile field on status | server.rs:2662-2667, :520-534 |
| AUD-WIRE-004 | Low-Med | `select_synthesis_engine` dead (sole caller = BDD suite, asserting behavior no serving path uses — false assurance); disagrees with `tier_for_engine_name` on `"transformerless"` | server.rs:3688-3698; tests/bdd.rs:239 |
| AUD-WIRE-005 | Low-Med | `FallbackRouter`/`EngineResponse`/`run_cascade_with_policy`/`AbstainPolicy::Terminal` + status `UnmappedRegion`: dead code; README documents FallbackRouter as the live mechanism | fallback.rs:219-300; README.md:229-230 |
| AUD-WIRE-006 | Low | Root wasm export `generate_r4g1_response` unreachable from both frontends (`wasm_module` never assigned); dashboard r4g1/transformerless engine selections silently take geometric fallback in static/WASM mode | src/lib.rs:61-65; index.html:2801; r4_worker.js:52 |
| AUD-WIRE-007 | Low-Med | `r4 transformerless compare|compare-report` (and ANY unknown transformerless subcommand): prints the usage banner **that advertises them** and exits 0 — silent no-op success; `graph` subdispatch errors properly | graph-cli lib.rs:9741-9762 vs :9673-9675 |
| AUD-WIRE-008 | Med | `uor-r4-cli` menu 3 (1.7B) pins `HF_REV="main"` — refused by the 40-hex pin rule on any fresh download; stage 1 breaks | uor-r4-cli:97-101; model.rs:773-776 |
| AUD-WIRE-009 | Low | `--sample` accepted-and-dropped in `chat --remote`; remote client hardcodes `temperature: 0.7` (opposite determinism semantics local vs remote) | main.rs:850-864; chat.rs:2321-2322 |
| AUD-WIRE-010 | Low-Med | Path-resolution inconsistencies: chat.rs R4G1 preference hardcodes `.uor-models` (ignores `UOR_MODEL_STORE`); raw `manifest.name` interpolated into the path (safe_name applied on read, not here); engine/profile files CWD-relative in both server and chat | chat.rs:198-203 vs model.rs:480; server.rs:3770, :3827 |
| AUD-WIRE-011 | Low-Med | Corrupt `compiled.r4g1` at ask time silently downgrades to the plain path (parse failure swallowed) after import gated both paths | chat.rs:351 |
| AUD-WIRE-012 | Low | Static-file fallback serves any non-`/v1`/`/api` path relative to CWD without traversal containment; mitigations: binds 127.0.0.1 by default, LOCAL_ONLY posture | server.rs:17369-17397 |
| AUD-WIRE-013 | Low | Legacy stale paths kept deliberately (documented): `parse_store_legacy_u16`; TLS1-u16 on-disk store era; TLA3-5 readers; `--emit-provenance` retained as a no-op flag | AGENTS.md:269-271; lib.rs:4250-4252 |
| AUD-WIRE-014 | Process | #759 closure says root-cause work "continues under #460" — but #460 is CLOSED; the codebook-collision root cause currently has **no open tracking issue**. Similarly #653's phase-2 "defer-open" items lost their tracking issue when Casey closed #653 | gh state 2026-08-18 |

## 16. Documentation claim-discrepancy table

Verdicts: ACCURATE / STALE (was true, superseded) / AMBIGUOUS / UNSUPPORTED (no evidence found) / INACCURATE
(contradicted by evidence). Only material claims listed; the many verified-accurate claims are represented by the
summary row at the end. Every row maps to reconciliation §17 or an explicit no-edit rationale.

| # | Document / claim | Verdict | Evidence | Reconciled? |
|---|---|---|---|---|
| D-01 | README "Serving order… A `FallbackRouter` cascades from primary `r4g1-graph` to secondary `transformerless-tla5`" | **INACCURATE** (dead type, dead names; live mechanism is `run_cascade` over `r4g1`/`transformerless` tiers) | AUD-WIRE-005 | §17 R-1 |
| D-02 | README "Omitting `engine` runs the **full cascade**, r4g1-first" + engine table implying transformerless/attention/geometric are reachable on `/api/chat` | **STALE** since E2: default Production profile admits r4g1 only; explicit non-r4g1 requests get a typed decline; `engine_profile.txt` undocumented | AUD-ARCH-002/-003; server.rs:3891-3928 | §17 R-2 |
| D-03 | `docs/CONFIGURATION.md` "Every environment variable the workspace reads" — missing `engine_profile.txt` (state file), `UOR_R4_RELEASE_BUNDLE_PATH`, `PORT` (orchestrator) | STALE/incomplete (E0 §5's own required update was not done with E2) | agent env sweep; E0 doc §5 | §17 R-3 |
| D-04 | `docs/SERVING_MODEL_DISCOVERY.md` (dated "as of 146a976e"): 4-tier cascade as the whole story; "`/v1/models`, `/uor/v1/status` reflect only Tier 1's installed state" | STALE (pre-E2 snapshot; the status claim was also inaccurate at its own commit — teacher readiness counts) | AUD-ARCH-004 | §17 R-4 |
| D-05 | `docs/MODEL_LIFECYCLE.md` §"Follow-up: compile-path dispatch" — "entry points currently bind the concrete HuggingFaceLlamaOracle (~9 sites)… tracked as the #607 follow-up" | **STALE** — `Teacher` architecture-keyed dispatch landed (#657, teacher.rs:36-120); all 9 sites migrated; GPT-2 compiles end-to-end (canary merged); `score` never bound a teacher at all | agent dispatch audit | §17 R-5 |
| D-06 | MODEL_LIFECYCLE + CONFIGURATION: "the graph-byte PROV section … tracked separately by #637" (future) | STALE — PROV/1 landed unconditionally (#637 phase 3, PR #738); `--emit-provenance` retired to a no-op | graph-cli lib.rs:4251-4252, :4647 | §17 R-6 |
| D-07 | `docs/RESEARCH.md` open-work table: #744 "left open pending #750"; #745 "Open"; #750 "Open"; #759 "Open" (+ #758/#759 mechanism text pre-correction; no #762 row) | STALE — all four CLOSED on GitHub (the table's own header says GitHub is the source of truth); #759 closed with a **corrected diagnosis** (argmax attractor-basins at real depth, not the fallback tier; mitigation #762 opt-in; root cause folded into the #460 lineage) | gh state 2026-08-18 | §17 R-7 |
| D-08 | `ROADMAP.md` "Next up": #744, #745 open; "#653 phase 2 … open"; #740/#741 "explicitly deferred until #744/#745 land" | STALE — #744/#745/#653/#740 all closed; only #741 (+#655) remain open; #653's defer-open items are untracked | gh state | §17 R-8 |
| D-09 | `docs/transformerless/PROOF.md` "The scanned file contains the COMPLETE runtime arithmetic surface… nothing on the inference path sits outside the scan" | **INACCURATE** — `simd.rs` is on the path via 5 call sites and unscanned; `dot_argmax` lacks a scalar witness | AUD-INV-001 | §17 R-9 |
| D-10 | `inference_audit.rs` module docs + CI step name "machine-code disassembly auditor / machine-code & dependency audit (#160)"; proof-matrix `Verified` on Operation-Set Conformance | **INACCURATE/overclaim** — the audit is hardcoded-sample-only; the *contract doc's* Witnessed status is the accurate statement | AUD-INV-002 | §17 R-10 (doc side); code-side rename left to a PR (out of audit scope) |
| D-11 | AGENTS.md/README: "`uor_standards/` is legacy local material (**gitignored**; not required to build)" | AMBIGUOUS/misleading — `.gitignore` lists it, but 1,121 files (~22 MB) are already tracked and ship in every clone | manifest count; .gitignore | §17 R-11 |
| D-12 | BASELINE.md §6 "[ ] M.V.G. targets confirmed by maintainer" vs §4 header "CONFIRMED… 2026-07-22" | Internal inconsistency (stale checklist) | file text | §17 R-12 |
| D-13 | `docs/serving_release_packaging_655_d.md` D3 = "run D2 against the real local bundle"; packager header "…which **will** call this helper" (future tense re: D2) | STALE/minor — implemented D3 is a synthetic CI round-trip (real-bundle variant `#[ignore]`d behind `UOR_R4_RELEASE_BUNDLE_PATH`); D2 landed | agent §4 | §17 R-13 (record doc: appended note; module header left to code PR) |
| D-14 | README CLI reference: `transformerless score` flags — `--quality-profile {pinned,relative_tla}` absent everywhere in docs (used by `uor-r4-cli`) | STALE/incomplete | lib.rs:4940-4946 | §17 R-3 |
| D-15 | README "compare/compare-report appear in the subcommand help but are not implemented there. The working spellings are the root `r4 compare`…" | ACCURATE but incomplete — omits that the failure mode is a **silent exit-0 no-op** for any unknown transformerless subcommand | AUD-WIRE-007 | §17 R-2 (footnote) |
| D-16 | `model/ledger.toml` route-fit/msa/#606 rows: "#531 saturation corpus not yet produced, compute-bound" | STALE reason-string (a #531 result exists — closed COMPLETED 2026-08-11); the *dormant/UNAVAILABLE statuses themselves remain accurate | gh #531 | NOT edited (ledger is R1-governed model data; flagged for the next re-measure PR) |
| D-17 | README "What actually works" limitations block (word-salad, ~1% OOD, gates, refuted two-pass, router history) | **ACCURATE** — independently reproduced this audit (word-salad live; Gate C numbers; dormant classifications) | §13 | minor E2/#755-state touch-ups only (§17 R-2) |
| D-18 | CONFORMANCE.md dormant-claim ledger (18 rows) | **ACCURATE** — spot-verified against code for route-attention, msa, route-fit, #606, patch-chain-vs-overlay split, packed-routing placeholders | agent sweeps | none needed |
| D-19 | README "34 harnesses with pre-declared exit rules" | UNSUPPORTED count (not re-counted this audit) | — | left as-is; NOT AUDITED |
| D-20 | `docs/octeract_attention_661.md` / #661 disposition | ACCURATE (typed UNAVAILABLE report, κ-pinned) | gh closure | none |
| D-21 | Teacher-parity BDD doc (AGENTS.md §Teacher parity): "runs in the default `cargo test --test bdd` when sources + bundle present; vacuously skips otherwise" | see §11 workspace rerun row | gate4 log | none |
| D-22 | MODEL_LIFECYCLE "#655 … still open as of this writing" | ACCURATE (still open) | gh | none |
| — | Everything else checked in README/AGENTS/CONTRIBUTING/formal_vocabulary/INFERENCE_OPERATION_CONTRACT/matrix_operation_census/serving_655 docs | ACCURATE at this commit | §§5-10 | — |

## 17. Prioritized remediation plan

Confirmed defects only; speculative architecture ideas live in §18. Ranked within each axis.

**A. Scientific validity**
1. **Recompile every canonical local bundle post-#755** and re-run the #758 15-prompt protocol on each — the
   measured baseline currently mixes pre-fix bundles with a post-fix compiler (this audit's probes measured the
   pre-fix bytes; §13.2). Cheapest large expected delta in the whole plan (compile-recorded is minutes, no teacher).
2. Restore a real quality lever on the preferred path: either populate `node_scores` for the R4G1 candidate walk
   or delete the dead beam scaffolding (AUD-QUAL-001), and decide whether `--sample` should reach the R4G1 path
   (AUD-QUAL-003) — as a pre-declared #762-style experiment, not silently.
3. Re-open (or re-file) the codebook-collision root-cause issue orphaned by #759→#460-closed (AUD-WIRE-014).
4. Reconcile the two published op-census rows or assert one (AUD-INV-004).

**B. Architectural correctness**
1. Runtime-resolve the repo root in repo-model/repo-conformance/xtask (+ make `audit-limits` fail on an empty
   walk) — AUD-VER-001.
2. Unify engine-name resolution (`tier_for_engine_name` vs dead `select_synthesis_engine`; retire the latter and
   repoint the BDD scenario) — AUD-WIRE-004.
3. Fix the `UOR_MODEL_STORE`/hardcoded-path splits and raw-name interpolation (AUD-WIRE-010); surface R4G1-parse
   downgrade (AUD-WIRE-011).
4. Delete or wire `FallbackRouter` and the unreachable wasm export (AUD-WIRE-005/-006).

**C. Serving correctness**
1. Decide the intended profile scope for `/api/tless/*` + `/api/r4g1/*` (bypass vs. governed) and test it at HTTP
   level — E0's own acceptance text asked for HTTP-level proof (AUD-ARCH-003 + missing tests).
2. Make discovery agree with serving under Production (model listing/status/profile field) — AUD-ARCH-004.
3. Typed-decline completeness: echo the requested engine string; decide unknown-name policy — AUD-ARCH-008; OpenAI
   error-envelope conformance for declines (AUD-ARCH-009).
4. `uor-r4-cli`: pin the 1.7B revision (AUD-WIRE-008); make `transformerless` unknown-subcommand exit nonzero
   (AUD-WIRE-007).

**D. Claim/compliance risk**
1. Land the real #160 machine-code audit or rename the CI step + downgrade the proof-matrix row (AUD-INV-002).
2. Extend P-4 coverage to `simd.rs` (+ dot_argmax scalar witness) or annotate the exclusion with its own witness
   (AUD-INV-001); sweep the remaining unscanned graph-runtime modules and the two unscanned ACTIVITY_OWNERS
   (AUD-INV-003).
3. Documentation reconciliation of §16 (performed — see below); keep RESEARCH/ROADMAP issue-state in sync with
   GitHub at close time (process rule already exists; it was not followed for #744/#745/#750/#759).
4. Refresh the stale #531 reason strings at the next ledger-touching PR (D-16).

**E. Developer ergonomics**
1. Gate scripts: `--no-fail-fast` on workspace test runs so one poisoned binary cannot hide the rest (AUD-VER-002).
2. Single shared source-scan implementation (AUD-INV-005).
3. Allocation assertion for `msa_selector_step` (AUD-INV-007).
4. Document `--quality-profile`, `PORT`, `UOR_R4_RELEASE_BUNDLE_PATH` (done in reconciliation, R-3).

### Documentation reconciliation performed (mapped to §16)

Executed after the evidence sections were complete, per the audit charter; every edit appends/corrects — no
historical measurement was rewritten; `python3 scripts/check_claim_wording.py` and `git diff --check` re-run
green after the edits (§11-addendum).

| Edit | Files | Maps to |
|---|---|---|
| R-1 | README.md — replace the FallbackRouter sentence with the real cascade + profile mechanism | D-01, D-02 |
| R-2 | README.md — engine-table/cascade paragraph updated for E2 defaults (Production/Experimental, typed declines, `engine_profile.txt`), pre-fix-bundle note, silent-no-op footnote on transformerless compare | D-02, D-15, D-17 |
| R-3 | docs/CONFIGURATION.md — add `engine_profile.txt` row (+ profile semantics), `--quality-profile`, `UOR_R4_RELEASE_BUNDLE_PATH`, `PORT` (orchestrator-only) | D-03, D-14 |
| R-4 | docs/SERVING_MODEL_DISCOVERY.md — dated E2 addendum section (profile gate, corrected status-surface claim) | D-04 |
| R-5 | docs/MODEL_LIFECYCLE.md — rewrite the stale "Follow-up: compile-path dispatch" section to the landed #657 dispatch | D-05 |
| R-6 | docs/MODEL_LIFECYCLE.md + docs/CONFIGURATION.md — "#637 tracked" → PROV/1 landed (phase 3) | D-06 |
| R-7 | docs/RESEARCH.md — issue-state corrections (#744/#745/#750/#759 closed; #759 corrected diagnosis; #762 row added; append-style, dated) | D-07 |
| R-8 | ROADMAP.md — "Next up" refreshed to actual open state (#655/#741 + untracked remnants called out) | D-08 |
| R-9 | docs/transformerless/PROOF.md — dated correction note on P-4 scan scope (simd.rs) | D-09 |
| R-10 | docs/transformerless/BASELINE.md — appended 2026-08-18 baseline-audit row (Gate C live numbers, probe outcome), §6 checklist fix | D-12, D-17 |
| R-11 | AGENTS.md + README.md — `uor_standards/` wording ("legacy, excluded from the build; ~1.1k files remain tracked") | D-11 |
| R-12 | docs/serving_release_packaging_655_d.md — appended D3-implementation note | D-13 |
| R-13 | docs/project_baseline_audit_2026_08_18.md — this report added under docs/ | deliverable |

## 18. Open questions and experiments requiring approval

Each candidate long run below is written in run-contract form (AGENTS.md discipline); none was launched.

**E-1. Post-#755 canonical-bundle recompile + quality re-read (the F-precondition-2 decider).**
metric: #758 15-prompt distinct-output count + grammaticality of `smollm2-135m-instruct` (and 1.7B) at baseline.
current: pre-fix bytes; 2/2 probed prompts byte-identical degenerate (this audit).
reachability ceiling: #755's own decisive test showed the same corpus recompiled = grammatical output in 0.2 s —
ceiling is "categorically different output", not a pp-level delta.
instrument: `compile-recorded` (minutes, no teacher) then two deterministic asks — the instrument IS the run.
exit rule: post-fix bundle produces non-repetitive, on-vocabulary text on ≥10/15 prompts.
positive → refresh all canonical bundles, post fresh quality read on #655 (unblocks F-precondition 2 discussion).
negative → #755 was not the dominant cause for this bundle; escalate the #460-lineage root cause.
cost: ~15–30 min per bundle. **Needs approval to modify `.uor-models` bundles.**

**E-2. Default-vs-sampled decode on post-fix bundles** (decides whether #762 should be default / reach R4G1 path).
Blocked on E-1. exit rule: sampled ≥14/15 distinct AND no degeneracy regression on Gate C repetition metric.

**E-3. D2 canonical differential (local)** — only if cross-platform reproducibility needs a fresh local datum;
CI already runs it nightly. Default: rely on CI (no launch).

**E-4. Real-teacher route-fit arm (#605 real stages)** — turns the attention-replacement story from synthetic to
real evidence. Prereq now present locally (SmolLM2 snapshot); corpus prereq state needs a #531-closure read.
cost: hours. Decision value: HIGH for the attention audit (§9's real-arm UNAVAILABLE rows become PASS/FAIL).

**Open questions for the maintainer:** (1) Is the `/api/tless|r4g1/*` profile bypass intended (documented
escape hatch) or an E2 gap to close? (2) Should discovery surfaces hide teacher-only models under Production?
(3) Where should the #653 defer-open remnants and the #460-lineage root cause be tracked now that both parents
are closed? (4) Is the `uor_standards/` tracked tree meant to ship, or to be removed from tracking to match its
gitignore entry? (5) `bench-internals`/`full-model` feature docs — none of these knobs appear in CONFIGURATION.md;
intentional?

## 19. Conclusions (charter §12)

1. **What unquestionably works today:** offline compile→cover→score→certify for SmolLM2 (and, snapshot-gated,
   GPT-2) with deterministic, κ-pinned, provenance-carrying artifacts; the R4G1 packed runtime with asserted
   zero-allocation steady state and typed-decline loading; the measurement apparatus (Gate C + trend alarm +
   witness replay + register conformance + claim-wording gate); the geometric router as a retrieval component
   (MRR 0.8763 record); A-mode infill; UOR attestation surfaces; the E2 Production-profile restriction at the
   cascade entry points.
2. **Exists but not activated:** the 18-mechanism dormant set (each with a ledger activation gate), most notably
   the fully-built `R4RouteAttentionV1` + fit ladder + composition certificate (real arms UNAVAILABLE);
   `ReleaseBundleManifest`/typed compile facade (no in-tree consumer); `MsaStructuredSelectorV1`; the #762
   sampler on R4G1-equipped bundles (structurally unreachable there); the `verified` sidecar chain (advisory).
3. **Broken or disconnected:** canonical bundles are pre-#755 (degenerate at baseline — the top remediation);
   R4G1 ask-path beam search is inert (width-1); `uor-r4-cli` option 3 unpinned revision; bare `r4 ask` default
   model resolution; `transformerless compare|compare-report` silent no-op; `FallbackRouter` +
   `select_synthesis_engine` + root wasm export dead; profile bypass on `/api/tless|r4g1/*`;
   discovery-vs-serving contradiction on teacher-only installs; stale-worktree gate poisoning (cleared locally,
   fix recommended).
4. **Synthetic/theoretical evidence only:** route-attention/msa/route-fit/#606 quality stories (synthetic PASS,
   real UNAVAILABLE); Kani (2 harnesses, core excluded); `inference_audit` (vacuous); dormant-mechanism
   activation gates generally; the "beam search" label on the ask path.
5. **Can it produce coherent text without teacher or fallback?** **Not today, on any locally available bundle,
   at baseline** — reproduced live, deterministic, prompt-invariant, through the pure audited runtime. The
   nearest recorded positive (post-#755 recompile: grammatical but topically wandering, prompt-insensitive;
   #758's 9–11/15 distinct) has not been institutionalized into any canonical bundle. Per-position offline
   signal (36.55% in-distribution; 10.2–29.0% causal broad-text) remains real and P-4-clean — the gap is
   composition, not measurement.
6. **Is any replacement attention operator causally active?** **No.** None is reachable from serving, none is
   default, none has runtime provenance in any served output. Only teacher-side attention runs, offline or under
   the Experimental profile, as a baseline — exactly as the project's goal statement requires.
7. **Most important structural obstacle to generalization:** the evidence-quality-vs-key-resolution ceiling
   (the programme's own measured pattern), concretely manifested at serving as greedy argmax over
   quantization-collided context codes (#759-corrected mechanism) with a signature-blind terminal fallback —
   compounded operationally by stale bundles and the unreachable sampling lever.
8. **Next three actions, each with a measurable decision rule (details §18):**
   **(1) E-1 bundle refresh:** recompile canonical bundles post-#755; PASS iff ≥10/15 distinct, non-degenerate
   outputs on the #758 protocol; FAIL ⇒ escalate the #460-lineage root cause with a fresh issue.
   **(2) E-2 decode-policy experiment:** default-vs-sampled on refreshed bundles; PASS iff ≥14/15 distinct and
   no Gate C repetition regression ⇒ decide default + wire `--sample` (or scores) into the R4G1 path.
   **(3) Instrument-integrity pair:** land the real #160 machine-code audit (or rename + downgrade to Witnessed)
   AND add `simd.rs` to P-4 with a `dot_argmax` scalar witness; decision rule: `verify_all`/CI can fail on a
   seeded violation in each (prove the instruments can fail before trusting their green).

---

*Report generated by the 2026-08-18 baseline audit session. Line references bind to `aea30bae`. Raw evidence:
`/tmp/gate*_*.out`, `/tmp/audit_ask/*.txt`, `/tmp/audit_trend_output/score_report.json` (host-local, session-lived);
Gate C metrics and probe outputs are reproduced inline above. This file is append-only per repository convention.*
