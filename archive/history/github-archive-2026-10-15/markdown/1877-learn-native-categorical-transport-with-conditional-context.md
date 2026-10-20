# #1877 — Learn native categorical transport with conditional Context action updates

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T05:27:27Z
- **Updated:** 2026-10-08T06:04:54Z
- **Closed:** 2026-10-08T06:00:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1877
- **Merged:** 2026-10-08T06:00:38Z
- **Merge commit:** 6b19c5f5eff197dcb113729fb67cb976c49c2597
- **Base / Head:** main / UOR-Foundation:codex/astra-reference-replay-20261008
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The native factual read-to-emission map previously had Context credit but no trainable categorical action carrier. This adds an opt-in carrier initialized to the exact parent map, full-120 shared-key action utility, native one-marker/zero-bias export, current-map/master identity checks, and a deterministic conditional Context-plus-action proposal with rollback.

The declared 588-term development experiment rejected its only paired proposal: combined native CE was **9.083995 parent → 12.295309 Context control → 12.223459 pair**. The action partially compensated for the Context move, restoring one frozen successful position with no new losses, but remained worse than the parent. Exact restoration retained **8/512 complete replies, all original eight, and all 512 parent sequences unchanged**. This delivers the learning mechanism and bounded negative evidence, with no model promotion or useful-chat improvement claim. Unselected full own-prefix rollouts were not run.

Validation at executed source `2d5006d6bf229022c00d3f07d3f8949efc53ffb6`: formatting, 15 bridge tests including explicitly executed CPU/CUDA full-120 action/query/source adjoints, 33 fitter tests, and CUDA release build pass. Initial/final actual graph/native parity covers all588 terms. Independent saved-evidence audits verify proposal ranking, exact master/native deltas, objective arithmetic, restoration, output membership and prior-candidate comparisons; separate Rust verification passes complete report file sets and BLAKE3 seals. The retained first CUDA visibility setup failure is not a model result. Final delivery validation and preservation receipts are posted below.

The current-state record defines the next distinct question: action learning directly at the intact accepted parent, under a newly declared single-proposal rule. This is causal isolation, not an expanded search, joint-learning qualification or permanent Context freeze.

References #820.


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 6053071415 — 2026-10-08T05:32:22Z — Casey-allard

Exact source 2d5006d6bf229022c00d3f07d3f8949efc53ffb6: changed formatting PASS; 15 focused bridge tests PASS with --include-ignored, including full120 categorical action/query/source CUDA-versus-CPU analytic parity; 33 fitter tests PASS; release CUDA build PASS. No failed or ignored tests at this head. Commands executed on leased4090 pod tuq3ezm6yq6rzj,2CPUthreads,CARGO_TARGET_DIR=/root/codex/target,CUDA12.8/SM89. Timings: format0.16s,bridge compile+tests88.67s,fitter compile+tests65.79s,releasebuild10.86s. Independent exact-head source review PASS; owner-allowed self/root review covers the narrow receipt changes authored by reviewer.

First cec541 check failed because runner masked CUDA_VISIBLE_DEVICES; execution failure retained, repaired by uor-pod run onGPU0 and fullmodule testfilter. Earlier82e8 checks passed13bridge/33fitter but excluded one explicitly ignored CUDA test; final15/33 supersedes that coverage. The intervening source commits only repaired a legal-range fixture and added direct CUDA action-adjoint coverage, without production changes.

Claim heartbeat/model admission: authenticated original Source48/Generate64 parent, zeroU, fixed512 data and588 reached/success terms are restored; frozen two-block rule is issue820/comment6052811970. Current executable is being frozen with exact source/configuration/review receipts before the sole declared model run. Model behavior NOT_RUN at this posting. CI transport acknowledgements are separate from these actual checks.


### comment 6053432613 — 2026-10-08T05:59:45Z — Casey-allard

Exact-head delivery evidence for `478f10314304e570c706f5cee1f7331d89054c55` (manual procedural fallback; CI acknowledgement statuses are not these tests):

- Independent exact-head source/claim review PASS. The final delta after executed `2d5006d6bf229022c00d3f07d3f8949efc53ffb6` is 28 current-state documentation lines; every Rust blob is unchanged.
- Actual final-head formatting PASS (0.14s); CUDA-feature bridge tests **15 passed /0 failed /0 ignored**, including full-120 action/query/source CPU/CUDA adjoints (93.59s compile+test); fitter tests **33 passed /0 failed /0 ignored** (68.29s); CUDA release build PASS (11.00s). Private build directory, 2 CPU threads, leased RTX4090, CUDA12.8/SM89. Producer log `/workspace/uor-r4/jobs/codex/20261008T055505Z-astra-categorical-20261008-gpu0.log` ends exit0. Claim-wording gate and git diff whitespace check pass locally.
- One initial final-validation dispatch had an incorrect commit suffix; Git refused checkout before any build/test/model. The failed log is retained separately. Correct full HEAD above was then executed; no model rerun.
- Sealed model run was executed at2d5006, not the documentation-only final head. Both588-position graph/native checks pass. Only paired candidate rejected; exact parent restored, 8/512 complete, original8 retained and all512 output sequences unchanged. Independent math, endpoint and prior-candidate saved-evidence readers pass; separate Rust BLAKE3 checks pass run/donor/plan. No backward-gradient replication or held-out/general-chat/energy claim.
- Full negative result, control/candidate checkpoints, output/telemetry/config, audit readers/results and outcome reviews preserved in private HF `caseyallard/uor-r4-store`, revision `8b63e63645204dba7ce2ba3a41c65bf6de2540bb`, prefix `codex/astra-categorical-action/categorical-0001-2d5006d6bf`. Archive97258539B, SHA256 `898cbbc3b2df8e29769bb166ff9f97f6d0f5ef30158e22641584e8d15b864bd5`; fresh archive and manifest downloads verified.

No blocking review findings remain. Ready for normal protected merge into main. Goal#820 remains open; useful conversation is not achieved.


### comment 6053505069 — 2026-10-08T06:04:54Z — Casey-allard

Delivered #1877 through the protected merge queue: merge `6b19c5f5eff197dcb113729fb67cb976c49c2597` is on fetched main; its tree `e96b3f55c05b109199197e3037b55c52210e2d67` exactly equals reviewed/tested head `478f10314304e570c706f5cee1f7331d89054c55`. The owner local main checkout was fast-forwarded to that merge; all existing untracked token-optimizer files were preserved unchanged.

Final validation/review/resource snapshot/merge receipt and executed-to-final documentation patch are preserved in private HF `caseyallard/uor-r4-store`, revision `99f8860a2bfb1ce2b04d7487b025d4e8ecea3430`, prefix `codex/astra-categorical-action/final-delivery-478f103143`. All17 files were freshly downloaded and verified; manifestSHA256 `aab819212eab94ca1952e0a303cf8615e47b12b9b32f5dc5240d10ddfd41dae0`. The full result/control/rejected candidate and independent audits remain at revision `8b63e63645204dba7ce2ba3a41c65bf6de2540bb`.

Result: categorical action learning implemented and exercised; one Context-plus-action proposal rejected, exact parent retained at8/512 complete and alloriginal8, with no sequence changes or useful-model promotion. The completed work card is released. Next recorded causal dependency is original-parent-local action learning under a new single-proposal rule, explicitly NOT_RUN and not permanent Context freezing. #820 and the persistent native-chat goal remain open.



---
_Comment/review ids in this file: 6053071415, 6053432613, 6053505069_

