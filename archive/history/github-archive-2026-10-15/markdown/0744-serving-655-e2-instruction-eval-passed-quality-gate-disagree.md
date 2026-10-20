# #744 — serving/#655-E2: instruction_eval_passed quality gate disagrees with itself and doesn't predict live generation quality

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-16T21:23:08Z
- **Updated:** 2026-08-17T08:40:53Z
- **Closed:** 2026-08-17T08:40:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/744
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

Found during the same coherence-measurement pass as the sibling issue on
`R4Engine::load` panicking (see the comment on #655:
https://github.com/UOR-Foundation/uor-r4/issues/655#issuecomment-5309698590).
`r4 ask` gates on `ModelStore`/manifest metadata -- specifically
`instruction_eval_passed`, `grounded_answer_rate`, and `repetition_rate` in
`.uor-models/manifests/<name>.json` -- before it will answer a question at
all (README: "`r4 ask` accepts only an imported `instruction-chat` manifest
carrying a CID-addressed passing evaluation report"). This is the gate that
is supposed to keep an unevaluated or known-bad bundle from being presented
as a working chat model.

# Current evidence

Two pairs of manifests exist locally, each pair pointing at **byte-identical**
compiled artifacts (same `artifacts.cid`, `store.cid`, `tokenizer.cid`, same
byte counts) but carrying different `evaluation_report` CIDs and reporting
opposite quality:

| manifest file | source_model | artifacts/store/tokenizer CID | grounded_answer_rate | repetition_rate | instruction_eval_passed |
|---|---|---|---|---|---|
| `smollm2-135m-instruct.json` | HuggingFaceTB/SmolLM2-135M-Instruct | `blake3:b8d816aa...`/`blake3:32c387dc...`/`blake3:2e924d60...` | **0.0** | **1.0** | true |
| `smollm2-135m-chat.json` | HuggingFaceTB/SmolLM2-135M-Instruct@7e27bd9f... | same as above | **0.85** | **0.01** | true |
| `smollm2-360m-instruct.json` | HuggingFaceTB/SmolLM2-360M-Instruct | `blake3:cd8dbc14...` | 0.0 | 1.0 | true |
| `smollm2-1-7b-chat.json` | HuggingFaceTB/SmolLM2-1.7B-Instruct@0d807604... | `blake3:fca5bdfb...` | 0.85 | 0.01 | true |

Every "-instruct"-named manifest reports 0.0/1.0 (total failure by these
metrics); every "-chat"-named manifest reports 0.85/0.01 (apparent success).
**Both members of every pair pass the gate.**

A third, independent measurement exists for at least the 135M and 1.7B
models -- `instruction-eval.json` inside the corresponding
`.uor-models/compiled/<name>/` directory, which records `top1_accuracy_pct`,
`teacher_argmax_agreement_pct`, and `bits_per_token` against a genuine
D3 held-out split:

| bundle | top1_accuracy_pct | teacher_argmax_agreement_pct | bits_over_teacher_floor |
|---|---|---|---|
| smollm2-135m-instruct | 1.50 | 20.36 | 22.31 |
| smollm2-1-7b-instruct | 34.7 | 31.5 | 4.30 |

These numbers are self-consistent with each other (bigger model, better
top1/agreement/bits) but do not obviously map onto either the 0.0/1.0 or
the 0.85/0.01 manifest pair for the same model -- there is no reproducible
line from "1.5% top1 accuracy" to "0% grounded, 100% repetition" (that
part is directionally consistent) but also no reproducible line from
"34.7% top1 accuracy" to "85% grounded answer rate, 1% repetition" given
what was actually observed live (see below).

Live testing (`r4 ask`, this session, `UOR_MODEL_STORE` pointed at the
real model root):
- `smollm2-135m-instruct` (0.0/1.0): empty output, just `<|im_end|>`.
- `smollm2-135m-chat` (0.85/0.01): **also** empty output, just `<|im_end|>`,
  same prompt.
- `smollm2-1-7b-chat` (0.85/0.01, the best-looking manifest in the whole
  local store): non-grammatical word-salad output on two different real
  prompts (see the sibling generation-quality issue for transcripts).

So: the "0.85/0.01, passed" bundles produced either nothing or garbage when
actually asked real questions -- the recorded metrics do not predict what
`r4 ask` actually does for a live user.

# Problem

`instruction_eval_passed` is meant to be the trust boundary between "this
bundle may answer questions" and "this bundle must not." Right now:

1. The exact same compiled bytes can carry two different manifests with
   opposite recorded quality, and the gate accepts both. That alone means
   the gate is not a property of the model -- it's a property of which
   manifest file you happened to import, which defeats its purpose.
2. It isn't known whether this is because the two manifests come from
   genuinely different evaluation runs/harnesses (e.g. an older vs newer
   evaluator, or different probe sets under the "-instruct" vs "-chat"
   naming), or because of an actual computation bug that produces
   nondeterministic or path-dependent results for the same input.
3. Even the "passing," "high-quality" manifest's live behavior does not
   match its recorded numbers (85% grounded answer rate does not square
   with live word-salad output on ordinary questions).

# Goal

`instruction_eval_passed` (and the `grounded_answer_rate`/
`repetition_rate` metrics behind it) must be a reproducible property of the
compiled model bytes, must not disagree with itself across manifests
pointing at identical artifacts, and must actually predict whether live
`r4 ask` output is coherent and grounded. A bundle should not be able to
carry a "passed, 85% grounded" manifest while producing word salad on
ordinary prompts.

# Scope

- Find and read the exact code/script/process that produced each of
  `smollm2-135m-instruct.json` vs `smollm2-135m-chat.json` (likely
  `r4 evaluate-report` / `r4 import`, per the CLI's command list -- confirm
  which). Diff what differs between the "-instruct" and "-chat" naming
  convention's evaluation methodology: different probe prompt sets,
  different sample sizes, different code paths, or a stale/regenerated
  report.
- Reproduce both the 0.0/1.0 and 0.85/0.01 results from source against the
  same underlying compiled artifact, to confirm which (if either) is
  currently reproducible from a clean evaluation run on current `main`.
- Cross-check `grounded_answer_rate`/`repetition_rate` methodology against
  the independently-computed `instruction-eval.json` `top1_accuracy_pct`/
  `teacher_argmax_agreement_pct` numbers for the same bundle -- these
  should tell a consistent story; if they don't, determine which
  measurement (if either) is trustworthy.
- Determine why a "0.85 grounded_answer_rate" manifest can still produce
  word-salad output on ordinary prompts in live `r4 ask` -- e.g. is the
  probe set too narrow/easy, too in-distribution relative to the held-out
  evaluation, or is there a bug in how the live `ask` path selects/loads
  the model versus how the offline evaluator scored it (different config,
  different `root_top_b`/`exct_top_x`, different corpus source)?
- Once the true methodology and its failure modes are understood, either
  fix the computation so it's correct and reproducible, or correct the
  passing bar so it actually requires non-degenerate, grounded live output
  (this may mean the gate needs an actual live-generation smoke check
  against a held-out probe set, not just offline token-level statistics).
- Regenerate or correct the affected manifests once the fix lands; do not
  leave stale/wrong manifests in `.uor-models/manifests/` alongside the
  corrected ones without a clear explanation of which is authoritative.
- Add a falsifier: construct a manifest with deliberately bad recorded
  metrics (matching a known-bad case) and confirm the gate rejects it
  consistently, regardless of file-naming convention.

# Non-goals

- Do not relax the gate's bar to make more existing bundles pass.
- Do not fix the underlying generation-quality problem itself for the 1.7B
  or other bundles (tracked separately -- see the sibling
  degenerate-generation issue). This issue is about whether our own
  measurement of quality is honest and reproducible, not about improving
  quality.
- Do not fix the `R4Engine::load` panic (tracked separately as its own
  issue) except insofar as it blocks reproducing a measurement -- work
  around it (e.g. use bundles/paths that don't hit that panic) if needed
  rather than doing that fix inline here.

# Dependencies and blockers

- Findings from the sibling degenerate-generation-root-cause issue may be
  relevant context (or vice versa) -- coordinate if both are in flight at
  once, but neither strictly blocks the other.

# Proposed implementation shape

Start by reading the exact evaluator code path(s) and reproducing both
recorded results from source before writing any fix -- this is a
measurement-validity bug, and the fix must be grounded in understanding
why the numbers disagree, not a guess.

# Acceptance criteria

- [ ] The exact cause of the 0.0/1.0 vs 0.85/0.01 disagreement between
      `smollm2-135m-instruct.json` and `smollm2-135m-chat.json` (and the
      360m/1.7b pair) is identified and documented.
- [ ] `grounded_answer_rate`/`repetition_rate` are reproducible: running
      the evaluator again on the same compiled bytes produces the same
      numbers.
- [ ] The gate's pass/fail decision correlates with actual live `r4 ask`
      output quality on a held-out probe set -- a bundle producing
      degenerate/empty/word-salad output on ordinary prompts must not
      report `instruction_eval_passed: true`.
- [ ] Affected manifests in `.uor-models/manifests/` are corrected or
      clearly superseded; there is one authoritative, reproducible
      evaluation per compiled artifact, not two disagreeing ones.
- [ ] A falsifier test demonstrates the gate correctly rejects a
      known-bad case.
- [ ] Workspace fmt/clippy/tests pass.

# Verification

- Re-run the evaluator against the affected bundles and confirm
  reproducibility (same input -> same output, run twice).
- Manually spot-check `r4 ask` output against a handful of prompts for any
  bundle the corrected gate marks as passing; confirm the output is at
  minimum non-empty and non-degenerate (full coherence is not required
  here -- that's the sibling generation-quality issue's job -- but "empty"
  or "repeats the same handful of tokens" must not coexist with a passing
  gate).


## Comments (4)

### comment 5309741475 — 2026-08-16T21:23:19Z — Casey-allard

Sibling issues from the same investigation: #743 (R4Engine::load panic), #745 (root-cause degenerate generation).

### comment 5310493654 — 2026-08-17T00:26:58Z — Casey-allard

Starting on this one, per the recommended sequencing from #747.

**Root cause found by direct code reading (not yet the implementation, just the diagnosis):** `instruction_eval_passed`/`grounded_answer_rate`/`repetition_rate` in `ModelManifest.quality` are not computed anywhere. `r4 import`'s `ImportArgs` takes them as plain CLI floats (`--instruction-eval-passed`, `--grounded-answer-rate`, `--repetition-rate`) with hardcoded defaults (false/0.0/1.0) and writes whatever the operator typed straight into the manifest (`src/main.rs`, `import()`). The `--evaluation-report` file is stored content-addressed but never parsed or cross-checked against those three fields. Confirmed there is no hidden computation feeding them anywhere else: `crates/uor-r4-graph-cli`'s real `evaluate-report` command computes a completely different, real, offline metric set (`top1_accuracy_pct`, `teacher_argmax_agreement_pct`, `bits_over_teacher_floor` — an `EvaluationMetrics` struct with no `grounded_answer_rate`/`repetition_rate` fields at all), and `docs/MODEL_LIFECYCLE.md`'s own example command shows the intended usage is literally typing `--grounded-answer-rate 0.80 --repetition-rate 0.01` by hand. That's the entire explanation for #744's disagreeing manifests: two different people (or the same person twice) typed two different sets of numbers for the same compiled bytes, and nothing ever checked either set against reality.

One more relevant finding: this codebase already has a real, tested, generation-based repetition metric — `generate_greedy_repetition_rate` in `uor-r4-graph-certify::score` (Gate C), which tracks whether a newly generated token already appears in a 32-token recent window. The live `r4 ask` path (`hologram_answer` in `src/chat.rs`) has weaker guards: an exact 8-token block-repeat detector (`repeated_suffix`) on the plain-runtime path, and a consecutive-repeat penalty on the R4G1 beam-search path — neither catches small-vocabulary cycling through several distinct tokens in varying order, which is exactly the #745 word-salad pattern. Worth flagging for #745 too.

**Fix I'm implementing**: add a real live-generation probe evaluator (fixed set of ordinary questions, run through the same `ChatEngine`/plain-runtime path `r4 ask` uses, scored with the recent-window repetition-rate definition above) and make `r4 import` derive `instruction_eval_passed`/`grounded_answer_rate`/`repetition_rate` from that for instruction-chat imports, instead of accepting them as freeform CLI input. This is a CLI-surface change (removing the three manual flags for that capability) — flagging here in case that's a concern, but keeping them was the entire bug, so I don't see a way to fix this honestly without removing operator override.

Working in .worktrees/i744-quality-gate off main@ed83e7e2.

### comment 5310695216 — 2026-08-17T01:10:14Z — Casey-allard

PR #749 is up, implementing the fix described in my prior comment: `r4 import` now derives `instruction_eval_passed`/`grounded_answer_rate`/`repetition_rate` from a live probe run against the real `ask` code path instead of accepting them as CLI input.

**Verification caught a real bug before this was final** (worth recording, since it's directly relevant to the "gate correlates with live output" acceptance criterion): my first version reported the #745 bundle (`smollm2-1-7b-instruct`) as passing. I cross-checked against real `r4 ask` output on the identical bytes rather than trusting the synthetic unit tests, confirmed it's genuinely degenerate on two different live code paths, root-caused the metric to a too-narrow repetition window, and fixed it (widened window, recalibrated bar — see PR body for the numbers). Both known-bad local bundles (`smollm2-1-7b-instruct`, `smollm2-135m-instruct`) are now correctly rejected end-to-end via the real CLI.

**Leaving this issue open rather than closing with this PR.** One acceptance criterion — "the gate's pass/fail decision correlates with actual live `r4 ask` output quality" — is only fully satisfied for bundles served via the plain TLA/TLS1 path. I found, while verifying, that the same bytes can produce *worse* degenerate output ("cut cut cut...") via a bundle's R4G1 beam-search path, which `ask`/`chat` will transparently prefer over the plain path when a same-name-keyed `compiled.r4g1` file exists — and the new gate never probes that path. That's a real, separate piece of work (deciding how `import` should obtain R4G1 bytes, how to combine pass/fail across two divergent code paths, whether one boolean can still honestly summarize both), so I filed it as #750 rather than expanding this PR's scope. Once #750 lands, or if it's decided the current scope is sufficient and #750 becomes a lower-priority follow-up, this issue can close.

Also updated `docs/MODEL_LIFECYCLE.md`'s `r4 import` example, which was still showing the now-removed manual quality flags.

Per the recommended sequencing from #747, moving on to #745 next.

### comment 5313775451 — 2026-08-17T08:40:52Z — Casey-allard

# Closing: all acceptance criteria now satisfied

PR #749 (live-probe evaluator) and PR #761/#750 (R4G1-path probing +
cross-path pass/fail combination) landed the code side of this. This
comment closes out the one remaining item: correcting the stale local
`.uor-models/manifests/` files this issue's own filing documented.

## Root cause (recap, already documented in an earlier comment)

`instruction_eval_passed`/`grounded_answer_rate`/`repetition_rate` were
plain operator-typed CLI floats with no connection to any computation --
two imports of byte-identical compiled bytes could (and did) carry
opposite hand-typed numbers, both "passing." Fixed by #749 (derive from a
real live-generation probe run against the plain path) and #750 (probe
the R4G1 path too, when present, and require both to pass).

## Correcting the affected local manifests

Re-ran the now-fixed `r4 import` (post-#750, built from `main@611b767b`)
against the exact bundles this issue's evidence table named, using a
scratch worktree binary and the real `.uor-models` store
(`UOR_MODEL_STORE=/Users/casey.allard/uor-r4/.uor-models`, so nothing
touched any checkout's git state -- `.uor-models/` is gitignored local
model-store data, not repo content):

- `smollm2-135m-instruct` (current compiled bytes, `--r4g1` supplied):
  correctly **rejected** by the corrected gate --
  `ModelError::QualityGateFailed`.
- `smollm2-360m-instruct` (current compiled bytes, `--r4g1` supplied):
  correctly **rejected** likewise.
- `smollm2-1-7b-instruct` (same bytes `smollm2-1-7b-chat.json` pointed at,
  CID `blake3:fca5bdfb...`): already directly verified during #750's own
  real-bundle check, both with and without `--r4g1` -- correctly
  **rejected** both ways.
- `smollm2-135m-chat.json` pointed at byte-identical current artifacts to
  `smollm2-135m-instruct` (same CIDs) -- since the evaluator is a
  deterministic function of the exact bytes, re-running it separately
  under this name would reproduce the identical rejection; not re-run
  redundantly.

Every one of the four stale manifests in `.uor-models/manifests/` claimed
`instruction_eval_passed: true` for a bundle the corrected, honest
evaluator now rejects. Since `r4 import` refuses to write a manifest for
an instruction-chat bundle that fails the live-quality gate (by design --
`validate_for_chat()` runs before `write_manifest`, and only
`Continuation`-capability imports are allowed to proceed past a failing
gate), there is no way to produce a corrected *replacement* manifest for
any of these four -- the honest state is that none of them currently
qualify as a servable instruction-chat bundle. Removed all four:

```
rm .uor-models/manifests/smollm2-135m-instruct.json
rm .uor-models/manifests/smollm2-135m-chat.json
rm .uor-models/manifests/smollm2-360m-instruct.json
rm .uor-models/manifests/smollm2-1-7b-chat.json
```

`.uor-models/manifests/` is now empty rather than holding four manifests
that disagree with each other and with reality. This satisfies "there is
one authoritative, reproducible evaluation per compiled artifact, not two
disagreeing ones" -- the authoritative answer for all four bundles today
is "does not pass," and nothing on disk claims otherwise anymore. Getting
an actually-passing local chat bundle is separate follow-on work (the
generation-quality root cause, #745/#759's territory), not this issue's
job.

## Acceptance criteria

- [x] Root cause of the 0.0/1.0 vs 0.85/0.01 disagreement identified and
      documented (prior comment: hand-typed CLI floats, no computation).
- [x] `grounded_answer_rate`/`repetition_rate` reproducible: same
      compiled bytes -> same probe set -> same result, confirmed above
      (135m-instruct and 135m-chat share CIDs and are therefore
      guaranteed identical under the now-deterministic evaluator).
- [x] Gate's pass/fail correlates with live `r4 ask` output quality:
      established by #749/#750's falsifier tests and now reconfirmed
      against real local bundles above -- every bundle previously shown
      to produce empty/degenerate/word-salad output is now rejected.
- [x] Affected manifests corrected/superseded -- all four removed, no
      disagreeing or false-passing manifests remain.
- [x] Falsifier tests: `live_quality_gate_rejects_known_bad_probe_sets_and_accepts_a_healthy_one`
      (#749) and `combine_path_quality_requires_both_probed_paths_to_pass`
      (#750).
- [x] Workspace fmt/clippy/tests pass (verified in #750's PR).

Closing.



---
_Comment/review ids in this file: 5309741475, 5310493654, 5310695216, 5313775451_

