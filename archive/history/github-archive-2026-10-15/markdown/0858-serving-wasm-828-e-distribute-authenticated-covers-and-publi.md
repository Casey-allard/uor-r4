# #858 — serving/WASM/#828-E: distribute authenticated covers and publish the final capability record

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:43:57Z
- **Updated:** 2026-08-25T23:32:18Z
- **Closed:** 2026-08-25T23:32:18Z
- **Labels:** enhancement, conformance, status: on-hold
- **Milestone:** R4 Intelligence / S7 — Proof, release, and distribution
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/858
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #828

## Context

Remote specialized covers are useful only after deterministic local composition is certified. Network loading adds trust, rollback, replay, resource-exhaustion, partial-download, and hidden-dependency risks. The final programme record is required whether or not network composition triggers.

## Problem

A client that fetches unbounded or incompatible bundles, changes composition by arrival order, or trusts a hash without an explicit authority policy would violate deterministic/offline guarantees. Conversely, making optional distribution block the final research verdict would conflate product scope with intelligence evidence.

## Goal

Keep network composition on hold until its trigger passes; if activated, reuse local semantics exactly with authenticated bounded fetch/activation. In all cases publish the final Gate A–K capability, conformance, negative-result, and open-limit record.

## Scope

- [ ] Keep `status: on-hold` until #852 local composition is certified and sibling A freezes network trust/threat assumptions.
- [ ] Post an activation decision based on a concrete distribution use case, supported hosts, trust roots, revocation/rollback policy, and resource envelope; allow `NOT TRIGGERED`.
- [ ] If triggered, define signed/content-addressed discovery/fetch manifests binding URI, CID/root, size, compatibility, capability, authority/signature, expiry/version, and composed result.
- [ ] Enforce byte/count/time/retry/concurrency/storage limits, streaming verification, atomic cache/activation, rollback, downgrade/replay policy, and typed failure.
- [ ] Verify CID/signature/trust, base/tokenizer/operator/format/capability/report compatibility before activation and reuse #852 precedence/conflict semantics exactly.
- [ ] Make network access explicit and opt-in; ordinary/offline inference performs no hidden fetch or telemetry.
- [ ] Provide equivalent supported HTTP and WASM host behavior or state the exact host limitation.
- [ ] Test malicious/tampered/oversize/partial/truncated/reordered/stale/revoked/wrong-base bundles and hostile endpoints.
- [ ] Publish a final Gate A–K matrix covering prompt causality, selective risk, generation, planning, geometry, instruction, scale, composition, operation contract, proofs, reproducibility, and distribution.
- [ ] Classify every row as guarantee, witnessed property, empirical criterion, negative result, limited capability, open research, NOT TRIGGERED, or UNAVAILABLE with exact evidence links.
- [ ] Issue the programme verdict: `GENERALIZATION GATE PASSED`, `LIMITED CAPABILITY`, or `RESEARCH FALSIFIER REACHED`; verify all public/release claims against it.

## Non-goals

- Make network access implicit or required for the offline product.
- Invent composition semantics different from local loading.
- Treat a BLAKE3 digest alone as authorization/trust.
- Delay the final capability record solely because optional network composition is NOT TRIGGERED.

## Dependencies and entry gate

Network work is on hold pending #852 and sibling A. The final capability record requires every mandatory stage verdict and sibling D release root; it can publish with network composition NOT TRIGGERED.

## Acceptance criteria

- [ ] If network work triggers, all downloaded layers authenticate, validate, compose, witness, and rollback identically to certified local semantics within hard limits.
- [ ] Offline mode makes no network calls and remains a first-class supported product.
- [ ] Malicious/incompatible/partial/replayed inputs cannot activate or corrupt the prior admitted state.
- [ ] Final Gate A–K record links every claim to exact evidence/status and includes all negative/null/unavailable outcomes.
- [ ] README/release notes/capability manifests match the final verdict with no broader language.

## Verification

- [ ] Local-versus-HTTP/WASM composition differential and deterministic arrival/order tests if triggered.
- [ ] Signature/trust/CID/size/time/partial/retry/revocation/downgrade/replay/wrong-base malicious fixtures.
- [ ] Network-disabled/no-hidden-fetch/telemetry test and atomic activation/rollback/restart tests.
- [ ] Automated completeness check across #820 children, Gate A–K record, conformance/proof matrices, release root, and public claims.
- [ ] Independent review/replay of the final capability record and programme verdict.

## Compatibility and migration

Network manifest/protocol is versioned and optional. Unsupported clients remain local/offline. Cache/activation never changes normative bytes or precedence; incompatible versions fail closed. Final records remain immutable/content-addressed by release era.

## Repository conformance

- **Execution scope:** Optional authenticated HTTP/WASM distribution feeding the certified normative local R4Engine composition path; mandatory final programme evidence record; each proof/check/report is credited only to the exact reachable bytes, target, build mode, and obligation it covers.
- Preserve P-4, zero-allocation, deterministic artifacts, bounded work/state, typed errors, no_std, unsafe prohibitions, witness replay, and compatibility where normative.
- Guarantee/assumption/empirical/proof-status language follows the formal vocabulary and includes identities, platform scope, and PASS/FAIL/UNAVAILABLE.
- **Conformance mapping:** RF-06, RF-09, RF-10, RF-11, RF-13, RF-15, RF-17 plus composition/distribution and all promoted capability IDs.; do not mark a capability built merely because a reference model or harness exists.
- Built capability order remains IDs TOML → Gherkin → failing marker/behavior test → implementation → regenerated conformance.
- Proof and release evidence updates the source model; generated conformance is never hand-edited.
- Append reproduction/measurement records and reconcile every public claim to the final claim map.
- Remote data never becomes trusted through transport alone; admission requires authority policy, content identity, compatibility, and capability evidence.

## Claim status and next action

Network composition is currently ON HOLD and may close NOT TRIGGERED. The final record must still publish the honest programme boundary.

## Comments (1)

### comment 5418399151 — 2026-08-25T23:32:17Z — Casey-allard

Closed as not planned / not triggered by the geometric-decoder architecture reset #948 / #949. Proving, auditing, reproducing, or distributing the current graph product before coherent generation is exactly the sequencing error being corrected. Historical proof and release assets remain intact. The successor release decision is #955, after the geometric decoder is product-viable. This is not completion evidence.


---
_Comment/review ids in this file: 5418399151_

