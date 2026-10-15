# #836 — format/runtime: lower and certify the selected prompt-state scorer (item C of #822)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:35:24Z
- **Updated:** 2026-08-21T01:55:14Z
- **Closed:** 2026-08-21T01:53:55Z
- **Labels:** enhancement, conformance
- **Milestone:** R4 Intelligence / S1 — Prompt-conditioned state
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/836
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #822

## Context

A prompt-state mechanism belongs in production only after sibling item B selects a bounded arm under causal and equal-budget controls.

## Problem

A positive offline/reference result does not automatically preserve packed semantics, historical artifacts, P-4, allocation, latency, or witness replay on R4Engine.

## Goal

Encode the selected state/evidence arm in a validated R4G1-compatible form, execute it through the normative scorer, and issue a production-scope promotion or retirement verdict.

## Scope

- [ ] Record the exact positive selection report and limits that activate this issue; close `NOT TRIGGERED` if no arm is selected.
- [ ] Define a versioned optional section or prove an existing section encodes the state without ambiguity.
- [ ] Add structural then semantic validation for capacities, offsets, IDs, score ranges, saturation, and compatibility.
- [ ] Implement compiler packing and a borrowed/zero-copy view where applicable.
- [ ] Execute the selected transitions and score contributions in the #831 normative R4Engine path using caller-owned fixed-capacity state.
- [ ] Extend witnesses with state lane, candidate-support, residual, saturation, and eviction attribution.
- [ ] Add reference-to-packed and scorer-to-production differential tests plus planted semantic mutations.
- [ ] Expose deterministic session/reset/streaming behavior consistently across CLI, API/server, and supported WASM.
- [ ] Run the frozen causal benchmark on the packed production path and record quality/resource deltas.

## Non-goals

- Implement an unselected or negative arm.
- Add inference-time teacher/network access, floating point, multiplication, division, or hot-path allocation.
- Change old-artifact behavior when the optional section is absent.

## Dependencies and entry gate

Blocked by a `SELECT` verdict in sibling item B and by #831. Requires the frozen #832 report and #833 canonical baseline.

## Acceptance criteria

- [ ] Production packed outputs/witnesses are bit-equivalent to the selected reference semantics on all differential fixtures.
- [ ] Old artifacts preserve exact behavior; malformed/incompatible sections fail closed with focused typed errors.
- [ ] The hot path remains P-4, allocation-free, bounded, deterministic, and no_std-compatible.
- [ ] The packed deployed result re-clears the causal-relevance gate and resource/non-regression budgets.
- [ ] The final verdict is `PROMOTE`, `REVISE`, `RETIRE`, or `NOT TRIGGERED`, with ledger and public claim updates.

## Verification

- [ ] Two-stage format validation, round-trip/golden bytes, corrupt-offset/capacity/range fuzz cases.
- [ ] Reference/packed and packed/production differential suites with planted negatives.
- [ ] Workspace tests no-fail-fast, clippy, fmt, graph-format no_std ladder, wasm target check, deterministic rebuild.
- [ ] Allocation census, P-4 source/operation scan, witness replay/self-consistency, streaming/reset/concurrency tests.
- [ ] Frozen causal benchmark replay with EXCT-on/off attribution and byte/latency/table-read reporting.

## Compatibility and migration

Use an optional versioned section with absent-section identity behavior. Document artifact-era, API-state, cache, session, and recompile consequences; preserve deterministic loading of supported historical bundles.

## Repository conformance

- **Execution scope:** Normative deployed R4Engine serving path across CLI/native API/server and supported WASM. Evidence outside this scope is not credited as serving.
- Preserve applicable P-4, zero-allocation steady state, deterministic bytes, bounded state/work, typed errors, `no_std`, and witness obligations.
- Normative equations/statements follow `docs/formal_vocabulary.md`; empirical criteria include distribution, protocol, n, uncertainty, identities, and status.
- **Conformance mapping:** RF-01, RF-21, RF-22 and the selected extension of RF-27/RF-28 or a justified new prompt-state capability.. Extend existing capability suites unless the implementation creates a genuinely new capability.
- Built capability order is `model/ids.toml` → tagged Gherkin → failing marker/behavior test → implementation → regenerated `CONFORMANCE.md`.
- Dormant/unselected arms remain ledgered with activation or retirement gates. Generated conformance is never hand-edited.
- Claim-changing measurements append a per-issue record and reconcile all current summary assertions.
- Verification must exercise the actual production request path, not only R4G1Runtime, certifier, or reference models.

## Claim status and next action

Do not merge a quality claim with the implementation. Promotion requires the independent frozen production-path report and precise scope.

## Comments (2)

### comment 5361107553 — 2026-08-20T20:09:48Z — Casey-allard

### #836 activated — increment 1 (PSTATE format section) in PR #876

**Activation report (the positive selection that triggers this issue):** #834's teacher-grounded run found the deployed model suffix-local (#874), but the #835 **segment lane** reference arm is a positive — whole-prompt content is predictive beyond the suffix: +17.5‰ (95% CI [15.9, 19.0]) and follows 10/4,722 minimal pairs where the suffix baseline follows 0 (#875, `docs/prompt_arms_bakeoff_834.md` §6.2). Limits: the effect is **modest** (+1.75pp overall; few hard same-suffix pairs resolved), so this is pursued with calibrated expectations, and the **packed** path must re-clear the causal gate (this issue's own acceptance).

**Increment plan** (mergeable, gated, one PR each):
1. **Format section (PR #876, this increment):** the optional versioned R4G1 `PSTATE` section + two-stage validation + borrowed view + tests. Additive/non-serving (absent-section identity); no `model/ids.toml` row yet.
2. Compiler packing — write PSTATE from the fitted segment lane (quantized residual table), deterministic bytes.
3. Hot-path scorer + witnesses — consume PSTATE in the #831 normative R4Engine path on the multiplication-free/allocation-free hot path with caller-owned fixed-capacity state; extend witnesses with lane/candidate-support/residual attribution. This is where the new serving capability + its `ids.toml` row → Gherkin → failing marker → implementation → regenerated `CONFORMANCE.md` land.
4. Causal-gate re-run on the packed path + resource/non-regression deltas → `PROMOTE` / `REVISE` / `RETIRE` verdict + ledger/claim updates.


### comment 5364242412 — 2026-08-21T01:55:14Z — Casey-allard

## #836 complete — segment lane lowered & certified; verdict **REVISE** (dormant)

The #835 segment prompt-state scorer is lowered end-to-end onto the deployed
`R4Engine` serving path and certified. Closing with an evidence-backed **REVISE**:
the mechanism is deployed and safe, but its whole-prompt-content effect is real
yet **sub-floor**, so the lane stays **dormant** (no `model/ids.toml` serving
row, `CONFORMANCE.md` unchanged, no quality claim).

### What landed (all gated, additive, absent-section-identity preserving)

- **#876/#877** — R4G1 `PSTATE` section: layout, validation, borrowed no_std view.
- **#878/#879** — P-4 hot-path `SegmentRing` + `SegmentSession` lifecycle.
- **#880/#882/#883** — `R4Engine` wiring: `predict_decision_candidates_with_segment[_witness]`, witness attribution, and (4c-i, #883) consumption of the **learned** content→candidate residual table (`table_score`/`table_contribution`, P-4).
- **#884 (4c-ii)** — `segment_fit::fit_segment_table` (faithful compiler-side lowering of the #834 §6.2 arm: content→teacher-argmax tally, top-K, quantized to integer `ScoreQ`) and `convert_with_segment_table` emission; fit→emit→load→consume proven end-to-end.
- **#885 (4c-iii)** — verdict docs: `docs/segment_lane_836_verdict.md`, `docs/segment_lane_836_result.json`.

### Verdict rationale (reachability arithmetic — no hours-long run launched)

The max whole-prompt-content signal measured anywhere is the §6.2 reference arm:
**+17.5‰, 95% CI [15.9, 19.0]**, against the pre-registered **20‰** causal floor
(`CAUSAL_FLOOR_PERMILLE`). The optimistic ceiling **19.0‰ < 20‰**, and the
deployed lane is strictly weaker (integer-`ScoreQ` quantization + top-8 candidate
bound). Ceiling < floor is **unconditional**, so per the long-run discipline no
full causal run was launched. The mechanism gap the original 4c contract flagged
(recurrence proxy vs learned table) is now **closed** by 4c-i/4c-ii, so the
verdict rests purely on this arithmetic.

### Safety

The `PSTATE` section is optional; every existing artifact carries none and is
byte-identical under the new code. The re-rank only reorders the already-decided
top-8 — it never fabricates a token and never turns a serve into an abstention.
Full P-4 / determinism / zero-allocation / no_std / witness contracts preserved;
gate ladder green on every increment.

### Follow-ups (implementation order, children of #822)

1. **#886** — bounded minimal-pairs spot-check of the deployed lane on the real
   bundle (cheap validation; no blocker).
2. **#887** — governance review of the 20‰ floor for a re-ranking lane (the
   pivotal gate; follows #886).
3. **#888** — widen candidate support beyond top-8 (blocked by #887 + #886).

Ordered roadmap recorded on the tracker: #822 (comment above). Closing as
**completed** (REVISE recorded; lane dormant).



---
_Comment/review ids in this file: 5361107553, 5364242412_

