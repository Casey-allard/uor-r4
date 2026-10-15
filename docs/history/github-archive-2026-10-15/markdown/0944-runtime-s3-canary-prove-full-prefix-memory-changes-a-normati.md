# #944 — runtime/S3 canary: prove full-prefix memory changes a normative production decision

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-25T20:59:07Z
- **Updated:** 2026-08-25T21:26:15Z
- **Closed:** 2026-08-25T21:25:43Z
- **Labels:** enhancement
- **Milestone:** R4 Intelligence / S3 — Free-running coherence
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/944
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #824
Predecessor: #942
Canonical admitted artifact: #933

## Problem

#942 carried a deterministic fixed-size signature of the complete admitted prompt plus committed generated prefix into the normative R4G1 production loop. That establishes reachability, but not a behavioral effect. The runtime consults the session signature only when the primary context routing probe admits no node within its calibrated radius. It is therefore still unknown whether the exact production artifact contains any case where the new lane changes candidates or the served decision.

## Goal

Run one bounded, teacher-free, production-scope canary against the exact schema-2 envelope ratified by #933 and issue a truthful non-vacuity verdict before any broader free-running evaluation.

## Scope

- Load and re-verify the exact schema-2 production envelope through the normative admission boundary.
- Deterministically inspect at most 512 one-step held-out cases.
- For each case, compare the same newest 8-token scoring window under complete-prefix and suffix-only token-history signatures.
- Capture whether the context probe missed, whether the session probe admitted nodes, candidate/served-token identity, score status, and stable artifact/input identities.
- Emit a deterministic JSON result with a typed final verdict.
- Add planted positive and inert fixtures so the canary cannot pass vacuously.

## Run contract

metric to move: count of paired cases whose normative candidate set or served token changes only when earlier-prefix memory changes
reachability ceiling: at most 512 deterministic one-step cases; the lane is reachable only after a context-probe miss
instrument + verdict: the bounded canary must report at least one exact production case with a session-admitted route and changed candidates/token to return EFFECT_ESTABLISHED
exit rule: stop immediately after the first effect, or after 512 cases with none
if positive: authorize only a small production-scope slice of the frozen #841 protocol
if negative: report INERT for this artifact and route the next fix to observation/compiler signature-addressable records; do not run #841
cost estimate: seconds to low minutes, teacher-free; no compile, corpus observation, or broad generation run

## Acceptance

- The exact envelope either passes admission or the result is UNAVAILABLE, never a fixture skip reported as success.
- Complete-prefix and suffix-only cases have identical newest 8 tokens and distinct signature bytes.
- A positive verdict requires a changed normative candidate set or served token attributable to the session lane; signature-byte inequality alone is insufficient.
- An inert result is retained as a valid negative product result.
- Focused tests, formatting, and claim wording pass; exhaustive certification remains nightly/manual.

## Claim boundary

This canary may establish only that full-prefix memory changes at least one normative production decision for the exact artifact. It cannot establish coherent generation, prompt relevance, or clear the frozen #841 bar.

## Comments (2)

### comment 5416856060 — 2026-08-25T21:11:52Z — Casey-allard

## Bounded canary result — INERT

The exact #933 schema-2 envelope admitted successfully. The predeclared 512-case ceiling completed in 53.6 seconds with no teacher, compile, observation, or broad generation run.

- inspected: 512
- explicit context-row resolutions: 492
- primary context-signature probes: 20
- primary probe misses: 0
- secondary session probes attempted/admitted: 0 / 0
- candidate-list changes: 0
- served-token changes: 0
- tested-position CID: blake3:f108e9caab5edc0b7b4d312cf5ae187608d9653ea3d4e24bfa00ef68a07507b7
- ordered-observation CID: blake3:c6876af09d4c416f10f7098b18eda94cf8e6e4217e9cf28bb66a73f8f86748fe

Decision against the run contract: INERT for this exact artifact. #942 is structurally connected, but the primary route never yields to the secondary session lane on this population and no normative behavior changes. The broader #841 rerun is NOT_RUN. The next representation change must make complete-prefix information behaviorally reachable before repeating this canary.

Implementation/evidence commit: 05ad0dc9

### comment 5417023191 — 2026-08-25T21:26:14Z — Casey-allard

DoD evidence (merged)\n\n- PR #945 merged through the protected queue as cf22879a35c896ece3df886ccdaca8716d4b3ee6.\n- Exact schema-2 #933 artifact canary verdict: INERT.\n- 512 held-out positions inspected: 492 context-row hits; 20 primary context probes attempted; 20 primary probes admitted; 0 primary misses; 0 session-probe attempts/admissions; 0 candidate-list changes; 0 served-token changes.\n- Tested-position CID: blake3:f108e9caab5edc0b7b4d312cf5ae187608d9653ea3d4e24bfa00ef68a07507b7\n- Observation CID: blake3:c6876af09d4c416f10f7098b18eda94cf8e6e4217e9cf28bb66a73f8f86748fe\n- Decision: the full-prefix signature path is structurally connected but behaviorally inert on the admitted production artifact because primary routing never yields to the session probe. Per the declared contract, broader #841 is NOT_RUN.\n- Focused local gates passed: fmt, diff check, claim wording, runtime fallback test, API classifier, exact canary, touched-package checks, no-default-features runtime check, and WASM boundary check.\n- PR required gate passed in 6m31s: https://github.com/UOR-Foundation/uor-r4/actions/runs/32899735014\n- Merge-group required gate passed in 5m56s: https://github.com/UOR-Foundation/uor-r4/actions/runs/32900377691\n- Nightly/certification suites were not run; the merge workflow correctly skipped them.\n\nThe next technical action is compiler/observation work that creates signature-addressable state or an explicitly changed routing composition; another larger runtime-only measurement cannot change this result.


---
_Comment/review ids in this file: 5416856060, 5417023191_

