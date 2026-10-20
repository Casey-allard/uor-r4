# #1215 — Repair query-owner geometric selection across competing revisions

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-09T17:54:08Z
- **Updated:** 2026-10-09T13:17:48Z
- **Closed:** 2026-09-09T17:55:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1215
- **Merged:** 2026-09-09T17:55:26Z
- **Merge commit:** 43f3e7056258107aa9fc41e459cc3fb9cb5a0877
- **Base / Head:** main / UOR-Foundation:codex/query-owner-source-selection
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

With two correctly stored revision chains, a recent question about the first owner could select the other owner's old value. The direct geometric reader gave both candidates the same H4 state and rank. Refitting three existing code rows separates the requested current occurrence (score 14) from the wrong one (score 13), without changing serving features, candidate admission or tie-breaking.

Add pre/post-hint diagnostic states and scores, Rust preparation bound to actual baseline dispatch, opt-in native API query-owner cases and causal allocation/checkpoint tests. Fitting remains the existing shared signed-H4 coordinate learner over frozen writer parent 231d; source vocabulary remains 286 codes.

The 24 competing-owner contrasts pass 24/24, versus retained baseline 22/24. Complete construction passes 82/82 labelled answers and 1,182/1,182 inherited comparisons against frozen writer parent `231d6950`; all 1,172 actual output/EOS/source/record comparisons against retained reader `7f70029e` remain exact. The earlier reader stress now passes 16/16 versus 15/16, preserving every previously correct answer. Writer construction/stress/prior-fresh panels and the earlier reader fresh draw pass unchanged. New post-selection transfer passes **72/72** labelled answers versus baseline **66/72**, all 36 current selections and all 36 inherited comparisons, without refit. All previously correct fresh baseline answers retain their exact output and applicable identities. These are familiar forms with freshly drawn whole owner/value spellings, not general language qualification.

Local validation: All 14 prior semantic/control reports retain exact applicable outputs and identities, as do 12 multi-turn histories. Word/Rust transfer passes 24/24; Copy→Add and action each pass 24/24 ordinary and 24/24 stress. Twenty-five focused tests pass, including the trace invariant, integer-kernel source guard and both actual current-source/query-owner allocation/checkpoint checks. The scoped native API records 63 PASS and 1 explicit NOT_RUN item for inner writer-only removal controls. Actual generated Rust equalities and identifier-return fragments compile and execute. Both current-source control names remove the same contribution and compare against `231d6950`; they are not independent ablations. `7f70029e` is separately compared. Full historical API, blanket suite, new browser/WASM/HTTP and complete-path energy measurement remain NOT_RUN.

Preserve failed preparation and unselected 39575f74 with their exact label-mismatch scope. Current/general-language and energy limits remain explicit; next is requested historical-version selection and emission.

At the immutable evidence snapshot, this cycle used **1726.541 model seconds** and **321.781 engineering-command seconds**; cumulative model use is **32231.496 / 34050.000 seconds**. The necessary **1,800-second standing-authorized increment** was recorded before use; the prior 196.620-second reservation leaves 1621.884 seconds. Complete projection: 3,000 model seconds, 1,200 engineering-command seconds, one model process, two threads, 4 GiB model/6 GiB build RAM, 1 GiB new storage, 512 new-input tokens/4,096 bytes and 96 output tokens. Inherited windows remain 8,192 tokens/65,536 bytes and five turns with up to 1,056 padding tokens per turn. Accounted storage is **38246420480 / 39158181888 bytes**, with 777543680 bytes remaining after the 128 MiB stop margin. No storage extension, unique deletion or paid external compute occurred. Final small formatting/claim/document/Git closure charges append to the local mutable receipt after this snapshot.

Formatting and claim wording checks are executed locally. GitHub's five historical statuses are compatibility acknowledgements, not test results.

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

