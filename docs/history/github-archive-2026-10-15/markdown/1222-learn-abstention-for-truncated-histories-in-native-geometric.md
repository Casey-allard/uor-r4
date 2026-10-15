# #1222 — Learn abstention for truncated histories in native geometric model

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-10T14:26:23Z
- **Updated:** 2026-10-09T13:08:49Z
- **Closed:** 2026-09-10T14:28:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1222
- **Merged:** 2026-09-10T14:28:22Z
- **Merge commit:** 89fd8e3a4455d26bafdcb1a3d5d9efc53ba7817a
- **Base / Head:** main / UOR-Foundation:codex/truncated-history-abstention
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

When later facts evict a chain's root from the sixteen-slot record ring, an initial-version request could only fall back to the parent's previous-value answer. The retained version-intent witness now carries a versioned `abstention` option: a request-named revised chain whose validated links reach no genuine root is offered to the same learned selector as a truncated candidate competing for the parent's no-read action, so the model answers `Unknown.` instead of the oldest survivor. Complete chains, the ancestor path proof, owner-role masking, the per-view dictionary and every inner parameter are unchanged; legacy artifacts keep their exact wire form and never abstain.

Retain `e44a9ae4`, refit from frozen parent `f0901dad` over the extended construction: 594/594 construction targets (486 exact root answers, 108 abstentions through actual no-read decisions) with 3,195/3,195 preservation cases. The separately frozen fresh draw passes 594/594 targets (108/108 abstentions) with 778/778 preservation cases, complete removal reproducing the parent on all 594. Against retained `e3a906c8`, outputs are identical on 3621/3669 construction-4 rows, 746/794 first-draw rows, 842/890 second-draw rows and 1204/1252 third-draw rows; every differing row is a declared evicted-root initial request that now answers `Unknown.` instead of the surviving previous value. Complete removal reproduces the parent on every target; withholding only the truncated candidate returns the parent's answer on 84 abstention rows and the surviving previous version on 24. The first fit attempt appended too few unrelated facts to the two-chain layout; the fit refused the mislabeled row before learning and the attempt is preserved.

Validation executed locally:
- Optimized offline Rust core/API/example builds; formatting and claim-wording checks.
- - Focused verification: 31 checks on `e44a9ae4` (six ancestor, three version-format and nineteen field/dispatch unit tests, the integer-kernel guard, the actual-artifact test over ten conversations and 40 turns with two evicted-root abstentions and disabled-control parity, and an allocation census of zero allocations and zero bytes including one abstention); the first focused chain failed on a stale class-bound assertion and was rerun, both charged.
- 2,376 native API completion/checkpoint checks (1,188 construction, 1,188 fresh), including every abstention.
- All 72 query-owner, 216 historical-transfer and 764 handoff-fresh comparisons are exact. All fourteen older behavior/control reports have zero checked differences, including their negative interventions. Twelve multi-turn histories replay with identical outputs, records, versions and sources, and 24 word/Rust cases are preserved. Copy/Add and action each pass 24/24 ordinary and 24/24 stress cases on the new artifact, and actual generated Rust is compiled and executed.

The immutable evidence records 2,377.008 model seconds and 570.968 engineering seconds (cumulative 52,355.724 / 54,450.000 model seconds after one standing-authorized 3,000-second extension recorded before use, 196.620 reserved), no monitor stops, and 46,211,366,912 / 47,748,116,480 accounted bytes leaving 1,402,531,840 after the 128 MiB margin. Final delivery charges append locally. No unique deletion or paid external compute occurred. Queue CI statuses are compatibility acknowledgements, not tests. Full-suite, HTTP/browser/WASM and complete-path energy checks are NOT_RUN.

These are bounded authored forms with fresh spellings; general prose, reasoning/program synthesis and alpha/frontier capability remain unfinished. Abstention is learned only for revised heads whose validated links never reach a root; intermediate-version intent is not claimed. Next: intermediate-version intent with exact depth labels.

References #973. The writer-context contribution obligation under #1139 remains separate and open.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

