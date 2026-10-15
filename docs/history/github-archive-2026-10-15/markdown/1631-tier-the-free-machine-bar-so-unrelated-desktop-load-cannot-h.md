# #1631 — Tier the free-machine bar so unrelated desktop load cannot hold the run

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T17:46:56Z
- **Updated:** 2026-10-02T17:48:13Z
- **Closed:** 2026-10-02T17:48:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1631
- **Merged:** 2026-10-02T17:48:13Z
- **Merge commit:** 521a0eff62f72d2fee0ab09ee6e4f7c667ae3617
- **Base / Head:** main / UOR-Foundation:codex/post-epoch-eval-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why

The waiter required `load<4` for its entire life. That **conflates two different things, and one of them cannot cost anything**:

- **a peer trainer competing for the same cores** — this is what I measured at **0.87 cores and 43.4 h instead of 4.7 h**, so it must always disqualify;
- **unrelated desktop load** — a browser and the app harness routinely hold this machine at **load 10–30**, which has nothing to do with the queued run.

As written, the second could **hold the run indefinitely while the first was absent** — defeating an objective whose entire point is reducing wall time. I observed exactly that: `others=2, load=13–31` driven substantially by a browser, with my run gated on a number the browser controls.

## Change

**`others=0` stays absolute at any load.** The load bar relaxes after `STRICT_POLLS` (default 48 ≈ 2 h) from **4 → 8**.

The relaxed bar still excludes a busy machine: with **8 cores, load 8 means every core already has a runnable task**. The strict window is kept first so a genuinely idle machine is still preferred when one is available.

## Exact-head evidence

**Reviewed head:** `ccc7a7a1`. `bash -n` clean. The running waiter executes this file, so the change takes effect on the queued launch with no further action.

`test-monitor-logic.sh` executed — **10 cases**, all four combinations for each tier:

| condition | result | expected |
|---|---|---|
| others=2, load=10.85 | wait | ✓ |
| others=2, **load=2.0** | wait | ✓ (a peer job disqualifies at any load) |
| others=0, load=12.0 | wait | ✓ |
| others=0, **load=1.5** | **FIRE** | ✓ (strict tier) |
| others=0, **load=3.99** | **FIRE** | ✓ (strict boundary) |
| others=0, load=4.01 | wait | ✓ |
| poll 10, **load=5.0** | wait | ✓ (still strict) |
| poll 49, **load=5.0** | **FIRE** | ✓ (relaxed tier) |
| poll 100, load=9.0 | wait | ✓ |
| poll 100, **load=7.9** | **FIRE** | ✓ |

Live state through the same expression: `others=2 load=18.27 → wait`.

## Scope

Orchestration only; **no model behaviour change, no capability claim.** The run this gates will be reported separately, labelled **geometry-on-principle, not a demonstrated advantage**.

References #1512, #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

