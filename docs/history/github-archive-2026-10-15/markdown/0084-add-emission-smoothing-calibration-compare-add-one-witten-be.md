# #84 — Add emission smoothing calibration: compare add-one, Witten-Bell, and absolute-discounting on held-out D3 partition

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-23T01:12:08Z
- **Updated:** 2026-10-09T14:17:23Z
- **Closed:** 2026-07-23T03:33:47Z
- **Assignees:** auser, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/84
- **Merged:** not merged
- **Base / Head:** main / UOR-Foundation:copilot/uor-foundation-add-one-vs-smoothing-comparison
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Emission tables were always compiled with add-one smoothing (`P(v|n) = (count+1)/(total+V)`) without empirical justification. This adds a calibration step that compares all three smoothing families on held-out bits/token and top-1 agreement, records the winner in `score_report.json`, and uses the chosen rule for compilation.

## New infrastructure (`score.rs`)

- **`EmissionSmoothingRule` enum** — `AddOne`, `WittenBell`, `AbsoluteDiscounting(f64)` with a `label()` method for deterministic report strings
- **Smoothing probability helpers** — `add_one_prob`, `witten_bell_emission_prob`, `absolute_discounting_emission_prob`; all back off to the add-one-smoothed root prior
- **`evaluate_smoothing_rule`** — evaluates one rule on the held-out set given precomputed region evidence; returns `(bits_per_token, top1_agreement)`
- **`calibrate_emission_smoothing`** — public API; builds evidence with the same semantics as `compile_emissions` (top-3 teacher-weighted counts over covered binary top-1 members), evaluates add-one + Witten-Bell + absolute-discounting at δ ∈ {0.1, 0.2, …, 1.0}, sorts candidates by bits/token, and returns `EmissionSmoothingCalibration`
- **`SmoothingCandidate` / `EmissionSmoothingCalibration`** — serializable structs embedded in `score_report.json` under `smoothing_calibration`; record all losers' numbers alongside the chosen rule and margin

## Schema changes

- `ScoreConfig` gains `emission_smoothing_rule: String` — set by the calibration result before `compile_emissions` runs
- `ScoreReport` bumped to **schema 3**; adds `smoothing_calibration: EmissionSmoothingCalibration`
- `ScoreReportConfig` exposes `emission_smoothing_rule` so the report is self-describing

## Pipeline (`command.rs`)

`score_command` runs `calibrate_emission_smoothing` immediately after evidence is available (before `compile_emissions`), injects the winning rule into `ScoreConfig`, and passes the calibration struct through to `build_score_report`.

## Determinism

Evidence computation is identical to `compile_emissions` (BTreeMap accumulations, content-addressed observation order); candidate sort is stable (bits/token ascending, then label). Two runs on identical inputs produce byte-identical calibration JSON.

<!-- START COPILOT CODING AGENT SUFFIX -->

- Fixes #67

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5054006261 — 2026-07-23T03:33:47Z — auser

Superseded by 139209c (already in main): the five-variant smoothing sweep ran (add-one 9.8612 vs WB 14.18 / abs-disc 15.83/14.68/14.20), add-one kept as default with byte-exact preservation proof, and the Smoothing enum + CLI flag landed. Everything here duplicates merged work.


---
_Comment/review ids in this file: 5054006261_

