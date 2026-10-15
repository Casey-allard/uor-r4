# #816 — cli: D4 abstention gate on the ask/chat graph path (#811)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-19T02:13:04Z
- **Updated:** 2026-10-09T13:08:54Z
- **Closed:** 2026-08-19T02:34:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/816
- **Merged:** 2026-08-19T02:34:14Z
- **Merge commit:** cc7537e7a58c57da9d07f93f8581b6bc771bd125
- **Base / Head:** main / UOR-Foundation:issue-811-ask-abstention
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Closes #811 per the maintainer decision (2026-08-19, #655 issuecomment-5336515423): the CLI ask/chat graph path now applies the **same deployed D4 policy** the server tier applies — one implementation (`R4Engine`'s serve / widen-once / abstain path, novel-seen FIFO included), never a re-derivation.

- **ChatEngine** carries the deployed policy engine over the same bundle bytes the walk decodes from (both build paths; `graph/score_report.json` status-policy override honored exactly like the server-side loader). **Per-step D4 gate on the default sampled path** — the server contract exactly: on an abstention, partial tokens are dropped and no token is guessed. **Question-boundary gate on the opt-in greedy beam** (speculative per-hypothesis windows would pollute widen-once bookkeeping; recorded limitation). `ChatAnswer` gains a typed `abstention`; `ask`/`chat` render it as an explicit honest line.
- **Served answers are byte-identical to the ungated walk by construction** — the gate only decides *whether* a step serves, never *what* it serves.
- **`R4Engine::load_accepting_quality`** (api): `load` minus the serving-admission quality gate *only* — the bundle's recorded quality verdict is a serving-admission concern, while the D4 policy is well-defined regardless; the CLI has already decided (with its own recorded warning) to decode local bundles. Serving admission continues to use `load()` unchanged. The chat layer also probes `GraphScorer::from_artifact` first, so legacy/converter-era graphs get a documented gate-absent warn instead of the R5/#510 self-produced-defect panic — a latent crash this work surfaced (a structurally-parseable HEAD-only graph panicked `load`).
- `engine_from_bytes*` (the #750 live-quality-gate constructor) stays deliberately ungated so that gate keeps measuring the raw decode.

## Measured on smollm2-360m-broad (release build)

| check | result |
|---|---|
| in-domain default output, gated vs the archived decode-flip canary | **byte-identical, 0 diffs** — the gate changes nothing on served prompts |
| the five F-p2 probes | policy engine loads, gate runs, **D4 itself resolves every probe window as servable — zero abstentions** |

The #811 **wiring gap is closed**: probe-serving is now attributable to the substrate's novelty resolution (a #784-class property, identical on any bundle the server admits), not to missing CLI wiring. That substrate question is recorded on the issue close.

## Tests

End-to-end over a real scored two-region fixture through the real chat tokenizer: a Novel question abstains typed on both decode paths (no token served, correct witness surface); a covered question serves byte-identically to the ungated engine; no-policy-engine keeps pre-#811 behavior. Full gates: workspace clippy `-D warnings` clean, fmt clean, chat 14, status_policy 14, census, api 27+19, bin 7.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

