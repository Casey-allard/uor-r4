# #1343 — Learn contextual text roles; repair causal evidence and define argument binding

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-22T01:48:56Z
- **Updated:** 2026-10-09T13:17:12Z
- **Closed:** 2026-09-22T02:27:17Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1343
- **Merged:** 2026-09-22T02:27:17Z
- **Merge commit:** 6ee9d0f53feea5e03dfaa408ff6a08776e2eabc8
- **Base / Head:** main / UOR-Foundation:codex/contextual-text-roles
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The observed-text learner produces complete multiword answers on the authored panel, but its runner conflated two learning counters, used incompatible membership encodings, and discarded inherited all-arm and checkpoint evidence. This change corrects those instruments and the module score/supervision/restore contracts, retaining the byte-identical learned artifact and all 24 primary answers.

The independently audited corrected replay gives 24/24 development and exposed-regression answers, 20/24 membership continuation, 18/24 maximum-two-read and 0/24 NoRead. Observation learning is 0/48→48/48; action learning is 14/24→24/24. All 811 serialized checkpoints reproduce complete continuations. A valid alternate-person source edit changes the complete answer and read path; actual terminal-evidence removal follows two redirects. The original Office Park parsing failure remains a retained counterexample. These are exposed authored-task results, not fresh general-language or geometry-superiority evidence.

The principal review identifies the next representational limit: subject and object are derived from fixed prefix/marker/suffix placement. The comprehensive successor brief calls for learned argument/query spans across ordinary forms, raw surface provenance separated from lexical identity across BPE contexts, and ordered H4 span potentials compared with the adapted categorical scorer. README, roadmap, current state, evidence and resource records reflect this scope. The corrected #1342 dependency is incorporated through an actual merge, restoring nine omitted review/evidence/handoff files.

Validation: 13 focused module tests; 21 runner tests passed and one legacy artifact smoke test explicitly ignored; offline release build; actual corrected generated output; independent sealed source/event/restore audit; formatting, claim wording and 1,060 relative document targets. The first partial corrected replay exposed a cycle-attempt counting error; its source/executable/report are retained and the subsequent corrected replay passes. Queue compatibility statuses are not tests. Complete-path D0-b, latency, energy and broad language remain unqualified.

References #820, #973, #1139, #962, #963, #964. Supersedes the conflicting delivery of #1342 while preserving its corrected history and contents.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5770355796 — 2026-09-22T02:28:04Z — Casey-allard

Protected delivery verified: PR #1343 merged as `6ee9d0f53feea5e03dfaa408ff6a08776e2eabc8`. Its complete tree equals reviewed head `6606d989cc5412671ec3b65e5f6fa3656f170e70`; all 32 changed paths match. Corrected parent `f5919d8e` is preserved as an actual ancestor with no parent paths deleted. Conflicting PR #1342 is closed as superseded, not itself merged. Owner checkout remains clean at `74fef088`.

The principal review and comprehensive next brief are on main: `docs/integration/contextual-text-roles-review-2026-09-22.md` and `docs/integration/deepseek-structured-argument-binding-step-2026-09-22.md`. Source-linked knowledge includes 20 source/issue records and 20 relationships, with a separate verified delivery receipt. The next milestone is learned structured argument/query spans and exact lexical identity across BPE contexts through the same runtime, retaining an adapted categorical comparison for ordered H4 span potentials. The current 24/24 component result remains an exposed authored-text result. No broad issue acceptance is closed.



---
_Comment/review ids in this file: 5770355796_

