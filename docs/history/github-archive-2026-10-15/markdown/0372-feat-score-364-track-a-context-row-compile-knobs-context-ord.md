# #372 — feat(score): #364 track A — context-row compile knobs (--context-order / --context-entries) enable the trigram A/B

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-03T14:26:57Z
- **Updated:** 2026-10-09T13:09:11Z
- **Closed:** 2026-08-03T14:42:12Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/372
- **Merged:** 2026-08-03T14:42:12Z
- **Merge commit:** 7a96d03efa7d32cea2d666d2f2b1138537bb530d
- **Base / Head:** main / UOR-Foundation:track-a-context-knob
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## #364 Track A: the context-row compile knob — makes the trigram A/B runnable

Since #362, NGRAM rows compile **unconditionally** and take most-specific precedence over the geometric path at serving. The #364 verdict positions trigram rows as a candidate *replacement* for emission conditioning (geometric path retained for routing) — but that comparison could not be run: there was no way to build a rows-free bundle.

- `ScoreConfig.context_order`: **0** = no NGRAM rows (geometric path serves everywhere), **1** = bigram only, **2** = bigram + trigram (default); `ScoreConfig.context_entries` (default 64). **Defaults reproduce the shipped artifact byte-exactly**; non-default values change artifact bytes and κ by design — that is the A/B.
- CLI: `--context-order` / `--context-entries` beside the emission flags, validated; `cover_sweep` call site updated.
- `score_report.json` records both knobs (schema 13 → 14, additive) so A/B bundles stay attributable — continuing #371's discipline; with #371's `exact_context_ngram` split, the two bundle rows decompose cleanly.
- New unit test pins the order gating (0 → empty, 1 → bigram-only, entries bound); CLI defaults test extended.

The A/B itself (default bundle vs `--context-order 0` bundle on the fixture corpus, rule12 + per-status rows) follows once this lands.

Refs #364, #362. Second PR of the dual-track series; Track B (cover-sweep × contrast shrinkage, then induction distinctiveness term) next.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_018UJdEQFdckSK2et9G44jsm


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

