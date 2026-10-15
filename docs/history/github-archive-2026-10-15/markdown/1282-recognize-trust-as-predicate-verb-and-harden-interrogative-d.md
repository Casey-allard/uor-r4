# #1282 — Recognize 'trust' as predicate verb and harden interrogative detection in contextual role disambiguation

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-17T06:56:35Z
- **Updated:** 2026-10-09T13:17:11Z
- **Closed:** 2026-09-17T06:57:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1282
- **Merged:** 2026-09-17T06:57:37Z
- **Merge commit:** 5fb3c99a705c72a8ba3b1b3fcf857a40700493a6
- **Base / Head:** main / UOR-Foundation:codex/contextual-role-trust-fix
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- Complete predicate verb coverage in crates/uor-r4-core/src/native_geometric/dependent_language/contextual_role.rs: added development verb 'trust' to is_verb alongside call, help, visit, follow, guide.
- Resolves an edge-case defect where a declarative sentence with governing verb 'trust' followed by an object containing 'will' (or preceded by a subject containing 'will') failed to recognize 'trust' as a verb, causing 'will' to be misclassified as a governing auxiliary verb (true) instead of content (false).
- Hardens clause_ranges to identify interrogative clauses starting with 'who' even if punctuation marks are absent.
- Adds comprehensive regression tests for declarative sentences with 'trust' verb and interrogative sentences without trailing '?'.
- Crate qualification verification: passed official crate qualification test native_geometric::dependent_language::independent_neighbor_report::independent_neighbor_evaluate_report with 2,304/2,304 (100%), 288/288 recovered, 7,680/7,680 historical retained cases bitwise equal.
- Invariants: 0 warnings (cargo check --all-targets), clean cargo fmt --check, clean scripts/check_claim_wording.py.

References #973, #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

