# #1836 — docs: record fixed-parent route capacity and fifth matched fit

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T07:46:13Z
- **Updated:** 2026-10-09T13:17:43Z
- **Closed:** 2026-10-07T07:47:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1836
- **Merged:** 2026-10-07T07:47:57Z
- **Merge commit:** 4c3fe85005e703e4976c4509f4065dda21d870d3
- **Base / Head:** main / UOR-Foundation:codex/native-route-capacity-record-20261007
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The current-state record still describes the fifth matched fit as running and leaves a fixed-parent routing experiment undecided. Record its completed, independently audited result and the completed zero-update route-capacity replay, with exact source, executable and report identities.

The fixed checkpoint-0000 result excludes Potential-only entry repair on four construction rows; it does not classify the six trained checkpoint-0128 exports. The next action is to finish the final unchanged fit and assess all completed exports before choosing a distinct learning intervention. The experiment driver stays on its RECORD branch rather than entering production code.

Validation at this documentation head:

- `git diff --check` passes.
- `python3 scripts/check_claim_wording.py` passes.
- Numeric claims and artifact boundaries are checked against the independently audited results on [#820](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6033325044).

This changes one documentation file. Rust/CUDA source and the six running-fit conditions are unchanged. The eight focused tests and actual CUDA replay cited in the document belong to experimental source `b4bc1ac95bc54c31fc4db5ee25da9a87478a2956`, not to a new model run in this PR.

References #820.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6033428250 — 2026-10-07T07:46:51Z — Casey-allard

Exact-head self-review: `1d32da99df7f89b5207d7721723cbc89a37a774b` — READY.

This is a single-file documentation change based directly on main `3fbf69f12bd382799a8e94e292186c91ee117a1d`. The experimental Rust driver stays on its pushed RECORD branch; no model, runtime, training condition or generated output enters this PR.

Executed at this head: `git diff --check` PASS; `python3 scripts/check_claim_wording.py` PASS. No Cargo run is needed for this documentation-only change. The Rust/CUDA checks cited in the text were executed at their separately named experimental source.

Independent read-only artifact reviewer: READY at this exact documentation head. Fifth-fit metrics/hashes and all 176 physical routes/44 entry certificates match audited reports. Ties require the lower-ID rule; later positions use teacher prefixes; CUDA/CPU timings have different scopes. The checkpoint-0000 exclusion is not extrapolated to trained exports. The next discriminator applies to all six checkpoint-0128 artifacts before selecting a distinct intervention. The independent artifact review does not claim to re-execute build or pod lifecycle receipts.

No new fit or GPU allocation was made for this documentation change.



---
_Comment/review ids in this file: 6033428250_

