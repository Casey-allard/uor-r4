# #1151 — Repair learned relation writes and record the NoWrite reuse regression

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-06T17:11:29Z
- **Updated:** 2026-10-09T13:17:33Z
- **Closed:** 2026-09-06T17:35:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1151
- **Merged:** 2026-09-06T17:35:07Z
- **Merge commit:** 2242cf037a0684a19cb5ff4dca7331e5a4447432
- **Base / Head:** main / UOR-Foundation:codex/geometric-writer-binding
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The learned writer previously attached some `Now owner in value` revisions to `Now`, and question text could create spurious records. This change gives the writer a construction-bound cue vocabulary, excludes labeled payload spelling from cue scoring, and refits the existing sparse integer owner/value/action learner. It also trains the missing adjacency between a previous question and the next user assertion. Exact records, version semantics, dependent reader parameters and copying remain unchanged.

Retain artifact `8dbf1367` as a functional development result, preserving `8070c006` as the reader/cost comparator:

- 48/48 dependent answers and exact write sequences, repairing all eight prior revision failures.
- 62/62 earlier responses, 24/24 transfer, 28/28 long-context answers/writes and 5/5 persistent turns, with restore and isolation.
- 28/28 replacement reserved-name answers/writes after selection; familiar grammar, not general-language qualification.
- Actual CLI returns ` Zurich.\n` after the second link is revised. Four unchanged generated Rust functions compile and pass 12 semantic assertions. The broader owner-of-box question remains Unknown.

**Cost regression:** the new cue namespace produces no hits in the old exact NoWrite cache on the long-context set. Writer row comparisons rise from 580,944 to 226,101,330 despite equal correct writes/answers; descriptive generation subtotals rise from 34.5 ms to 767.4 ms. This is not a serving-efficiency promotion. The immediate next #1139 task recompiles existing bounded admission metadata for this frozen writer before expanding composition. No transformer or runtime matrix operation is added; startup validation still uses floating point.

Three unsuccessful attempts are preserved. An earlier fresh set was accidentally evaluated after a failed session-preservation guard; it remains explicitly OPEN. A separately named replacement reserve was evaluated only after successful selection. Full #1139 and #1140 handoffs remain unmet.

Validation: release core/example/native CLI build; five focused relation state/geometry/cue tests; the actual artifact's ingest/select/observe/copy allocation census (0 allocations, 0 bytes); generated Rust compile/execution; formatting, architecture policy and claim wording checks. Protected CI is pending; its compatibility-only statuses do not run broad audit/fuzz/WASM/Gate-C checks.

Local cycle model work: 123.484/150 s; cumulative 1731.154/1800 s, leaving 68.846 s. Engineering commands: 1048.088/1200 s. Necessary +128 MiB storage recorded under standing owner authorization; material preserved, no deletion or paid compute.

See `docs/native_geometric_writer_binding_1139.md`, its evidence JSON, and the updated current-state/roadmap. Continues #1139; does not close it.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

