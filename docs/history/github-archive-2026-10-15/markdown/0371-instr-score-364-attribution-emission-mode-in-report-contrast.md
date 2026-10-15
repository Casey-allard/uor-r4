# #371 — instr(score): #364 attribution — emission mode in report, contrast stats promoted, NGRAM/EXCT exact-context split (schema 13)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-03T13:52:59Z
- **Updated:** 2026-10-09T13:08:53Z
- **Closed:** 2026-08-03T14:23:11Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/371
- **Merged:** 2026-08-03T14:23:10Z
- **Merge commit:** 9a0e75c8e789f5fbbb885ba8191243175033651d
- **Base / Head:** main / UOR-Foundation:instr-364-attribution
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Attribution prerequisites for the #364 dual-track

Both #364 follow-on tracks (trigram-rows-as-replacement, cover-induction distinctiveness) are A/B measurements. Today the measurements are unattributable three ways; this PR closes the gaps with instrumentation only — **no artifact-byte or κ change, witness bytes untouched**.

1. **Report records the emission mode.** #366/#368/#370 are merged but opt-in, and `score_report.json` could not say which mode produced it. `config.emission_selection` / `config.emission_shrinkage` added (labels match the CLI strings).
2. **Contrast stats promoted from stderr into the report.** The per-region selection/contrast statistics (`[emission-selection]` stderr line) now land in the report as `emission_selection_stats` (normalized per-region means) — machine-readable, so contrast-distribution comparisons across covers stop being copy-paste.
3. **NGRAM row vs EXCT probe split.** Since #362, an explicit NGRAM row hit and an EXCT full-depth probe hit both report `ScoreStatus::ExactContext` — a per-status row post-#362 measures a different population than #358/#360 did. New `ExactContextSource` provenance (`NgramRow | ExctProbe`) threads scorer → `StepOutcome`/`ScoreOutcome` → api engine (`PredictOutcome.ngram_hit`) → C serving row (`StatusBreakdown.exact_context_ngram`, stdout line now prints `exact N (M ngram)`); Gate C splits `rule12_status_counts.exact_context` the same way.

`ScoreReport` schema 12 → 13 (additive, #293 convention). `ScoredGraphInfo` drops `Eq` (f64 means), keeps exact `PartialEq`.

Scoped gates green: fmt, clippy `--all-targets --all-features`, certify (all suites) / api / cli / status-policy tests.

Refs #364, #362. First of the dual-track series decided 2026-08-03; the context-row config knob (A/B enabler) and the induction distinctiveness term follow in separate PRs.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_018UJdEQFdckSK2et9G44jsm


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

