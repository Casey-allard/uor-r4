# #381 — Repetition penalty costs ~10pp top-1 on windowed Gate C — measure it on natural distributions and sweep the magnitude (#364 follow-up c)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-03T16:27:38Z
- **Updated:** 2026-08-04T05:49:27Z
- **Closed:** 2026-08-04T05:49:27Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/381
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up (c) from the #364 campaign synthesis (results: #374). Measured with the Gate C window mode (#375/#378): enabling real recent-token windows WITHOUT rows drops rule12 from 36.47% to **26.69% top-1** (+10.3 bits) on the fixture distribution — the −2,000,000 recent-token repetition penalty is the mechanism. Serving always pays this penalty on real windows.

Open question, needs measurement not assumption: is the penalty a net cost on natural serving distributions too, or is the fixture (teacher-forced stories with benign repetition) unrepresentative? Scope: (1) quantify on the D3-style natural corpus; (2) sweep the penalty magnitude (0, −2M, intermediate) at fixed everything-else; (3) decide whether the penalty should be evidence-based (e.g. suppressed when the repeated token is corpus-frequent after this context) rather than flat. The flat −2M was an anti-degeneracy guard, not a measured operating point.


## Comments (2)

### comment 5175131015 — 2026-08-04T05:49:25Z — Casey-allard

## Both-distributions answer: the fixture's ~10pp penalty cost was unrepresentative — on natural text it costs 0.8pp

Natural corpus: `simple-wiki-20231101` full 3,000-article observation pass (this run's obs corpus, merged κ `blake3:998f93bc…`; 533,832 construction / 133,239 held-out positions, D3 article-hash split). Pipeline compiled ON the natural corpus per arm (window-on rows-off: `--gate-c-context-window on --context-order 0 --quality-profile relative_tla`), penalty swept 0 / −1M / −2M:

| arm | Rule 1 (graph chain) | Rule 1+2 (chain+EXCT) | bits (1+2) |
|---|---|---|---|
| penalty 0 | 26.9% / 21.35 | **36.5%** | **19.239** |
| penalty −1M | 26.9% / 21.83 | 35.7% | 19.689 |
| penalty −2M | 26.9% / 22.37 | 35.7% | 20.193 |
| TLA3 store baseline | — | 36.3% | 32.140 |

Fixture reference (#385 record, teacher-forced stories): window-on rows-off 26.69% at −2M vs 36.47% window-off — **~10pp cost**.

Findings:

1. **The fixture was unrepresentative for this question, in the direction suspected when this issue was filed.** On natural text the −2M penalty costs **0.8pp top-1 and ~0.95 bits/token** — real, monotone in magnitude (bits 19.24 → 19.69 → 20.19), but an order of magnitude smaller than the fixture's 10pp. Teacher-forced stories carry benign repetition that the flat penalty punishes; wiki text mostly doesn't.
2. **The penalty remains a pure cost at every measured magnitude on both distributions** — no arm anywhere shows a benefit. The original design intent (anti-degeneracy guard for free-running generation) is not exercised by these teacher-forced measurements; that caveat stands.
3. Recommendation stays as filed, with lowered urgency: an evidence-based penalty (suppressed when the repeated token is corpus-frequent after this context) dominates a flat −2M, but on natural serving distributions the flat guard costs little. Magnitude 0 vs −2M is now a measured 0.8pp/0.95-bit decision, not an assumed one.

Closing per batch-flow: question ("net cost on natural distributions too, or fixture unrepresentative?") answered with the sweep; artifacts and reports under /tmp/nat_pen_* in the session container, headline rows recorded above. Follow-on (evidence-based penalty design) belongs to a new issue if/when serving priorities call for it.


### comment 5175131115 — 2026-08-04T05:49:26Z — Casey-allard

Closing with the both-distributions record above: fixture ~10pp penalty cost does not transfer — natural-text cost is 0.8pp top-1 / ~0.95 bits at −2M, monotone in magnitude, still pure cost. Evidence-based penalty design deferred to a future issue.


---
_Comment/review ids in this file: 5175131015, 5175131115_

