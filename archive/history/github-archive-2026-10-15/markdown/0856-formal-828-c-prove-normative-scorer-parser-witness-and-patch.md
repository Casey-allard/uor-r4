# #856 — formal/#828-C: prove normative scorer, parser, witness, and patch invariants with Kani and Lean

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:43:55Z
- **Updated:** 2026-08-25T23:32:15Z
- **Closed:** 2026-08-25T23:32:14Z
- **Labels:** conformance
- **Milestone:** R4 Intelligence / S7 — Proof, release, and distribution
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/856
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #828

## Context

Executable proof obligations and the existing GNAF proof provide useful foundations, but final claims must attach to the stable normative scorer, format validator/parser, witness rules, and patch/composition semantics actually used in production.

#859 owns the early prime-router reference mathematics. This issue owns later refinement of stable normative production semantics and must not silently broaden that reference theorem boundary.

## Problem

Proofs can be vacuous, cover reference-only code, omit bounds/overflow/error branches, or establish a mathematical model without a refinement link to packed bytes and Rust execution.

## Goal

Freeze obligation statements, build non-vacuous Kani and Lean artifacts for the stable model, connect reference mathematics to packed/Rust semantics, and update the proof matrix with exact proved and assumed boundaries.

## Scope

- [ ] Enumerate obligations for structural/semantic validation, safe borrowed access, bounds/count arithmetic, score saturation/ties, deterministic state transitions, fixed work/capacity, typed decline/error, witness self-consistency/replay, patch precedence/composition, and no hidden fallback.
- [ ] Import or reference #859 theorem IDs rather than restating them without provenance.
- [ ] For every reused prime-router theorem, prove the required refinement to stable packed/Rust semantics or mark the router component reference-only/excluded from the release.
- [ ] State each theorem/contract under the formal vocabulary with explicit preconditions, postconditions, assumptions, finite bounds, and target code/spec identity.
- [ ] Use Kani for reachable Rust invariants, panic freedom under validated inputs, bounds/overflow/error paths, and patch/state/scorer properties where tractable.
- [ ] Use Lean for mathematical reference semantics, determinism, transition/witness validity, composition properties, and refinement lemmas where tractable.
- [ ] Connect packed bytes → validated view → normative step/state → witness with explicit refinement/differential artifacts.
- [ ] Add non-vacuity checks: cover success and failure branches, mutation/planted-counterexample tests, meaningful state spaces, and harness reachability.
- [ ] Keep GNAF GEMM proof scope explicit and do not use it as proof of unrelated serving components.
- [ ] Publish machine-readable proof artifacts, versions, hashes, reproduction commands, resource bounds, and proof-status matrix.
- [ ] Mark unproved obligations assumed/open and narrow release claims rather than using aspirational wording.

## Non-goals

- Prove the entire Rust compiler, LLVM, OS, or hardware.
- Re-prove #859's pure mathematical kernel or treat its reference theorems as proof of Rust/f64 or deployed behavior.
- Use source scan or exhaustive bounded testing as an unqualified mathematical proof.
- Refactor stable bytes solely to make a broad proof claim unless separately justified.

## Dependencies and entry gate

Blocked on #855, #859, and stable scorer/format/patch/local-composition semantics. Incremental proof work may begin, but final status binds exact release identities.

## Acceptance criteria

- [ ] Every mandatory obligation is proved, explicitly assumed/open, or removes/narrows the corresponding claim.
- [ ] Every #859 theorem relevant to a release claim is reused with an explicit refinement link or classified as reference-only/excluded.
- [ ] Proof harnesses reach the normative implementation/model and pass non-vacuity/mutation checks.
- [ ] Packed/reference/Rust refinement chain is explicit for every claimed theorem.
- [ ] Artifacts reproduce with pinned Lean/Kani/Rust dependencies and resource limits.
- [ ] Proof matrix distinguishes formal proof, executable/model check, source check, binary check, empirical test, and assumption.

## Verification

- [ ] Run all Kani harnesses and Lean builds from clean pinned environments.
- [ ] Inject planted counterexamples/mutations for tie, overflow, bound, patch-order, witness, and decline properties and verify failure.
- [ ] Coverage/non-vacuity audit across success/error/capacity branches.
- [ ] Reference/packed/Rust differential suites tied to theorem examples.
- [ ] Claim wording/register/proof-matrix completeness checks.

## Compatibility and migration

Proofs bind exact spec/code/artifact eras. Semantic or byte changes invalidate the mapped obligations and require proof updates; historical proof artifacts remain content-addressed evidence for their era.

## Repository conformance

- **Execution scope:** Lean reference semantics and Kani harnesses reaching stable normative Rust parser/scorer/state/witness/patch/composition code; each proof/check/report is credited only to the exact reachable bytes, target, build mode, and obligation it covers.
- Preserve P-4, zero-allocation, deterministic artifacts, bounded work/state, typed errors, no_std, unsafe prohibitions, witness replay, and compatibility where normative.
- Guarantee/assumption/empirical/proof-status language follows the formal vocabulary and includes identities, platform scope, and PASS/FAIL/UNAVAILABLE.
- **Conformance mapping:** RF-06, RF-09, RF-10, RF-11, RF-13, RF-15, RF-17 plus relevant built capability IDs.; do not mark a capability built merely because a reference model or harness exists.
- Built capability order remains IDs TOML → Gherkin → failing marker/behavior test → implementation → regenerated conformance.
- Proof and release evidence updates the source model; generated conformance is never hand-edited.
- Append reproduction/measurement records and reconcile every public claim to the final claim map.
- Lean/Kani establish only encoded obligations; hardware instruction claims remain sibling B's scope.

## Claim status and next action

Publish obligation-specific statuses. “Fully formally verified” is prohibited unless sibling A's entire claimed TCB is actually covered.

## Comments (1)

### comment 5418398659 — 2026-08-25T23:32:14Z — Casey-allard

Closed as not planned / not triggered by the geometric-decoder architecture reset #948 / #949. Proving, auditing, reproducing, or distributing the current graph product before coherent generation is exactly the sequencing error being corrected. Historical proof and release assets remain intact. The successor release decision is #955, after the geometric decoder is product-viable. This is not completion evidence.


---
_Comment/review ids in this file: 5418398659_

