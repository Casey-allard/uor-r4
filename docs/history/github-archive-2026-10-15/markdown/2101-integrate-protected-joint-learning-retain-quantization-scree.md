# #2101 — Integrate protected joint learning; retain quantization-screen negative

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T22:22:06Z
- **Updated:** 2026-10-09T23:20:19Z
- **Closed:** 2026-10-09T23:11:45Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2101
- **Merged:** 2026-10-09T23:11:45Z
- **Merge commit:** 924fedd1fe12ea257e254cd4097dce2b4dfa66ad
- **Base / Head:** main / UOR-Foundation:codex/protected-joint-construction-20261009
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What changed and why

Task-only Prefix proposals lowered conditional CE but failed original winner protections. This adds opt-in protected joint Prefix/Generate learning: 380 unweighted pooled winner/rival Jacobians constrain the 31-graph task/reference direction before a fixed four-radius quantized transaction. It retains all native gates and adds no serving exceptions.

## Measured result

At the original coupled initializer with frozen U, 411 backwards / 822 training-graph forwards yield a continuously admissible direction. Quantized radii 1/2/4/7 violate 104/170/98/93 surrogate constraints respectively. No native proposal is scored or committed. The independently reloaded 391-state artifact remains at 5/15 conditional winners, 17/17 references, 380/380 original guards and combined CE 5.111520730202647. Candidate REJECTED; actual-nine/full512/fresh/multi-turn NOT_RUN_CONSTRUCTION_NEGATIVE. Accepted Source48/Generate64 development replies remain 8/512.

This is a bounded quantized-displacement screen result, not measured native winner loss or global infeasibility. Saved arithmetic audit PASS; all 826 compared gradient/master/mass files match the failed-reader attempt and corrected fresh retry. The record retains every failed attempt and cost. The measured artifact stays bound to source `2c31a22e6fbef3bd37dade8cbb7daae79f13876a`; delivery source only corrects stale receipt descriptions/count metadata, with no numerical rerun or weakening of policy equality.

## Validation and preservation

31 focused CUDA-feature tests passed at the measured source. Final-head tests and exact-head review receipts are posted in comments before queueing; the compatibility acknowledgements do not execute tests. Saved audit is not independent native scoring or complete proposal reconstruction. Full source history, runtimes, configs, sealed reports, audits and failed attempts are preserved under `icloud:UOR-R4/results/codex/codex-protected-joint-20261009.tar`; verified digest is posted before closeout.

## Checklist

- [x] Checks run at this exact head (commands and outcomes posted on this PR)
- [x] Dated research record with exact artifacts/data/config, numbers, decision and limits
- [x] Newest-first current-state entry ending in **Next:**
- [x] Stale documentation search and affected README/crate README/geometry updates
- [x] STATUS.md / ROADMAP.md / #2028 headline unchanged (accepted 8/512)
- [x] Geometry 29/29, 15 regenerated figures and claim-wording PASS
- [x] Independent exact-head source review; saved outcome mathematical review
- [x] After merge: fresh-main verification, branch/worktree deletion, iCloud preservation and hygiene

References #2030


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 6090822728 — 2026-10-09T23:10:51Z — Casey-allard

Exact-head validation and independent review at **956367f552e11d475d147f98de482d2c02214631**:

- Rust/Cargo source inventory verified on the approved CUDA pod before compilation. `cargo test --release --locked -p uor-r4-training --features cuda --example geometric-frozen-map-fit coupled_episode_learning -- --test-threads=2`: **31 passed, 0 failed, 0 ignored**, 111 filtered; compile/test invocation143.328480s, test execution0.03s. Per-lab target, two workers. Receipt SHA256 `0b9476f8dcf375eceeaa792d1bc6af2318c9b45a959c4b7e7ec12b89a8e836a9`.
- Changed Rust formatting and diff check PASS; geometry verifier **29/29 PASS**;15 figures regenerate with no tree changes; claim-wording PASS. Stale-doc search resolved current claim locations, preserved dated history.
- Independent source/adversarial review PASS at exact956367; preceding mathematical saved-result review PASS. Merge resolution retains all peer current-state entries. Reviewers did not run or independently replicate the model.
- Measured model source remains2c31a22e6fbef3bd37dade8cbb7daae79f13876a. Source delta changes receipt descriptions/count metadata and a fixture, not numerical behavior; immutable artifact and strict policy equality preserved. Full391-state reload completed; no native proposal scored, no update committed,5/15 unchanged,17/380 retained, actual-nine/full512NOT_RUN_CONSTRUCTION_NEGATIVE. No rerun was needed for descriptor correction.
- Saved arithmetic audit PASS at its stated scope,826/826 retry files byte-identical, complete downloaded file-set/BLAKE3 verification PASS. iCloud digest/pod closeout receipt follows before final closeout.

The five GitHub compatibility statuses are acknowledgements, not these tests.


### comment 6090858890 — 2026-10-09T23:14:21Z — Casey-allard

Preservation and merge receipt: protected merge **924fedd1fe12ea257e254cd4097dce2b4dfa66ad** verified on fresh origin/main; all20 changed file blobs equal reviewed head956367f552e11d475d147f98de482d2c02214631. M2 body/result updated. Remote delivery branch deleted; Casey checkout clean main.

Complete evidence/source history stored at `icloud:UOR-R4/results/codex/codex-protected-joint-20261009.tar`: **482,161,152 bytes**, MD5 **467c102adc510a6352951596b969f1be**. cloud-store upload/download check PASS, then fresh index plus independent full-stream byte count/MD5 PASS. Includes failed attempts, measured runtime/config/source inventories, sealed model+observations, raw gradients/masses, audits, source bundles, final-head tests and expert reviews.

Podqa0kzpevw5jaq0 released and **deleted**23:13:14UTC; controller lifetime3423s at$1.19/h (estimated$1.13). No peer pod touched. Local owned worktree removal and prescribed hygiene now closing; cumulative elapsed charge and final cleanup receipt follow.


### comment 6090921741 — 2026-10-09T23:20:19Z — Casey-allard

Closeout complete. Protected PR#2101 is merged and fresh-main blob verification passed. Both GitHub and local delivery branches are deleted; the managed worktree is removed; Casey checkout is clean main. No local or GitHub `codex/*` branches remain at this check. Evidence is in the verified482,161,152-byte iCloud archive (MD5467c102adc510a6352951596b969f1be). Podqa0kzpevw5jaq0 is deleted.

`uor-hygiene --apply` completed with exit0. Its attempted removal of Claude's clean worktree was deliberately blocked with a temporary lock to preserve peer work; all temporary locks were restored afterward. No peer worktree/pod was removed. Additional hygiene reclamation0GB; laptop currently50GiB free after this deliverable's workspace removal. `uv cache prune` was skipped because uv was busy.

Complete preparation/implementation/review/build/retry/model/preservation/merge/cleanup elapsed about76.8minutes, charged to the cumulative ledger without reset. CUDA model observations total873.561seconds across failed launch, failed-reader attempt and completed retry;822 backwards/1644 training-graph forwards cumulative across the latter two,411/822 in the completed result. No actual-nine or full512 qualification was run. Five production/final-head compile invocations plus focused-test invocations and their failures are retained separately. Controller-recorded pod lifetime3423s at$1.19/h, estimated$1.13. Persistent programme goal remains active; next is the declared saved-displacement attribution after this completed merge/cleanup, without repeating the model experiment.



---
_Comment/review ids in this file: 6090822728, 6090858890, 6090921741_

