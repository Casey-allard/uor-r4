# #813 — Sampled decode machinery: step candidate lists + engine generate_sampled_into (#655)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-19T01:00:34Z
- **Updated:** 2026-10-09T13:08:50Z
- **Closed:** 2026-08-19T01:17:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/813
- **Merged:** 2026-08-19T01:17:24Z
- **Merge commit:** 3d3427daf2d5151594367bcba6997818a33d6ed0
- **Base / Head:** main / UOR-Foundation:decode-sampled-machinery
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Slice 1 of the maintainer decision on #655 (2026-08-19): **sampled decode becomes the default everywhere, pinned seed, greedy opt-in**. This slice is purely additive machinery — no serving surface calls it yet; slice 2 wires the CLI + server defaults and docs.

- **`StepCandidates`** (certify): the bounded top-8 `(token, score)` list of one deployed scoring step, ranked under the canonical selection rule (score desc, token asc) so `ranked()[0]` **is** the greedy selection. `score_step_candidates_coded_with_recent` assembles it alongside the byte-identical plain step — one shared `score_step_impl`, list assembly outside the op census (the reference scorer's own sort convention).
- **`R4Engine::generate_sampled_into`** (api): seeded weighted sampling over the serving probe's own candidate list, through the **same D4 policy path** as `generate_into` — one shared policy implementation (counters, widen-once, novel-seen FIFO). Abstention is identical by construction: sampling only replaces *which served candidate* is emitted. Weights are the #762/#785-C2 scheme (order-preserving positive shift, soft ~1000-per-occurrence emitted-token penalty with floor 1, shared division-free `SampleRng::draw`).
- All new engine code sits inside the delimited integer-only status-policy block and passes its source scan (no float, no `*`/`/`/`%` value arithmetic, no `unsafe`).

## Tests

- `step_candidates_match_the_plain_step_and_rank_the_selection_first` — parity vs the plain step on every probe signature × both membership widths × penalty on/off; selection-first, strict canonical order, count bound.
- `sampled_decode_is_seeded_and_abstains_exactly_like_greedy` — OOD probe abstains with the same typed outcome (no guessed token under sampling); served run byte-reproducible from its rng seed; every emitted token stays in the fixture's emission universe; policy counters show one serve per emitted token.
- Full gates: workspace clippy `-D warnings` clean, fmt clean, status_policy 14 passed (incl. the integer-only source scan), census suite pass, api 27+19, certify 41.

## Refs

#655 (decode-default decision, 2026-08-19) · #762 / #785-C2 (sampling scheme provenance) · #784 (the attractor-basin substrate issue this mitigates at serving time)

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

