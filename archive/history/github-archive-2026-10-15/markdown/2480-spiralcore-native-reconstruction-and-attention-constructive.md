# #2480 — SpiralCore native reconstruction and attention: constructive review and byte evidence

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T17:25:05Z
- **Updated:** 2026-10-14T17:29:20Z
- **Closed:** 2026-10-14T17:27:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2480
- **Merged:** 2026-10-14T17:27:56Z
- **Merge commit:** bbbbdf7fbb28c5c8f6f5cd83ab7aecf48e7d4889
- **Base / Head:** main / UOR-Foundation:codex/spiralcore-native-attention-20261014
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why

This completes Casey's requested constructive reassessment of SpiralCore as an exact reconstruction substrate that a native attention read can use. It corrects the previous report's incomplete treatment of the active HTML score/instrument/FBS loop, explains compression before tokenization, traces the current carrier/read/writer, and supplies a concrete finite relational-head interface and evidence-dependent roadmap.

Six native Codex specialist assignments (math, networking, ML, physics, CS and systems integration), followed by math/ML adversarial cross-review, are retained with the synthesis. They are agent review roles, not external-lab endorsements. The prior report and all peer source/results remain preserved.

## Result and scope

Decision: NOT YET PROMOTED. The architecture is coherent; no replacement law or model configuration is adopted. Exact root packing and the comparator-specific sixteen-frame top-K index are conditional source-derived proposals. The relational head remains unimplemented and untested. No model/reference execution, tokenization, attention simulation, CUDA, paid resources or training occurred; this is byte/data-structure evidence and research documentation. No owner hold is released.

Fresh original-code checks pass: 65,536 symbolic-prefix round trips, 10,000 address cases, legacy radix checks; current golden-codec byte packing and ordered FBS checks; and 813 selective byte ranges / 2,374 assertions in a dependency-free Rust reader over actual Showcase records. A 23-byte interval touches eight node records / 384 serialized bytes after store construction/loading, not an end-to-end network or memory benchmark. The primary agent independently reran the scripts and Rust experiment within the same lab and obtained identical result JSON/text.

Line: owner-requested architecture assessment · count 0/3 (no model line advanced) · headline 0 → 0 newly qualified replacement heads. M1/M2 milestone headline and their existing line counts remain unchanged. This completes the requested review and creates no waiting review or FIT dependency for Kimi or Claude. A future model design/qualification remains separately subject to prospective FIT and owner authorization.

## Validation and delivery record

At head `3080e96aa214db09d42083d6649607c2c920a9ae`: D23 ratchet PASS (1641 existing sites), D25 ratchet PASS (93 existing sites), claim-wording PASS, standalone Rust formatting PASS, 19 evidence-file SHA-256 checks PASS, source-codec reruns match retained results, and `git diff --check` PASS. The retained Rust reader is byte-identical to the compiled/tested primary reproduction. Ledger shard check: six shards, 440 receipts, none lost. Final independent agent document review: PASS, explicitly not FIT or model validation; its reproduction-path nit is corrected.

- Research record: `docs/labs/spiralcore-native-attention-review-2026-10-14/README.md` with reviews, runnable adapters, results and resource observations.
- Current-state entry and earlier report cross-link updated. STATUS/ROADMAP/tracker headline and curated README stay unchanged because no capability, milestone or active mechanism changed.
- Documentation/evidence only; no model-compute preregistration applies to the executions in this PR.
- Protected merge, fresh-main content verification, preservation and owned-workspace cleanup will complete delivery.

References #2029

- [x] Checks run at this exact head: `uor-harness ratchet d23`, `uor-harness ratchet d25`, `python3 scripts/check_claim_wording.py`, `rustfmt --check --edition 2021 <retained range_reader.rs>`, `shasum -a 256 -c SHA256SUMS`, source-codec reproduction and `git diff --check HEAD^ HEAD`, all PASS.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6174260429 — 2026-10-14T17:26:43Z — Casey-allard

Codex / Sol exact-head research-document review: PASS at `3080e96aa214db09d42083d6649607c2c920a9ae`.

The six specialist memos and math/ML adversarial reviews are retained. Final ML document review found no substantive false claim, missing causal caveat or contradiction; its publication-path nit was corrected before this head. I checked the synthesis against the codec/reader evidence, verified all 19 evidence SHA-256 entries, checked all 17 relative Markdown links, reran both source-codec scripts at this head with identical results, and verified the committed Rust reader equals the independently compiled/tested reproduction. D23/D25 ratchets, claim wording, Rust formatting, diff whitespace and append-only shard checks pass.

This approves the report's stated evidence and feasibility scope. It is not a TEST FITNESS verdict, new model qualification or owner release. Exact packing/index proofs remain conditional and unimplemented; predictive value of the relational head remains NOT_RUN. No source/model/configuration adoption or experiment is authorized here.

PR lint has zero errors and two registry advisories because this owner-requested architecture review is not a registered model line and its zero newly qualified heads is not M1's quality headline. The body explicitly preserves the milestone's headline/count; no registry or headline has been changed to make those advisories disappear. Required compatibility acknowledgements are not represented as executed tests.



---
_Comment/review ids in this file: 6174260429_

