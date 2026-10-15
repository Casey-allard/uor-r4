# #1867 — Add native reference replay and record its failed first proposal

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T02:13:48Z
- **Updated:** 2026-10-08T02:42:37Z
- **Closed:** 2026-10-08T02:42:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1867
- **Merged:** 2026-10-08T02:42:37Z
- **Merge commit:** 60c98435dea034508ff5773c2a29532c8ac12e14
- **Base / Head:** main / UOR-Foundation:codex/astra-reference-replay-20261008
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The first joint native Source update improved its drawn batch while losing all eight accepted complete replies. This adds a frozen supervised reference term from all authenticated parent-correct teacher positions, combines its gradient with the unchanged B8 objective before one shared clip/Adam step, and keeps Context/Potential/Generate/U active. Rust preparation seals eligibility and weights without model calls; sparse replay preserves complete teacher prefixes. Actual exported native evaluation checks the original eight replies before full512 evaluation, without a teacher-retention veto.

The fixed one-proposal pilot at source `f52b5dec2ce50a6c7ceb9d907d2f2e41f3ddff95` is a retained negative:

| Endpoint | Parent | Replay |
|---|---:|---:|
| Complete own-prefix replies /512 | 8 | 0 |
| Prior-correct reference positions /398 | 398 | 46 |
| Current-batch native CE | 7.2967103431 | 6.9683028935 |
| Reference native CE | 2.5308361672 | 4.7397594763 |
| Combined native objective | 9.8275465103 | 11.7080623698 |

No promotion or longer unchanged fit is justified. The recorded successor concerns bounded native proposal learning with objective acceptance; no such successor result is claimed here. General chat, held-out transfer, geometric advantage and complete-path D11/energy remain unestablished.

The owner also clarified delivery policy: branches and PRs are temporary; completed validated work must merge into main, and opening a PR or entering the queue is not completion. AGENTS now states that requirement explicitly.

Validation: changed-file formatting,22 focused CPU tests and CUDA89 release build; actual one-proposal CUDA fit, native reload/parity and own-prefix evaluation; independent saved-row/objective audit; separate Rust complete-file seal verification. Source/math/adversarial reviews pass. Final-head checks and review are posted separately before queueing. CI status names are compatibility acknowledgments, not these tests.

Full sealed negative, checkpoints, outputs and audit are freshly verified in private HF revision `0d746d0dbcac16e549ec59a73bba26ba4c9d053b`; runtime/source/plan at `8c804432848ed49198700a49b61cff529e8c99f4`. Owned GPU was released/deleted after preservation. Accepted parent remains retained.

References #820.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6051077150 — 2026-10-08T02:41:45Z — Casey-allard

Exact-head delivery validation at `3d6354786258f1cc87630cbb2e78d2faca80df55`:

- Changed-file rustfmt: PASS (0.07s).
- Actual CUDA-feature release fitter tests: **22 passed,0 failed,0 ignored** (build+execution140.58s), GPUs masked, two admitted remote CPU threads.
- Release fitter build: PASS (21.71s), CUDA12.8 / sm89.
- Claim-wording and `git diff --check`: PASS.
- Exact-head adversarial review: PASS, no blocker. The fitter is byte-identical to executed model sourcef52b5dec2; only AGENTS/current-state differ from that executed source.

The actual one-proposal run is a **retained negative**, not a passing model:0/512 complete replies, all8 accepted replieslost,46/398 referencepositionsretained; combined native objective9.8275465103→11.7080623698. Independent auditPASS concerns authentic saved evidence, not capability. Separate Rust complete-file sealsPASS on run,donor,plan. HF archiveandpublicationmanifest freshlydownloadverified at0d746d0dbcac16e549ec59a73bba26ba4c9d053b. OwnedGPU released/deleted afterpreservation.

Merge delivers the implementation, measurednegative, prospective successorboundary andownerclarified mandatory-main-delivery rule. It does not promote the candidate or authorize a repeatdose. References#820. FiveCIstatuses are transportacknowledgments, nottests. Delivery-evidence check advisory/manualfallbackUNVERIFIED; these areactualscopedchecks andrecordedexact-headreview.



---
_Comment/review ids in this file: 6051077150_

