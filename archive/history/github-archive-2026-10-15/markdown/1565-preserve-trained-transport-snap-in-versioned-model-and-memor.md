# #1565 — Preserve trained transport snap in versioned model-and-memory inference checkpoints

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-10-01T06:23:27Z
- **Updated:** 2026-10-01T06:43:29Z
- **Closed:** 2026-10-01T06:43:28Z
- **Labels:** roadmap:historical
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1565
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Saved geometric-model checkpoint continuation

Parent #973 / #962 / #964; follows the owner-adopted #1563 / PR #1564 roadmap. This is a complementary persistence task; Claude and DeepSeek retain classifier/data/emitter work on #1552.

**Observed blocker.** `StackModel::save/load` now preserves trained icosian transport, but `stack_checkpoint::save_checkpoint` still rejects every transport-snap model and its exact file list excludes `transport.json`. The inference checkpoint therefore cannot preserve snapped R1/S4 plus exact stored memory. The refusal is safe; simply deleting it would create an invalid artifact.

**Deliverable.** A versioned inference-checkpoint format that saves and independently reloads the actual declared transport snap with its model and optional exact store. Preserve old schema-1 unsnapped checkpoints and refusal of unpersisted served/QAT representations. This does not claim a learned compiler, generated-history session or D11 export is complete.

**Causal change and ownership.** Codex owns `crates/uor-r4-training/src/stack_checkpoint.rs`, its focused tests and a concise integration/result note only. Reuse existing `StackModel` transport serialization. Do not edit `geometric_stack.rs`, `relation_compiler.rs`, `m-world.rs`, teacher data or another lab's worktree without a fresh coordination record. No new training.

**Fixed conditions and outcomes.** Existing model/transport values remain unchanged; no recalibration or unsnapping. A new schema binds the transport file digest and presence; old free-transport format retains its semantics. Success means exact loaded same-backend logits/greedy continuation and exact-store reads with the snap still active. Missing/extra/mismatched/tampered transport or unsupported served mode must fail explicitly. If a supported actual parent cannot round-trip, retain the refusal and diagnose the contract; never silently load unsnapped.

**Checks.** Focused old-format and snapped model+store round trips; greedy/logit behavior on changed causal inputs; malformed/presence/digest cases after resealing; existing refusal cases; relevant Rust compile/format. Use an actual saved lineage for a bounded reload witness if the available CLI/source and live resource admission support it; otherwise report that witness NOT_RUN rather than substitute a toy result. No full model-quality campaign or repeat training is justified.

**Resource projection.** Initial source/review/delivery allowance 90 minutes; under 10 MiB new source/receipts. Prefer free GitHub public-repo compile/focused tests at the exact PR head, using the existing cache, with a 45-minute execution ceiling rather than a learning-dose claim. No local Cargo/model worker is launched by this card; any actual-artifact witness gets an updated local resource projection before execution. Live derived ledger view read at this pickup: cumulative790,330,180ms / limit1,130,000,000ms; this does not reset charges or substitute for immutable accounting. Internal free42,532,992KiB, workspace438,242,472KiB at pickup; refresh before material allocation. Preserve the128MiB stop margin and current host/worker limits. No paid compute or cleanup.

Partial code delivery references this issue. Completion requires the full stated checkpoint behavior, protected merge and verified delivered patch; it does not close the parent capabilities.


## Comments (2)

### comment 5926062151 — 2026-10-01T06:34:21Z — Casey-allard

Checkpoint boundary claim renewed: source is frozen at `2d5cfeacd38369d5e61e5f91aa1944fd2959c202`, PR #1566. Two independent exact-head reviewers are active. Free GitHub runner [36825407561](https://github.com/UOR-Foundation/uor-r4/actions/runs/36825407561) explicitly checks out that head for compilation and focused tests; results are pending. No local Cargo/model process or slot was started. The same two-file ownership remains, and no classifier/teacher source is touched.

Static review identified an additional reload risk at this same boundary: an independently resealed configuration declaring a served/QAT representation could otherwise be loaded into float variables because the underlying training loader intentionally does not restore the served representation. The checkpoint now refuses that case; tests exercise both snapped and unsnapped records. A trained R1/S4 witness and fresh-process dialogue reload remain NOT_RUN.


### comment 5926170023 — 2026-10-01T06:43:27Z — Casey-allard

**Delivered and verified:** protected PR #1566 merged at `bbdecac23e7e9bc44c8e2a410a3c679ce2435c41` on2026-10-01T06:42:07Z. Refreshed `origin/main` contains that merge, and the **entire merged tree equals** reviewed/tested head `2d5cfeacd38369d5e61e5f91aa1944fd2959c202`. No protection bypass or direct main push.

Actual [CI36825407561](https://github.com/UOR-Foundation/uor-r4/actions/runs/36825407561) passed formatting/claim/JSON/whitespace checks, library+m-world compilation and **10 checkpoint tests,0 failures,0 ignored**. Two nonauthor reviewers approved the exact head. The advisory publisher's Not Found error is preserved separately in the validation comment; it was not claimed PASS.

The completed scope is the versioned checkpoint API: snapped model plus optional exact store, unchanged schema1 output for unsnapped models, transport/root/digest validation and explicit QAT refusal on both save and load. Tests actually exercised newly loaded CPU models, exact logits, short greedy continuation and store updates. They use an initialized fixture. **Trained R1/S4 witness and fresh-process learned dialogue are NOT_RUN**; the issue's optional artifact witness was unavailable through the existing consumer and no stand-in model-quality claim is made. This API has no dialogue CLI consumer and does not retain compiler/transcript/cache state.

This satisfies #1565's scoped persistence deliverable. Parent #962/#964/#973 and the overall model objective remain open. The next integrated #962 work card consumes the real saved compiler with exact store, trained emitter and generated-history/fresh-process continuation. Claude's compiler/data work and jobs remain theirs.

Accounting: local Cargo/model workers0. Free GitHub compile1m04s; test compilation4m58s; tests0.16s. Complete orchestration/source/review/delivery elapsed is charged cumulatively through the internal immutable resource ledger, not reset from the stale external model-time view. Receipts/reviews/logs are retained in the task's `uor-r4-plan-adoption/checkpoint-snap` archive; no unique material was removed.



---
_Comment/review ids in this file: 5926062151, 5926170023_

