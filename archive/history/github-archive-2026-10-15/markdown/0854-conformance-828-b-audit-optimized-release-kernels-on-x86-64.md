# #854 — conformance/#828-B: audit optimized release kernels on x86_64, aarch64, and WASM

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:43:53Z
- **Updated:** 2026-08-25T23:32:11Z
- **Closed:** 2026-08-25T23:32:11Z
- **Labels:** conformance
- **Milestone:** R4 Intelligence / S7 — Proof, release, and distribution
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/854
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #828

## Context

The P-4 source scan and executable proof models constrain source-level operations, but optimizing compilers choose target instructions. The existing GNAF GEMM proof has a narrower boundary than the full normative scorer/load/patch path.

## Problem

A source expression may lower to unexpected multiply/divide/floating instructions, a forbidden operation may live in a reachable helper, or the scan may cover a non-serving implementation. Conversely, scanning an entire server would include legitimate out-of-kernel code and overstate the kernel boundary.

## Goal

Build the actual supported optimized release artifacts, identify the reachable inference-kernel call graph from production entry points, and produce reproducible target-specific instruction/opcode censuses with planted-negative detection.

## Scope

- [ ] Consume sibling A's exact kernel boundary and production entry points for x86_64, aarch64, and WASM.
- [ ] Pin Rust/LLVM/toolchain, target triples/features, profiles/LTO/codegen flags, feature sets, link inputs, and binary/module hashes.
- [ ] Build actual release library/binary/WASM artifacts used by supported deployment, not a synthetic substitute.
- [ ] Retain symbols/maps or deterministic address metadata sufficient to establish reachability from load/predict/step/patch/compose kernel entry points.
- [ ] Disassemble x86_64/aarch64 and decode WASM opcodes; census multiply, divide, floating point, SIMD variants, calls/imports, allocation, atomics, nondeterministic/time/RNG instructions as scoped by the contract.
- [ ] Separate reachable normative kernel code from unreachable or explicitly out-of-kernel server/tokenizer/compiler code and publish both boundary and exclusions.
- [ ] Correlate source P-4 scan, call graph, proof obligations, symbols, and machine instructions.
- [ ] Add planted forbidden-operation and hidden-helper fixtures that the binary audit must catch.
- [ ] Emit signed/content-bound census reports and reproduction commands per target.

## Non-goals

- Claim the entire application contains no multiply/float if only the inference kernel is intended.
- Rely on debug builds or source grep as machine-code evidence.
- Call cross-platform binary bytes identical unless explicitly established.

## Dependencies and entry gate

Blocked on sibling A's frozen boundary and stable S6 bytes/semantics. Target builders/toolchains must be available; otherwise status is UNAVAILABLE, not PASS.

## Acceptance criteria

- [ ] Every supported target has an actual release artifact, reachable call graph, opcode census, hashes, toolchain/config, and exclusions.
- [ ] No forbidden operation/import exists in the claimed reachable kernel or the claim is narrowed/failed.
- [ ] Planted direct and transitive forbidden operations fail the audit on each target.
- [ ] Source/proof/binary symbols can be traced through the evidence map.
- [ ] Reports are reproducible and release admission rejects missing or mismatched censuses.

## Verification

- [ ] Rebuild and hash/check the exact optimized artifacts; repeat census.
- [ ] Run direct/transitive/feature-gated/LTO/inlined forbidden-op planted negatives.
- [ ] Cross-check reachability with symbols/maps and production smoke traces.
- [ ] Run existing P-4/allocation/witness tests and compare their boundary with binary census.
- [ ] Independent spot review of disassembly/WASM opcode classifications.

## Compatibility and migration

Audit tooling/reports are additive. Pin platform/toolchain-specific identities; a toolchain/target/feature change invalidates only the affected census and requires regeneration.

## Repository conformance

- **Execution scope:** Actual optimized reachable inference kernel in supported x86_64, aarch64, and WASM release artifacts; each proof/check/report is credited only to the exact reachable bytes, target, build mode, and obligation it covers.
- Preserve P-4, zero-allocation, deterministic artifacts, bounded work/state, typed errors, no_std, unsafe prohibitions, witness replay, and compatibility where normative.
- Guarantee/assumption/empirical/proof-status language follows the formal vocabulary and includes identities, platform scope, and PASS/FAIL/UNAVAILABLE.
- **Conformance mapping:** RF-06, RF-09, RF-10, RF-11, RF-13, RF-15, RF-17.; do not mark a capability built merely because a reference model or harness exists.
- Built capability order remains IDs TOML → Gherkin → failing marker/behavior test → implementation → regenerated conformance.
- Proof and release evidence updates the source model; generated conformance is never hand-edited.
- Append reproduction/measurement records and reconcile every public claim to the final claim map.
- Machine-code operation claims require target-binary disassembly/opcode evidence; source scans remain a separate guarantee.

## Claim status and next action

Each target receives its own PASS/FAIL/UNAVAILABLE verdict. Do not summarize missing target evidence as universal PASS.

## Comments (1)

### comment 5418398032 — 2026-08-25T23:32:11Z — Casey-allard

Closed as not planned / not triggered by the geometric-decoder architecture reset #948 / #949. Proving, auditing, reproducing, or distributing the current graph product before coherent generation is exactly the sequencing error being corrected. Historical proof and release assets remain intact. The successor release decision is #955, after the geometric decoder is product-viable. This is not completion evidence.


---
_Comment/review ids in this file: 5418398032_

