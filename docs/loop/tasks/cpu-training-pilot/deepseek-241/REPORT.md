# REPORT: cpu-training-pilot, deepseek-241

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](./ORDER.md). Base: `973bc0750`.

**Forcing the role-correct value exposes the current model's main opening.** On
the sealed Claude-322 artifacts, ordinary generation produced **0/256**
complete replies under every seed and rendering. Forcing only the correct first
token produced **20-41/256**. Forcing the complete correct value produced
**200-250/256** and **75-122/128** complete-distinct pairs. Forcing the paired
other-relation or a shuffled value produced **0/256** and **0/128** in every
case. The dominant gap is selecting and copying the role-correct source value;
continuation supports most forced correct values, with a smaller residual
termination/continuation gap.

## What did not work, first

- **Free generation still completes nothing.** The one-user controls reproduce
  Claude-322 exactly: EOS stops are 109/256 and 86/256 at seeds 1 and 2,
  first-line asked values 14 and 2, and other-relation values 11 and 6. None is
  a complete accepted answer.
- **The correct first token is insufficient.** Forcing only that token reaches
  41 complete rows at seed 1 and 20-21 at seed 2. Most replies still finish with
  the wrong continuation, so an opening-only patch does not clear the bar.
- **Forced correct content is not a perfect oracle.** The model still misses
  26/256 and 56/256 correct-value rows on the one-user renderings and 6/256 and
  43/256 on per-turn. Some adds text before EOS; others hit the cap.
- **Wrong values terminate without becoming answers.** Other-relation values
  reach 215-254 EOS stops and shuffled values 211-252, but both have 0 complete
  and 0 complete pairs. Stopping is not correctness-aware.

## What changed

- Added `cpu-forced-copy-panel`, a CPU development-research instrument with five
  matched arms: free, correct-first-token, complete correct value, paired
  other-relation value and deterministic shuffled value.
- Reused the exact protocol-v2 panel rendering and nearest served head. The
  local stop contract uses token 1 as EOS, matching `served-reply-panel`; the
  first attempt incorrectly reused a legacy helper that stops on token 0 and is
  retained as an instrument fault.
- Reused one parsed CPU forward across 32 independent row workers. Each final
  full five-arm sweep takes about 2.3-2.7 seconds.
- Added fail-closed artifact/tokenizer/panel hashes, unique case and pair checks,
  exact frozen-answer scoring, exact per-stratum pair metrics, paired controls,
  CPU identity and executable SHA-256.
- Review repair separates the two matched-treatment comparisons from the two
  structural identity controls in the machine-readable report. Identity
  controls retain their descriptive counts but carry `passes: null`; they cannot
  be mistaken for inferential wins. Arm summaries now carry both `pairs: 128`
  and `complete_distinct_pairs`.
- The served-forward parity input is now retained as the exact 40-token panel
  slice, its case identity, derivation and source hash. Re-running both artifact
  audits from that file reproduced the previous parity report hashes exactly.

## Measurements

The selected artifacts are seed 1
`86e51d5ee2c00a4041bcde4d2183884e351099f28f155fb1e6a7516f13064dfe`
and seed 2
`8838cd1044ae7152934d8ec09016e268f2dabb0c702dd45e6ab06d7b5198bdae`.
The executable is
`a6cc69daad3f5dfd9af1ed64dd3a573a8086fa8a0b77d8d200e476c1795cc000`.

| Seed | Rendering | Free complete/distinct pairs | First-target complete/distinct pairs | Correct-value complete/distinct pairs | Other value complete | Shuffled value complete |
|---|---|---:|---:|---:|---:|---:|
| 1 | `one-user-turn/v1` | 0/0 | 41/12 | **230/102** | 0 | 0 |
| 1 | `per-turn/v1` | 0/0 | 41/12 | **250/122** | 0 | 0 |
| 2 | `one-user-turn/v1` | 0/0 | 20/0 | **200/75** | 0 | 0 |
| 2 | `per-turn/v1` | 0/0 | 21/0 | **213/86** | 0 | 0 |

Overall paired gains passed the `max(2, ceil(2*sqrt(discordant)))` threshold:

- first-target vs free: gains 41 and 20/21;
- correct-value vs first-target: gains 189, 209, 180 and 192;

The other-value and shuffled arms are identity controls, not statistical
evidence of a gain: their wrong prefix remains in every reply, and exact
accepted-answer scoring therefore prevents completion. Their result is that
215-254 and 211-252 respectively still reached EOS, showing termination without
the role-correct answer.

The per-stratum `first-target-vs-free` comparison passes `length2` and
`reassert`, but fails `length4`, `length8` and `update` at gain 0 versus
threshold 2 in every seed/rendering. The per-stratum `correct-value-vs-first`
comparison passes all 20 cells, with gains 16-64 against thresholds 8-16. These
values are recorded in [summary.json](./evidence/summary.json).

Correct-value completion by stratum, seed 1/seed 2:

| Rendering | `length2` | `length4` | `length8` | `reassert` | `update` |
|---|---:|---:|---:|---:|---:|
| `one-user-turn/v1` | 56/45 | 58/44 | 58/58 | 30/27 | 28/26 |
| `per-turn/v1` | 62/50 | 64/54 | 64/50 | 28/31 | 32/28 |

First-target completion by stratum, seed 1/seed 2:

| Rendering | `length2` | `length4` | `length8` | `reassert` | `update` |
|---|---:|---:|---:|---:|---:|
| `one-user-turn/v1` | 27/14 | 0/0 | 0/0 | 14/6 | 0/0 |
| `per-turn/v1` | 29/13 | 0/0 | 0/0 | 12/8 | 0/0 |

The seed-1 one-user repeat differs only in wall time and peak RSS; every
decision, reply, tally and paired count is identical.

The exact artifacts also passed a panel-token served-forward audit at 40
positions each: 10,240 Q12 coordinates, 40 token decisions and 320 kept cells
with zero query, token or kept-cell mismatches. The committed source is case
`fresh-diverse-length2-00-swap0-q0-forward-job`, token slice `[0:40]`,
`reply_start: 39`, with sequence hash
`c44886565e763cee62096af3c82e4e1427d7bd8fcd9f952969d25a230af47a7b`.
Audit report hashes are `11271dc5...` and `61aeab45...`.

The canonical D84 ledger is
[`resource-ledger.json`](../../../../labs/cpu-training-pilot-forced-copy-2026-11-08/resource-ledger.json),
with the detailed retained-run receipt at
[`evidence/resources.json`](./evidence/resources.json): 24 retained reports,
79.524 seconds measured probe time, 137,456 KiB maximum peak RSS,
570,541,430 bytes of retained evidence/inputs, 5,813,363,277 bytes of
regenerable target, and 144.406 GiB free. The ledger preserves the legacy
37,808.048816343-second projection chain and separately reconciles the
95,627.677487699-second latest recorded admitted cumulative, the 1,140-second
readout-refit projection, and this unit's 1,800-second increment. The admitted
cumulative is 98,567.677487699 seconds. The canonical shared ledger
`.uor-models/native-joint-learning-2026-09-04/model-time.json` was `ABSENT`.
Downloads, export, builds, tests and delivery are explicitly unmeasured.

## Checks run at the head

- `cargo test --offline --locked -p uor-r4-cpu-train --bin
  cpu-forced-copy-panel`: 13 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --lib`: 455 passed,
  4 ignored.
- `cargo test --release --offline --locked --features cpu-parity --bin
  served-reply-panel`: 15 passed.
- `cargo check --offline --locked -p uor-r4-cpu-train --bin
  cpu-forced-copy-panel`: passed.
- Targeted `rustfmt --check`, driver Clippy with no changed-code warnings,
  `git diff --check`, report check, local PR policy and full loop gates pass.
- Five v5 reports from the exact reviewed source cover two seeds, both
  renderings and the exact-schema repeat. All 256 rows and 128 pairs are present
  in every arm. Inputs, settings, arm rows, replies, tallies and decisions match
  v4 and v3; v5 changes only executable identity, timing, RSS and the repaired
  paired-comparison metadata.

## Skills used

No system-provided Codex skill applied. The repository's geometry, D84 CPU
research and loop-delivery rules were used directly.

## Still open

- Implement a lawful role-conditioned value reader/pointer. Correct-value
  forcing shows a large capability ceiling, but it does not provide the
  mechanism that finds that value without a label.
- Preserve continuation after the located value. The residual 6-56 failures
  show that opening/copy alone is not sufficient.
- Do not tune on these forced prefixes or merge them into serving. The panel is
  now consumed and this unit is an attribution diagnostic only.
- Claude-325 and Claude-323 remain the main training-track experiments. This
  branch neither waits for nor modifies them.
- Fresh exact-head review, protected queue and fresh-main verification remain.

## Plain summary

The model already has most of the machinery needed to finish an answer after it
is handed the right value: 200 to 250 of 256 replies become complete. Handing it
only the first correct token is not enough, and handing it the other relation's
value or a shuffled value never counts. So the next useful mechanism is not
another global point table or more stop tuning. It is a lawful way to choose the
role-correct value from the prompt and copy it into the reply, followed by the
existing geometric continuation and termination path.
