# #829 — docs/governance: publish the R4 Intelligence Completion Plan and evidence-first issue forms (item A of #821)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:34:06Z
- **Updated:** 2026-08-19T23:29:50Z
- **Closed:** 2026-08-19T23:28:00Z
- **Labels:** documentation, conformance
- **Milestone:** R4 Intelligence / S0 — Truth and inference closure
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/829
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #821

## Context

The repository has strong evidence discipline but no authoritative post-v0.1 completion plan or issue forms. `AGENTS.md` still names the original graph-compiler plan as project direction; the old `Phase 0`–`Phase 11` milestones remain open despite having no open issues.

## Problem

Live summaries retain superseded interpretations: ROADMAP/RESEARCH still frame #784 as context/code collision; `.github/DOCUMENTATION_OVERHAUL_PLAN.md` remains Draft/Active; the historical and current planning eras are not clearly separated.

## Goal

Adopt one canonical completion-plan document, link every public entry point to it, add evidence-first implementation/research issue forms, and explicitly classify the historical milestones without rewriting evidence history.

## Scope

- [ ] Create `docs/r4_intelligence_completion_plan.md` with the #820 hierarchy, dependencies, promotion gates, kill/redesign criteria, global invariants, and definition of completion.
- [ ] Include #859 as the cross-cutting F0 lane and document the two-level proof strategy: early reference mathematics in #859, later production refinement in #856.
- [ ] Distinguish the R4 prime-router Lean package from `research/riemann-lean`, the vendored GNAF proof, Rust executable proof models, Kani, and target-binary evidence.
- [ ] Update `AGENTS.md`, `ROADMAP.md`, `docs/RESEARCH.md`, and the README roadmap link so the new document is authoritative for post-v0.1 intelligence sequencing.
- [ ] Add a status banner to `docs/r4_graph_compiler_implementation_plan.md`: retained as the original graph-compiler engineering plan and superseded only for post-v0.1 sequencing.
- [ ] Mark `.github/DOCUMENTATION_OVERHAUL_PLAN.md` superseded and correct its live architectural assertions.
- [ ] Correct current-summary prose after #784, #811, and #605/#804 while preserving measurement records append-only.
- [ ] Distinguish the W(3,3) visualization/router, R4G1 graph scoring, route-attention reference work, and the normative production path.
- [ ] Add `.github/ISSUE_TEMPLATE/research-experiment.yml` with hypothesis, pinned identities, reachability, nulls, cheap instrument, thresholds, outcome branches, costs, evidence record, and claim status.
- [ ] Add `.github/ISSUE_TEMPLATE/implementation.yml` with execution scope, dependencies, acceptance, compatibility, conformance mapping, verification, documentation, and claim status.
- [ ] Audit the 12 historical milestones and record a maintainer-visible disposition: completed/historical, retained open for a named reason, or superseded. Do not repurpose their titles.

## Non-goals

- Change runtime behavior or empirical thresholds.
- Delete or rewrite historical measurement records.
- Assign speculative roadmap work to maintainers.

## Dependencies and entry gate

None beyond #820. Use the issue tree and milestone URLs created with this programme as canonical targets.

## Acceptance criteria

- [ ] One canonical plan owns stage order and all public entry points link to it without duplicating a drifting phase table.
- [ ] Current docs state that #784 found continuation-distribution convergence rather than full-depth code collision and that #811 did not establish semantic abstention.
- [ ] Both issue forms validate on GitHub and encode the repository run/conformance contracts.
- [ ] Every old milestone has an explicit recorded disposition and remains distinguishable from `R4 Intelligence / S0–S7`.

## Verification

- [ ] Run `python3 scripts/check_claim_wording.py`.
- [ ] Run repository documentation/link checks and preview both issue forms.
- [ ] Search current-summary documents for the superseded collision, route-fit-unavailable, and router-integration claims.
- [ ] Confirm measurement records were appended/cross-referenced rather than historically rewritten.

## Compatibility and migration

Documentation/governance only. Existing issue URLs, old milestone names, artifact formats, and CLI/runtime behavior remain stable.

## Repository conformance

- **Execution scope:** Documentation and GitHub governance. Evidence outside this scope is not credited as a deployed-serving result.
- Preserve the applicable P-4, zero-allocation steady state, deterministic-byte, bounded-work, focused typed-error, `no_std`, and `forbid(unsafe_code)` contracts.
- Classify normative mathematics under `docs/formal_vocabulary.md`; empirical criteria name distribution, protocol, sample count, uncertainty, provenance identities, and status.
- **Conformance mapping:** No new capability ID expected; preserve the mappings and claim boundaries of RF-01…RF-29.. Reuse/extend an existing capability unless a genuinely new capability requires a new RF ID.
- A new built capability lands in order: `model/ids.toml` row → tagged Gherkin → failing marker/behavior test → implementation → regenerated `CONFORMANCE.md`.
- `CONFORMANCE.md` is generated and must not be edited directly. Retained off-serving mechanisms stay `open` in `model/ledger.toml` with an activation gate.
- Measurements that revise a claim append a per-issue record and reconcile README, ROADMAP, RESEARCH, lifecycle/configuration, and ledger assertions.


## Claim status and next action

This closes only when #820 is the visibly authoritative roadmap and the forms are usable. It does not promote any intelligence capability.

## Comments (1)

### comment 5349244536 — 2026-08-19T23:29:50Z — Casey-allard

## Definition of Done — #829 complete

**Merged:** PR #860 → `main` @ `d1d1f7357c44b7bcd30765a38236f7f56a40ef04` (2026-08-19T23:27:59Z), through the protected merge queue (no admin bypass; full queue-side ladder green).

### Deliverables on `main`
- **New canonical plan** `docs/r4_intelligence_completion_plan.md` — readable mirror of programme root #820: F0 + S0–S7 hierarchy with native children, dependency order, promotion gates, kill/redesign + global falsifiers, programme invariants, definition of completion; the **two-level proof strategy** (reference math #859 → production refinement #856); **proof-asset boundaries** (R4 prime-router Lean pkg vs `research/riemann-lean` vs vendored GNAF vs `uor-r4-proof-model`/Kani vs monograph vs target-binary evidence); **mechanism boundaries** (W(3,3) vs f64 router vs R4G1 scoring vs dormant route-attention vs the normative path); and a **historical-milestone disposition table**.
- **Evidence-first issue forms** `.github/ISSUE_TEMPLATE/research-experiment.yml` and `implementation.yml` — encode the repository run contract (hypothesis, pinned identities, reachability, nulls, cheap instrument, thresholds, outcome branches, cost, evidence record, claim status) and the conformance contract (execution scope, deps, acceptance, compatibility, RF mapping, verification, docs, claim status).
- **Entry points linked to the plan:** `AGENTS.md`, `ROADMAP.md` (also corrected the now-stale "zero open issues" line), `README.md`, `docs/RESEARCH.md`.
- **Status banner** on `docs/r4_graph_compiler_implementation_plan.md` (retained engineering plan; superseded only for sequencing); **`.github/DOCUMENTATION_OVERHAUL_PLAN.md` marked superseded** with live-assertion corrections.
- **Current-summary corrections** (records append-only): #784 = continuation-distribution convergence, not a full-depth context-code collision; #811 did not establish semantic abstention; #605/#804 S1 verdict FAIL, operator dormant.
- **12 historical milestones (Phase 0–11)** given a recorded disposition — a table in the plan **and** a live disposition banner prepended to each milestone description (titles unchanged, not repurposed).

### Acceptance criteria
- [x] One canonical plan owns stage order; all public entry points link to it without duplicating a drifting phase table.
- [x] Current docs state #784 found continuation-distribution convergence rather than full-depth code collision, and that #811 did not establish semantic abstention.
- [x] Both issue forms validate (local schema check PASS; merge queue accepted the change).
- [x] Every old milestone has an explicit recorded disposition and remains distinguishable from `R4 Intelligence / S0–S7`.

### Verification / evidence
- `python3 scripts/check_claim_wording.py` → **passed**.
- Issue-form schema validation (types, unique ids, required attributes, dropdown options) → **PASS** for both forms.
- `cargo fmt --check` → clean; merge-queue full ladder (fmt/clippy/tests/no_std/κ, Gate C trend, wasm-pack, fuzz smoke, cargo audit, register conformance) → **all success** on the speculative merge.
- Measurement records (`docs/<topic>_<issue>.md`) untouched — append-only preserved. Untracked `obs-text/` and `tf1-pkg/` deliberately not staged.

### Scope note
Documentation/governance only — no runtime, threshold, capability-ID, or generated-file (`CONFORMANCE.md`) changes. Does **not** promote any intelligence capability. Next sequential S0 item: **#830**.



---
_Comment/review ids in this file: 5349244536_

