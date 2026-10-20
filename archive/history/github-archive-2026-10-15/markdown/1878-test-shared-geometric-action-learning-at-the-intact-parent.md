# #1878 — Test shared geometric action learning at the intact parent

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T06:16:48Z
- **Updated:** 2026-10-08T06:54:16Z
- **Closed:** 2026-10-08T06:52:17Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1878
- **Merged:** 2026-10-08T06:52:17Z
- **Merge commit:** f54050ca1f0d61bae09f0e4465e41a56b3ed789e
- **Base / Head:** main / UOR-Foundation:codex/astra-reference-replay-20261008
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The previous conditional Context/action proposal damaged the accepted parent before action learning could repair it. This change adds a default-off `categorical_action_only` path that computes one original-parent gradient and changes only the globally best shared action contrast, retaining the existing two-block behavior. Independent native export/reload accepts strict native loss descent or restores the exact parent; Context, Potential, Generate, U and all other masters stay unchanged for this attribution test.

The executed one-proposal experiment lowers combined native CE **9.08399492844048 → 9.07236859663813** by changing lane3/key30/action105→50. Actual complete replies remain **8/512**, all original8 retained, with10 changed output sequences and4 wrong-to-wrong entry changes. This is bounded optimization evidence; the accepted serving parent is preserved and no useful-model, held-out, general-chat or geometric-advantage claim follows.

Validation: 34 focused fitter tests (0 failed/ignored), changed formatting and CUDA release build pass at executed source3a7c9fdae9f4d278df5d850f3f3602272e6b5115. Initial/final graph/native parity each cover588 positions. Independent saved-evidence checks reconstruct proposal ranking, unchanged masters, native packing/objectives,1024 initial/final own-prefix rows and full512 oracle membership. Separate Rust complete-file BLAKE3 checks pass for run/donor/plan; pinned comparisons with1872/1875/1877 retain the same8 complete outputs. These checks are not independent model/autodiff replication. A derived state-reader schema error was preserved and corrected without rerunning the model.

Current state records artifact identities, preserved negatives, cumulative costs and the reviewed next dependency. Exact final-head validation and preservation receipts are posted separately before protected merge.

References #820


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 6053742969 — 2026-10-08T06:21:27Z — Casey-allard

Actual validation at source `3a7c9fdae9f4d278df5d850f3f3602272e6b5115`:

- Independent exact-head source review PASS; only the fitter and categorical proposal module changed.
- Formatting PASS (0.16s), focused CUDA-feature fitter tests PASS (220.44s including compilation), CUDA release build PASS (11.09s). The admitted runner uses 2 CPU threads, private CARGO_TARGET_DIR, RTX4090 and CUDA12.8/SM89. Job log `20261008T061527Z-astra-parent-action-20261008-gpu0.log` is terminal exit0.
- The unchanged library's 15 bridge tests, including explicitly executed CPU/CUDA full-120 action/query/source adjoints, remain bound to the previous delivered source and are reused; they were not rerun for this example-only change.
- Original accepted parent/donor/plan/config identities were verified from the retained shared volume. New config SHA256 `5a958ca54bb8a5abc71deebf6a0dcb82d2ac23c7afc6d8a69c21661fbf636ec1` adds action-only mode and a fresh exclusive output root.

No model outcome is claimed by these checks. The only forthcoming experiment is the prospectively declared parent-action rule in #820/comment6053558247. Queue acknowledgement statuses are not substitutes for these executed checks.


### comment 6054236636 — 2026-10-08T06:51:24Z — Casey-allard

Exact-final-head validation PASS at **1b5fa46babd0f6cd2880619cc8acf2ad5e96884b**, tree **e0abd2fca4b6dcd3aaed678327f39babdb1e4c0a**, including current main41234d24b. Independent source/scientific-claim review PASS with no blocking findings; changed Rust file bytes match executed3a7c.

Actual admitted RTX4090/CUDA12.8 validation,2 CPU threads, per-lab target directory:
- Changed formatting PASS0.120646s.
- `cargo test --locked --offline --release -p uor-r4-training --features cuda --example geometric-frozen-map-fit`: **34 passed,0 failed,0 ignored**; compile/test161.894819s.
- `cargo build --locked --offline --release -p uor-r4-training --features cuda --example geometric-frozen-map-fit`: PASS12.303843s.
- Claim-wording check and git diff whitespace check PASS locally.

Log20261008T064742Z ends exit0 at2026-10-08T06:50:37Z. Initial final-check dispatch064556 exited2 before compilation because the script was absent at the chosen path; unchanged helper uploaded and same-head retry passed. Preserve the setup fault separately; it is neither a model negative nor a Rust test failure.

The sole model run remains the authenticated3a7c execution: native CE9.08399492844048→9.07236859663813,8/512complete with all8 retained,10 changed sequences/4wrong-to-wrong entries. Initial/final588-position graph/native parity, independent saved-evidence endpoint, separate Rust complete-file BLAKE3 and pinned1872/1875/1877 comparisons PASS. Fullrun preserved at private HF1eae263d4d1f002a50c92eb82ed03aa65106cda1, archive148e3ad4c7b72c60f15a5ae4ce34caa1897fb80eed3fabdd806e88ae5fff1983, fresh verified. No useful-model promotion, held-out/chat/geometric-advantage/energy claim. The sampled243092803bytes is report-directory size.

Five GitHub historical status names are compatibility acknowledgements, not these actual tests. Protected merge remains separate until GitHub and fresh main confirm it.


### comment 6054283902 — 2026-10-08T06:54:16Z — Casey-allard

Delivered through protected **PR1878**, merged **f54050ca1f0d61bae09f0e4465e41a56b3ed789e** at2026-10-08T06:52:17Z. Fresh origin/main contains the merge, and its tree exactly equals reviewed/tested1b5fa46babd0f6cd2880619cc8acf2ad5e96884b: e0abd2fca4b6dcd3aaed678327f39babdb1e4c0a. Owner main checkout was fast-forwarded; local token-optimizer files preserved. Same existing worktree/branch reused.

One original-parent shared action replacement lowers native CE9.08399492844048→9.07236859663813, but complete native replies remain8/512 with all8 retained,10 changed sequences and4 wrong-to-wrong entry winners. Accepted parent retained; no useful-model promotion. This is optimization/evidence progress, not accomplished chat or durable-memory capability. Mathematical, architectural/engineering and adversarial reviews support the scoped conclusion and prospective shared Context update construction against exact native recurrent rounding/ties. Full120 utilities already exist; no unchanged action-key/dose/seed repeat is scheduled.

Exact-final-head formatting,34 fitter tests (0failed/ignored), CUDA release build and independent review PASS. Actual native initial/final588-position parity and saved-evidence endpoint/Rustseal/prior-artifact comparisons PASS. A derived reader's absent state field was corrected with both versions retained; a final-check missing-helper dispatch fault was retained and repaired before successful same-head tests. No model rerun.

Private HF caseyallard/uor-r4-store:
- Fullrun revision **1eae263d4d1f002a50c92eb82ed03aa65106cda1**, prefix codex/astra-parent-action/parent-action-0001-3a7c9fdae9; archive SHA256148e3ad4c7b72c60f15a5ae4ce34caa1897fb80eed3fabdd806e88ae5fff1983,74747558bytes; fresh archive/manifest verified.
- Final delivery revision **6b4cf377d7e5f42362eb2568fcd05abb0e650247**, prefix codex/astra-parent-action/final-delivery-1b5fa46bab; all17files freshly download-verified, manifest01a59ba26db2066daa0987047d4cec201656049f3f4d07274fb4e7848801924f. Includes exact-head checks, source delta, merge receipt, resource snapshot, engineering review and amended adversarial review plus dispatch fault.

Producer612.596s, supervisor657.475s; sampled report-directory243092803bytes, RSS1282320KiB/GPU1564MiB. All preparation/build/review/audit/preservation/delivery remains cumulative under projection360870s. Owned podofqn9sjw0y2ul2 lease released and pod deleted after preservation; no active model job left. Goal remains active and unachieved. Next source-design dependency is recorded in current state and must yield a reviewed concrete workcard before more model compute.



---
_Comment/review ids in this file: 6053742969, 6054236636, 6054283902_

