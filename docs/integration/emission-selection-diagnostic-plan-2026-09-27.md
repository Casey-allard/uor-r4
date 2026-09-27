# Same-checkpoint emission/selection diagnostic plan (September 27, 2026)

**Status boundary.** This plan executes the
[recommended single work card](current-state.md#next-single-work-card--failure-localization)
after the completed full-context language continuation. It is **read-only**: the
same sealed step-15,672 checkpoints, the same frozen prompts, seeds, sampling
policy and mode; no weight change, no decoding change, no fit, no new acceptance
panel, no threshold change. It states the decision the observation can change and
the complete prospective cost before execution, as the work card requires. It
cannot overturn the [frozen prose verdict](language-continuation-result-2026-09-26.md)
or promote any failed candidate.

## Question and witnesses

At the witnessed malformed decisions, distinguish a **poorly ranked semantic
token** (the model's own top-ranked candidate is the malformed token, so the
ranking itself is at fault) from a **stochastic departure** (the draw selected a
low-probability token while a better-ranked alternative existed). Secondary
attribution: whether the selected token's mass came from the **vocabulary**
component or the **copy** component of the output mixture.

Retained witnesses (continuous Read lane, both arms, story seed 2014):

| Arm | Checkpoint | Decisions | Witnessed clause |
|---|---|---|---|
| Quaternion | `fit-quaternion-6/checkpoint-final` (15,672) | 37–42 | "a very big, grey man had an idea **called out in.**" (decision 39 selects token 1028 " called", raw model mass 0.0017458474, greedy token 16 ".") |
| Householder pair | `fit-householder_pair-6/checkpoint-final` (15,672) | 25–30 | "He wanted the most **so exciting!**" (decision 27 selects token 1996 with copy gate 0.908; decision 29 selects 2935 with greedy 69) |

Fixed inputs (all already retained):

- Campaigns: `campaigns/{quaternion,householder_pair}-resume-6.json`.
- Evaluator: `docs/integration/reference-evaluator-v2.json`; its `prompt_source`
  (`prompts.json`, sha256 `e9b1f0e387fb770d0e4e137de67899d6569d03347856442ed6eeae3eaa8f0387`),
  5 prompts, seeds 2014–2018, horizon 128, mode Read, incremental batch 1.
- Sampling policy: `joint-probability-top-k-q32-splitmix64/1;score=ln(f64(probability));temperature=0.8;top-k=40;rank=probability-desc-token-asc`.
- Retained comparison records:
  `evaluate-{quaternion,householder_pair}-continuous-read-6/generations.json`.

## Instrument (additive)

New subcommand `joint-emission-trace` in `crates/uor-r4-training`:

```text
uor-r4-training joint-emission-trace CAMPAIGN_JSON SEALED_CHECKPOINT RETAINED_GENERATIONS_JSON NEW_REPORT_ROOT cpu {read|no-read}
```

It replays each of the five prompts with the same checkpoint, session stepping,
seeds and sampler, and records per-decision evidence the retained packet does not
contain. Existing commands and their numerical behavior are unchanged; the trace
is a separate function used only by this command. The report root is claimed
exclusively before model load and sealed/verified on completion, like the other
joint commands.

**Captured per decision.** Tier 1 (all decisions, all five stories): selected
and greedy tokens with decoded text; raw model probability of each; full-row
rank of the selected token; the exact top-10 Q32 sampling table (token, raw
probability, integer weight, sampling probability, cumulative bound); draw
threshold and fraction; sampling probability and rank of the selected token;
ratio selected/best; vocabulary- mass, copy-mass and contribution split for the
selected and greedy tokens; copy contributors; mixture reconstruction error.
Tier 2 (additionally for story seed 2014, covering both witness ranges): the
full 40-entry sampling table and the full source-weight row (occurrence, token,
mass) with the top copy contributors per reported token.

**Parity gate.** Every replayed story and decision is compared against the
retained `generations.json`: prompt, seed, generated token ids, stop reason, and
per-decision `selected_token`, `greedy_token`, `selected_probability`,
`selected_model_nll_nats`, `probability_sum`, `probabilities_sha256_le_f32`,
`no_read_mass`, `copy_gate`, `effective_copy_mass`, `top_read_*` and sampler
states must match exactly. Any mismatch reports `PARITY_FAILED`; the instrument
is then invalid and no localization claim is made.

## Predeclared classification rule

For each decision let `r` be the selected token's rank in the Q32 sampling order
(0 = best) and `q` the sampling probability (`integer weight / total`).

- `top_choice`: `r == 0` — the malformed token was the model's own best
  candidate; no sampling alternative existed.
- `near_tie`: `r >= 1` and `q_selected >= 0.5 * q_rank0` — a stochastic outcome
  between comparable alternatives.
- `departure`: `r >= 1` and `q_selected < 0.5 * q_rank0` — the draw departed
  from a materially better-ranked alternative.

Secondary tag: `copy_dominated` when the copy contribution exceeds the
vocabulary contribution of the selected token's final mass. Counts are reported
for the witness ranges and for the whole seed-2014 story; raw percentages
accompany every class. The principal separately records the decoded semantic
reading of the selected and best-ranked candidates; mechanical classes and the
semantic read are reported as distinct facts.

## Decision this observation can change

| Observed pattern over the witness ranges (both arms) | Interface supported | What changes |
|---|---|---|
| Predominantly `top_choice` | emission/ranking | A sampling change cannot repair these decisions; the next work card targets the model's ranked predictions, consistent with the retained greedy source-panel regressions. |
| Predominantly `departure` / `near_tie` | selection policy | A separately authorized bounded selection experiment (no weight change) becomes the supported interface; it must still explain the source-panel regressions. |
| Selected-token mass `copy_dominated` | read/copy attribution | The next card targets the copy/read path rather than the vocabulary head. |
| Mixed | dominant interface only | Report exact counts; mark the remainder `UNRESOLVED`; select by the majority pattern. |

It does not change: the frozen prose verdict, gates, accepted parents, or
authorize decoding/weight changes, a replay campaign or an alternate rollout.

## Complete prospective cost

| Item | Projection |
|---|---|
| Implementation + predeclaration | ≤75 min orchestration wall; no model compute |
| Release build (`uor-r4-training`, features `metal,cpu-accelerate,reference-accelerate`) | ≤10 min CPU; ≤8 GiB new SSD target (`/Volumes/UOR-Workspace/BuildCaches/uor-r4-emission-20260926-1`); no internal-disk target |
| Execution | Two CPU runs (one per arm), one model process at a time; ≤5 min model wall total |
| New retained storage | ≤20 MiB report roots in the existing model store; ≤100 KiB committed evidence |
| Ledger | Actual elapsed charged to the shared cumulative model-time ledger (<0.2% of the current limit) |

No external or paid compute; no deletion of unique material; no stop-margin
reduction.

## Stop condition

Finish with one supported implementation decision or an explicit `UNRESOLVED`
finding. No further trace, fit, panel or sweep follows from this plan. Results
and the exact evidence paths are recorded in a companion result document and
delivered through a protected PR referencing #973 and #820.
