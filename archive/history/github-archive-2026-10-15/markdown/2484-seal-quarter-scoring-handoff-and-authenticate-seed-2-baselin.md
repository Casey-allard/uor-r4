# #2484 — Seal quarter-scoring handoff and authenticate seed-2 baseline before apply

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T18:45:39Z
- **Updated:** 2026-10-14T18:59:22Z
- **Closed:** 2026-10-14T18:57:49Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2484
- **Merged:** 2026-10-14T18:57:49Z
- **Merge commit:** 5ce408c6d1a46237497dbc82e514dda314bdca97
- **Base / Head:** main / UOR-Foundation:codex/completion-scoring-seed2-20261014
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The quarter-scoring handoff was missing its exact pod pull/verify steps, and seed-2 baseline preservation did not authenticate checkpoint-zero u and the complete count reference before apply. This delivers the Q2–Q4 runbook and a seed-2-only metadata admission guard.

The runbook freshly pulled sealed Q1 artifacts and reproduced Kimi's entire audit JSON byte-for-byte. The audited scorer changes only its local checkpoint-path resolver, retaining the original FilePin hash and all scoring logic. Seed-2 authentication binds its original zero checkpoint, plan/admission/lineage/model, full frozen reference and row-derived u; it seals an immutable receipt/pin and checks them on resume before issuing update admission.

Validation: 12 existing scorer tests; complete Q1 transfer/hash/scoring rehearsal; wrong checkpoint override refused before output; 5 focused authentication metadata controls including the pinned archived report; full default-feature metadata suite 29 passed, 1 external-artifact check explicitly ignored there and passed separately; D23/D25 ratchets 1641/93, formatting, shell syntax and claim wording pass. Q1 receipt audit: 0.34s wall / 85,114,880B RSS. No model/GPU compute.

Source delivery only: CUDA-gated seed-2 integration is not qualified. The operational driver identity changes and requires exact-source review, a new qualified build/seed-2 packet and Casey's personal release before use. Existing numerical source, seed1 executable and released plans are untouched; seed2/M2 remain HELD. No bars are decided before cursor16404, and no architecture successor is adopted.

Canonical records: `docs/labs/completion-quarter-runbook-2026-10-14/README.md` and `docs/labs/seed2-baseline-authentication-2026-10-14/README.md`; current-state and prior scoring handoff updated. Line: evaluation integrity; headline unchanged; no new model result/count.

References #2029, #2028.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6175778761 — 2026-10-14T18:47:15Z — Casey-allard

Independent Codex subagent source review (`delivery_review`), exact head `b4cd2fad7a4a3107f39dc74777779638318a05bd`: **APPROVE source delivery**. No actionable correctness findings. Reviewer inspected final diff, immutable baseline/pin recovery, digest domains, gate ordering before apply, runbook/read-only artifact flow, incoming Antigravity document preservation, and final test log. This is a distinct non-author source pass, not Claude's binding/FIT review and not CUDA qualification.

Actual checks: 30/30 default-feature metadata controls including the expressly supplied authenticated pilot artifact; 12/12 scorer controls; fresh Q1 pull/verify/score output byte-identical to Kimi; wrong local checkpoint override rejected before output; D23/D25 1641/93; package/scorer fmt, shell syntax, claim wording, diff check pass. Numerical source and existing release packet unchanged. Test artifacts and runbook identify synthetic versus stored-GPU receipt evidence.

Future seed2 execution still requires exact-source adoption review, a new qualified CUDA build and plan/lineage/executable pins plus personal owner release. No existing seed1 binary/plan is replaced; no model compute was performed. Approval is for source delivery only.


### comment 6175992506 — 2026-10-14T18:58:36Z — Casey-allard

**Kimi PM exact-head review — `b4cd2fad7`: APPROVE.**

Reviewed in two parts. (1) Small hunks, read directly: `completion.rs` adds exactly the disclosed `#[path]` include + one `require_before_apply(&driver, &release, &a.plan, &p)?` call immediately before `issue_corpus_update_admission`; `training.rs` adds the one `include_bytes!` line to `driver_source_sha256()` (the disclosed driver-identity change); the scorer change formalizes my Q1 checkpoint-path override — byte-identical to the scratch change I verified Q1 with, scoring logic untouched. The two head-advancing commits (`326e64572`, `b4cd2fad7`) carry zero `crates/` delta. (2) The 556-line `completion_baseline.rs`, deep-reviewed by a fresh-eyes reviewer against the design README: **CLEAN** — no unwrap/expect/panic, immutability via symlink-aware require-absent + create_new under the run lock, u computed solely from seed-2's own rows (105 appears only in an ignored pilot-recount test; guard requires `pilot_parent.is_none()` + seed-2 plan/model binding, so no seed-1 path can substitute it), full-length digest comparisons only, no float/softmax/matmul, correct cursor semantics (`completed == 0`, 224×64+15 = 14,287). Seed-1 resumes early-return `Ok` at the seed≠2 guard — documented scoping, no silent wrong-auth, and a future rebuild fails closed at plan validation since both seed plans pin the pre-change driver hash `cb090e85…`.

Notes recorded, none blocking: driver-identity change is fail-closed until a future seed-2 re-pin (by design); reference digest `b92da0bf…` is anchored by the recorded archived-report run (5/5, u=105 reproduced); test-only `create_dir` nit under untracked `local/`.

**Seed-1 divergence boundary (PM):** on merge, main's discrete source diverges from the frozen run source `33564b75e` for the first time — disclosed and safe: seed-1's pinned pod binary (`d4fbe26c…`), admission, plan and packets are untouched, the archived scorer and my Q1-verified copy remain pinned by hash, and Q2–Q4 verification uses the same scoring logic. The 10-file frozen invariant now reads: main = post-#2484 state; frozen RUN source = `33564b75e` on-pod. Seed-2 and M2 remain HELD; this merge admits nothing.


---
_Comment/review ids in this file: 6175778761, 6175992506_

