# #2149 — M2: archive joint reply fit at 0/512 and close ordinary recipe

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T09:15:08Z
- **Updated:** 2026-10-10T09:27:08Z
- **Closed:** 2026-10-10T09:17:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2149
- **Merged:** 2026-10-10T09:17:23Z
- **Merge commit:** ce08d10d8aa8063fb08576b3432a17e934bee294
- **Base / Head:** main / UOR-Foundation:codex/m2-joint-reply-20261010
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Joint Context–prototype training changes native parameters but regresses complete replies from **8 to 0/512** and from **8 to 0/24** trained rows. **REJECT** the candidate; accepted M2 remains **8/512** (target 256). All eight accepted successes are lost, with no gains.

**Line: ordinary reply-completion gradient learning · count 4/3 (the single D21 decisive continuation after the 3/3 pivot) · headline 8/512 → 8/512.**

This is the pre-registered decisive run from [the pivot](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6095599251), [registered before compute](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6095654613). Compared with #2148 it adds all nine Context families to the same accepted-parent prototype-enabled 24-row/96-update recipe. Actual native exports change 12,307/8,963,136 Context Q4 coefficients and 25,718/32,768 prototype codes. Lower teacher-prefix CE does not qualify replies.

The ordinary 24/96 recipe is now closed: no further family-by-family, dose/rate/seed continuation. The new joint mode is retained as an indexed source patch, **not activated on main**; the net Rust source diff is zero. The canonical record, source/failed-build archives, exact artifact identities, prior-candidate comparisons and current-state/STATUS/ROADMAP updates land together. The constructor/attribution/solver line stays closed. No preparation PR.

**Counterfactual before opening:** yes. Retained new complete replies would keep a candidate; 24/24 fit would support expanded qualification. The observed regression instead closes the recipe. [Result decision](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6095974330).

Validation: producer1833cb4 passed the optimized CUDA build and 10 focused reply tests after a preserved compile-only failure; the sealed run exited0. Independent review authenticated all1,024 saved endpoint rows and both native payload crossing counts, plus all three prior reports. Final delivery-head checks and review will be posted before queueing, separately from producer validation. Exact archive patches reproduce stated revision blobs. Whole iCloud archive round-trip MD5 and six-column index readback passed (168,509,440 bytes).

Measured model process30.481 min, peak RSS1,463,804 KiB, sampled GPU1,702 MiB. One authorized RTX5090 at$1.19/h; full cumulative cycle/cleanup and actual pod receipts follow at delivery. No serving-cost or impossibility claim follows.

References #2030. A result PR ends this cycle, not the continuous goal.


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 6096031304 — 2026-10-10T09:15:59Z — Casey-allard

Independent exact-head review: **PASS**, no blocking findings, at `876f06b6f9a3e86442008d3b5a6dd60c50ed59bc` (read-only reply_review agent).

The net Rust diff versus published base52022ae is empty. Both source archives exactly match declared Git diffs and restore the stated producer/compile-failed blobs; both bundles independently pass git bundle verify with published base52022ae. README/current-state/STATUS/ROADMAP agree on 8→0/512, 8→0/24, REJECT, accepted8/512, count4/3 and recipe closure. Committed measurements, saved-row reader/results and config match independently reviewed originals. Preservation identities/receipts are consistent, including failed-build material.

Independent saved-result review checked all1,024 endpoint rows, exact IDs/hashes, own-prefix/chosen-token traces, EOS and frozen membership; all three prior comparisons; and directly reproduced native Context12,307/8,963,136 and prototype25,718/32,768 crossings. No model rerun or mutation by reviewer.

Final-head executed checks are separate and still running. Full-diff whitespace checking flags only four expected single-space context lines inside the exact archived patch; excluding archive patches passes. The archive itself is retained byte-for-byte.


### comment 6096034533 — 2026-10-10T09:16:24Z — Casey-allard

Exact-head self-review and executed validation: **PASS** at `876f06b6f9a3e86442008d3b5a6dd60c50ed59bc`.

I reviewed the complete base-to-head delivery diff: native code is unchanged; producer mode and compile-failed variant are losslessly archived; result/KEEP decision, original-eight losses, prior-candidate comparisons, count4/3 and narrow limitations agree across the canonical record and status documents. Independent exact-head review is recorded above. No blocking findings.

Actual checks at this head on the admitted owned pod:
- Scoped rustfmt check on both relevant Rust files: PASS.
- `cargo test --release --locked --offline -p uor-r4-training --features cuda --example geometric-frozen-map-fit reply_ -- --test-threads=2`: **8 passed,0 failed,0 ignored**;95.595s;peak child RSS2,331,788KiB. These are main's active tests, separate from the producer's10 tests.
- Claim-wording gate: PASS.
- Archived negative patch applies cleanly; isolated-index reconstruction exactly matches both declared tips.
- Base-to-head diff whitespace check excluding byte-exact archive patches: PASS (patch context-space exception described above).

The five historical queue status names are compatibility acknowledgements and are not the test evidence. Logs bind this exact head and will be preserved with protected-delivery receipts.


### comment 6096115581 — 2026-10-10T09:27:08Z — Casey-allard

Delivery and cost closeout for #2149:

- Protected merge `ce08d10d8aa8063fb08576b3432a17e934bee294` verified on fresh origin/main; entire tree `51786e488702ae3e9fa03019fb11e28c23082bd0` equals reviewed/tested head876f06b.
- GitHub delivery branch, owned remote worktree and its local branch removed; no laptop worktree/branch was created.
- `uor-hygiene --apply` completed exit0, freed about0GB,21GiB free. Other labs' active/recent/unarchived material was preserved.
- Main evidence archive:168,509,440 bytes,MD5 `7d5a401263c9b267b4a8ef79f724776b`. Delivery closeout:`codex-m2-joint-reply-20261010-closeout.tar`,81,920 bytes,MD5 `c5f32ac730a403fa367807636ace6217`. Both under `icloud:UOR-R4/results/codex/`; full streamed download verification and standard six-column index readback PASS. No local archive copy. Closeout includes exact-head checks, delivery bundle, merge verification and cleanup/hygiene receipts.
- Own pod8uckvwxtq0oruw released09:26:17Z and deleted09:26:36Z. Ledger uptime3,676s at$1.19/h ≈**$1.215**. No live Codex model job remains.
- Complete cycle08:21:30→09:26:55Z charged **3,925,000ms (65m25s)** including preparation, failed build, model/evaluation, review, preservation, delivery and cleanup. Shared cumulative **1,475,947,627→1,479,872,627ms**, limit1,485,000,028ms; concurrent ledger fields preserved. Next cycle starts09:26:55Z so the final posting tail is charged there.

REJECT0/512; accepted8/512 remains. [Stop/archive pivot](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6096048923) closes the ordinary24/96 recipe at4/3. The continuous goal remains active; next cycle starts now.


---
_Comment/review ids in this file: 6096031304, 6096034533, 6096115581_

