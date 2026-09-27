# Selection-policy diagnostic plan (September 27, 2026)

**Status boundary.** This plan executes the supported next work card recorded by
the [emission/selection diagnostic result](emission-selection-diagnostic-result-2026-09-27.md):
a **same-checkpoint selection-policy diagnostic** — a deterministic (greedy)
replay of the frozen packet at the same step-15,672 checkpoints. It is read-only:
no weight change, no serving-decoding change, no new panel or threshold. It
cannot overturn the frozen prose verdict and promotes nothing. The deterministic
condition is a measurement, not a candidate: a better-ranked token is not a
coherent alternate trajectory until actual generated text says so.

## Question

With the draw removed (argmax selection) at the same checkpoints and prompts:

1. Do the witnessed malformed free-prose clauses persist?
2. What exactly do the retained **greedy source-panel regressions** fail to
   explain? The source panel is the 32 greedy source-edit completions per arm
   (`run_story_probes`; first-sentence stop, 32-token cap). At these checkpoints
   complete correct rows are retained at **21/32 (quaternion)** and
   **23/32 (householder pair)**, with retained lost/gained rows versus the
   accepted parent (`5/2` and `7/4` in the
   [continuation source comparison](../evidence/language-continuation-source-comparisons-2026-09-26.json)).
   These were produced without sampling, so they cannot be selection artifacts.

## Fixed inputs

| Quantity | Value |
|---|---|
| Checkpoints | `fit-quaternion-6/checkpoint-final`, `fit-householder_pair-6/checkpoint-final` (step 15,672) |
| Campaigns / evaluator | `campaigns/{quaternion,householder_pair}-resume-6.json`; `docs/integration/reference-evaluator-v2.json` (5 frozen prompts, seeds 2014–2018 for the sampled comparison only) |
| Retained comparison roots | `evaluate-{quaternion,householder_pair}-continuous-read-6/` (`story-probes.json`, `generations.json`) |
| Greedy policy | `seed=None` argmax selection, Read mode, same session semantics; free prose 128-token horizon; story probes unchanged (32 max new tokens, first-sentence stop) |
| Sampled reference | Retained `generations.json` at the same checkpoint (for divergence reporting only) |

## Instrument (additive)

```text
uor-r4-training joint-selection-replay CAMPAIGN_JSON SEALED_CHECKPOINT RETAINED_EVALUATE_ROOT NEW_REPORT_ROOT cpu
```

- Replays the unchanged `run_story_probes` (greedy) and requires **exact parity**
  against the retained `story-probes.json`: per pair and side, `first_noun_correct`,
  `complete_correct`, `pair_complete_correct`, `outputs_differ`, response text,
  generated ids, stop, and per-decision selected/greedy tokens, probabilities and
  hashes. Any mismatch reports `PARITY_FAILED`; the instrument is invalid.
- Runs the five frozen prompts under greedy and records full texts, stop reasons
  and the first divergence (index, greedy token, sampled token) versus the
  retained sampled trajectories.
- Writes `selection-replay.json` and `parity.json` in a claimed/sealed root.
  Existing commands and behavior are unchanged.

## Predeclared decision rule

| Greedy free-prose outcome (both arms, principal read against the same four criteria) | Decision |
|---|---|
| Malformed witnessed clauses persist or ≥4/5 remain unacceptable | Selection policy cannot produce acceptable prose here; the next work card targets **ranking/emission** (the failing source rows are the concrete target). |
| Broadly acceptable (≥3/5 in both arms, no witnessed malformed clause) | Record a scored positive for deterministic selection on this exposed packet only; the next card is a separately authorized bounded selection-policy experiment on fresh contexts. The source panel still bounds the claim. |
| Mixed | Report the exact split per arm; select the next card by that split; the source panel remains a ranking-side obligation. |

Regardless of branch: the replayed source-panel failures (21/32, 23/32 and the
retained lost rows) are reported exactly and marked not selection-explainable.
No branch changes weights, serving decoding, gates or candidates.

## Cost (prospective)

| Item | Projection |
|---|---|
| Worktree | fresh `~/uor-r4-worktrees/selection-policy-diagnostic-20260927` (created; ~0.4 GiB internal) |
| Build | reuse SSD release cache `/Volumes/UOR-Workspace/BuildCaches/uor-r4-emission-20260926-1`; incremental ≤5 min CPU; no new large cache; one cargo process at a time (fourth-lab slot etiquette) |
| Execution | two CPU runs, one model process at a time; ≤2 min model wall total |
| New storage | ≤10 MiB report roots; ≤100 KiB committed evidence |
| Ledger | actual complete elapsed charged once (<0.3% of the current limit) |

No fit, no external compute, no deletion; the 128 MiB stop margin is untouched.

## Stop condition

Finish with one supported implementation decision or an explicit `UNRESOLVED`
finding, delivered through a protected PR referencing #973/#820, with the run
handoff updated.

## Corrigendum (2026-09-27, after execution; predeclared rule unchanged)

The Question section cites `5/2` and `7/4` as the continuous source panel's
lost/gained versus the accepted parent. That attribution is wrong: those are the
new F32-hard artifact's complete-row figures. The step-15,672 continuous greedy
panel is 21/32 (quaternion) and 23/32 (householder pair) complete, with
lost/gained **9/2** and **6/5** versus the accepted September25 parent
(first-noun `5/2` and `1/2`), as verified independently in the
[result](selection-policy-diagnostic-result-2026-09-27.md) and the retained
[row adjudication](../evidence/language-continuation-source-comparisons-2026-09-26.json).
The predeclared question, classification rule and cost are unchanged.
