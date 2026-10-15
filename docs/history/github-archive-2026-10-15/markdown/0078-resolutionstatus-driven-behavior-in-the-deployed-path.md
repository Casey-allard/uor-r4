# #78 — ResolutionStatus-driven behavior in the deployed path

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-23T00:05:38Z
- **Updated:** 2026-07-23T05:49:35Z
- **Closed:** 2026-07-23T05:49:35Z
- **Labels:** graph-compiler
- **Milestone:** Phase 5: Allocation-free graph runtime
- **Assignees:** auser
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/78
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Depends on: #64 (statuses exist in the scorer), residualized EXCT. The scorer emits ExactContext/Graph/Novel statuses but the deployed adapter (src/r4g1.rs) has no behavior attached. Wire the D4 manifest policy: ExactContext → serve; Graph → serve; Novel → widen candidate set once, then abstain with explicit status (never silently guess); Contradictory → abstain. Acceptance: per-status behavior table in the manifest; server responses surface status; adversarial/OOD probe suite shows abstention instead of pathological output (the current reject guard becomes the last line, not the only one).


---

## Definition of Done
- The D4 manifest policy is wired into the deployed adapter: per-status behavior for ExactContext / Graph / Novel / Contradictory is declared and enforced.
- Server responses surface the status; adversarial/OOD inputs abstain rather than emit pathological text.

## Acceptance criteria
- [ ] Manifest carries the per-status behavior table (serve / widen-once / consult EXCT / abstain).
- [ ] API responses include the ResolutionStatus field.
- [ ] A probe suite (OOD + adversarial) shows widen-then-abstain behavior with the status visible; the pathological-output reject guard fires only as the last line.
- [ ] Zero allocation and integer-only contract preserved in the new path (census test updated).

## Comments (1)

### comment 5054934686 — 2026-07-23T05:49:35Z — auser

Complete — merged via the merge queue as PR #93 (mergedAt 2026-07-23T05:46:46Z).

DoD evidence: (1) Policy table declared as data in src/r4g1.rs: ExactContext → Serve, Graph → Serve, Novel → WidenOnce (retry once at 2xTOP_M) then Abstain, Contradictory → Abstain (arm implemented and unit-asserted); optional config.status_policy override in score_report.json with lenient fallback. (2) API responses include status/widened/abstained: POST /api/r4g1/predict returns {abstained:false,prediction:{status,token,widened}} on serve and HTTP 200 {abstained:true,status:'novel',widened:true} on abstain (repeat: widened:false via widen-once memory); chat abstain is HTTP 200 with generation_mode r4g1-abstained and an explicit r4g1 status block — never a silent fall-through; the pathological guard stays the last line for served output. (3) Probe suite 11/11: OOD → widen → abstain (counters widen_attempts=1, abstains=1), covered → serve (ExactContext/Graph), identical Novel probe → no second widen (widen_skipped_seen=9), 16 repeated probes deterministic, generate mid-abstain stops with status, step↔reference parity at both widths, out-of-vocab typed error (no panic). (4) Contract preserved: the new step layer (score_step + StepState) is integer-only (source-scanned block) and allocation-free — census: 0 allocations/0 bytes across warm-up, predict(serve), predict(abstain), generate(16), and 8 repeated OOD probes; predict_window delegate 0/0. (5) Verified by maintainer agent: 49 suites, clippy -D warnings, fmt clean, κ-reproduction green; PR checks all green; merged through the queue. Live endpoint examples: served {'status':'graph','token':10}, abstained {'status':'novel','widened':true}, real smollm2 bundle serves 'exact_context' per reference.


---
_Comment/review ids in this file: 5054934686_

