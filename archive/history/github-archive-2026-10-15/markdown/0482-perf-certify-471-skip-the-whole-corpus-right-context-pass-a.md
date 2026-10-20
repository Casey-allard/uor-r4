# #482 — perf(certify): #471 — skip the whole-corpus right-context pass a sampled run never reads

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-07T17:58:05Z
- **Updated:** 2026-08-07T18:27:04Z
- **Closed:** 2026-08-07T18:16:56Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/482
- **Merged:** 2026-08-07T18:16:56Z
- **Merge commit:** 3401c607ea1012e2c6558da6a883394f28bf6dbe
- **Base / Head:** main / UOR-Foundation:issue-471-gate-c-arm-skip
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Gate C now prints a **per-phase wall clock**, and the profile revised this issue's diagnosis before any code was written for it.

## The profile (500k fixture, 2 cores, `R4_GATE_C_SAMPLE=10000`)

| phase | seconds | share |
|---|---:|---:|
| scorer construction | 2.60 | 3.0% |
| forward-anchor table (#399 M2) | 0.08 | 0.1% |
| **right-context code pass (#446)** | **51.05** | **59.0%** |
| left code pass (#469 lever A, sidecar-served) | 0.09 | 0.1% |
| two-sided table build (#446 M1) | 0.27 | 0.3% |
| latent right-context tables (#446 M2) | 0.38 | 0.4% |
| unigram null (#390) | 0.02 | 0.0% |
| scoring 10,000 positions | 31.70 | 36.6% |
| reduction, rollups, replay probes | 0.38 | 0.4% |
| **total** | **86.56** | |

The issue named three things. **Two of the three are 0.8% combined** — the table builds are not the problem. All of it is `derive_right_codes`, and the left code pass is free because the store build already wrote the #469 sidecar earlier in the same process. There is no case for a `forward_anchor` group either (0.08s).

## The knob

`R4_GATE_C_SKIP_ARMS`, comma-separated **arm-group names**, unset behaviour-identical, unknown name **panics** (a typo that silently measured everything hands back an 85-minute run when you asked for a 30-second one; a typo that silently skipped nothing looks exactly like a knob that does not work — both are invisible).

One group: `right_context`, the #446 M1 + M2 families, because they are exactly the closure of the right-context code pass. Grouped by *what they cost*, not by issue number, so the knob does not become a list of ad-hoc flags as arms accumulate.

| | control | skipped |
|---|---:|---:|
| Gate C phase | 86.56s | **32.14s** (−62.9%) |
| whole `score` pipeline | 138.1s | 82.8s (−40.1%) |

Linear in `n` — 105.7 and 102.1 µs/record at 250k and 500k — so the saving grows with the corpus and shrinks with the sample. On a **census** run it is ~16%, not two thirds; this is a decision-run lever by construction.

**Not option (a).** A κ-keyed sidecar for the right codes misses on exactly the runs that need it: a decision run is one that changed the artifact or the corpus, which is what invalidates the key.

## Absence is not zero

The twenty-six #446 rows move into `gate_c.right_context_arms: Option<..>` (schema 25 → 26), `null` on a skipped run, `skipped_arm_groups` naming what was dropped, CLI printing `SKIPPED` in each row's own place. **The M2 exit-rule verdict is suppressed rather than printed as NOT MET** — a pre-declared exit rule reported as unmet is a result, and this one would never have run.

A flat skip would have published `rule12_latent_mix.top1_agreement: 0.0` across the arm set: the all-zero pattern this repo has learned to read as a harness bug.

## Verification

- **A schema-25 report from before this change and a schema-26 control report from after it agree on every value of every `gate_c` key**, key sets equal, nothing missing or added. The restructure is a move, demonstrated rather than argued.
- `tests/gate_c_arm_skip.rs` (ignored; spawns the shipped binary twice, no `unsafe` env mutation) compares the two runs key by key — 45 keys identical, plus compile inputs and emitted graph. It asserts the control arm is non-degenerate **before** comparing, so it cannot pass by comparing two empty runs.
- **That test earned its keep on its first run**: `win_loss.twosided_vs_rule12_live` and its shuffled twin were still flat and published all-zero cross-tabs beside five populated ones. The vacuous-instrument pattern, reproduced in a change written to avoid it, and caught by a machine.
- 5 env-parse units, `gate_c_trend.sh` green (`rule12 0.3655/8.3222, baseline 0.3917/8.4985`), workspace clippy `-D warnings`, fmt, claim wording, no_std ladder, wasm target check.

## What this does NOT explain

**The 85 minutes.** 102 µs/record extrapolates to ~4 minutes at 2.11M, and peak RSS differs by only 5.6% (1,382 → 1,305 MB), so the memory hypothesis is recorded as a measured negative. The residual is unexplained and named in the record, with the leading remaining candidate being that the 85 minutes was not all inside `evaluate_gate_c` — which nothing at the time could tell. That is why the phase timing ships permanently rather than as scaffolding: the next 2.11M run attributes its own wall clock.

Record: `docs/gate_c_arm_skip_471.md`

Closes #471


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

