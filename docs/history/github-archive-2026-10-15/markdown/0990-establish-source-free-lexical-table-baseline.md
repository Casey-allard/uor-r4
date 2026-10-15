# #990 — Establish source-free lexical table baseline

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-28T13:39:12Z
- **Updated:** 2026-10-09T14:17:22Z
- **Closed:** 2026-08-28T13:40:16Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/990
- **Merged:** 2026-08-28T13:40:16Z
- **Merge commit:** a5745a1d2e8b1de420b2dd7a3edaf699f612847a
- **Base / Head:** main / UOR-Foundation:issue-989-source-free-table-baseline
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #989

## Outcome

Ships the first working source-free corpus-to-decoded-text baseline: deterministic construction-only lexical unigram/bigram/trigram tables, D3 held-out evaluation, exact payload inversion, bounded continuation, canonical artifact bytes, and a visible source-free-table CLI.

The pinned 3,000-document run scored 99,362/446,342 known held-out targets (22.261404%) versus 24,163/446,342 (5.413561%) for unigram, a +16.847843 percentage-point uplift. The fixed prompt emitted 16 valid UTF-8 units without a period-1/2 cycle. Two complete runs produced byte-identical reports and 35,655,288-byte artifacts.

Terminal: ESTABLISH_TABLE_NATIVE_LEXICAL_BASELINE.

## Activated evidence

- focused source_free_table_baseline_989 test: 3/3
- focused compile of uor-r4-core and the r4 CLI: clean
- named Rust formatting: clean
- real 3,000-document command twice: identical report and artifact bytes

All broad/workspace/BDD/teacher/graph/Gate C/kappa/audit/fuzz/WASM/conformance/formal/performance/product-chat/release checks remain NOT_RUN.

## Claim boundary

This establishes only a construction-trained statistical lexical table baseline with held-out next-route evidence and bounded exact decoding. It does not establish semantics, attention, geometry, correctness, reasoning, chat quality, performance superiority, multiplication-free serving, or release readiness.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

