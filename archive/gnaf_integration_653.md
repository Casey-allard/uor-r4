# GNAF integration matrix (#653 phase 2)

Classifies every top-level layer and claim of the vendored
`proofs/wasm-gemm-gnaf/` tree (phase 1, #742, pinned at
`171652cd95c0b8e8620f76151b7e0c485e30ccfc` — see
[`gnaf_import_provenance.md`](gnaf_import_provenance.md) for the exact
provenance) per #653's own scope: `adopt-active`, `retain-formal-reference`,
`defer-open`, or `reject`, each with its r4 destination/owner, reason, and
evidence. This is a classification and decision record, not an
implementation — no adapter code, CI wiring, or Atlas/falsifier porting
happens in this phase. Those are the concrete `defer-open` items below,
each scoped enough to become its own follow-up issue rather than staying an
open-ended aspiration.

## What the vendored proof actually establishes

Read `README.md`/`CERTIFICATION.md`/`model/claims.json` at the pin before
classifying anything, per this repo's own discipline of verifying against
the real content rather than the issue text's summary. The registry holds
45 claims: 28 `formalProof` (kernel-checked, sorry-free, axiom closure
`propext`/`Quot.sound`/`Classical.choice`), 2 `authority`, 3
`buildEvidence`, and **12 `open`** — the repository's own terminal answer
for its release theorem is `WorkloadIncomplete`, by design, and `just vv`
is meant to fail at gate step 9 today. The 12 open claims include the
headline `GO-001` (`released_wasm_gemm_gnaf_global_optimal`) and its
exhibition/coverage/semantics dependencies (`GO-006`, `GO-007`, `WS-001`,
`UV-001`, `LB-001`, `BI-002`, others). **None of this proves anything about
r4's own kernels.** The proved half (`Universal.exists_globalOptimal_of_nonempty`,
`Cost.coordinate_le_score`, `Atlas.semantic_closure_least`,
`Atlas.incremental_eq_full_rebuild`, etc.) is about a *reference* WASM GEMM
module under an abstract 36-coordinate cost objective defined in the
authority's own `SPEC.md` §9 — not about `uor-matmul`, the table-native
transformerless kernels, or any artifact this repo actually compiles or
serves. Every classification below keeps that boundary explicit, per
#653's own non-goal: "Do not claim that its Wasm GEMM results prove r4
model correctness, quality, equivalence, or optimality."

## Classification

| Layer | Classification | r4 destination / owner | Reason | Evidence |
| --- | --- | --- | --- | --- |
| `authority/` (pinned GNAF spec, Wasm Core pin, frozen `WGG-GO-1` proposition) | `retain-formal-reference` | `proofs/wasm-gemm-gnaf/authority/` (as vendored) | Immutable pinned inputs to the proof; nothing in r4 consumes them directly. Provenance already independently re-verified (phase 1). | `gnaf_import_provenance.md` |
| `WasmGemmGnaf/Foundation/` (canonical identity, ordering, result algebra) | `retain-formal-reference` | none (Lean-only) | Generic proof scaffolding for this specific theorem; no r4 Rust type currently mirrors it, and porting a result-algebra pattern without a concrete consumer would be speculative, not active. | `README.md` layout table |
| `WasmGemmGnaf/Cost/` (36-coordinate cost vector, canonical objective, `coordinate_le_score`) | `defer-open` | future: `uor-r4-graph-certify`/`uor-r4-proof-model` cost/witness fields | #653's own scope names this exact pattern as the phase-2+ target: "Apply the typed plan/scope/resource/cost/witness pattern to at least one active pipeline seam. The #602–#606... path is the initial candidate." Not started — requires designing an r4-native cost-vector schema for the #602–#606 seam, not importing the Lean cost type itself (that type is scoped to the WASM GEMM profile, not r4's operators). | `model/claims.json` (`CO-001..CO-004`, all `formalProof`, `discharged`) |
| `WasmGemmGnaf/Wasm/` (Core Wasm subset semantics, encode/decode round-trip, cost-transparency) | `retain-formal-reference` | none | Models a WebAssembly binary target. r4 does not compile to or serve WASM as part of this proof's chain; the modelled subset (`i32` only, per `README.md`'s "What is not proved" table, `WS-001`) is also explicitly incomplete relative to Core 3.0. No current or planned r4 consumer. | `README.md` ("What is not proved") |
| `WasmGemmGnaf/Gemm/` (the GEMM implementation being proved optimal) | `reject` (for correctness transfer) / `retain-formal-reference` (as source) | none | This proves properties of a *reference* GEMM module under GNAF's abstract cost objective — not `uor-matmul` or any kernel r4 actually ships. Treating its results as evidence about r4's own matmul correctness or optimality would be exactly the "green-washing" #653's own Problem section warns against. Retained only as inert source. | #653 non-goals: "Do not claim that its Wasm GEMM results prove r4 model correctness..." |
| `WasmGemmGnaf/Atlas/` (least-closure, incremental-vs-full-rebuild equality, coverage-seal scope-blindness) | `defer-open` | future: `xtask`/compiler incremental-compile seam (e.g. `compile-recorded`'s resumable path) | #653 names this explicitly: "Apply the Atlas update/rebuild discipline to a current incremental seam (patch/epoch or resumable compile)." `Atlas.incremental_eq_full_rebuild` and `Atlas.universalCoverCompleteCheck_scope_blind` are both `formalProof`/discharged upstream, but adopting the *discipline* (not the Lean code) into an r4 incremental-compile fixture is unstarted work with its own acceptance test (byte-identical incremental-vs-clean rebuild). | `model/claims.json` (`AT-*` ids); `README.md` proved-declarations table |
| `WasmGemmGnaf/Universal/` (existence/subsingleton theorems backing `GlobalOptimal`) | `retain-formal-reference` | none | Proof-theoretic machinery specific to establishing the GEMM release theorem; not a pattern r4 has an active consumer for. | `README.md` proved-declarations table |
| `WasmGemmGnaf/Conformance/` | `retain-formal-reference`; the *discipline* is `defer-open` via `Tools/` (below) | see `Tools/gen_conformance.py` row | The Lean namespace is specific to this proof's own conformance predicates. The generated-`CONFORMANCE.md` *mechanism* parallels r4's own generated conformance doc and is the real transferable piece — tracked under `Tools/` below, not here. | directory listing |
| `WasmGemmGnaf/GNAF/` (claim/status vocabulary: `formalProof`/`buildEvidence`/`measurement`/`open`/`authority` levels, execution/optimization status types) | `adopt-active` (partial — vocabulary only, not wired) | `uor-r4-naf` crate (already landed, #623/PR#631) | The GNAF claim/status vocabulary was already adopted into `uor-r4-naf`'s `core + integer + tensor + address` slice before this repo's Lean source was ever vendored. What's still `defer-open` is wiring that vocabulary into live `graph-certify`/target-operator certificate and API result records — #653's scope says this explicitly ("Wire the already-adopted GNAF claim vocabulary into actual graph-certify/target-operator certificate and API result records") and it has not happened; the vocabulary exists in `uor-r4-naf` but nothing in `uor-r4-graph-certify` or the server's result types constructs or reads it yet. | `crates/uor-r4-naf` (landed); grep of `uor-r4-graph-certify`/`src/server.rs` for GNAF-vocabulary usage (none found) |
| `WasmGemmGnaf/Theorems/` | `retain-formal-reference` | none | Assembled top-level theorem statements for this proof; downstream of `Universal`/`Cost`/`Wasm`, same reasoning applies. | directory listing |
| `WasmGemmGnaf/Artifact/` (`Release.lean` — the committed release module, `DeciderAnswersAdmissible`) | `retain-formal-reference` | none | Handles the Lean proof's own target binary/release object; unrelated to r4's own artifact formats (`TLA5`/`TLS1`/`R4G1`). Also where 2 of the 12 open claims live (`UV-003`, `WS-003`), i.e. actively incomplete upstream. | `model/claims.json` (`UV-003`, `WS-003`, both `sourceModule: WasmGemmGnaf/Artifact/Release.lean`) |
| `Tools/*.py` (`axioms.py`, `firewall.py`, `gate.py`, `gen_conformance.py`, `manifest.py`, `mutation.py`, `releasepath.py`, `required.py`, `root.py`, `scan.py`) | `defer-open` | future: `xtask` subcommands (Rust ports of the *disciplines*, not the Python scripts) | #653 names this directly: "Adapt generic proof-integrity mechanisms into existing repo-conformance/xtask ownership rather than creating parallel r4 registries: authority/tool pins, spec-derived required-declaration inventory, compiled-root/axiom audit, dependency firewall, acyclic staged manifests, and deterministic generated conformance records." r4's `repo-conformance`/`xtask` crates already implement the equivalent disciplines for r4's own artifacts in Rust; porting means designing r4-native equivalents of each script's *check*, not vendoring Python into a Rust workspace. Each of the ~9 scripts is its own scoped follow-up (e.g. "port `firewall.py`'s dependency-firewall check as an `xtask` subcommand over r4's own crate graph"). | `Tools/` directory listing; `xtask`/`repo-conformance` crates (existing, unrelated implementation) |
| `.github/workflows/{mutation,reproducible,verify}.yml` | `defer-open` | future: r4 CI (queue-side gate vs. PR-side stub, per `AGENTS.md`'s documented CI split) | Vendored as files only; **not** registered as r4 workflows today (verified: no `lean-toolchain` install step anywhere in `r4`'s own `.github/workflows/`). #653's own scope flags the real open question: whether every merge-queue run pays a Lean-toolchain-install+build cost indefinitely, or whether Lean verification runs on a narrower trigger (e.g. only `proofs/wasm-gemm-gnaf/**` changes) — that's a CI-cost/policy decision for whoever picks this phase up, not a default to assume either way. | `gnaf_import_provenance.md` ("Not in this phase") |
| `model/*.json` (`claims.json`, `dependencies.json`, `falsifiers.json`, `profiles.json`, `reproducibility-plan.json`, `spec-deviations.json`) | `retain-formal-reference`; bridging is `defer-open` | future: cross-reference from r4's own `model/ids.toml`/`model/ledger.toml`, if/when the adapter layer above lands | Upstream's own registry, self-consistent and already independently spot-checked (45 claims, level counts above). r4's `model/ids.toml`/`model/ledger.toml` track r4's own compiled bundles and have no current field for a GNAF claim id — adding one is part of the deferred adapter-wiring work, not a standalone task. | `model/claims.json` (spot-checked, 45 claims) |
| `vendor/wasm-spec` | `retain-formal-reference` | none | Vendored-by-upstream WebAssembly spec text; pure reference material for the Lean proof, two levels removed from r4. | directory listing |
| `Tests/`, `fixtures/`, `artifacts/` (upstream's own, currently placeholder `README.md` only at this pin) | `retain-formal-reference` | none | Empty/placeholder at the vendored pin; nothing to classify beyond "present, inert." | directory listing (each contains only `README.md`) |

## Summary

- **`adopt-active` (partial):** GNAF claim/status vocabulary (already in
  `uor-r4-naf`). Wiring it into live certificate/result records remains
  `defer-open`.
- **`defer-open` (real, scoped future work, each independently pickup-able):**
  the `#602`–`#606` cost/witness seam (`WasmGemmGnaf/Cost/`), the
  incremental-compile Atlas discipline (`WasmGemmGnaf/Atlas/`), the
  proof-integrity tooling port (`Tools/*.py` → `xtask`), CI wiring for
  Lean verification (`.github/workflows/*.yml`), and the `model/`
  registry cross-reference (once the adapter layer exists).
- **`retain-formal-reference`:** everything else — the proof content
  itself (`Foundation`, `Wasm`, `Universal`, `Theorems`, `Artifact`,
  `Conformance` Lean namespaces), the pinned `authority/`, and the
  vendored `vendor/wasm-spec`.
- **`reject` (for correctness transfer only, not deletion):**
  `WasmGemmGnaf/Gemm/` — its optimality results describe a different,
  reference GEMM module, not r4's own kernels, and must never be cited as
  evidence about them.

No layer is unclassified, silently duplicated, or left as unregistered
dead code — every directory and script family under `proofs/wasm-gemm-gnaf/`
appears in the table above with a destination and a reason.

## What this phase does not do

Per #653's own non-goals, this document does not itself: wire any GNAF
vocabulary into `graph-certify`/API result records, port any `Tools/*.py`
check into `xtask`, register any CI workflow, apply the Atlas discipline to
any r4 seam, or bump the vendored pin. Each `defer-open` row above is
sized to become its own follow-up issue when picked up, rather than a
single monolithic "phase 3."
