# #2097 — Trace earliest-query influence through native scoring

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T22:00:22Z
- **Updated:** 2026-10-09T22:04:56Z
- **Closed:** 2026-10-09T22:02:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2097
- **Merged:** 2026-10-09T22:02:35Z
- **Merge commit:** 0ad73899ad57e18989f12b569527381ec0e8ebdc
- **Base / Head:** main / UOR-Foundation:codex/early-query-causal-20261009
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why

References #2030. The input-access review left a causal question: does an answer-relevant early query token reach native scoring? This adds a sealed Rust input-only single-token intervention and an opt-in serialization of the existing entry trace, with no serving changes or model fit. The retained Source48/Generate64 parent has no U field.

## Result

Two original development job/home controls complete correctly with EOS (2/2). The novel diagnostic requests differ solely at token zero, with identical remaining11 tokens, bank and empty prefix: 0/2 correct entry tokens and0/2 complete replies. The change survives all8 final state/post-bridge lanes and changes3981/4096 Generate and11/11 Copy raw scores; Source candidates and donor remain identical. KEEP this bounded influence witness; proceed to previously declared protected-margin integration. No broad512 cause, semantic-sufficiency or capability claim; accepted8/512 unchanged.

Actual own-feedback run binds source ab247227a3a10672144c25651d4f61b67c9d52d0 and binary77407b3c28758e893d1f53a603c60ab61e64ba6815bcf7b209c036f67f890c94. Final-head Rust model/evaluator blobs are identical. Preserve offline dependency setup failure, sealed128MiB storage failure, unchanged512MiB complete retry, and both saved-analysis versions. The strengthened saved-only analysis binds cross-file IDs/hashes and recomputes every pooled winner; no additional model pass. Full identities, scope, counts, cost and next action are in the dated research record.

## Checklist

- [x] Exact-head checks and review receipts posted before queueing
- [x] Dated research record and newest-first current-state entry ending **Next:**
- [x] Stale-doc search completed; prior dated records link follow-through; geometry text and figure regenerated
- [x] STATUS / ROADMAP / tracker headline unchanged: no promoted model or changed milestone
- [x] README unchanged: no new qualified capability, product command or runtime mechanism
- [x] Claim-wording passes; geometry connections29/29 PASS
- [x] After merge: fresh main/source equality verified, iCloud evidence preserved, branch/worktree/pod deleted and scoped hygiene applied

The required five CI status names acknowledge transport compatibility; actual focused tests and review are reported separately.



## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6089984396 — 2026-10-09T22:01:41Z — Casey-allard

Exact-head validation/review: 36d3dc83f848fa6b226fe1cfab8e61bac4ffd429.

- Release focused example tests: 19 passed, zero failures. Changed Rust paths rustfmt check: PASS. The broad cargo fmt --all --check reports existing unrelated formatting differences (first addressed-attention-arith-fit.rs); these are retained and not repaired outside scope. Initial exact-head shell invocation lacked cargo in PATH; absolute rustup path repaired this setup, with both logs preserved.
- Claim wording and git diff --check: PASS. Geometry connections: 29/29 PASS; figure regeneration committed.
- Independent exact-head adversarial review: PASS. Cross-file attribution and configuration-serialization wording findings resolved. Mathematical outcome review supports the bounded learning/scoring successor; no semantic-sufficiency, component-isolation or full512 causal claim.
- Actual native generation ran at ab247227a3a10672144c25651d4f61b67c9d52d0. The two changed Rust blobs are identical through this final head. Both original controls complete correctly; both novel one-token interventions fail, with the detailed score influence in the record. Saved v2 analysis passes and equals v1 numerically; no repeat model pass for review.
- Submitted and sealed evaluation configs parse identically; normalized formatting explains their hash difference. All three rows retained before the first attempt's storage failure have identical SHA256 in the successful retry.

Required CI statuses remain compatibility acknowledgements, not these tests. Protected merge and cleanup are pending.


### comment 6090023131 — 2026-10-09T22:04:45Z — Casey-allard

Merged #2097; fresh origin/main 0ad73899ad57e18989f12b569527381ec0e8ebdc verified with all ten delivered blobs equal to reviewed head36d3dc83f848fa6b226fe1cfab8e61bac4ffd429.

On retained Source48/Generate64, no U: original development job/home controls2/2 complete withEOS; novel one-token-zero intervention0/2 correct entry and0/2 complete (32-token bound, noEOS). Same bank,11-token suffix and empty prefix. All8 final recurrent/post-bridge lanes,3981/4096 Generate raw scores and11/11 Copy scores differ; Source candidates and donor are identical. This rules out total early-cue erasure before scoring on this pair, not semantic insufficiency/sufficiency or the cause of all512 errors. Accepted8/512 unchanged. Next: previously declared shared protected winner/rival and authenticated Prefix credit integration, preserving all native acceptance conditions.

19 focused tests and changed-Rust formatting pass at the exact head; geometry29/29 and claim wording pass; independent exact-head review PASS. Broad workspace formatting still reports unrelated existing differences. Failed offline setup, report-storage attempt and analysis correction retained. Full own-feedback model time8.59s including failed attempt; no training/CUDA. Shared elapsed charge28.15min through delivery/preservation, cumulative1,444,171,663ms of1,449,000,028ms (cleanup/report tail continues from22:03:32.634649Z).

Private evidence verified twice: icloud:UOR-R4/results/codex/codex-early-query-causal-20261009.tar,17,122,304bytes,MD5 bfef6383e93969f073eb9058d6dd90ba. Codex pod f138jobfuq5l0m released/deleted. Remote/local delivery branch and worktree removed; owner checkout clean main; scoped hygiene applied with peer worktrees preserved. Local free space approximately52GiB.



---
_Comment/review ids in this file: 6089984396, 6090023131_

