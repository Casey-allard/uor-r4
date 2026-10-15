# #787 — conformance: make the #160 machine-code audit real or honest, and close the P-4 scan coverage gaps (simd.rs, dot_argmax witness)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-18T14:51:44Z
- **Updated:** 2026-08-18T21:37:32Z
- **Closed:** 2026-08-18T21:37:32Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/787
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem

Two of the repository's honesty instruments overclaim, found by the 2026-08-18 baseline audit (findings AUD-INV-001/-002; line refs at `aea30bae`). Both violate the repo's own "an instrument that cannot fail is indistinguishable from one that passes" rule.

**(a) The "machine-code audit" CI gate is vacuous.** `uor-r4-proof-model::inference_audit` (CI step "Inference contract machine-code & dependency audit (issue #160)", ci.yml:107-108) scans a **4-line hardcoded assembly string** (`inference_audit.rs:135`), checks a **3-element hardcoded dependency array** (`:146`), and returns `steady_state_allocations: 0` as a **literal** (`:161`). No disassembly, no allocator hook (the real counting allocator in `allocation_proof.rs` is never called from here), no real manifest read. Meanwhile `proof_matrix.rs:41-48` marks Operation-Set Conformance `Verified` while its own description says "until disassembly audit lands", and `verify_all` can only fail on a status (`Unverified`) that is never instantiated — the matrix is a hardcoded 8/8-Verified table. The *accurate* statement already exists: `INFERENCE_OPERATION_CONTRACT.md` §6/§8 (source-scan evidence is **Witnessed** until the disassembly audit lands).

**(b) The P-4 source scan does not cover the hot path it claims to.** `runtime.rs` delegates to `transformerless::simd.rs` at five sites (`runtime.rs:572, 702, 866, 875, 1229`); `simd.rs` is outside every scan list and holds all of core's hot-path `unsafe` (14 sites). The SIMD **hamming** path has a scalar-equivalence witness (`test_simd_hamming_equivalence`); the **`dot_argmax`** path does not (the layouts test compares two layouts through the same dispatcher, `simd.rs:670-695` — on AVX2/NEON hosts neither side is scalar). Also unscanned: 5 of 11 graph-runtime modules (`vp_tree.rs:142` carries a load-time `/ 2` — legal under the steady-state boundary, but unwitnessed), 2 of 9 `ACTIVITY_OWNERS` modules; one owner names a function that does not exist (`generate_into`); two contract version constants coexist (0.1.0 at `inference_contract.rs:56`, 1.0.0 at `:184`) with no cross-check. `PROOF.md`'s "COMPLETE runtime arithmetic surface" sentence has already been corrected by an appended scope note (pending docs diff).

## Scope

- **E-a** — maintainer decision, then implement ONE of: (1) land the real #160 audit — objdump/symbol disassembly of the release binary's contract-owned functions + a real `cargo metadata` denylist walk + wire the existing counting allocator; or (2) rename the CI step to what it does and downgrade the proof-matrix row to the honest Witnessed-equivalent status. Either restores truthfulness; (2) is an hour, (1) is real work.
- **E-b** — add `simd.rs` to the P-4 scan (or give its exclusion an explicit witnessed-equivalence note per function), add a `dot_argmax` scalar-vs-SIMD differential, sweep the remaining unscanned graph-runtime modules and the two unscanned activity owners, fix the phantom `generate_into` owner string, reconcile the two contract versions.
- **E-c** — consolidate the four divergent source-scanner copies (core `mod.rs`, `msa_selector_643.rs`, `r4g1_runtime_test.rs`, `bott_fock.rs`) onto one implementation.

## Decision rule (applies to every sub-item)

Each instrument must be shown to **fail on a seeded violation** (temporary branch demo recorded in the PR) before its green is trusted — the same rule `docs/RESEARCH.md` applies to measurement harnesses.

**Next action:** the E-a decision (real vs. rename) — everything else is independent and can start with E-b's `dot_argmax` witness.


## Comments (1)

### comment 5334369357 — 2026-08-18T21:37:31Z — Casey-allard

**Resolution — closing as completed.** Delivered by PR #801, merged to `main` as `4ca5519e`. Every scope item below maps to shipped code; per this issue's own decision rule, each instrument was demonstrated red on a seeded violation before its green was trusted (falsifier ledger at the end).

## E-a — maintainer decision: option (2), "make it honest" (Casey, 2026-08-18)

- CI step renamed to what it does: **"Inference audit engine self-test (source-witness level; #160 disassembly audit pending)"** (was "Inference contract machine-code & dependency audit (issue #160)").
- `inference_audit` rewritten as an honest engine self-test: `audit_all` → `audit_reference_fixtures`; the literal `steady_state_allocations: 0` and its never-fireable `UnexpectedAllocationDetected` error are **removed** (they measured nothing); `is_certified` → `fixtures_clean`. The module no longer claims disassembly, dependency-manifest, or allocator evidence it does not gather.
- Proof matrix gains a **`ProofStatus::Witnessed`** tier; the Operation-Set Conformance row is downgraded `Verified` → `Witnessed`, matching what `INFERENCE_OPERATION_CONTRACT.md` §6/§8 already said (source-scan evidence is Witnessed until the disassembly audit lands). `verify_all` can now actually fail — first falsifier the matrix has ever had.
- The certify-side mirror follows: `target_operator_certificate`'s Operation-Set link status is `"Witnessed"`.
- The **real** #160 audit (objdump disassembly + `cargo metadata` denylist walk + wired counting allocator) remains future work under #160, exactly as the decision scoped.

## E-b — scan coverage + witnesses

- **`dot_argmax` differential**: new proptest `simd_dot_argmax_matches_scalar_reference` compares the runtime dispatcher against the scalar reference on both compact and legacy layouts. The enabling fix: `dot_argmax_scalar` (+ its two helpers) was `#[cfg(not(target_arch = "aarch64"))]` — compiled out on exactly the host family (NEON) whose SIMD path most needed a witness. It now compiles on all targets (`cfg_attr(aarch64, allow(dead_code))` where unused outside tests).
- **`simd.rs` exclusion is pinned, not silent**: `p4_simd_exclusion_is_pinned_to_its_equivalence_witnesses` asserts the `simd::` delegation count in the runtime is exactly **11** — any new call site fails the pin until it gets its own equivalence witness.
- **Full graph-runtime coverage**: the P-4 scan now walks **all 11** graph-runtime modules plus `reference_state.rs` and `src/r4g1.rs` (the two previously-unscanned `ACTIVITY_OWNERS` modules). New coverage immediately caught `vp_tree.rs:59/60` (`checked_mul(8)`) alongside the known `:142` (`/ 2`) — all load-time construction, legal under the steady-state boundary, now carrying explicit `p4-allow(load-time):` markers with the allowance list **pinned byte-for-byte to exactly those 3 lines** (a 4th allowance, or drift in any of the 3, fails the test).
- **Contract versions reconciled**: `InferenceContractVersion::CURRENT` is now derived from `INFERENCE_OPERATION_CONTRACT_VERSION`; the stray `V1_0_0` is removed; a 3-way sync test parses the normative doc's `- **Version:**` line so doc/constant/report cannot drift silently. The BDD contract-version step asserts against the canonical constant instead of a hardcoded string.
- **Corrected audit finding**: the audit's "phantom `generate_into` owner" claim (AUD-INV-002 sub-item) was **wrong** — verification found the function exists (`src/r4g1.rs:410`, plus `uor-r4-api::engine::generate_into`). The owner string `uor-r4-wasm-router::r4g1::{encode_into,decode_into,generate_into}` was accurate all along; no change made, and this record corrects the audit doc's claim so it doesn't propagate.

## E-c — one canonical scanner

The four divergent scanner copies (`transformerless/mod.rs`, certify's `msa_selector_643.rs`, `r4g1_runtime_test.rs`, `bott_fock.rs`) are consolidated onto **`uor_r4_core::transformerless::source_scan`**: comment-stripping and string/char-literal masking (lifetime-tick-safe — a bare `'a` never opens a mask, so `'a * b'`-shaped real operators can't hide), method-form needles (`.wrapping_mul(` etc.), a float variant, and an explicit `p4-allow(<scope>):` allowance protocol whose per-caller expected lists are exact-matched (offenders **and** unexpected allowances both fail).

## Falsifier ledger (decision-rule compliance)

Demonstrated red before trusting green, recorded in the PR run: (1) scanner falsifier suite incl. `lifetime_mul` (a `*` adjacent to a lifetime tick **must** flag) and char-literal `'/'` (**must not** flag); (2) `verify_all_fails_on_a_seeded_unverified_entry` (matrix); (3) the allowance-list exact pin (seeded extra allowance fires); (4) the delegation-count pin (seeded 12th `simd::` call fires); (5) the 3-way version-sync test (seeded doc bump fires).

Gates on the merged tree: fmt, clippy `-D warnings` (workspace), core 58 / graph-runtime 11 / certify-msa 26 / graph-format 7 / proof-model suites green, registered 29/29, R1/R4/R5, wasm32 lib, BDD in CI. AUD-INV-001/-002 are resolved: no repository instrument now claims evidence it does not gather.



---
_Comment/review ids in this file: 5334369357_

