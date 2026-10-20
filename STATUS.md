# Status

**Pre-alpha. 0 of 8 milestones are complete, and no model built under the project's rules holds a
useful conversation.** This page is a pointer: the measured detail, with a source for every number,
is in [where we are](docs/start/where-we-are.md), and the live one-page plan is
[NOW.md](docs/integration/NOW.md), which wins if the two disagree.

![Bar chart of correct first guesses out of 14,287 test positions: trained model seed 1 at 65, seed 2 at 463, unigram table at 1,226, next-word table at 6,300, with a dashed line at 6,300 marking the new M1 bar.](docs/figures/start/status-vs-baselines.svg)

*The best trained model against two word-count tables (measurements from the
[K16 outcome](docs/labs/k16-outcome-call-2026-10-19/README.md) and the
[count tables](archive/labs/host-diagnostics-2026-10-17/README.md)).*

| Milestone | Issue | Status | Where it stands |
|---|---|---|---|
| M1 Language base | [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) | in progress; nothing passes | Best top-1 65 and 463 of 14,287 (top-1 = first guesses that are right), against the bar of beating 1,226 and 6,300 on each of two seeds through the integer export |
| M2 Grounded reply from exact memory | [#2030](https://github.com/UOR-Foundation/uor-r4/issues/2030) | held | 437/512 saved development replies, 0/128 on a transfer check; a genuinely fresh test is NOT_RUN |
| M3 Durable conversation memory | [#2031](https://github.com/UOR-Foundation/uor-r4/issues/2031) | paused | Evaluator and session built, not qualified; waits on an M1 or M2 model |
| M4 One served model (integer export and CLI) | [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032) | held | Integer serving engine built. No model trained under today's rules is exported to it; older float stacks were served in October and missed the old 0.90 bits-per-byte bar |
| M5 Laptop cost | [#2033](https://github.com/UOR-Foundation/uor-r4/issues/2033) | not started | Energy, memory and tokens per second on an M1 laptop: not measured |
| M6 Reasoning and coding | [#2034](https://github.com/UOR-Foundation/uor-r4/issues/2034) | not started | Exact arithmetic is the visible gap |
| M7 API, browser and Studio | [#2035](https://github.com/UOR-Foundation/uor-r4/issues/2035) | engine delivered; no model to load | Browser engine landed in [#2654](https://github.com/UOR-Foundation/uor-r4/pull/2654); not yet connected to the Studio |
| M8 Alpha release | [#2036](https://github.com/UOR-Foundation/uor-r4/issues/2036) | not started | Owner decision |

Training is held: each GPU run needs a reviewed one-page plan and a release typed by the owner
([charter](CHARTER.md) §5.1). Milestone bars and the road to them: [ROADMAP](ROADMAP.md) and
[where we're going](docs/start/where-were-going.md). Live tracker:
[#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028). Per-line results:
[research notebooks](docs/research/notebook/README.md).

This page changes only when a milestone's state changes; it is not a log.
