# #1635 — Implement persistent integer geometric attention and Stack replay bridge

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T18:43:21Z
- **Updated:** 2026-10-09T13:17:18Z
- **Closed:** 2026-10-02T23:44:01Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1635
- **Merged:** 2026-10-02T23:44:01Z
- **Merge commit:** 1dc78d1bfc3ade45868d7f35fcee30ca5e264d56
- **Base / Head:** main / UOR-Foundation:codex/geometric-attention-session-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Persistent integer geometric attention is implemented with cached occurrence state, head-local geometric compatibility/NoRead and value composition, plus an unchanged-parent replay bridge. This PR does not qualify the float Stack tail or claim language improvement.

Native M1 run37067792966 at Rust source4a0b0224 passes3 session tests,5 context tests and1 allocation census. The census measures256 successful pushes and2 resets with0allocations/bytes/reallocations/deallocations, excluding admission/warm-up/errors/reporting. An actual compiled19-symbol integer scan initially found3 MADD from48/104byte context descriptor indexing. Private64/128byte padding removes all3 in the rebuilt binary:0multiply/divide/FP arithmetic/conversion/FMOV across those emitted integer ranges. Additional64bit descriptor storage16V+24HL, max65,728B before allocator overhead; coefficients/artifacts/algebra unchanged. Independent exact-source review finds no blocker. External libc/panic callees remain unscanned.

Hosted Linux and Mac driver builds pass. Repaired native binary SHA256a5abfdee0a3e92d7409df0fe25955011d94138d1fd01bad16fa59d55968e85de; all145 compiled Rust-source hashes verified. Final documentation-only head preserves executed Rust bytes. Claim-wording and diff checks pass. Source-bound execution receipt and retained prior failure are linked in docs.

**Both loaded-parent lineages now complete**, after DeepSeek exits and the declared2thread/3GiB slot is admitted. All512 retained rows preserve saved answers and all-position whole-window/session logit bits; retained correct totals93/110 and124/119 (446total),0optimizer updates. B2/short-prefix/reset/capacity/dependency checks pass. Workers take8.78s and9.46s, peakRSS843,300,864 /735,199,232B. Independent saved-output review passes512 answer vectors and1,883,664 causal score pairs, reconstructs57,952 head reductions and matches1,024 parent query heads;0mismatches. Full-position equality remains pinned executed-driver assertions, distinct from saved-output joins. This preserves the prior conversion outcome, not a quality improvement. No local Cargo/private artifact upload; GroundedSession untouched.

References #1512

Native evidence: https://github.com/UOR-Foundation/uor-r4/actions/runs/37067792966


## Timeline (5 entries; chronological: conversation comments, reviews, review comments)

### comment 5959559356 — 2026-10-02T19:08:47Z — Casey-allard

Independent focused Stack seam/admission review completed at exact head5e9dfb52c9e16266260710a8c71ec7bd86a48862. The reviewer inspected the six-line caller repair and found no remaining material source blocker within this scope. Host restrictions, old-held/new-context timing, batch reset, residual site, donor-map bypass and checked i64 head addition before identical Q16 float conversion agree with the native composition reference. Source SHA bindings independently checked after receipt.

Review SHA256:0566414b23211ea7c3f20bcb8ef540758f02d55329b465077a2c5ac9d515bb2c. Full note retained in the project knowledge store under `uor-r4:geometric-attention-session:2026-10-02:stack-review-5e9dfb52`.

This is source-only review. The earlier missing-field blocker is retained as a corrected implementation defect. Compilation, loaded-parent parity, allocation and opcode checks remain NOT_RUN, and validate_stack is not full live float-tail authentication. Four actual peer trainers are live in the latest observation, including the new Metal trace; the shared registry already reserves8 threads before the unregistered DeepSeek worker. No own compute or foreign-process changes. Keep draft pending scoped execution.


### comment 5961414398 — 2026-10-02T21:01:32Z — Casey-allard

Owner-directed hosted-runner route is working: [run37063718908](https://github.com/UOR-Foundation/uor-r4/actions/runs/37063718908). Claude's throwaway CI-branch convention was reused: exact source head5e9dfb52 plus one workflow-only commit edb2f636. Source identity guard passed in both independent hosted Ubuntu jobs. No product source/workflow change and no local Cargo/model run.

Focused release integer session job completed: all3 named attention_session_ cases passed,0 failed/ignored; release compilation31.54s, execution0.04s, Rust1.97.1/x86_64-unknown-linux-gnu. Downloaded raw logs/compiler/source bindings; all12 changed Rust file hashes independently match tested head. Successful dispatch alone was not counted as PASS.

Parallel release replay-driver build is still IN_PROGRESS. Saved-parent model comparison, compiled allocation/opcodes on M1 and whole-model serving remain NOT_RUN. No private model data uploaded. The hosted binary will be Linux; it is not interchangeable with a local macOS executable.

The clean, pushed temporary runner checkout is removed; workflow/Git branch retained. Measured free-space increase1,317,023,744bytes; product worktree, cache marker and research untouched. Full temporary checkout size had exceeded the initial256MiB projection (1,275,204KiB); this projection error and the explicit1.5GiB temporary-checkout correction are retained, rather than silently claiming it fit.


### comment 5961507369 — 2026-10-02T21:08:12Z — Casey-allard

Hosted evidence update: first integer job passes3/3 release cases on Linux, with exact source binding verified. First driver job compiles the training library but fails E0508 at the packet-record comparison. Failure log/artifact retained. e19bbd33 changes only that access to a borrow; the integer implementation remains byte-identical.

Driver-only exact-head [retry37064609925](https://github.com/UOR-Foundation/uor-r4/actions/runs/37064609925) is live, with35min ceiling within the remaining declared build allowance. Current PR head3ab7d257 adds only result/current-state/plan documentation after the executed source head; no additional Rust change. No local Cargo/model or private artifact upload. Loaded model parity and M1 allocation/opcode checks remain NOT_RUN.


### comment 5961642937 — 2026-10-02T21:19:00Z — Casey-allard

Hosted compilation route confirmed by actual execution. Linux3/3 release integer tests pass; repaired release driver build passes6m03s, ELF SHA4f355d320a784db17336128f14d835d5aa0c683f195b708ce41c53d2ab828b1d verified. Source/current-state/plan and evidence receipt are pushed at59442042; Rust unchanged after repaired build head e19bbd33.

Native [M1/arm64 run37065787933](https://github.com/UOR-Foundation/uor-r4/actions/runs/37065787933) also passes all3 intended release integer tests,0 ignored,0.02s execution, Rust1.97.1/aarch64-apple-darwin. Downloaded source bindings match all12 owned changed Rust files. Its native driver build remains live. Standard hosted public runners only; no local Cargo/model or private artifact upload.

This removes the compile-slot dependency and produces platform-bound fixture evidence. Loaded-parent model comparison and successful-push allocation/opcode audit remain NOT_RUN; no complete serving/language/energy claim. No automatic fit, abandoned geometry, GroundedSession edit or foreign-process change.


### comment 5963218804 — 2026-10-02T23:44:23Z — Casey-allard

Exact-head delivery review APPROVE at395d2c6f75bd2ac69720cb53332a1c437dfb88b2 against sourcebase8f732a9feb8ac9501231e07bfc7c48d3697c616c. Root self-review underD19 plus disclosed independent specialist source/result audits. Rust unchanged since executed4a0b0224; hostedM1run37067792966 passes3session/5context/1allocation checks+release driver;19integer symbols0mul/div/FP/FMOV. 145compiledsource hashes+binarySHAverified. Both parents pass sealed512-row replay,0updates. Independent saved-data audit512answerf32 vectors/28976context-value traces/1883664causal scorepairs/57952head reductions/1024query heads0mismatch. Independent inventory/sizes/SHA/source/binary binding pass; independentBLAKE3 unavailable, distinctfrom executedRustseal+verify. All-position equality is fail-closed executed-driver assertion, onlyanswerlogitvectorssaved. Directrustfmtcheckall14ownedRustpaths passed(skip_children=true); firstformatcommand selectedunrelatedadvancedmainpaths andfailedbeforechecking, retainedlog, correctedsourcebase selection. Claim-wording/diff-checkpass. No remaining requiredfix inboundedPR. Event/agewidth,selectedaccess,floattail/natural-language remainseparate. References1512partial. D19manualfallback applies; advisorypublishernotrequiredbyactualrules; no forgedstatus/branchbypass.



---
_Comment/review ids in this file: 5959559356, 5961414398, 5961507369, 5961642937, 5963218804_

